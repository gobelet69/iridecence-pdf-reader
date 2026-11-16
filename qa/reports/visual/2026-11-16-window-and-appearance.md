# Window behavior and restrained appearance — 2026-11-16

Implemented the requested stable window behavior and native, restrained appearance. Kept the existing reader, workspace navigation, inspector modes, annotations, OCR, shortcuts and command routing.

## Changes

- Fixed split panes resize the reader instead of enlarging the window. Opening a panel fits the PDF to its viewport.
- Home/Reader transitions preserve the selected frame. Automatic initial/restored placement respects the current screen, including small secondary displays.
- Long document tabs are clipped independently of window fitting, and use a responsive overflow menu. Tests verify that tab controls remain visible.
- A 616-point window retains the requested panel and closes the opposite panel when two panels cannot fit.
- Removed spectral color tokens, gradient surfaces, custom emblem illustration, empty-state halos and colored metadata capsules. Native semantic colors and materials supply the chrome. Annotation colors are unchanged.
- QA uses synthetic PDFs, a distinct bundle identity and a separate workspace cache.

## Evidence

- Regression tests failed before the fixes: document opening resized an 800×600 window; panel opening retained 300% zoom; small-screen placement exceeded its screen; long tabs expanded the window to 1433 points.
- Full working-tree suite: 24 tests, zero failures. Includes the pre-existing document-handoff tests.
- `scripts/verify.sh`: passed build, tests and Release bundle build. Installed bundle signature verified; installed executable matches the tested Release executable.
- Six runtime geometry checks: 800/1024/1200 points in light/dark, all preserve frame, remain onscreen, fit the PDF and retain the current page. Evidence: `/tmp/pdf-reader-verified/window-layout.json`.
- Captures reviewed: Home light/dark, standalone/workspace reader, compact reader, search, multiple tabs, empty annotations, palette search results. Neutral hierarchy, no tinted backgrounds, and readable document content.
- Native UI interaction: inspector close/open changed zoom 194% → 154%, retained the window and page; search and command palette remained available.
- Existing unrelated uncommitted handoff changes were preserved.

## Verification limits

CLI window capture was unavailable and the harness used cached-view renders; these do not establish native material compositing. Native app screenshots and real clicks were also inspected separately. Tests log a transient AppKit constraint warning while replacing the Home root controller; final frame and control-bounds assertions pass. Xcode reports an unrelated CoreSimulator version mismatch, but the macOS Release build succeeds. This is a local development update, not a published release.
