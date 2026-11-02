# Iridescence PDF Reader — Agent Guidelines & Invariants

This document serves as the mandatory operational manual and architectural contract for all AI agents working on the **Iridescence PDF Reader** codebase.

---

## 1. Project Overview & Architecture

Iridescence PDF Reader is a high-performance, native macOS PDF reader and workspace manager written in **pure Swift 6.0** with Apple's native AppKit and PDFKit:

- **UI & Window Management**: AppKit (`NSWindow`, `NSSplitViewController`, `NSVisualEffectView`) with native Liquid Glass styling and custom toolbar controls.
- **PDF Engine**: Apple PDFKit (`PDFView`, `PDFDocument`, `PDFPage`, `PDFAnnotation`) with native text rendering, annotations, and outlines.
- **OCR Engine**: Apple Vision framework (`VNRecognizeTextRequest`) running on background threads via `Task.detached`.
- **Index & Search**: Custom binary serialization format (`IRIDX1`) with bounded memory and fast full-text substring scanning.
- **Workspace Navigation**: Asynchronous directory trees (`NSOutlineView`), iCloud status monitoring, and drag-and-drop support.

---

## 2. Fundamental Safety & Boundary Invariants

### 2.1 The MASTER Directory is Read-Only
- **STRICT PROHIBITION**: Never touch, modify, write to, or delete files in `/Users/theodeville/Documents/MASTER` under any circumstances.
- This invariant is enforced in code ([`AnnotationsManager.swift`](file:///Users/theodeville/Documents/Iridescence/iridecence-pdf-reader/Sources/IridescenceCore/AnnotationsManager.swift) and [`AppDelegate.swift`](file:///Users/theodeville/Documents/Iridescence/iridecence-pdf-reader/Sources/IridescenceCore/AppDelegate.swift)) and must be honored by every agent.

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

1. **Zero Preventable Crashes**:
   - Never use force unwraps (`!`) on Cocoa notification payloads, user inputs, filesystem queries, or thread locks.
   - For `NSOutlineView` delegate callbacks (`outlineView:child:ofItem:`), always perform explicit bounds checking against negative indices and out-of-range lengths before indexing.

2. **Strict Resource Bounds**:
   - Memory buffers must be bounded (e.g., `LOG_BUFFER` capped at `MAX_LOG_BUFFER_SIZE = 2000`).
   - File parsers and deserializers must enforce defensive bounds:
     - Item count ≤ 50,000
     - Path length ≤ 4,096 bytes
     - Page count ≤ 25,000
     - Text length per page ≤ 5,000,000 bytes
   - Directory scans must detect symlink cycles (`HashSet<String>`) and enforce recursion depth caps (depth ≤ 25).

3. **Responsive Main Thread (Zero Freezes)**:
   - File I/O, PDF indexing, and OCR requests must execute on background threads.
   - Dispatch UI updates back to the main actor via `MainActor.run` or `Task { @MainActor in ... }`.

4. **iCloud & Target Synchronization**:
   - macOS iCloud Desktop & Documents sync can cause build lockouts. Always configure Swift builds and tests with scratch paths in `/tmp`:
     ```bash
     swift test --scratch-path /tmp/pdf-test
     ```

---

## 4. Verification Workflow

Before concluding any session or committing code, run the verification commands:

```bash
# 1. Automated Unit & Integration Tests (20 tests)
swift test --scratch-path /tmp/pdf-test

# 2. Build release application bundle
./build_app.sh

# 3. Full verification script
./scripts/verify.sh
```

---

## 5. Liquid Glass & macOS UI/UX Architecture Rules

To maintain the native macOS 26+ aesthetic and prevent visual regressions:

1. **Window Translucency & Compositing**:
   - The main window must have `setOpaque(false)`, `setBackgroundColor(NSColor.clear)`, and `setHasShadow(true)`.
   - Never fill root container views with opaque background colors (`windowBackgroundColor`).
   - Root containers must be backed by `NSVisualEffectView` with `NSVisualEffectView.BlendingMode.behindWindow` and material `NSVisualEffectView.Material.underWindowBackground` (or `sidebar` for sidebar panels).

2. **Table View Styles**:
   - Modern AppKit tables must use `.style = .sourceList` for sidebars or `.style = .inset` for content/command lists.

3. **Dynamic Semantic Colors**:
   - Never hardcode static RGB colors for UI text or icons.
   - Always use macOS semantic dynamic colors: `NSColor.labelColor`, `secondaryLabelColor`, `tertiaryLabelColor`, `separatorColor`, and `controlAccentColor`. This guarantees high contrast and seamless appearance adaptation in both Light and Dark modes.
