# Iridescence PDF Reader — Final Performance Remediation — 2026-10-30

## Outcome

**Status: WARN.** All reproducible application defects from the performance audit are fixed and verified. Idle Home memory now passes the documented RSS budget. The remaining warnings are external evidence limits: Xcode 27 did not finalize an exportable Animation Hitches trace, and the disposable File Provider upload did not complete within 120 seconds.

No release, tag, push, or publication was performed. All PDFs were disposable fixtures under `/tmp`; no personal document or `Documents/MASTER` content was opened or modified.

## Corrected findings

| Priority | Finding | Correction | Verification |
|---|---|---|---|
| P1 | PDF construction on main-thread open/history/reload paths | All paths use the existing PDF worker and reject obsolete completions | Regression tests and 116-state runtime suite |
| P1 | Synchronous short-document search | All non-empty queries use incremental PDFKit search | Regression test and search states 31–45 |
| P1 | Home directory enumeration on the main thread | Generation-scoped worker performs enumeration and metadata I/O | 10,000-entry baseline plus regression tests |
| P1 | Workspace cancellation race | Monotonic generations reject stale workers | Regression test |
| P1 | Hover tracking raised unrecognized `mouseEntered:` exceptions | Home, Workspace, and Palette tracking owners implement enter/move/exit | Selector regression test and real mouse movement in the full suite |
| P2 | Detached PDFPage diagnostic during OCR | Keep at most four recently closed documents alive for an individual 60-second PDFKit completion window | Exact OCR transition rerun: diagnostic absent; 116/116 states pass |
| P2 | Idle Home RSS over 85 MiB | Rebuilt final binary and repeated isolated-state measurement | 68,128 / 68,368 / 69,600 KiB; median 68,368 KiB (66.8 MiB), 0.0% CPU |

The deferred-document queue is bounded to four entries. Each entry carries its own expiry deadline, so an older timer cannot release a more recently closed document early.

## Before / after

| Metric | Before | Final | Result |
|---|---:|---:|---|
| Warm first window median | 354 ms | 277 ms | PASS, about 22% faster |
| First cold window | 1,711 ms | 1,086 ms | Improved; cache-sensitive |
| Launch to PDF callback median | 306–342 ms | 260 ms | PASS against 600 ms large-document budget |
| Idle Home RSS, 3 runs | 89.2–98.5 MiB | 66.5–68.0 MiB; median 66.8 MiB | PASS against 85 MiB |
| Idle physical footprint | Not recorded consistently | 29 MiB; peak 30 MiB | Informational PASS |
| Visual/runtime assertions | 116/116, 115 unique | 116/116, 116 unique | PASS |
| PDFKit detached-page diagnostics | 1 during OCR | 0 in exact rerun | PASS |

## Remaining evidence limits

- **Core Animation:** three `xctrace` Animation Hitches attempts exercised search, pages, thumbnails, and document switching. Xcode 27.1 remained blocked in `com.apple.dt.tap` after target exit and produced trace packages without an exportable template. The incomplete temporary traces were deleted; no frame-time claim is made.
- **iCloud:** the disposable PDF was recognized as an ubiquitous File Provider item and coordinated reads matched byte-for-byte. A strict second attempt required upload completion before eviction, but `ubiquitousItemIsUploaded` remained false for 120 seconds. The fixture was removed. Cloud-only → downloading → current remains WARN rather than a false PASS.

## Verification

- `cargo fmt --check`: PASS.
- `cargo check`: PASS.
- `cargo test`: **43 passed, 0 failed**.
- `cargo clippy --all-targets`: PASS; historical style warnings remain non-fatal.
- `CARGO_TARGET_DIR=/tmp/iridecence-final-verify-target ./scripts/verify.sh`: PASS.
- `git diff --check`: PASS.
- Release bundle and disposable DMG construction: PASS.

