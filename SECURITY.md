# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.4.x   | :white_check_mark: |
| < 0.4.0 | :x:                |

---

## Security Model & Guarantees

**Iridecence PDF Reader** is designed with the following security principles:

1. **Memory Safety by Construction**: Core logic, session management, and state caches are implemented in safe Rust. Objective-C FFI boundaries are isolated and audited for lifecycle and memory correctness.
2. **Untrusted PDF Isolation**: PDF file parsing is delegated directly to macOS system PDFKit and CoreGraphics frameworks, benefiting from Apple's kernel sandboxing and memory hardening.
3. **Atomic File Serialization**: Saving document changes or annotations writes to an isolated temporary file before verifying size and performing an atomic rename. At no point is an original file truncated prior to complete write verification.
4. **Protected Directory Guard**: A compile-time and runtime invariant prohibits modifying files within configured read-only workspaces (e.g. `MASTER`), preventing accidental data loss or corruption of archival files.

---

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

- **Do NOT** open a public GitHub issue.
- Please email details to the project maintainers or open a confidential Security Advisory via GitHub's Security Advisory interface.
- Provide a clear description, reproduction steps, and potential impact.
- We will acknowledge receipt within 48 hours and work on an expedited patch.
