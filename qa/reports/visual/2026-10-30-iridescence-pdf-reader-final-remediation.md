# Iridescence PDF Reader — Final Visual Remediation — 2026-10-30

## Outcome

**Status: PASS for the application UI; WARN for external iCloud/trace evidence.** The actual release binary completed the full deterministic suite with **116/116 assertions passing, 116 unique screenshots, and zero duplicate groups**. Native AppKit/PDFKit Liquid Glass and the subtle iridescence identity are preserved.

## Additional corrected defects

| Priority | Reproduction | Root cause | Result |
|---|---|---|---|
| P1 | Move the pointer across Palette, Home, or Workspace rows during scrolling | Tracking areas requested `mouseEntered:` but owners did not implement it | Fixed and regression-tested |
| P2 | Highlight, underline, and strikeout appeared identical/invisible | PDFKit quadrilateral points were in page coordinates instead of annotation-relative coordinates | Fixed; all three rasters are distinct |
| P2 | Close an inactive annotated tab, open the scanned fixture, then run OCR | PDFKit could finish drawing a page after its document was released | Fixed with bounded deferred document lifetime; diagnostic absent |

## Before / after annotation evidence

- Before highlight SHA-256: `4da816bb4399e7162a5376db5a2c3940f7c0e5acdf00c4abc11daeca92251005`.
- Before strikeout SHA-256: identical to highlight.
- After highlight SHA-256: `ff7cc47643b87d8c74614deae10cbf04fd7f20206d1676efeb345a1a09ac10c3`.
- After strikeout SHA-256: `b7fadecc8ba30d183bbfc540e4ee9bf05d53b88df135a9bc8965250fecaf0c46`.

Evidence:

- `qa/reports/visual/evidence/2026-10-30/followup-before/`
- `qa/reports/visual/evidence/2026-10-30/followup-after/`

The after set also includes Home dark, both sidebars, iCloud downloading, search results, selected thumbnail, Command Palette, compact layout, the real WindowServer Liquid Glass composite, the state report, and the duplicate report.

## Final runtime result

- Actual native application launched with isolated Home state.
- Home, recent files, workspace explorer, reader, both sidebars, Palette, search, thumbnails, iCloud states, small windows, near-max window, light/dark, focus, presentation, OCR, and error states exercised.
- Assertions: **116/116 PASS**.
- Screenshot hashes: **116/116 unique**.
- Duplicate groups: **0**.
- `Drawing a PDFPage when its PDFDocument is nil`: **not emitted** in the exact final OCR rerun.
- `mouseEntered:` exceptions: **not emitted** after the fix.

No personal PDF or `Documents/MASTER` content was used. No release was published.

