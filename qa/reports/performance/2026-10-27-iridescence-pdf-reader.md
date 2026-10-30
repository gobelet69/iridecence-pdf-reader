# Iridescence PDF Reader Performance Audit — 2026-10-27

## Outcome

**Overall: WARN.** Confirmed main-thread stalls in PDF loading, short-document search, and large Home directory enumeration were removed. Workspace-index cancellation is now generation-scoped. The application still misses the documented launch and idle-RSS budgets on this machine, and live iCloud plus frame-timing proof remains unavailable without touching personal state.

## Environment and safety

- Apple M2, 8 GB RAM, macOS 27.0 (26A428), arm64.
- Source: `gobelet69/iridecence-pdf-reader`, baseline `19765dd` (`main`, v0.5.0).
- Release builds used `/tmp/iridescence-pdf-reader-audit-target` or `/tmp/target`.
- All fixtures lived under `/tmp/iridescence-perf-audit.Ar8AAb` and contained no personal data.
- Fixtures: 20-page/28 KB text PDF, 1,200-page/1.1 MB text PDF, 32-page/144 MB raster PDF, nested 5,000-file workspace, and flat 10,000-file workspace.
- `/Users/theodeville/Documents/MASTER` was neither read nor modified. No release, tag, push, or publication was performed.

## Before / after evidence

| Scenario | Baseline | Corrected result | Status |
|---|---:|---:|---|
| Warm launch to first window (4 runs) | 317–416 ms; median 341 ms | 334–363 ms; median 354 ms | **WARN** — variability overlaps, but median is 4 ms above the 350 ms budget |
| First cold launch in each run set | 3,655 ms | 1,711 ms | **WARN** — cache-sensitive and far above budget |
| Small-PDF search call (20 pages) | synchronous completion 25.9–31.9 ms | incremental dispatch 0.041–0.060 ms | **PASS** |
| Large-PDF search call (1,200 pages) | synchronous completion 1,453–1,519 ms; app already used async above 500 pages | incremental dispatch 0.045–0.651 ms | **PASS** |
| Flat 10,000-entry Home directory | main-thread enumeration 49.8–56.2 ms warm; 438 ms cold | worker dispatch 0.015–0.064 ms; completion 40.7–82.0 ms | **PASS** |
| PDF construction dispatch | synchronous first-open PDFKit work 48–207 ms across fixtures | worker dispatch 0.020–0.031 ms | **PASS** |
| Actual launch to `pdfDidLoad:` callback | not reliably captured before correction | 306–342 ms across all three fixtures | **PASS** against 600 ms large-PDF budget; launch-inclusive small PDF exceeds the 250 ms document-only budget and is not directly comparable |
| Idle Home RSS after 5 seconds | previous documentation claimed ~62 MB; not reproducible in this audit | 89.2–89.3 MB, 0.0% CPU | **WARN** — exceeds 85 MB budget by ~5% |
| Rapid workspace-index replacement | boolean cancellation could be reset by an older worker after 50 ms | monotonic generations invalidate every older worker | **PASS** — deterministic regression test |

## PDFKit rendering, thumbnails, and scrolling proxy

Native PDFKit rendering was retained. Forty distributed page thumbnails were measured per document where possible:

- 1,200-page text PDF: p95 0.51–0.56 ms.
- 144 MB raster PDF: warm p95 6.99–7.39 ms; first cold pass 26.85 ms.
- 20-page text PDF: p95 9.50–12.64 ms.

**Status: WARN.** Warm rendering is within a 16.6 ms frame, but the cold raster pass is not. These are page-render/thumbnail proxies, not Core Animation frame-time captures during live scrolling. The native `PDFView`/`PDFThumbnailView` architecture was left unchanged.

The Home list no longer calls `metadata()` while constructing visible rows. File sizes are collected during background enumeration, removing iCloud/file-system I/O from the scrolling cell path.

## Corrections

1. Local URL opens and post-iCloud-download opens now construct `PDFDocument` on a worker and dispatch only presentation to the main thread. Existing pending-path validation discards obsolete completions.
2. PDF search now always uses PDFKit incremental search with cancellation instead of synchronously scanning documents up to 500 pages.
3. Local, iCloud, and workspace Home directories enumerate on workers. Monotonic generations prevent stale results from replacing newer navigation.
4. Home row file sizes are prefetched off the UI path, eliminating per-cell metadata I/O during scrolling.
5. Workspace indexing uses monotonic generations rather than a shared boolean whose reset could reactivate stale workers.

An attempted allocation-free ASCII workspace-search scan was **not shipped**: on a 50,000-page disposable corpus it took 91–103 ms versus 30–32 ms for the existing lowercase implementation. The current implementation was restored rather than accepting a measured regression.

## Memory and lifecycle

- Idle Home: 89.2–89.3 MB RSS, 0.0% CPU after five seconds.
- 144 MB PDF process snapshot: ~93.5 MB RSS after the document callback; PDF rendering was still active at the early sample.
- `leaks` snapshot: 434 allocations / 21,536 bytes reported leaked. The snapshot cannot distinguish app-owned objects from framework lifetime allocations and is therefore not a confirmed app leak.
- Existing `FileNode::load_children` keeps an explicit retain until process exit because the current callback does not release it. This remains an open memory-management finding; it was not changed without a reliable lifecycle regression test.

**Status: WARN.** No unbounded Rust collection regression was found, but the RSS budget and worker-retain concern remain open.

## iCloud, downloads, and concurrency

- Directory metadata collection and download polling run off the main thread.
- `ACTIVE_DOWNLOADS` still deduplicates requests.
- PDF construction after download completion now also runs off the main thread.
- Index replacement and Home directory results now reject stale generations.

**Status: WARN.** No disposable iCloud File Provider domain was available. Triggering real iCloud transitions would have touched personal state, so cloud-only → downloading → current behavior was reviewed in code but not live-validated.

## Document switching and application launch

- Tab state and stale-load tests pass, and PDF parsing no longer blocks launch/open dispatch.
- Actual tab-switch frame latency was not captured; switching still refreshes outline and bookmark UI synchronously after `PDFView.setDocument`.
- Back/forward history navigation and external-file reload still construct `PDFDocument` synchronously so they can preserve page/history state. They remain confirmed main-thread risks outside the corrected open/download paths.
- Warm launch is near the stated 350 ms budget but did not consistently meet it. Cold launch remains substantially above budget.

**Status: WARN.** Functional regressions are covered, but document-switch frame timing needs a future Instruments/Core Animation run.

## Verification

- `cargo test --release`: **38 passed, 0 failed**.
- `CARGO_TARGET_DIR=/tmp/iridescence-pdf-reader-audit-target ./scripts/verify.sh`: `cargo check`, 38 tests, and `.app` packaging completed successfully.
- `git diff --check`: clean.
- Release binary actual launch and PDF callback were exercised with all three disposable PDFs.

## Open blockers / follow-up

- **WARN:** capture actual Core Animation frame times for scrolling, thumbnails, and document switching.
- **WARN:** investigate warm launch median (~354 ms) and cold launch variability.
- **WARN:** reduce idle Home RSS below 85 MB and isolate the 21.5 KB `leaks` snapshot.
- **WARN:** add a disposable File Provider/iCloud test domain before claiming end-to-end iCloud performance.
- **WARN:** replace or balance the `FileNode` worker retain with a lifecycle-tested callback ownership model.
- **WARN:** move history navigation and external-file reload parsing to the worker path with an explicit pending-page/history context.
