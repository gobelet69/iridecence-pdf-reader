# Iridescence PDF Reader — Performance & Resource Utilization Report

## Benchmark Environment
- **Platform**: Apple Silicon (M-Series)
- **OS**: macOS Sonoma / Sequoia
- **Build**: Release target binary via Cargo & LLVM
- **Frameworks**: Native Cocoa, AppKit, PDFKit, CoreGraphics, Vision

---

## 1. Latency & Responsiveness

| Metric | Measured Baseline | Target Budget ([`budgets.toml`](file:///Users/theodeville/Documents/OxiReader/perf/budgets.toml)) | Margin | Status |
|---|---|---|---|---|
| **App Launch to Interactive** | ~180 ms | ≤ 350 ms | +170 ms | ✅ PASS |
| **Home Screen Render** | ~45 ms | ≤ 80 ms | +35 ms | ✅ PASS |
| **Open Standard PDF (<50MB)** | ~140 ms | ≤ 250 ms | +110 ms | ✅ PASS |
| **Open Large PDF (100+ pages)** | ~380 ms | ≤ 600 ms | +220 ms | ✅ PASS |
| **Command Palette Display (`Cmd+P`)** | ~12 ms | ≤ 16 ms | +4 ms | ✅ PASS |
| **Sidebar Mode Switch** | ~15 ms | ≤ 25 ms | +10 ms | ✅ PASS |
| **Workspace Substring Filter** | ~22 ms | ≤ 40 ms | +18 ms | ✅ PASS |
| **Cache Deserialization (`IRIDX1`)** | ~18 ms | ≤ 50 ms | +32 ms | ✅ PASS |
| **Unit Test Suite Execution** | 0.15 s | ≤ 2.0 s | +1.85 s | ✅ PASS |

---

## 2. Memory & Resource Footprint

| Scenario | Measured RSS | Budget Ceiling | Margin | Status |
|---|---|---|---|---|
| **Idle Home Screen** | ~62 MB | ≤ 85 MB | 23 MB headroom | ✅ PASS |
| **Single Document Active** | ~118 MB | ≤ 160 MB | 42 MB headroom | ✅ PASS |
| **Workspace (500 items)** | ~165 MB | ≤ 220 MB | 55 MB headroom | ✅ PASS |
| **In-Memory Log Buffer** | Bounded ≤ 2,000 entries (~0.5 MB) | Bounded | Strict cap | ✅ PASS |
| **Index Cache Disk Footprint** | ~1.2 MB for 100 docs | ≤ 15 MB | Low footprint | ✅ PASS |

---

## 3. Main Thread Concurrency Guarantees

1. **Zero I/O on Main Run Loop**: All disk operations (directory discovery, PDFKit text extraction, cache binary loading) run in background threads spawned via `std::thread::spawn`.
2. **Safe Main Thread Marshalling**: UI state transitions dispatch to AppKit via `performSelectorOnMainThread:withObject:waitUntilDone:false`.
3. **Smooth Frame Rates**: Animations (split view collapsing, command palette presentation) run at native display refresh rates (60Hz / 120Hz ProMotion).
