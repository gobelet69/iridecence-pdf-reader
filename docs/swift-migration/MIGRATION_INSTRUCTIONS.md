# Iridescence PDF Reader — Native Swift Build, Test & Migration Instructions

## 1. Prerequisites

- **macOS**: macOS 14.0+ (optimized for macOS 27 Liquid Glass on Apple Silicon `arm64`)
- **Xcode**: Xcode 16+ / Xcode 27.1 (`swift-driver version 1.150.2`, Swift 6.0 / 6.4)
- **External Dependencies**: **None** (100% Apple native frameworks: `AppKit`, `PDFKit`, `Vision`, `UniformTypeIdentifiers`, `Foundation`)

---

## 2. Building & Packaging the Application

### Option A — Automated Build & App Bundle Packaging (`./build_app.sh`)
```bash
./build_app.sh
```
This compiles the `Release` target via `xcodebuild` into `/tmp/xcode-build/Release/Iridescence` and packages the signed macOS bundle at:
```text
Iridecence PDF Reader.app
```

To launch the packaged app:
```bash
open "Iridecence PDF Reader.app"
```

### Option B — Opening in Xcode IDE
```bash
open Iridescence.xcodeproj
```
Select the **Iridescence** scheme (`My Mac`) and press `Cmd+R` to build and run with full LLDB debugging and Instruments profiling support.

### Option C — Swift Package Manager CLI (`Package.swift`)
Because macOS iCloud Desktop & Documents sync can interfere with local `.build` directories inside `~/Documents`, always pass `--scratch-path /tmp/swift-build`:
```bash
swift build --scratch-path /tmp/swift-build
```

---

## 3. Running Automated Tests & Verification

### Full Verification Gate (`./scripts/verify.sh`)
```bash
./scripts/verify.sh
```
This script verifies:
1. Zero legacy Rust source files (`Cargo.toml` or `src/`) remain in the repository.
2. `swift build --scratch-path /tmp/swift-build` compiles cleanly with 0 errors.
3. `swift test --scratch-path /tmp/swift-build` passes all 15 unit and integration test suites.
4. `./build_app.sh` builds and packages `Iridecence PDF Reader.app` via `xcodebuild`.

### Automated 22-State Visual QA Suite (`./scripts/visual_qa.sh`)
```bash
./scripts/visual_qa.sh /tmp/iridescence-swift-visual-qa
```
This launches the compiled application in deterministic Visual QA mode (`IRIDESCENCE_SWIFT_VISUAL_QA=1`) and captures 25 high-DPI screenshots across Light Mode, Dark Mode, Home Screen, Standalone Reader, Workspace Mode, Pages/Outline/Search/Bookmarks/Annotations inspectors, Multi-Document Tabs, Command Palette (`Cmd+P`, `>`, `//`), Document Info, Vision OCR HUD & Results, Settings, Error Alert, Debug Console, and Fullscreen Presentation recovery.

---

## 4. User Data & Persistence Compatibility

The Swift rewrite preserves 100% compatibility with user data created by previous versions of Iridescence PDF Reader:
- **Preferences & Recent Documents**: Stored in `UserDefaults` (`com.iridecence.pdfreader`) using identical keys (`IridescenceRecentFiles`, `IridescenceRecentWorkspaces`, `IridescenceFavoriteFiles`, `IridescencePagePositions`, `IridescenceDisplayMode`, `IridescenceHighlightColor`, etc.).
- **Bookmarks (`bookmarks.tsv`)**: Stored at `~/Library/Application Support/Iridescence/bookmarks.tsv` using the identical 4-column TSV schema (`pdf_path \t page_index \t timestamp \t label`).
- **Full-Text Search Index (`IRIDX1`)**: Stored at `/tmp/iridescence_workspace_index.bin` using the identical little-endian `IRIDX1` binary format with strict defensive deserialization bounds.
- **Safety Invariant**: Saving directly into `/Users/theodeville/Documents/MASTER` remains strictly prohibited and enforced by `AnnotationsManager.isMasterProtectedPath(_:)`.
