# Iridescence PDF Reader Visual Fix Verification - 2026-10-30

## Outcome

**Status: PASS for the requested visual states; WARN overall because one P3 evidence duplicate and one non-fatal PDFKit diagnostic remain.** The actual release application completed the deterministic 116-state suite with **116/116 invariant assertions passing**. The native AppKit/PDFKit Liquid Glass presentation and the subtle iridescence identity were preserved.

## Findings recovered from the visual audit

| Priority | Historical problem / reproduction | Final result |
|---|---|---|
| P1 | Multiple hover states after scrolling Home or workspace lists | PASS; single-hover invariant and states 111/113 pass |
| P1 | Content under macOS traffic lights or incorrect window sizing | PASS across Home, reader, near-max, short, and compact windows |
| P1 | iCloud completion reopened the wrong document identity | PASS; canonical public/private path equivalence regression test passes |
| P1 | Async visual QA switched documents before PDF completion | PASS; the production loader is awaited and obsolete completions are discarded |
| P1 | Closing an inactive tab could detach live PDFKit pages during rendering | **Fixed now**; inactive closure does not reinstall the active document |
| P2 | Excessively grey backgrounds and weak Liquid Glass composition | PASS in light/dark and real WindowServer backdrop captures |
| P2 | Command Palette transparency, corners, and result layout | PASS in states 56-67 |
| P2 | Left/right sidebar resizing and both-sidebars layout | PASS, including 180/360 pt limits and compact auto-collapse |
| P2 | PDF tabs, dirty state, active identity, long names, and overflow | PASS in states 50-55 |
| P2 | Search results and PDF thumbnails | PASS in states 29-49 |
| P2 | iCloud unavailable/downloading/current visual states | PASS in deterministic synthetic states 26-28 |
| P3 | Highlight and strikeout evidence images are identical | Still reproducible; annotation counts/types pass, but PDFKit produces the same raster for states 82 and 84 in this fixture |

## Final runtime capture

- Actual release bundle launched with `IRIDESCENCE_QA_FIX_SUITE=1` and isolated Home state.
- 116 screenshots captured.
- Invariant assertions: **116/116 PASS**.
- Unique screenshot hashes: **115/116**.
- Duplicate groups: one (`82-annotation-highlight.png` = `84-annotation-strikeout.png`).
- The final JSON state report and duplicate manifest are stored with the selected evidence.

## Evidence set

The selected after set under `qa/reports/visual/evidence/2026-10-30/after/` includes:

- Home light/dark, recent files, and workspace explorer.
- Standalone viewer, left sidebar, right sidebar, and both sidebars.
- Command Palette, PDF search results, thumbnails, and tab overflow.
- iCloud cloud-only and download-progress states.
- Small reader, compact both-sidebars, near-max window, and real Liquid Glass composite.
- `qa-state-report.json` and `qa-duplicate-report.md`.

The prior before and after evidence remains under `qa/reports/visual/evidence/2026-10-29/` for comparison.

## Visual regression result

No requested visual regression was found after the performance and tab-lifecycle corrections. Window geometry, sidebars, PDF canvas width, palette shape, native materials, hover coordination, search, document identity, and iCloud state presentation all passed their runtime assertions.

## Remaining visual limitations

- P3: the highlight/strikeout screenshot pair remains identical even though annotation-type and count assertions pass.
- A real File Provider domain was not available, so iCloud UI states are deterministic production-view states driven by synthetic metadata rather than a personal iCloud transition.
- PDFKit emitted one non-fatal detached-page diagnostic during the OCR transition; the suite continued and all OCR states passed.

