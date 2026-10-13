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
