# Iridescence PDF Reader — Regression Testing Matrix

| ID | Feature / Component | Scenario | Expected Behavior | Automated / Manual Verification | Status |
|---|---|---|---|---|---|
| **REG-01** | Workspace Tree Navigation | Rapid expansion and collapse of directory nodes | No panic in `outlineView:child:ofItem:`, safe handling of stale indices | Automated (`test_tab_manager_out_of_bounds_remove`, bounded index checks) | ✅ PASS |
| **REG-02** | Symlink Directory Cycles | Directory tree containing cyclic symlink loop | Traversal terminates without stack overflow or hang | Automated (`test_collect_pdfs_safe_cycle_protection`) | ✅ PASS |
| **REG-03** | Corrupt Index Cache | Workspace index file (`workspace_index.bin`) corrupted or malformed | Safely rejected without crash; index starts empty | Automated (`test_corrupt_magic_rejection`, `test_truncated_stream_handling`) | ✅ PASS |
| **REG-04** | Index Memory DOS Attack | Malicious cache claiming >50,000 files or oversized strings | Bounded memory caps abort deserialization cleanly | Automated (`test_oversized_count_dos_protection`, `test_oversized_path_len_dos_protection`, `test_oversized_page_count_dos_protection`) | ✅ PASS |
| **REG-05** | Memory Bloat in Long Sessions | Excessive logging during extensive application use | `LOG_BUFFER` remains strictly bounded to ≤ 2,000 items | Automated (`test_log_buffer_bounded_capacity`) | ✅ PASS |
| **REG-06** | Encrypted / Locked PDFs | Opening a password-protected PDF document | Informational alert displayed; app remains stable | Code verified (`doc.isLocked()` check in `load_pdf_document`) | ✅ PASS |
| **REG-07** | Read-Only MASTER Guard | Attempting to save annotations into `/Documents/MASTER` | Save blocked with explicit user modal; file untouched | Guard in `src/annotations.rs` verified | ✅ PASS |
| **REG-08** | Multi-Document Tabs | Tab addition, navigation, wrap-around, and closing | Correct active tab tracking; no panic on out-of-bounds removal | Automated (`test_tab_manager_lifecycle_and_navigation`) | ✅ PASS |
| **REG-09** | Window Geometry Sanitization | Restoring corrupted/offscreen/NaN window geometry | Window clamped to visible bounds on current display | Automated (`test_window_geometry_sanitize_nan_and_inf`, `test_window_geometry_sanitize_offscreen_frame`) | ✅ PASS |
| **REG-10** | Command Palette (`Cmd+P`) | Opening palette with prefix routing (`>`, `/`, `p:`) | Spotlight-style panel appears with focused search input | Visual QA State 10 & 11 | ✅ PASS |
| **REG-11** | Full-Text Search in PDF | Searching keywords across pages | Search results listed with snippets and page jumping | Visual QA State 07 | ✅ PASS |
| **REG-12** | Right Sidebar Modes | Switching between Pages, Outline, and Search | Segmented control switches subview without header overlay | Visual QA State 05, 06, 07 | ✅ PASS |
| **REG-13** | OCR Text Extraction | Running OCR on scanned document | OCR text extracted asynchronously in side-panel | Visual QA State 14 | ✅ PASS |
| **REG-14** | Reading Position Memory | Re-opening previously read document | Restores last viewed page and scale factor | Preference persistence verified | ✅ PASS |
| **REG-15** | iCloud Download Badges | Viewing dataless documents in iCloud workspace | Cloud badge displayed; download triggers on click | Workspace delegate verified | ✅ PASS |
