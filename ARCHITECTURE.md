# OxiReader Architecture

## Overview

OxiReader is a modern, lightweight, open-source PDF reader for macOS, written in Rust. It prioritizes extremely fast startup, low CPU/memory consumption, and a native macOS user experience.

## Technology Stack Decision

After evaluating several options (Slint, egui, Tauri, AppKit via Rust bindings, PDFium, PDFKit), we have decided on the following stack:

- **Language:** Rust (for performance, safety, and modern tooling).
- **UI Framework:** `AppKit` via `objc2-app-kit`. 
- **PDF Engine:** `PDFKit` via `objc2-pdf-kit`.

### Why AppKit & PDFKit over Slint/egui/PDFium?

1. **Native Integration:** Using `AppKit` directly ensures 100% native macOS behavior, standard keyboard shortcuts, accessibility, and correct typography. Slint and egui draw their own controls which, while fast, do not look or behave exactly like macOS system components.
2. **Minimal Binary Size:** `PDFKit` is bundled with macOS. If we used `pdfium-render`, we would need to package and distribute the `libpdfium.dylib` (~10-20MB), which balloons the binary size.
3. **Built-in PDF Features:** `PDFKit` (`PDFView`) provides highly optimized native scrolling, zooming, selection, text search, and thumbnails out-of-the-box. Re-implementing these features reliably in egui/Slint using `pdfium` is complex and prone to edge-case bugs (e.g., trackpad rubber-banding, complex layout issues).
4. **Memory and CPU Efficiency:** Apple has heavily optimized `PDFKit` for macOS (it's what Preview uses). It uses native memory management and lazy page rendering at the OS level.
5. **No Electron/Web:** This fulfills the absolute requirement to avoid web technologies. 

## Application Architecture

The application is structured into the following conceptual modules, mapped onto the native AppKit MVC model:

### `app` (Application Lifecycle)
- Bootstraps the application via `NSApplication`.
- Sets up the `NSApplicationDelegate`.
- Handles main menu creation and macOS-specific lifecycle events (Open With, dragged files).

### `ui` (User Interface)
- `window_controller`: Manages the main `NSWindow`.
- `toolbar`: Sets up the native `NSToolbar` with essential actions.
- `sidebar`: Manages the thumbnail navigation panel (via `PDFThumbnailView`).

### `document` (Document Management)
- Wraps `PDFDocument` (from `PDFKit`).
- Manages file URLs, opening logic, and basic metadata extraction.
- Interacts with `NSOpenPanel` for native file picking.

### `renderer` & `navigation`
- Primarily handled by `PDFView`.
- We configure `PDFView` to handle continuous vertical scrolling, zoom levels, and single-page viewing modes.
- `PDFView` handles lazy rendering and caching automatically.

## Extensibility

To ensure the architecture remains extensible (e.g., adding OCR, Annotations later):
- We keep the Rust representations of documents distinct from the view layer.
- Future components like a custom `SearchService` or `OCRService` (possibly using Apple's Vision framework via `objc2-vision` in the future) can run on background threads in Rust and send results back to the main thread via Grand Central Dispatch or Rust mpsc channels.

## Roadmap Alignment

By leveraging `PDFKit`, we drastically reduce the implementation time for Phase 3 (Usability) and Phase 4 (Optimization), allowing us to deliver a highly polished MVP much faster, while strictly adhering to the design principles.
