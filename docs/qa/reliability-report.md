# Iridescence PDF Reader — Reliability Engineering Report

## Executive Summary

This report documents the reliability engineering outcomes achieved during the hardening phase of Iridescence PDF Reader. The primary objective was transitioning the application from a rapid prototype to production-grade stability, eliminating all preventable panics, bounding memory growth, and establishing a regression testing suite.

---

## 1. Identified & Resolved Hazards

### 1.1 Objective-C Delegate Index Panics
- **Component**: [`WorkspaceDelegate`](file:///Users/theodeville/Documents/OxiReader/src/workspace_delegate.rs)
- **Problem**: In `outlineView:child:ofItem:`, the index queried by AppKit was cast directly into a slice index: `children.as_ref().unwrap()[index as usize]`. During asynchronous workspace refreshes, AppKit could query an out-of-bounds index, triggering an immediate panic and termination.
- **Resolution**: Implemented defensive guards checking `index < 0` and `index as usize >= children.len()`, returning `std::ptr::null_mut()`. Replaced panicking `.unwrap()` calls in `nodeDidLoadChildren` and `outlineViewSelectionDidChange` with safe guard clauses.

### 1.2 Unbounded Memory Leaks in Logging
- **Component**: [`src/logger.rs`](file:///Users/theodeville/Documents/OxiReader/src/logger.rs)
- **Problem**: `LOG_BUFFER` appended every application log event into an in-memory vector without eviction. In long-running sessions, this vector grew indefinitely.
- **Resolution**: Defined `MAX_LOG_BUFFER_SIZE = 2000`. When buffer capacity is exceeded, oldest entries are evicted via `drain`. Handled mutex poisoning safely.

### 1.3 Denial-of-Service via Malformed Cache
- **Component**: [`WorkspaceIndex`](file:///Users/theodeville/Documents/OxiReader/src/workspace_index.rs)
- **Problem**: Binary deserialization trusted file headers unconditionally. A corrupted or malicious `workspace_index.bin` could specify huge entry counts or path lengths, causing gigabytes of RAM to be allocated immediately.
- **Resolution**: Added strict limits on entry count (≤ 50,000), path length (≤ 4,096 bytes), page count (≤ 25,000), and text length per page (≤ 5,000,000 bytes). Added unit tests confirming rejection.

### 1.4 Stack Overflow via Directory Symlink Cycles
- **Component**: [`collect_pdfs_safe`](file:///Users/theodeville/Documents/OxiReader/src/workspace_index.rs)
- **Problem**: Recursive directory traversal lacked symlink loop detection. If a workspace contained a circular symlink (e.g. `folder/link -> folder`), the crawler would recurse until thread stack overflow.
- **Resolution**: Integrated `HashSet<PathBuf>` canonical path tracking, a recursion depth limit (depth ≤ 25), and a file cap (10,000 files). Added automated test `test_collect_pdfs_safe_cycle_protection`.

### 1.5 Encrypted PDF Handling
- **Component**: [`load_pdf_document`](file:///Users/theodeville/Documents/OxiReader/src/main.rs)
- **Problem**: Password-protected PDFs attempted to load normally, resulting in blank views and unhandled delegate events.
- **Resolution**: Added `doc.isLocked()` inspection with user-facing `NSAlert` modal.

---

## 2. Test Suite Status

- **Unit & Regression Tests**: 18 tests registered across 5 subsystems.
- **Test Results**: 18 passed; 0 failed; execution time 0.15s.
- **Compilation**: Clean `cargo check` (0 errors).
- **Packaging**: Clean bundle generation via `./build_app.sh`.
