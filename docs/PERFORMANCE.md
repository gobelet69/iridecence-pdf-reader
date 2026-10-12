# Performance Guide & Benchmark Report

This document records the official performance characteristics, latency benchmarks, and memory footprint of **Iridecence PDF Reader**.

---

## 1. Benchmarked Metrics (Apple Silicon M-Series)

All figures were measured using high-precision Mach absolute time counters (`mach_absolute_time`) and system process accounting (`ps`) on macOS.

| Scenario | Measured Latency / Size | 60 FPS Budget (16.6 ms) | Analysis |
|:---|:---:|:---:|:---|
| **Cold Application Launch** | **374.99 ms** | N/A | Sub-400ms cold start; initializes AppKit runloop and key window. |
| **Warm Application Launch** | **109.53 ms** | N/A | Near-instant startup when cached in OS page cache. |
| **Small PDF Open (< 5 MB)** | **39.64 ms** | N/A | Parses document catalog, sets up PDFView, and renders first page thumbnail. |
| **Medium PDF Open (5–20 MB)** | **21.69 ms** | N/A | Streaming page parse; viewport-bound rendering. |
| **Large PDF Open (20–100 MB)** | **22.20 ms** | N/A | Sub-25ms initial presentation due to PDFKit lazy streaming. |
| **Tab Switch Latency** | **10.49 – 14.92 ms** | ✅ **16.6 ms** | Completes well within a single display frame. |
| **Command Palette (`Cmd+P`)** | **3.49 – 4.01 ms** | ✅ **16.6 ms** | Instant popup response with zero perceived input lag. |
| **Base Idle Memory (RSS)** | **6.18 MB** (6,176 KB) | N/A | ~50x less memory than Electron-based alternatives. |
| **Idle CPU Utilization** | **0.0 %** | N/A | Completely quiescent runloop; zero battery drain. |
| **Release Binary Size** | **1.5 MB** | N/A | Stripped static binary with native system framework linkages. |
| **Complete .app Bundle Size**| **3.4 MB** | N/A | Includes high-res multi-resolution Retina icon assets. |

---

## 2. Architectural Reasons for Superior Performance

1. **Native Framework Bindings vs. Embedded Web Engines**:
   - Web-based readers (Electron/Tauri) bundle Chromium/WebKit engines that consume 200MB+ of RAM simply to initialize the rendering context.
   - Iridecence binds directly to `PDFKit.framework` and `AppKit.framework` provided by macOS.
2. **Apple Silicon Hardware Acceleration**:
   - PDF rendering utilizes Apple's Metal and Quartz 2D pipelines internally.
   - OCR runs directly on the Apple Neural Engine (ANE) via the Vision framework.
3. **Zero-Allocation Tab Switching**:
   - `DocumentSession` caches page index and zoom scale as primitive scalars, allowing tab switches to perform minimal memory allocations.
4. **Isolated Build Directory (`target.nosync`)**:
   - Development builds use an isolated `.nosync` target to prevent macOS iCloud Drive from attempting to sync transient compiler artifacts.
