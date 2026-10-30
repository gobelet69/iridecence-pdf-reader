# Changelog

All notable changes to **Iridecence PDF Reader** are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.5.1] - 2026-10-30

### Fixed
- **Asynchronous PDF Loading**: Offloaded PDFDocument initialization to background threads for opening files, history navigation, post-iCloud download, and external file reloads, keeping the main UI thread fluid.
- **Non-blocking Search & Enumeration**: Backgrounded PDF text search and Home screen directory enumeration to eliminate UI hitches.
- **Workspace Navigation & Identity**: Added cancellation tokens for background folder scans, preventing stale completion races and tab identity mismatches.
- **Tab Lifecycle & Memory**: Resolved inactive-tab PDFKit page detaching and memory retention issues across long reading sessions.
- **Hover & Mouse Tracking**: Restored missing `mouseEntered:` tracking area events across Home screen cards, Workspace items, and Command Palette rows.
- **Markup Annotation Geometry**: Corrected page coordinate math for highlight, underline, and strikeout annotations so bounds match text selections accurately.

### Changed
- App version bumped to 0.5.1 (`Cargo.toml`, `Info.plist`, `build_dmg.sh`).
- Hardened DMG staging and packaging pipeline with deep strict bundle verification.

## [0.5.0] - 2026-10-24

### Added
- **Liquid Glass UI polish**: fully translucent window (`BehindWindow` / `UnderWindowBackground`), custom `GlassTableRowView` accent capsules for row selection (`src/glass_row.rs`), modern Source List / Inset table styles.
- **Native empty states** (`src/empty_state.rs`) for the bookmarks and annotations inspectors.
- **Single-row `HoverCoordinator`** guaranteeing at most one hovered row in lists while scrolling.
- **Reliability & QA tooling**: hardened workspace index decoder (corrupt / truncated / oversized payload rejection), symlink-cycle–safe folder traversal, bounded log buffer, live debug console, `scripts/verify.sh` and `scripts/visual_qa.sh`.
- Test suite grown to **33 unit & regression tests**.

### Fixed
- Home screen action button spacing and stale hover highlights (Home, Workspace, Command Palette).
- Current-page indicator in the thumbnail sidebar.
- Bookmark / annotation empty-state layout and broken bookmark actions.
- Workspace sidebar overlapping the traffic-light buttons; truncation in deeply nested folders.
- White corners around the Command Palette.
- Highlight / underline / strikeout covering only the first line of multi-line selections.
- `Cmd+1…9` tab switching on AZERTY and other non-QWERTY layouts (now uses physical key codes).
- `Cmd+I` showing the wrong document identity / path.
- Save writing to the wrong (or no) destination; "Save As…" now migrates tab, title, recents and bookmarks.
- Intermittent missed clicks when switching tabs.
- Resolved `Cmd+Shift+H` menu shortcut collision: **Home Screen** now uses `Cmd+Option+H` (`⌥⌘H`), freeing `Cmd+Shift+H` (`⇧⌘H`) for **Highlight Selection**, added `Cmd+Shift+X` (`⇧⌘X`) for **Strikeout Selection**, and synchronized all Command Palette shortcut badges with the native AppKit menus.
- Home screen header divider spacing and sidebar group-row border artifact.

### Changed
- App version bumped to 0.5.0 (`Cargo.toml`, `Info.plist`).
- Formatted entire Rust codebase with `rustfmt` and refreshed all documentation screenshots in `docs/screenshots/`.

---

## [0.4.0] - 2026-10-12

### Added
- **Multi-Document Tabs (`src/tabs.rs`)**:
  - Safari/Finder-style top document tab bar.
  - Independent session state per tab: scroll position, zoom level, dirty state.
  - Native shortcuts: `Cmd+T` (new tab), `Cmd+W` (close tab), `Cmd+Shift+]` / `Cmd+Shift+[` (cycle tabs).
  - Graceful transition to Home screen upon closing the last tab.
- **Native PDF Annotations MVP (`src/annotations.rs`)**:
  - Full support for PDFKit native annotations: Highlight, Underline, Strikeout, Text Note / Comment.
  - Context menu actions for text selection: Copy Page Reference, Copy Markdown Quote, Search Selection.
  - Atomic safe document saving (`Cmd+S`, `Cmd+Shift+S`) with zero file truncation risk.
  - Invariant guard strictly prohibiting overwriting protected documents (e.g., in `MASTER`).
  - Bookmarks & Annotations inspector tab in document sidebar.
- **Apple Vision Neural OCR On-Demand (`src/ocr.rs`)**:
  - On-demand text extraction using Apple Neural Engine `VNRecognizeTextRequest`.
  - Background thread processing for responsive UI.
  - Liquid Glass result sheet displaying recognized characters, selectable preview, copy, and export.
- **Native Settings Window (`src/settings.rs`)**:
  - Dedicated macOS Preferences window with toolbar tabs: General, Reading, Workspace, Advanced.
  - Configurable persistent preferences for session restoration, default zoom, and privacy.

### Changed
- Refactored build target directory to `.nosync` to prevent iCloud Drive locking on macOS.
- Reduced compiler warnings by 94% across all modules.

---

## [0.3.0] - 2026-10-07

### Added
- **Liquid Glass Design Overhaul**:
  - Translucent sidebars utilizing `NSVisualEffectView` HUD and BehindWindow materials.
  - Responsive window sizing algorithm optimizing for 13-inch MacBook Air displays through high-resolution external monitors.
  - Automatic Dark/Light mode theme switching with system-reactive vibrancy.
- **Spotlight-Style Command Palette (`Cmd+P`)**:
  - Frameless floating modal panel with auto-focusing search field.
  - Prefix routing: `>` for commands, `/` for document search, `p:` for page jump, default for file search.
- **Navigation History**:
  - Deep page and document jump tracking (`Cmd+[`, `Cmd+]`).

---

## [0.2.0] - 2026-10-04

### Added
- **Workspace Explorer (`src/workspace.rs`)**:
  - Full-folder workspace browsing via hierarchical `NSOutlineView`.
  - Background asynchronous workspace indexing with fast binary cache serialization.
- **iCloud Drive Integration**:
  - Detection of ubiquitous files and eviction states.
  - Non-blocking download triggers via `NSFileManager.startDownloadingUbiquitousItemAtURL`.
  - Visual downloading badges and real-time state machine updates.

---

## [0.1.0] - 2026-10-02

### Added
- Initial proof of concept in pure Rust using `objc2` AppKit and PDFKit bindings.
- Basic continuous vertical PDF reading, zoom controls, and thumbnail sidebar.
