# Contributing to Iridecence PDF Reader

Thank you for your interest in contributing to **Iridecence PDF Reader**! We welcome bug fixes, performance improvements, and documentation enhancements.

---

## Code of Conduct

Please be respectful, constructive, and collaborative in all discussions and code reviews.

---

## Development Environment Setup

### Prerequisites
- **Operating System**: macOS 14.0 (Sonoma) or macOS 15.0 (Sequoia) on Apple Silicon (arm64) or Intel (x86_64).
- **Xcode Command Line Tools**:
  ```bash
  xcode-select --install
  ```
- **Rust Toolchain**:
  Install via `rustup`:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup update stable
  ```

---

## Build & Test Workflow

### 1. Verification & Compilation
Run standard Cargo check and test commands:
```bash
cargo check
cargo test
```

### 2. Code Quality & Formatting
We maintain strict formatting and clippy lint standards:
```bash
cargo fmt --check
cargo clippy
```

### 3. Packaging the Application
Build the complete native `.app` bundle:
```bash
./build_app.sh
open "Iridecence PDF Reader.app"
```

---

## Autonomous Visual QA System

Iridecence uses an integrated visual QA regression harness:
- Test runner: `scripts/visual_qa.sh`
- Visual audit captures are persisted under `/tmp/iridecence-visual-qa/`
- Every milestone captures 19+ distinct UI states across both Light and Dark modes.
- Before submitting a major change, ensure no regressions against the baseline.

---

## Pull Request Guidelines

1. **Keep Pull Requests Focused**: Target a specific feature, defect fix, or performance improvement.
2. **Preserve Native Invariants**: Do not introduce Electron, WebKit, or non-native UI rendering libraries.
3. **Master Directory Safety**: Never disable or bypass safety guards protecting critical system directories or read-only workspaces.
4. **Sign-off**: Ensure code formats cleanly with `cargo fmt`.
