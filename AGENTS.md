# Iridescence PDF Reader — Agent Guidelines & Invariants

This document serves as the mandatory operational manual and architectural contract for all AI agents working on the **Iridescence PDF Reader** codebase.

---

## 1. Project Overview & Architecture

Iridescence PDF Reader is a high-performance, native macOS PDF reader and workspace manager written in **pure Rust** using Apple's modern Objective-C runtime via `objc2`:

- **UI & Window Management**: AppKit (`objc2_app_kit`) with native Liquid Glass (`NSVisualEffectView`), `NSSplitViewController`, and native toolbar controls.
- **PDF Engine**: Apple PDFKit (`objc2_pdf_kit`) with native text rendering, annotations, and outlines.
- **OCR Engine**: Apple Vision framework (`VNRecognizeTextRequest`) running on background threads.
- **Index & Search**: Custom binary serialization format (`IRIDX1`) with bounded memory and fast full-text substring scanning.
- **Workspace Navigation**: Asynchronous directory trees (`NSOutlineView`), iCloud status monitoring, and drag-and-drop support.

---

## 2. Fundamental Safety & Boundary Invariants

### 2.1 The MASTER Directory is Read-Only
- **STRICT PROHIBITION**: Never touch, modify, write to, or delete files in `/Users/theodeville/Documents/MASTER` under any circumstances.
- This invariant is enforced in code (e.g. [`src/annotations.rs`](file:///Users/theodeville/Documents/OxiReader/src/annotations.rs)) and must be honored by every agent.

### 2.2 Preservation of Existing Capabilities
Do not regress, rewrite from scratch, or drop existing features. Always preserve:
- Home Screen (Recent, Favorites, Continue Reading)
- Standalone PDF vs Workspace mode transitions
- Pages / Outline / Search right sidebar with segmented control
- Liquid Glass Command Palette (`Cmd+P`) and prefix routing (`>`, `/`, `p:`)
- iCloud status badges, download coordination, and progress tracking
- Multi-document tabs (`TabManager`)
- PDF annotations (Highlight, Underline, Strikeout, Notes) and safe saving
- OCR text extraction panel
- Visual QA automation tooling and screenshot captures

### 2.3 Non-Destructive Git Workflows
- **NEVER** run `git reset --hard` without explicit user instruction.
- **NEVER** overwrite uncommitted user modifications.
- Preserve commit history and build artifacts cleanly.

---

## 3. Reliability & Robustness Rules

1. **Zero Preventable Panics**:
   - Never call `.unwrap()` or `.expect()` on Cocoa notification payloads, user inputs, filesystem queries, or thread locks.
   - For mutexes, always recover from poisoning:
     ```rust
     let guard = match LOCK.lock() {
         Ok(g) => g,
         Err(poisoned) => poisoned.into_inner(),
     };
     ```
   - For `NSOutlineView` delegate callbacks (`outlineView:child:ofItem:`), always perform explicit bounds checking against negative indices and out-of-range lengths before indexing.

2. **Strict Resource Bounds**:
   - Memory buffers must be bounded (e.g., `LOG_BUFFER` capped at `MAX_LOG_BUFFER_SIZE = 2000`).
   - File parsers and deserializers must enforce defensive bounds:
     - Item count ≤ 50,000
     - Path length ≤ 4,096 bytes
     - Page count ≤ 25,000
     - Text length per page ≤ 5,000,000 bytes
   - Directory scans must detect symlink cycles (`HashSet<PathBuf>`) and enforce recursion depth caps (depth ≤ 25).

3. **Responsive Main Thread (Zero Freezes)**:
   - File I/O, PDF indexing, and OCR requests must execute on background threads.
   - Dispatch UI updates back to the main thread via `performSelectorOnMainThread` or GCD.

4. **iCloud & Target Synchronization**:
   - macOS iCloud Desktop & Documents sync can cause build lockouts if Cargo writes to `target/` within `Documents`. Always configure `CARGO_TARGET_DIR=/tmp/target` or use `target.nosync/`.

---

## 4. Verification Workflow

Before concluding any session or committing code, run the verification script:

```bash
./scripts/verify.sh
```

This verifies:
1. `cargo check` passes with 0 compilation errors.
2. `cargo test` passes all unit and regression tests.
3. `./build_app.sh` packages `Iridecence PDF Reader.app` cleanly.

---

## 5. Liquid Glass & macOS UI/UX Architecture Rules

To maintain the native macOS 27 aesthetic and prevent visual regressions:

1. **Window Translucency & Compositing**:
   - The main window must have `setOpaque(false)`, `setBackgroundColor(Some(&NSColor::clearColor()))`, and `setHasShadow(true)`.
   - Never fill root container views with opaque background colors (`windowBackgroundColor`).
   - Root containers must be backed by `NSVisualEffectView` with `NSVisualEffectBlendingMode::BehindWindow` and material `NSVisualEffectMaterial::UnderWindowBackground` (or `Sidebar` for sidebar panels).

2. **Table View Styles vs Deprecated APIs**:
   - Modern AppKit tables must use `setStyle: 1usize` (`NSTableViewStyleSourceList`) for sidebars or `setStyle: 2usize` (`NSTableViewStyleInset`) for content/command lists.
   - **NEVER** call deprecated `setSelectionHighlightStyle(SourceList)` as it triggers legacy drawing fallbacks and compiler warnings.

3. **Table Row Selection Highlights (`GlassTableRowView`)**:
   - Standard `NSTableRowView` creates private internal `NSVisualEffectView` subviews for row selection that render as solid pitch-black rectangles in offscreen/headless capture contexts (`cacheDisplayInRect:`).
   - All interactive table and outline views with selection must return `GlassTableRowView` from `tableView:rowViewForRow:` or `outlineView:rowViewForItem:`.
   - `GlassTableRowView` suppresses internal selection effect subviews in `layout()` and draws a native translucent rounded capsule using `NSColor::controlAccentColor().colorWithAlphaComponent(0.22)` in `drawSelectionInRect:`.

4. **Dynamic Semantic Colors**:
   - Never hardcode static RGB colors for UI text or icons.
   - Always use macOS semantic dynamic colors: `NSColor::labelColor()`, `secondaryLabelColor()`, `tertiaryLabelColor()`, `separatorColor()`, and `controlAccentColor()`. This guarantees high contrast and seamless appearance adaptation in both Light and Dark modes.

---

## 6. AppKit & Objective-C Runtime Lessons Learned

1. **`define_class!` Method Invocations**:
   - Objective-C methods defined in `define_class!` via `#[unsafe(method(...))]` expect hidden selector arguments (`_cmd: Sel`).
   - Calling them with standard Rust method syntax `self.foo(arg)` from within other methods fails compilation (`error[E0061]: method takes 2 arguments`).
   - Always invoke them dynamically via `unsafe { let _: () = objc2::msg_send![self, foo: arg]; }` or split the business logic into an internal helper function.

2. **`NSOutlineView` Click vs Selection Handling**:
   - In AppKit, single-clicking an already-selected row does not fire `outlineViewSelectionDidChange:`.
   - Setting `outline_view.setAction(Some(sel!(workspaceClicked:)))` in addition to `setDoubleAction:` ensures every user click responds immediately without requiring a deselection cycle.

3. **macOS Symlink Path Canonicalization**:
   - `/tmp` on macOS is a symlink to `/private/tmp`.
   - Path comparisons between `PDFDocument::documentURL()` and user-provided or CLI paths must always resolve canonical paths via `std::fs::canonicalize` to prevent false mismatches (e.g., in `already_open` tab routing).

4. **Command Palette Spotlight Geometry & Cursoring**:
   - Spotlight-style panels should use `NSWindowStyleMask::Borderless` (`0`), `setFloatingPanel: true`, floating window level 3 (`kCGFloatingWindowLevel`), and `NSVisualEffectMaterial::Popover`.
   - Dynamic height must shrink to `48.0 pt` when empty and hide the separator and table view.
   - When prefilling prefix characters (`>` or `p:`), explicitly invoke `setSelectedRange: NSMakeRange(prefix.len(), 0)` to place the cursor at the end without triggering AppKit's full-text blue selection highlight.


