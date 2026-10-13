# Iridescence PDF Reader — Reliability & Fault Tolerance Architecture

## Overview

Iridescence PDF Reader is engineered to operate reliably under extreme conditions, including corrupted files, circular directory hierarchies, network timeouts, encrypted documents, and intense multitasking on macOS.

---

## 1. Crash Hazard Elimination

### 1.1 AppKit & Cocoa Delegate Safety
In Objective-C bindings via `objc2`, delegate callbacks can receive unexpected parameters, such as negative indices or items during background node updates.
- **`outlineView:child:ofItem:`**:
  Previous implementations used unchecked index lookups (`children[index as usize]`). When an outline view reloaded asynchronously while a node collapsed, this caused panic index out of bounds. The hardened implementation checks `index < 0` and `index as usize >= children.len()`, returning `std::ptr::null_mut()` safely.
- **Notification Payloads**:
  Notification objects from `NSNotificationCenter` may occasionally be nil or unrelated types. All notification handlers now use defensive pattern matching (`if let Some(...) = ...`) rather than unwrapping.

### 1.2 Mutex Poison Recovery
In multi-threaded Rust code, an unexpected panic on a worker thread can poison locks. Iridescence uses poisoned lock recovery:
```rust
let guard = match LOCK.lock() {
    Ok(g) => g,
    Err(poisoned) => poisoned.into_inner(),
};
```
This guarantees that worker threads logging diagnostics or querying the workspace index never crash the main application.

---

## 2. Resource & Memory Bounds

### 2.1 Bounded Logging (`LOG_BUFFER`)
During extensive reading sessions, in-memory log accumulators can cause stealth memory bloat. Iridescence enforces a hard ceiling of `MAX_LOG_BUFFER_SIZE = 2000` entries in [`src/logger.rs`](file:///Users/theodeville/Documents/OxiReader/src/logger.rs), evicting the oldest entries via FIFO queue operations.

### 2.2 Deserialization Hardening (`IRIDX1`)
The binary workspace cache deserializer in [`src/workspace_index.rs`](file:///Users/theodeville/Documents/OxiReader/src/workspace_index.rs) includes strict validation gates:
- Entry count: ≤ 50,000 files
- Path length: ≤ 4,096 bytes
- Page count: ≤ 25,000 pages per document
- Text size: ≤ 5,000,000 bytes per page

Payloads exceeding these thresholds are rejected immediately without triggering large memory allocations.

### 2.3 Symlink Cycle & Recursion Protection
Naively traversing macOS directories can lead to infinite loops if recursive symlinks exist. [`collect_pdfs_safe`](file:///Users/theodeville/Documents/OxiReader/src/workspace_index.rs) tracks canonical paths using a `HashSet<PathBuf>`, caps traversal depth at 25 levels, and caps total collected files at 10,000.

---

## 3. Password & Encryption Handling

Password-protected and encrypted PDF documents previously resulted in blank canvas rendering or silent failures. Iridescence queries `doc.isLocked()` via PDFKit before loading. When an encrypted document is encountered:
- The application logs a diagnostic notice.
- A native macOS `NSAlert` modal is presented to the user explaining that encrypted documents cannot be unlocked automatically.
- The UI gracefully maintains its prior state without corruption.
