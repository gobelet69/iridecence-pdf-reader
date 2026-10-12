## Summary

Briefly describe the change and the rationale behind it.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Performance improvement
- [ ] Documentation update
- [ ] Code refactoring / hygiene

## Verification
- [ ] `cargo fmt --check` passes cleanly
- [ ] `cargo clippy` passes without warnings
- [ ] `cargo check` and `cargo test` pass
- [ ] Tested application bundle via `./build_app.sh`
- [ ] Visual QA regression checks pass without UI defects

## Checklist
- [ ] Preserves native AppKit/PDFKit architecture (no Electron/WebKit)
- [ ] Respects read-only file protection invariants
- [ ] Updates relevant documentation if applicable
