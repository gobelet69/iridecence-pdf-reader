# Iridescence PDF Reader — Release Guard Final Check — 2026-10-30

## Final status: WARN

The application defects identified by the performance and visual audits are corrected and verified. The code, tests, runtime UI, memory budget, annotation rendering, hover interaction, PDFKit lifecycle, and local packaging gates pass. Publication remains blocked by external release requirements: no Developer ID distribution signature, notarization, or exportable Core Animation trace, and no confirmed disposable iCloud eviction/download cycle.

No release, tag, push, GitHub mutation, or website update was performed.

## Problems corrected

- Main-thread PDF construction for open, history, post-download, and external reload.
- Synchronous PDF search and Home directory enumeration.
- Workspace cancellation and stale completion races.
- Incorrect document identity after asynchronous completion.
- Inactive-tab PDFKit lifecycle and detached-page rendering.
- Missing tracking-area `mouseEntered:` selectors in Home, Workspace, and Palette.
- Invisible/identical markup annotations caused by absolute quadrilateral coordinates.
- Missing local DMG construction and strict staged bundle verification.

## Modified files

- `src/annotations.rs`
- `src/home_screen.rs`
- `src/main.rs`
- `src/palette.rs`
- `src/pdf_search.rs`
- `src/tabs.rs`
- `src/visual_qa.rs`
- `src/workspace_delegate.rs`
- `src/workspace_index.rs`
- `build_app.sh`
- `scripts/build_dmg.sh`
- `docs/superpowers/`
- `qa/reports/` and selected visual evidence

## Gates

| Gate | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo check` | PASS |
| `cargo test` | PASS — 43 passed, 0 failed |
| `cargo clippy --all-targets` | PASS with historical non-fatal style warnings |
| `./scripts/verify.sh` with isolated target | PASS |
| `git diff --check` | PASS |
| Release bundle build | PASS |
| Visual QA | PASS — 116/116 assertions, 116 unique hashes |
| Idle Home memory | PASS — median 66.8 MiB RSS; 29 MiB physical footprint |
| PDFKit OCR lifecycle | PASS — prior detached-page diagnostic absent |

## Local artifact

- DMG: `/tmp/iridecence-release/Iridecence-PDF-Reader-v0.5.1-macOS-arm64.dmg`
- SHA-256: `951319e6561d84dd71a6e5c3f89835ca0897bb2a6a9a66b4d55831fecabc6c3e`
- `hdiutil verify`: PASS.
- Read-only mount: PASS; contains the application and `/Applications` symlink.
- Staged bundle `codesign --verify --deep --strict`: PASS.
- Architecture: arm64, consistent with the artifact name and project packaging.
- Signature: ad hoc with hardened runtime; not a Developer ID distribution signature.
- Notarization: not performed.

The DMG is a local verification artifact only and was not published.

## Remaining warnings

1. **P2 / WARN — Instruments:** Xcode 27.1 did not finalize an exportable Animation Hitches trace after normal target exit. No frame-time performance claim is made.
2. **P2 / WARN — iCloud:** a real disposable File Provider item was coordinated successfully, but upload did not complete within 120 seconds, so eviction and asynchronous redownload were not proven.
3. **P2 / WARN — distribution:** a Developer ID Application identity and notarization credentials are required before a publishable artifact can pass Gatekeeper.

## Decision

**WARN — suitable for local development and review; not approved for release publication.**

