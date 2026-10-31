# Iridescence PDF Reader — macOS Design & Technical Research Report

## 1. Executive Summary

Before architecting the native Swift rewrite of **Iridescence PDF Reader**, we conducted a thorough review of Apple's latest platform documentation (macOS 26/27 Liquid Glass design system, AppKit, SwiftUI, PDFKit, Swift 6 Concurrency, File Coordination / iCloud APIs) and analyzed best-in-class macOS productivity and document applications (**Apple Preview**, **Finder**, **Safari**, **Raycast**, **PDF Expert**, **Highlights**, **Skim**, **Things 3**, and **Bear**).

Our objective is to build an elite, lightweight, native macOS PDF reader and workspace manager that feels indistinguishable from a first-party Apple pro application while running effortlessly on 8 GB Apple Silicon Macs.

---

## 2. Official Apple Documentation & Platform Guidance

### 2.1 Liquid Glass & macOS Design Principles
- **References**:
  - [Apple Human Interface Guidelines — Materials & Liquid Glass](https://developer.apple.com/design/human-interface-guidelines/)
  - [Liquid Glass Technology Overview](https://developer.apple.com/documentation/technologyoverviews/liquid-glass)
  - [WWDC25 Session 310 — Build an AppKit app with the new design](https://developer.apple.com/videos/play/wwdc2025/310/)
- **Key Architectural Takeaways**:
  1. **Functional vs. Content Layer Separation**: Liquid Glass belongs strictly in the *functional navigation layer* (sidebars, unified toolbars, tab bars, floating command palettes, and transient HUDs). The *content layer* (the PDF document canvas) must remain optically pure and high-contrast for long-form reading.
  2. **Genuine Native Materials**: On AppKit, sidebars and window backgrounds should use `NSVisualEffectView` with `.behindWindow` blending (`NSVisualEffectView.Material.sidebar` for sidebars/inspectors, `.underWindowBackground` for root window containers, and `.popover` / `.hudWindow` for floating panels).
  3. **Concentric Rounded Geometry**: Floating panels (`16pt` corner radius), cards (`10pt` corner radius), and interactive table row pills (`6.5pt` corner radius) must maintain concentric visual harmony with macOS window geometry.
  4. **Accessibility & Appearance Adaptation**: Every surface and label must adapt dynamically to **Light Mode**, **Dark Mode**, **Reduce Transparency** (`NSWorkspace.shared.accessibilityDisplayShouldReduceTransparency`), **Increase Contrast** (`accessibilityDisplayShouldIncreaseContrast`), and **Reduce Motion** (`accessibilityDisplayShouldReduceMotion`). Semantic colors (`NSColor.labelColor`, `.secondaryLabelColor`, `.tertiaryLabelColor`, `.separatorColor`, `.controlAccentColor`) must be used exclusively instead of static RGB literals.

### 2.2 AppKit vs. SwiftUI Hybrid Strategy
- **References**:
  - [AppKit Documentation](https://developer.apple.com/documentation/appkit/)
  - [SwiftUI Documentation](https://developer.apple.com/documentation/swiftui/)
- **Key Architectural Takeaways**:
  - While SwiftUI excels at self-contained declarative layouts, **AppKit** provides superior control, lower memory overhead, and deterministic behavior for:
    - **Window & Titlebar Integration**: `NSWindow` with `.fullSizeContentView`, transparent titlebars, traffic-light clearance, and unified `NSToolbar`.
    - **3-Pane Split Views**: `NSSplitViewController` with `.sidebarWithViewController` and `.inspectorWithViewController` split items, holding priorities, and smooth animated collapse.
    - **Large Outline & Table Trees**: `NSOutlineView` and `NSTableView` with cell reuse, custom `NSTableRowView` (`GlassTableRowView`), single-click actions, and deterministic keyboard responder chains.
    - **Floating Spotlight Panels**: Borderless `NSPanel` (`FloatingPalettePanel`) with instant first-responder focus, dynamic height resizing (`48pt` collapsed to `420pt` expanded), and zero frame-lag keyboard navigation.
  - SwiftUI (`NSHostingView`) can be used selectively or paired with AppKit where appropriate, ensuring zero excessive view invalidation during rapid PDF scrolling or resize events.

### 2.3 PDFKit & Vision Framework Best Practices
- **References**:
  - [PDFKit Documentation](https://developer.apple.com/documentation/pdfkit/)
  - [Vision Framework — VNRecognizeTextRequest](https://developer.apple.com/documentation/vision/vnrecognizetextrequest)
- **Key Architectural Takeaways**:
  1. **Native `PDFView` & `PDFThumbnailView`**: `PDFView` handles hardware-accelerated tiled page rendering, pinch-to-zoom, continuous scrolling, text selection, and Find highlighting natively.
  2. **Multi-Line Markup Annotations**: When highlighting, underlining, or striking out text spanning multiple lines, creating a single bounding-box annotation creates an ugly giant rectangle across margins. Using `selection.selectionsByLine()` to compute per-line `quadrilateralPoints` (relative to the union bounds origin) produces crisp, typography-accurate markup identical to Apple Preview.
  3. **On-Demand Vision OCR**: Running `VNRecognizeTextRequest` (`.accurate` recognition level with `usesLanguageCorrection = true`) off the main thread on a 2x-rendered `CGImage` of the current `PDFPage` provides instantaneous, high-accuracy text extraction for scanned PDFs without bloating memory across the entire document.

### 2.4 Swift Concurrency & Resource Bounds for 8 GB Macs
- **References**:
  - [Swift Concurrency Documentation](https://developer.apple.com/documentation/swift/concurrency)
- **Key Architectural Takeaways**:
  1. **`@MainActor` Isolation**: All AppKit view controllers, windows, delegates, and UI state coordinators are isolated to `@MainActor`, preventing data races at compile time.
  2. **Bounded Caches & Lazy Loading**:
     - Directory trees load children lazily and enforce strict recursion depth (`<= 25`) and item count (`<= 50,000`) bounds with canonical symlink cycle detection.
     - Workspace full-text index (`IRIDX1`) extracts page text in a background task and releases intermediate `PDFDocument` instances immediately after indexing each file (`autoreleasepool`), keeping peak memory usage low on 8 GB systems.

---

## 3. Competitive & Reference Application Analysis

| Reference App | Studied Pattern | How Iridescence Adopts & Improves It |
|---|---|---|
| **Apple Preview** | Clean unified toolbar, multi-line `quadrilateralPoints` text markup, lightweight memory footprint. | Adopts Preview's uncluttered reading canvas and exact multi-line quad-point annotations, while adding a full Workspace Explorer, multi-document glass tab bar, and Command Palette. |
| **Apple Finder** | Source-list sidebar (`NSTableView.Style.sourceList`), iCloud status badges (`icloud.and.arrow.down`), breadcrumb/path context. | Adopts Finder's source-list geometry, SF Symbol hierarchy, and inline iCloud status/download indicators in both the Home Screen and Workspace Explorer. |
| **Safari** | Compact, translucent tab pills with close buttons, overflow menu, and `Cmd+1..9` / `Cmd+Shift+T` shortcuts. | Implements a custom Liquid Glass tab bar above the PDF viewport with dirty-state dot (`●`), overflow pill (`+N ▾`), right-click tab actions, and physical keycode `Cmd+1..9` support (works on both QWERTY and French AZERTY keyboards). |
| **Raycast / Spotlight** | Instant borderless floating panel (`Cmd+P`), prefix routing (`>`, `/`, `//`, `@`, `#`, `p:`), fuzzy match scoring, bottom keyboard shortcut hint bar. | Rebuilds the Command Palette as a native `NSPanel` with `16pt` squircle mask, `48pt` collapsed height, smart prefix modes, cursor placement without selection highlight artifacts, and a `26pt` footer hint bar. |
| **PDF Expert / Highlights** | Unified right-hand inspector combining Thumbnails, Outline, Search, Bookmarks, and Annotations with Markdown export. | Provides a 5-segment right inspector (`Pages`, `Outline`, `Search`, `Bookmarks`, `Annotations`) with instant one-click navigation and clean Markdown note/annotation export (`Cmd+Shift+E`). |
| **Things 3 / Bear** | Calm typography, generous whitespace, subtle hover pills, clear empty states with actionable guidance. | Uses semantic typography hierarchy, single-row `HoverCoordinator` tracking, and informative `EmptyStateView` components across every panel. |
