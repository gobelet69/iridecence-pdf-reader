# Iridescence PDF Reader — Testing Strategy & Test Suites

## Philosophy

Reliability in a systems application cannot rely solely on visual inspection. Iridescence couples automated unit and adversarial regression testing with visual state capture tooling.

---

## 1. Automated Test Suite (`cargo test`)

The test suite tests real system invariants across multiple subsystems:

### 1.1 Index Resilience & Adversarial Payloads ([`src/workspace_index.rs`](file:///Users/theodeville/Documents/OxiReader/src/workspace_index.rs))
- **`test_roundtrip_index_serialization`**: Verifies full fidelity roundtrip binary encoding/decoding of workspace metadata, unicode filepaths, page numbers, and text snippets.
- **`test_corrupt_magic_rejection`**: Confirms that non-`IRIDX1` headers are safely rejected without panicking.
- **`test_truncated_stream_handling`**: Confirms that streams cut off mid-entry abort without panics or undefined memory reads.
- **`test_oversized_count_dos_protection`**: Tests adversarial inputs claiming >50,000 files to ensure OOM protection is active.
- **`test_oversized_path_len_dos_protection`**: Tests adversarial inputs claiming >4,096 byte paths.
- **`test_oversized_page_count_dos_protection`**: Tests adversarial inputs claiming >25,000 pages per document.
- **`test_collect_pdfs_safe_cycle_protection`**: Creates a temporary directory structure containing circular symlinks (`dir/loop -> dir`) and verifies that directory traversal completes without stack overflow.

### 1.2 Multi-Document Tab Manager ([`src/tabs.rs`](file:///Users/theodeville/Documents/OxiReader/src/tabs.rs))
- **`test_tab_manager_initial_state`**: Verifies empty state guarantees.
- **`test_tab_manager_out_of_bounds_remove`**: Tests removal on invalid and empty indices.
- **`test_tab_manager_lifecycle_and_navigation`**: Simulates tab insertion, active index updating, circular navigation (`next_index`, `prev_index` wrap-around), and index shifting upon closure.

### 1.3 Memory Bounding ([`src/logger.rs`](file:///Users/theodeville/Documents/OxiReader/src/logger.rs))
- **`test_log_event_records_message`**: Validates diagnostic capture.
- **`test_log_buffer_bounded_capacity`**: Verifies that the internal buffer never exceeds `MAX_LOG_BUFFER_SIZE` (2,000 items).

### 1.4 Window Geometry Constraints ([`src/window_geometry.rs`](file:///Users/theodeville/Documents/OxiReader/src/window_geometry.rs))
- **`test_window_geometry_min_max_bounds`**: Confirms mathematical invariants `min <= max` across all window configurations (`Home`, `Reader`, `Palette`, `DebugConsole`, `Utility`).
- **`test_window_geometry_sanitize_nan_and_inf`**: Asserts that NaN and Infinite coordinates are repaired into valid screen frames.
- **`test_window_geometry_sanitize_offscreen_frame`**: Asserts that windows restored off-screen are clamped back within visible bounds.

### 1.5 Annotation Mappings ([`src/annotations.rs`](file:///Users/theodeville/Documents/OxiReader/src/annotations.rs))
- **`test_annotation_type_subtypes`**: Validates exact PDFKit string specifiers (`Highlight`, `Underline`, `StrikeOut`, `Text`).
- **`test_annotation_type_symbols`**: Validates SF Symbol names for native UI buttons.
- **`test_annotation_type_display_names`**: Validates human-readable labels.

---

## 2. Executing the Test Suite

```bash
# Run all unit and regression tests
cargo test

# Run tests with stdout output
cargo test -- --nocapture
```
