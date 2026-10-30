# Iridescence PDF Reader Performance Fix Verification - 2026-10-30

## Outcome

**Status: WARN.** Every reproducible main-thread construction path identified by the 2026-10-27 audit now routes through the existing PDF worker, including history navigation and external-file reloads. Warm launch and launch-to-PDF callback improved materially. Idle Home RSS remains above the 85 MiB budget, and live iCloud plus Core Animation frame-time evidence remain unavailable without touching personal state.

## Safety and scope

- Baseline report: `qa/reports/performance/2026-10-27-iridescence-pdf-reader.md`.
- Source baseline: `19765dd828ce0f1523d8fe721601c8ad981eca47` on `main`.
- All PDF and workspace fixtures were synthetic and located under `/tmp`.
- No personal document was opened or modified. No content under `Documents/MASTER` was read or written.
- No release, tag, push, or publication was performed.

## Findings recovered from the baseline

| Priority | Finding | Reproduction from the audit | Result |
|---|---|---|---|
| P1 | First PDF open constructed `PDFDocument` on the main thread | Open the text, 1,200-page, or 144 MB raster fixture and measure synchronous construction | Fixed in the earlier audit changes; worker dispatch retained |
| P1 | Short PDF search synchronously scanned up to 500 pages | Search the 20-page fixture and measure the call | Fixed in the earlier audit changes; all non-empty queries use incremental PDFKit search |
| P1 | Home enumerated large directories and queried metadata on the main thread | Open a flat 10,000-entry Home directory | Fixed in the earlier audit changes; enumeration and metadata run on a generation-scoped worker |
| P1 | Workspace cancellation used a resettable boolean | Replace an active workspace indexing request after 50 ms | Fixed in the earlier audit changes; monotonic generations reject stale workers |
| P1 | Back/forward history constructed another PDF synchronously | Navigate history to an entry in a different PDF | **Fixed now**; page, mode, and history suppression travel with the worker request |
| P1 | External-file reload constructed the replacement PDF synchronously | Modify the active fixture, then reactivate the window | **Fixed now**; current page is restored after worker completion |
| P1 | Closing an inactive tab reinstalled the already-active PDF document | Open a second disposable PDF, close the first tab while PDFKit is rendering, then enter OCR/thumbnail states | **Fixed now**; inactive closure no longer detaches and reinstalls the live PDF document |
| P2 | Warm/cold launch missed the documented budget | Launch repeatedly after one cold run | Warm now passes; cold remains WARN |
| P2 | Idle Home RSS exceeded 85 MiB | Launch Home with isolated state and sample after 5 seconds | Still WARN |
| P2 | Scrolling, thumbnails, and switching lacked real frame traces | Scroll/render/switch the large fixtures | Still WARN; visual runtime exercised, but no Core Animation trace was captured |
| P2 | Live iCloud state transitions were not disposable | Trigger cloud-only to downloading to current | Still WARN; only deterministic synthetic state coverage was safe |
| P3 | `FileNode` worker ownership was suspected to retain indefinitely | Inspect directory-load callback ownership | Stale finding: `finishLoading` reconstructs the retained node with `Retained::from_raw`, balancing the worker retain |

## Before and after measurements

| Scenario | 2026-10-27 corrected baseline | 2026-10-30 result | Status |
|---|---:|---:|---|
| Warm launch to first visible window, 4 runs | 334-363 ms; median 354 ms | 263-291 ms; median 277 ms | **PASS**, median improved about 22% and is below 350 ms |
| First cold launch | 1,711 ms | 1,086 ms | **WARN**, improved about 37% but remains cache-sensitive |
| Launch to `pdfDidLoad:` callback, 5 runs | 306-342 ms | 247-273 ms; median 260 ms | **PASS** against the 600 ms large-document budget; launch-inclusive result is not a document-only 250 ms measurement |
| PDF construction dispatch | 0.020-0.031 ms for the existing worker path | History and external reload now call that same worker path | **PASS** by implementation equivalence and regression test; no separate UI signpost was added |
| Idle Home RSS after 5 seconds, 3 runs | 89.2-89.3 MiB | 88.0-98.5 MiB; median 91.4 MiB, 0.0% CPU | **WARN**, above the 85 MiB budget |
| `leaks` snapshot | 434 allocations / 21,536 bytes | 416 allocations / 19,968 bytes | **WARN**, about 7% fewer bytes; sampled roots shown were system AppIntents allocations, not a confirmed app leak |
| Visual/runtime state assertions | 116/116 in the prior visual audit | 116/116 after the final fixes | **PASS** |

The search, 10,000-entry Home, workspace-generation, and PDF thumbnail proxy numbers remain those recorded with the same synthetic workload in the baseline report because the final corrections did not modify those implementations. Their regression tests and runtime states were rerun.

## Corrections made in this pass

1. Added `PendingPdfPresentation` so asynchronous completions preserve content mode, target page, and history suppression while still rejecting obsolete documents.
2. Routed back/forward cross-document navigation through `start_pdf_load_in_background_with_presentation`.
3. Routed external-file refresh through the same worker and restored the current page after completion.
4. Avoided reinstalling an unchanged `PDFDocument` when an inactive tab closes; active-tab closure now clears the removed document before activating its replacement.
5. Added focused regression tests for async history presentation context and inactive-tab closure behavior.

## Verification

- `cargo fmt --check`: PASS.
- `cargo clippy`: PASS with 265 pre-existing style warnings; the CI configuration does not treat warnings as errors.
- `cargo check`: PASS.
- `cargo test`: **41 passed, 0 failed**.
- `CARGO_TARGET_DIR=/tmp/iridescence-pdf-reader-target ./scripts/verify.sh`: PASS, including release bundle construction.
- `git diff --check`: PASS.
- Real release binary launch and PDF callbacks: PASS with disposable fixtures.

## Remaining performance limitations

- Idle Home RSS remains above budget and varies by roughly 10 MiB between clean processes.
- Actual Core Animation frame-time traces for scrolling, thumbnail generation, and document switching were not captured.
- Live File Provider/iCloud download coordination was not exercised because no disposable domain was available.
- PDFKit emitted one non-fatal `Drawing a PDFPage when its PDFDocument is nil` diagnostic during the 116-state OCR transition; the full suite continued and passed, but this remains a P3 framework-lifecycle observation.

