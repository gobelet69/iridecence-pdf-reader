# Iridecence PDF Reader

<p align="center">
  <img src="docs/screenshots/reader-workspace-light.png" alt="Iridecence PDF Reader Workspace" width="850">
</p>

<p align="center">
  <b>A blazing-fast, lightweight, native macOS PDF reader engineered in pure Rust with AppKit & PDFKit.</b><br>
  Zero Electron. Zero WebKit bloat. Native Liquid Glass materials. Apple Vision OCR on demand.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS%2014.0%2B-blue?logo=apple" alt="macOS 14+">
  <img src="https://img.shields.io/badge/language-Rust%202024-orange?logo=rust" alt="Rust 2024">
  <img src="https://img.shields.io/badge/ui-Native%20AppKit%20%2B%20PDFKit-black" alt="AppKit & PDFKit">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License MIT">
</p>

---

## Overview

**Iridecence PDF Reader** is a high-performance document reader crafted specifically for macOS power users, researchers, and developers. By pairing Rust's memory safety with direct Objective-C bindings (`objc2`) to Apple's native **AppKit**, **PDFKit**, and **Vision** frameworks, Iridecence achieves instant cold startup (<250ms), minimal memory footprint (<60MB base idle RAM), and seamless macOS system integration.

---

## Key Features

### 🪟 Native Liquid Glass Design
- Translucent sidebar materials (`NSVisualEffectView` HUD/BehindWindow) that blend naturally with macOS Sonoma and Sequoia.
- Pixel-perfect typography and native split view dividers with persistent panel layouts.
- Dynamic responsive window geometry tailored for 13-inch MacBook Air displays through 6K Pro Display XDR screens.
- Automatic or manual Dark Mode and Light Mode switching (`Cmd+Shift+D`).

| Home Screen (Light) | Home Screen (Dark) |
|:---:|:---:|
| <img src="docs/screenshots/home-light.png" width="400"> | <img src="docs/screenshots/home-dark.png" width="400"> |

### 📑 Native Multi-Document Tabs
- Safari/Finder-inspired top tab bar (`Cmd+T`, `Cmd+W`).
- Isolated document sessions per tab: individual reading positions, zoom scales, undo histories, and dirty annotation states.
- Clean transitions: opening the first PDF transitions from Home to Document mode; closing the final tab gracefully returns to the Home screen.
- Keyboard navigation: `Cmd+Shift+]` / `Ctrl+Tab` for Next Tab, `Cmd+Shift+[` / `Ctrl+Shift+Tab` for Previous Tab.

<p align="center">
  <img src="docs/screenshots/tabs.png" alt="Multi-Document Tabs" width="800">
</p>

### 🖊️ Annotations & Safe Atomic Saving
- Native PDFKit annotations: **Highlight**, **Underline**, **Strikeout**, and **Text Notes / Comments**.
- Direct context menu integration on text selection: Copy, Copy with Page Reference, Copy as Markdown Quote, Search Selection, and Annotate.
- Safe atomic persistence (`Cmd+S`, `Cmd+Shift+S`): Writes to a temporary file, verifies integrity, and atomically renames.
- **Protected Directory Guard**: Strictly blocks write operations to configured read-only repositories (e.g. `MASTER`), prompting for "Save As..." instead.

<p align="center">
  <img src="docs/screenshots/annotations.png" alt="Native Annotations" width="800">
</p>

### 🔍 Apple Vision OCR On-Demand
- Integrated Apple Neural Engine text recognition (`VNRecognizeTextRequest`).
- Extract text directly from scanned documents or embedded images with high accuracy.
- Native Liquid Glass results sheet displaying recognized characters, text view with selectable font sizes, and one-click "Copy Text" / "Save TXT".

<p align="center">
  <img src="docs/screenshots/ocr-panel.png" alt="Apple Vision OCR" width="800">
</p>

### ⚡ Spotlight-Style Command Palette (`Cmd+P`)
- Floating, keyboard-first palette with instant auto-focus:
  - `>` : Command palette mode (toggle sidebar, fit width, zoom, OCR, settings).
  - `/` : Full-text search across the active PDF document.
  - `p:` or `:` : Direct page jump navigation.
  - *Default* : Fast fuzzy filtering across current workspace files and recent documents.

<p align="center">
  <img src="docs/screenshots/palette-command.png" alt="Command Palette" width="600">
</p>

### 📁 Workspace Mode & iCloud Drive
- Open entire folders with recursive background hierarchy loading (`Cmd+Shift+O`).
- Native iCloud status indicators (Local, CloudOnly, Downloading, Error) with automatic background hydration via `NSFileManager.startDownloadingUbiquitousItemAtURL`.
- Fast background workspace text indexing and real-time status reporting.

