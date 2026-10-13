# Performance Guidelines & Budgets

This directory contains performance budgets and methodology for Iridescence PDF Reader.

## Budgets Overview

Defined in [`budgets.toml`](file:///Users/theodeville/Documents/OxiReader/perf/budgets.toml):
- **App Launch to Interactive**: ≤ 350 ms
- **Standard Document Open**: ≤ 250 ms
- **Memory (Idle Home)**: ≤ 85 MB RSS
- **Memory (Single Document)**: ≤ 160 MB RSS
- **Log Buffer**: Hard capped at 2,000 entries (FIFO eviction)
- **Workspace Index Deserialization**: ≤ 50 ms with safety bounds against malicious / corrupted cache payloads

## Measurement & Profiling Tools
- **Instruments (macOS)**:
  - *Time Profiler*: Measure UI thread responsiveness. Zero work on main thread during indexation or file loading.
  - *Allocations & Leaks*: Confirm no retained Cocoa or Rust objects upon closing documents or switching tabs.
- **Visual QA**:
  - Run `./scripts/visual_qa.sh <pass-name>` to capture all 19 application states for layout and geometry verification.
- **Cargo Test Suite**:
  - `cargo test` executes the adversarial regression tests verifying bounds and cycle detection.
