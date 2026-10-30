# Iridescence PDF Reader visual audit - 2026-10-29

## Result

**PASS with one non-blocking evidence limitation.** The actual native macOS application was built and launched against an isolated home profile and disposable PDFs. The final run captured all 116 states and passed all 116 state assertions. No release was published.

The application in this repository is implemented in Rust with AppKit and PDFKit rather than SwiftUI. The audit exercised the shipped native application architecture as-is.

## Safety and fixtures

- Runtime profile: `/tmp/iridecence-visual-audit-home`
- PDF workspace: `/tmp/iridecence-demo-workspace`
- Eight synthetic PDFs with searchable text, outlines, pages, long names, and nested folders
- No personal documents were opened or changed.
- The `MASTER` directory was neither read nor modified.
- The QA driver refuses to create Home fixtures unless `NSHomeDirectory()` is under `/tmp`.

## Before and after measurements

| Measurement | Before | After |
|---|---:|---:|
| States captured | 116 | 116 |
| State assertions passing | 111/116 | **116/116** |
| Assertion failures | 5 | **0** |
| Unique screenshot hashes | 115/116 | 115/116 |
| Unit and regression tests | 38 passing | **39 passing** |
| Full project verification | passing | **passing** |

The original runner initially captured 0 states because an Objective-C `CGColor` return value was bridged as `void *`; that crash was reproduced before correction. The preserved machine-readable baselines are in [`before/qa-state-report.json`](evidence/2026-10-29/before/qa-state-report.json) and [`after/qa-state-report.json`](evidence/2026-10-29/after/qa-state-report.json).

## Corrections

1. **iCloud completion and reopening** - download completion now compares canonical document identity instead of exact path strings. This handles equivalent macOS paths such as `/tmp/...` and `/private/tmp/...`. A regression test creates a real disposable completed file and verifies both the matching and obsolete-download cases.
2. **Visual runner crash** - all QA-only Core Graphics color bridges now use typed `CGColor` objects.
3. **Async-state sampling** - the QA driver waits for production asynchronous PDF loads, directory enumeration, PDF search, and the first PDFKit layout cycle before capturing.
4. **Disposable Home/iCloud states** - local and iCloud fixtures are created only inside the isolated `/tmp` profile, allowing these views to be tested without touching user files.
5. **Lazy workspace trees** - the disposable workspace is deterministically hydrated for tree-state captures while production workspace loading remains lazy and asynchronous.

## Visual review

| Area | Result | Evidence |
|---|---|---|
| Home, Recent, Local Files, workspace list | PASS | [`Local Files`](evidence/2026-10-29/after/06-home-local-files.png), [`iCloud Drive`](evidence/2026-10-29/after/07-home-icloud-drive.png) |
| Workspace explorer and PDF viewer | PASS | [`both sidebars`](evidence/2026-10-29/after/18-reader-workspace-both-sidebars.png) |
| Left/right sidebar sizing | PASS | 180 pt and 360 pt right-sidebar bounds passed; the narrow reader collapses the right sidebar to preserve the PDF canvas. |
| Search results | PASS | [`12 populated matches`](evidence/2026-10-29/after/32-sidebar-search-results.png) |
| PDF thumbnails | PASS | Pages mode, many thumbnails, selection, top/middle/end navigation, and responsive widths passed. |
| Command Palette | PASS | [`rounded native material panel`](evidence/2026-10-29/after/56-palette-empty-light.png) |
| PDF tabs and document identity | PASS | [`seven-document overflow state`](evidence/2026-10-29/after/52-tabs-many-documents.png) |
| iCloud availability and progress | PASS | [`cloud-only`](evidence/2026-10-29/after/26-workspace-icloud-cloudonly.png), [`downloading`](evidence/2026-10-29/after/27-workspace-icloud-downloading.png), [`local`](evidence/2026-10-29/after/28-workspace-icloud-local.png) |
| Small windows and traffic lights | PASS | [`small reader`](evidence/2026-10-29/after/107-small-window-both-sidebars.png) |
| Full screen / presentation | PASS | [`presentation mode`](evidence/2026-10-29/after/93-presentation-mode.png) |
| Light and dark modes | PASS | [`dark Home`](evidence/2026-10-29/after/02-home-dark.png) |
| Near-max window | PASS | [`near-max reader`](evidence/2026-10-29/after/110-near-max-window-reader.png) |

## Historical problem review

- **Multiple hover states after scrolling:** not reproduced. The coordinator tests pass and the captured workspace state contains a single hover target: [`hover state`](evidence/2026-10-29/after/113-hover-workspace-item.png).
- **Incorrect spacing / excessively grey backgrounds:** not reproduced in light, dark, small, or near-max layouts. Native semantic colors and material hierarchy remain legible without flattening the Liquid Glass identity.
- **Window sizing / content below traffic lights:** not reproduced. Small, short, near-max, focus, and presentation states maintain titlebar clearance and usable document space.
- **Command Palette transparency and corners:** rounded clipping, native material, selection, empty, command, search, and no-result states pass.
- **Sidebar resizing:** min/max widths and narrow-window auto-collapse pass.
- **PDF tabs / document identity:** multi-document, long-name, dirty, active-middle, and overflow states pass.
- **iCloud completion and reopening:** a real path-identity defect was corrected and regression-tested.

## Known evidence limitation

The highlight and strikeout annotation screenshots remain pixel-identical, so the hash report contains one duplicate pair. Their model assertions pass, and annotation rendering was outside this audit's requested focus. The complete duplicate manifest is in [`qa-duplicate-report.md`](evidence/2026-10-29/after/qa-duplicate-report.md). This does not affect the 116/116 state result, but it should be resolved before treating the suite as a general-purpose annotation visual regression oracle.

## Verification commands

```text
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/target cargo test --release
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/target ./scripts/verify.sh
HOME=/tmp/iridecence-visual-audit-home \
CFFIXED_USER_HOME=/tmp/iridecence-visual-audit-home \
IRIDESCENCE_QA_FIX_SUITE=1 /tmp/target/debug/oxireader
```

Final evidence: 39/39 tests passed, the build/package verification gate passed, and the rebuilt application completed all 116 visual states with zero assertion failures.
