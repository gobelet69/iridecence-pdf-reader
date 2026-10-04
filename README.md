# OxiReader

A lightweight, extremely fast native macOS PDF reader written in Rust.

## Features

- Native macOS UI using AppKit
- High-performance PDF rendering using PDFKit
- Extremely low memory footprint and fast startup time
- No heavy web engines (No Electron)
- Dark mode and Retina display support

## Architecture

OxiReader is built entirely in Rust, using `objc2` bindings to access native macOS frameworks:
- `AppKit` for the user interface, windows, and toolbars.
- `PDFKit` for PDF rendering, scrolling, and navigation.

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed design decisions.
See [ROADMAP.md](ROADMAP.md) for the development roadmap.

## Requirements

- macOS 11.0 or later
- Rust 1.80+

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```
