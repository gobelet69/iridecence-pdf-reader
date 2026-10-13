# Iridescence PDF Reader — macOS Native System Integration

## Overview

Iridescence is purpose-built for macOS, taking advantage of native Cocoa, AppKit, PDFKit, and CoreServices frameworks while adhering to Apple Human Interface Guidelines (HIG).

---

## 1. Window Geometry & Multi-Display Resilience

All window frames are managed by [`WindowGeometry`](file:///Users/theodeville/Documents/OxiReader/src/window_geometry.rs):
- **Visible Frame Bounds**: Dimensions are evaluated against `NSScreen.visibleFrame` rather than raw screen frames, preventing titlebars from slipping under the macOS menu bar or bottoms from being obscured by the Dock.
- **MacBook 13"/14" Baselines**: Default sizing targets ~1000x680 pt for reader mode, adapting down to 740x480 pt minimums on compact displays.
- **Display Reconfiguration Handling**: When an external monitor is disconnected, persisted window coordinates are sanitized through `sanitize_frame` to re-center the window onto the primary display.

---

## 2. Liquid Glass & Visual Materials

The user interface uses modern macOS visual effect materials (`NSVisualEffectView`):
- **Command Palette (`Cmd+P`)**: Uses `NSVisualEffectMaterial::HUDWindow` or `Popover` with `BehindWindow` blending mode, providing translucent Liquid Glass aesthetics.
- **Sidebars**: Native source-list visual materials with 1pt divider borders.
- **Tab Bar**: `NSVisualEffectMaterial::HeaderView` with automated dark/light mode switching.

---

## 3. iCloud Drive Integration

When browsing workspaces stored in iCloud Drive:
- **Dataless File Detection**: Files are queried for `NSURLUbiquitousItemDownloadingStatusKey` and `NSURLUbiquitousItemIsDownloadingKey`.
- **Badging & Live Progress**:
  - Dataless files are displayed with an inline cloud badge.
  - Downloading files show an animated indicator with current completion percentage.
  - Download triggers utilize `NSFileManager::startDownloadingUbiquitousItemAtURL:error:`.
- **Target Segregation**: To prevent iCloud sync from thrashing build outputs, all Cargo builds output to `/tmp/target` or `.nosync` directories.

---

## 4. Vision Framework OCR

Text extraction in non-searchable or scanned PDF documents utilizes Apple's native Vision framework:
- Uses `VNRecognizeTextRequest` with `VNRequestTextRecognitionLevelAccurate`.
- Runs asynchronously on background worker threads, posting progress notifications without blocking the main event run loop.
- Renders recognition results in a dedicated side-panel with copy and export facilities.
