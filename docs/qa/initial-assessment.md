# Iridescence PDF Reader — Initial Engineering Assessment

**Date**: 2026-10-13  
**Target**: `com.iridecence.pdfreader` (macOS native pure Rust + AppKit/PDFKit/Vision)  
**Assessor**: Autonomous Reliability & Performance Engineer  

---

## 1. Executive Summary

Iridescence PDF Reader is a high-performance native macOS PDF reader built in Rust using direct Objective-C runtime bindings (`objc2`). The application currently features exceptional baseline rendering speeds (20–40ms document load, 10–15ms tab switching, 6.2MB baseline RSS). However, an in-depth audit of the codebase reveals critical safety, resilience, and testing gaps:

1. **Zero Unit Tests**: The Rust test suite currently contains `0 tests`, leaving core domain logic (tab management, indexing, serialization, geometry clamping) untested against regression.
2. **Defensive Programming Hazards**: Several Cocoa delegate methods contain dangerous `.unwrap()` calls and unvalidated slice indexing (`children.as_ref().unwrap()[index as usize]`) that will trigger instant process panics if Cocoa sends unexpected inputs or during asynchronous race conditions.
3. **Adversarial Input Vulnerabilities**:
   - `collect_pdfs` in `src/workspace_index.rs` performs unbounded recursive directory traversal without symlink cycle detection, susceptible to stack overflow.
   - `load_from_disk` in `src/workspace_index.rs` trusts lengths read directly from disk (`u32::from_le_bytes`), risking multi-gigabyte heap allocations or OOM crashes if the cache is corrupt.
4. **Unbounded Resource Growth**: `LOG_BUFFER` in `src/logger.rs` pushes every event without an upper limit or ring-buffer eviction, causing unbounded memory growth during long sessions.
5. **Silent Document Failures**: Password-protected (`isLocked`) or corrupt PDFs fail silently without user-facing feedback or recovery options.

---

## 2. Compilation & Tooling Baseline

| Check | Status | Details |
| :--- | :---: | :--- |
| `cargo check` | **Pass (0 errors)** | 38 warnings: deprecated `msg_send_id!` macros and Apple AppKit bezel style constants (`NSBezelStyle::Rounded`, `NSTableViewSelectionHighlightStyle::SourceList`). |
| `cargo test` | **0 tests executed** | No `#[test]` modules registered in any source file. |
| `build_app.sh` | **Pass** | Generates valid macOS app bundle `Iridecence PDF Reader.app` with embedded Retina icons. |
| Visual QA Suite | **Pass (19/19)** | Automated QA runner (`scripts/visual_qa.sh`) successfully exercises and captures 19 UI states. |

---

## 3. Detailed Threat & Defect Inventory

### 3.1 P0 — Critical Hazards (Panic & Crash Vectors)

- **`WorkspaceDelegate::outlineView_child_ofItem` (`src/workspace_delegate.rs:112`)**:
  - Code: `let child = &children.as_ref().unwrap()[index as usize];`
  - Hazard: If children are currently reloading or `None`, or if `index < 0` or `>= children.len()`, a panic occurs inside the Objective-C delegate callback, triggering an immediate abort.
  - Fix: Guard with `if let Some(children) = children.as_ref() { if let Some(child) = children.get(index as usize) { ... } }` and return `std::ptr::null_mut()`.

- **`WorkspaceDelegate::nodeDidLoadChildren` (`src/workspace_delegate.rs:41`)**:
  - Code: `let node = node.unwrap();`
  - Hazard: If the notification object is nil or of an unexpected type, panics immediately.
  - Fix: Replace with `let Some(node) = node else { return; };`.

- **`WorkspaceDelegate::outlineViewSelectionDidChange` (`src/workspace_delegate.rs:63`)**:
  - Code: `let outline_view = outline_view.unwrap();`
  - Hazard: Panics if notification object is nil.
  - Fix: Replace with `let Some(outline_view) = outline_view else { return; };`.

- **`src/main.rs` Unchecked Unwraps on URLs and Superviews**:
  - Lines 775, 913, 1688, 1717, 1758: `let path = url.path().unwrap().to_string();`
  - Line 815: `let superview = scroll.superview().unwrap();`
  - Line 909: `self.ivars().pdf_view.get().unwrap().document()`
  - Hazard: Remote/malformed URLs or non-attached views cause immediate panics.
  - Fix: Convert to idiomatic `if let Some(...)` expressions.

### 3.2 P1 — Resource Management & Denial of Service

- **Unbounded Directory Traversal & Cyclic Symlinks (`src/workspace_index.rs:348`)**:
  - Code: `if path.is_dir() { collect_pdfs(&path, out); }`
  - Hazard: Follows symlinks without cycle detection or maximum depth limit. A cyclic symlink causes infinite recursion and stack overflow.
  - Fix: Maintain a `HashSet<PathBuf>` of canonicalized paths, cap depth at 25, and limit total collected PDFs to 10,000.

- **Unbounded Allocations in Index Deserializer (`src/workspace_index.rs:87, 129`)**:
  - Code: `let mut text_bytes = vec![0u8; text_len];`
  - Hazard: Reading a corrupted or hostile `workspace_index.bin` file with large `text_len` will allocate gigabytes of RAM or crash with OOM.
  - Fix: Validate length bounds before allocation (`path_len <= 4096`, `page_count <= 25_000`, `text_len <= 5_000_000`).

- **Unbounded In-Memory Log Buffer (`src/logger.rs:40`)**:
  - Code: `buffer.push(msg);`
  - Hazard: Every logged event is retained forever in memory.
  - Fix: Limit `LOG_BUFFER` capacity to 2,000 entries; evict oldest entries when full.

### 3.3 P2 — Missing Document State Handling

- **Locked / Password-Protected PDFs**:
  - Hazard: If a PDF requires a password, `PDFDocument::initWithURL` returns a document whose pages cannot be accessed, but no error or password prompt is provided.
  - Fix: Inspect `doc.isLocked()`; log and display informative status.

- **Empty Documents (0 Pages)**:
  - Hazard: An empty document (page count 0) can cause divisions by zero or invalid indexing when calculating thumbnails and jump targets.
  - Fix: Check `doc.pageCount() > 0` before initiating page-specific navigation.

---

## 4. Performance Baselines

Tested on Apple Silicon (macOS 15.x):

- **Cold Launch**: 374.99 ms
- **Warm Launch**: 109.53 ms
- **PDF Open (10 MB, 352 pages)**: 22.20 ms
- **Tab Switch**: 10.49 – 14.92 ms
- **Command Palette Search**: 3.49 – 4.01 ms
- **Idle RSS**: 6.18 MB
- **Idle CPU**: 0.0%