### ⚙️ Native Settings (`Cmd+,`)
- Dedicated macOS Preferences window organized by tabs: **General**, **Reading**, **Workspace**, and **Advanced**.
- Persistent user preferences via `NSUserDefaults`: restore sessions, default zoom/display modes, and privacy settings.

<p align="center">
  <img src="docs/screenshots/settings.png" alt="Settings Window" width="600">
</p>

---

## Keyboard Shortcuts

| Shortcut | Action |
|:---|:---|
| `Cmd + O` | Open PDF File |
| `Cmd + Shift + O` | Open Folder (Workspace Mode) |
| `Cmd + S` | Safe Save Document & Annotations |
| `Cmd + Shift + S` | Save As... |
| `Cmd + T` | New Document Tab |
| `Cmd + W` | Close Active Tab / Return to Home |
| `Cmd + Shift + ]` | Next Document Tab |
| `Cmd + Shift + [` | Previous Document Tab |
| `Cmd + P` | Quick Open / Command Palette |
| `Cmd + Shift + P` | Command Palette (Prefix `>`) |
| `Cmd + F` | Search in Current Document |
| `Cmd + L` | Go to Page |
| `Cmd + [` | Navigation History: Back |
| `Cmd + ]` | Navigation History: Forward |
| `Cmd + 1` | Sidebar: Pages (Thumbnails) |
| `Cmd + 2` | Sidebar: Document Outline |
| `Cmd + 3` | Sidebar: Full-Text Search |
| `Cmd + 4` | Sidebar: Bookmarks & Annotations |
| `Cmd + +` / `Cmd + -` | Zoom In / Zoom Out |
| `Cmd + 0` | Actual Size (100%) |
| `Cmd + 9` | Fit to Window |
| `Cmd + Option + F` | Toggle Zen / Focus Mode |
| `Cmd + Option + P` | Toggle Presentation Mode |
| `Cmd + Shift + D` | Toggle Dark / Light Appearance |
| `Cmd + ,` | Settings / Preferences |

---

## Getting Started

### Prerequisites
- macOS 14.0 (Sonoma) or macOS 15.0 (Sequoia) recommended.
- [Rust](https://rustup.rs/) (1.80+ or latest stable).
- Apple Xcode Command Line Tools (`xcode-select --install`).

### Build & Run

Clone the repository:
```bash
git clone https://github.com/theodeville/iridecence-pdf-reader.git
cd iridecence-pdf-reader
```

To build and run directly via Cargo:
```bash
cargo run --release
```

To build the self-contained macOS `.app` bundle:
```bash
./build_app.sh
open "Iridecence PDF Reader.app"
```

---

## Architecture & Design

Iridecence is structured around an event-driven Cocoa runloop paired with Rust concurrency primitives:

- **[`src/main.rs`](src/main.rs)**: AppKit application lifecycle, `NSApplicationDelegate`, window controller, toolbar coordination.
- **[`src/tabs.rs`](src/tabs.rs)**: Multi-document session manager, tab view controller, and isolated per-tab document state.
- **[`src/annotations.rs`](src/annotations.rs)**: Native PDFKit annotation mutations, safe atomic file serialization, and read-only protection guards.
- **[`src/ocr.rs`](src/ocr.rs)**: Integration with Apple's `VNRecognizeTextRequest` and background thread image dispatch.
- **[`src/palette.rs`](src/palette.rs)**: Spotlight-style floating `NSPanel` with prefix-based command and fuzzy search routing.
- **[`src/workspace_index.rs`](src/workspace_index.rs)**: Background thread file crawler, PDF text extractor, and fast binary cache.
- **[`src/settings.rs`](src/settings.rs)**: Tabbed native preferences window backed by `NSUserDefaults`.

For in-depth architectural diagrams and memory ownership models, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Performance

Measurable baselines verified on Apple Silicon (M-series):
- **Cold App Launch**: <220 ms
- **Small PDF Open (<5 MB)**: <15 ms
- **Large PDF Open (>100 MB)**: <65 ms (streaming page load)
- **Base Memory Footprint**: ~48 MB RAM
- **Binary Size**: ~8.4 MB (unstripped debug), ~3.2 MB (stripped release)

Detailed benchmark procedures and profiling methodologies are documented in [docs/PERFORMANCE.md](docs/PERFORMANCE.md).

---

## Contributing

Contributions, bug reports, and suggestions are welcome! Please review [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md) before opening a pull request.

---

## License

This project is licensed under the [MIT License](LICENSE).
