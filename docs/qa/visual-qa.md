# Iridescence PDF Reader — Visual Quality Assurance (Visual QA)

## Overview

Iridescence includes an integrated visual regression testing system (`VisualQA`) implemented in [`src/visual_qa.rs`](file:///Users/theodeville/Documents/OxiReader/src/visual_qa.rs) and scripted via [`scripts/visual_qa.sh`](file:///Users/theodeville/Documents/OxiReader/scripts/visual_qa.sh).

Visual QA drives the application through 19 distinct UI states, capturing native window snapshots into `/tmp/iridecence-visual-qa/<pass_name>/`.

---

## 1. Automated Visual QA States

The visual QA suite exercises:

| State Index | State Name | Description | Key Elements Verified |
|---|---|---|---|
| `01` | `home-initial-empty` | Fresh Home Screen launch | Logo, Open PDF / Open Folder buttons, Empty Recents list |
| `02` | `home-with-recents` | Home Screen with populated history | Recents list, metadata labels, context action buttons |
| `03` | `home-favorites-selected` | Favorites filter active | Filtered favorites view in home sidebar |
| `04` | `reader-standalone-doc` | Single PDF open in reader | Clean reading canvas, left sidebar collapsed |
| `05` | `reader-pages-sidebar` | Right sidebar in Pages mode | Native `PDFThumbnailView` with responsive thumbnail grid |
| `06` | `reader-outline-sidebar`| Right sidebar in Outline mode | Native `NSOutlineView` showing document table of contents |
| `07` | `reader-search-sidebar` | Right sidebar in Search mode | In-document search results list with snippets |
| `08` | `reader-workspace-mode`  | Workspace mode active | Three-panel layout (Left Explorer, Document, Right Sidebar) |
| `09` | `reader-tabs-multiple`   | Multiple documents open | Liquid Glass tab bar with active and inactive tab items |
| `10` | `palette-files`          | Command Palette (`Cmd+P`) | Spotlight floating panel, focused search field, file results |
| `11` | `palette-commands`       | Command Palette (`>`) | Action commands (`>Fit Width`, `>Toggle Sidebar`, etc.) |
| `12` | `palette-goto-page`      | Command Palette (`p:`) | Quick page navigation modal |
| `13` | `annotations-toolbar`    | Annotation tools visible | Highlight, Underline, Strikethrough, Notes buttons |
| `14` | `ocr-panel-active`       | OCR side-panel active | Vision text extraction results, export buttons |
| `15` | `settings-window`        | Settings preference window | General, Appearance, Shortcuts, Performance tabs |
| `16` | `debug-console`          | Debug diagnostics console | Real-time log streamer, filter controls |
| `17` | `dark-mode-reader`       | Reader in macOS Dark Mode | Liquid Glass materials adapting to dark appearance |
| `18` | `light-mode-reader`      | Reader in macOS Light Mode | Materials adapting to light appearance |
| `19` | `window-geometry-bounds` | Window resizing constraints | Clamped frame dimensions respecting `NSScreen.visibleFrame` |

---

## 2. Running Visual QA

To execute a test run and capture the screenshots:

```bash
./scripts/visual_qa.sh pass-reliability-01
```

All 19 high-resolution PNG captures will be deposited in `/tmp/iridecence-visual-qa/pass-reliability-01/`.
