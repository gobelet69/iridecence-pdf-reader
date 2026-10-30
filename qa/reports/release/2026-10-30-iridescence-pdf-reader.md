# Iridescence PDF Reader Release Guard Check - 2026-10-30

## Final status: WARN

The audited source and release bundle are functionally healthy and all requested runtime visual assertions pass. This is not a publishable release artifact yet: idle RSS remains over budget, live iCloud and Core Animation traces are incomplete, and the local bundle is arm64-only, ad-hoc/linker-signed, not notarized, and has no DMG in the worktree. Check mode performed no publication.

## Problems corrected

- Main-thread PDF construction for open, post-download, history navigation, and external-file reload.
- Synchronous short-document PDF search.
- Main-thread Home directory enumeration and row metadata I/O.
- Workspace index cancellation race.
- Stale PDF completion replacement.
- Async document identity and iCloud completion path matching.
- Tab lifecycle race caused by reinstalling an unchanged active PDF after closing another tab.
- Historical hover, spacing, window geometry, traffic-light inset, palette, sidebar, tabs, and iCloud presentation regressions covered by the 116-state suite.

## Modified source and evidence

- `src/home_screen.rs`
- `src/main.rs`
- `src/pdf_search.rs`
- `src/tabs.rs`
- `src/visual_qa.rs`
- `src/workspace_index.rs`
- `docs/superpowers/plans/2026-10-27-performance-audit-fixes.md`
- `qa/reports/performance/`
- `qa/reports/visual/` and selected screenshot evidence
- `qa/reports/release/2026-10-30-iridescence-pdf-reader.md`

## Test and build gates

| Gate | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo check` | PASS |
| `cargo test` | PASS - 41 passed, 0 failed |
| `cargo clippy` | PASS with 265 historical warnings |
| `./scripts/verify.sh` with isolated Cargo target | PASS |
| Release bundle build | PASS |
| Real application launch and PDF open | PASS |
| Visual QA | PASS - 116/116 assertions |
| `git diff --check` | PASS |
| Secret-pattern scan | PASS - no credential pattern found |

The latest remote CI run for baseline HEAD `19765dd828ce0f1523d8fe721601c8ad981eca47` completed successfully. Current uncommitted audit changes have only local-equivalent verification; they were not pushed.

## Performance comparison

| Metric | Before / prior corrected baseline | Final |
|---|---:|---:|
| Warm first window median | 354 ms | 277 ms |
| First cold window | 1,711 ms | 1,086 ms |
| Launch to PDF callback | 306-342 ms | 247-273 ms; median 260 ms |
| Idle Home RSS | 89.2-89.3 MiB | 88.0-98.5 MiB; median 91.4 MiB |
| `leaks` bytes | 21,536 | 19,968 |
| History/external PDF construction | 48-207 ms synchronous risk | existing 0.020-0.031 ms worker-dispatch path, then asynchronous construction |

## Visual comparison

- Prior evidence: `qa/reports/visual/evidence/2026-10-29/`.
- Final selected evidence: `qa/reports/visual/evidence/2026-10-30/after/`.
- Final state report: 116/116 assertions, 115 unique hashes, one P3 duplicate annotation pair.
- Light, dark, compact, near-max, both sidebars, palette, search, thumbnails, tabs, and synthetic iCloud states all pass.

## Packaging and release checks

- Version is consistently `0.5.0` in Cargo metadata, README, changelog, and bundle plist.
- Local and remote `main` baseline both resolve to `19765dd828ce0f1523d8fe721601c8ad981eca47`; remote tag `v0.5.0` exists.
- Bundle binary is Mach-O arm64 and links only system frameworks/libraries.
- Bundle signature is ad-hoc/linker-signed with no Team ID; Gatekeeper assessment fails.
- No DMG, PKG, ZIP, notarization ticket, or universal-binary artifact was produced.
- No release, tag, push, GitHub mutation, or website update was performed.

## Remaining issues

1. **P2 / WARN:** reduce and stabilize idle Home RSS below 85 MiB.
2. **P2 / WARN:** capture real Core Animation frame traces for scrolling, thumbnails, and document switching.
3. **P2 / WARN:** validate iCloud download coordination with a disposable File Provider domain.
4. **P2 / WARN:** produce and verify a properly signed/notarized distribution artifact only in an explicitly authorized publish workflow.
5. **P3 / WARN:** investigate the identical highlight/strikeout raster evidence.
6. **P3 / WARN:** monitor the non-fatal PDFKit detached-page diagnostic observed once during the OCR transition.

## Decision

**WARN - suitable for continued local development and review, not approved for release publication.**

