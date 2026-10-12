# Iridecence PDF Reader Roadmap

## Phase 1 — Research and Architecture (Completed)
- [x] Inspect the development environment.
- [x] Select the technical stack (AppKit + PDFKit via Rust `objc2`).
- [x] Create initial architecture specification and development roadmap.

## Phase 2 — Working Application Core (Completed)
- [x] Initialize pure Rust project (`objc2`, `objc2-app-kit`, `objc2-pdf-kit`).
- [x] Create macOS native application window with unified titlebar.
- [x] Implement native file opening and document lifecycle.
- [x] Integrate PDFKit rendering with continuous and single-page viewing modes.
- [x] Smooth zooming, fit presets, and page navigation.

## Phase 3 — Usability & Liquid Glass Foundation (Completed)
- [x] Responsive PDF rendering and asynchronous page streaming.
- [x] Functional native Liquid Glass materials (`NSVisualEffectView` materials and vibrant dividers).
- [x] Collapsible workspace and document sidebars.
- [x] Full macOS native application menus and keybindings.
- [x] Finder integration (`application:openURLs:` & drag-and-drop file opening).
- [x] Robust error handling, non-blocking asynchronous iCloud Drive faulting and downloads.
- [x] Responsive default window geometry (centered 960x640pt clamped to screen).

## Phase 4 — Productivity & Navigation (Completed)
- [x] Multi-tab document management with isolated per-tab sessions and dirty-state tracking.
- [x] Document sidebar with Pages thumbnail grid, hierarchical Outline tree, and Full-Text Search.
- [x] Command Palette & Quick Open (`Cmd+P`, `>`, `/`, `:`) with instant fuzzy filtering.
- [x] Back & Forward navigation history (`Cmd+[` / `Cmd+]`).
- [x] Reading position persistence (page index, scroll offset, zoom factor per canonical path).
- [x] Home screen dashboard: Continue Reading, Recents, and Pinned Favorites.
- [x] Text selection, Copy with Page Reference, and Markdown Quote copying.

## Phase 5 — Annotations, OCR & Preferences (Completed)
- [x] PDFKit-native annotations (Highlight, Underline, Strikethrough, Text Notes).
- [x] Document dirty tracking and atomic Save (`Cmd+S`) / Save As (`Cmd+Shift+S`).
- [x] Local on-device OCR using Apple Vision framework (`VNRecognizeTextRequest`) with page overlay.
- [x] Dedicated native Settings window (`Cmd+,`) for appearance, rendering, and defaults.
- [x] Focus Mode (`Cmd+Shift+F`) and Presentation Mode (`Cmd+Option+P`).
- [x] Automated visual QA regression suite and performance audit harnesses.

## Phase 6 — Advanced Document Utilities & Cloud Sync (Future)
- [x] Workspace indexing and rapid multi-file search.
- [ ] Merge and split PDF documents.
- [ ] Extract, reorder, delete, and rotate pages.
- [ ] Custom highlight color palettes and annotation export (JSON / Markdown).
- [ ] CloudKit sync for reading position and annotations across macOS devices.

