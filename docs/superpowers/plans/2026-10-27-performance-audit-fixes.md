# Performance Audit Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove confirmed main-thread stalls and indexing races while preserving the existing Rust/AppKit/PDFKit architecture.

**Architecture:** Reuse the app's established worker-thread plus main-thread selector pattern for file and PDF loading. Keep PDFKit rendering and thumbnails native, make search consistently asynchronous, and replace boolean workspace-index cancellation with monotonic request generations.

**Tech Stack:** Rust 2024, objc2, AppKit, PDFKit, XCTest-style Rust unit tests, disposable Swift/CoreGraphics PDF fixtures.

**Spec:** User request in this task; no separate design document.

## Global Constraints

- Never read or modify personal documents or `/Users/theodeville/Documents/MASTER`.
- Use disposable fixtures under `/tmp` only.
- Preserve existing Swift/Rust architecture and all current capabilities.
- Measure the same scenarios before and after changes.
- Do not publish, release, or push.

## Review Focus

- A stale PDF load must not replace a newer requested document.
- Rapid workspace switches must leave only the newest indexer active.
- Empty and replaced PDF-search queries must cancel cleanly.
- Hidden files and non-PDF files must remain excluded from Home directory results.
- Worker completion must not retain delegates or file nodes indefinitely.

---

### Task 1: Workspace indexing cancellation and search hot path

**Files:**
- Modify: `src/workspace_index.rs`
- Test: `src/workspace_index.rs`

**Interfaces:**
- Produces: generation-scoped workspace crawling/extraction and allocation-free ASCII substring matching.
- Consumes: existing `WorkspaceIndex`, `SearchResult`, and index serialization format unchanged.

- [ ] Add failing tests for stale-generation cancellation and ASCII/non-ASCII case-insensitive search behavior.
- [ ] Run the focused tests and confirm the expected failures.
- [ ] Replace boolean cancellation for background indexing with a monotonic generation token.
- [ ] Add an ASCII fast path that avoids lowercasing every page while retaining the existing Unicode fallback.
- [ ] Run focused tests and the full suite.

### Task 2: Non-blocking PDF load and search dispatch

**Files:**
- Modify: `src/main.rs`
- Modify: `src/pdf_search.rs`
- Test: `src/main.rs`, `src/pdf_search.rs`

**Interfaces:**
- Produces: background PDF construction routed through existing `pdfDidLoad:` stale-request validation; asynchronous PDFKit search for every non-empty query.
- Consumes: `pending_pdf_path`, `content_mode`, and existing PDFDocumentDelegate callbacks.

- [ ] Add failing policy tests proving local opens use background loading and all non-empty searches use incremental PDFKit search.
- [ ] Run focused tests and confirm the expected failures.
- [ ] Extract the existing background PDF construction path and use it for URL opens and iCloud-download completion.
- [ ] Route all non-empty PDF queries through `beginFindString` and existing cancellation callbacks.
- [ ] Run focused tests and the full suite.

### Task 3: Non-blocking large-directory Home loading

**Files:**
- Modify: `src/home_screen.rs`
- Test: `src/home_screen.rs`

**Interfaces:**
- Produces: generation-scoped background directory enumeration with main-thread result application.
- Consumes: existing Home table model and reload behavior.

- [ ] Add failing tests for directory filtering and stale-result rejection.
- [ ] Run focused tests and confirm the expected failures.
- [ ] Move directory enumeration to a worker and apply only the current generation on the main thread.
- [ ] Preserve loading and empty states without changing navigation behavior.
- [ ] Run focused tests and the full suite.

### Task 4: Verification and performance report

**Files:**
- Create: `qa/reports/performance/2026-10-27-iridescence-pdf-reader.md`
- Modify: performance test helpers only if required for repeatability.

**Interfaces:**
- Produces: reproducible before/after evidence, status per scenario, and explicit blockers.
- Consumes: the same disposable fixtures and machine conditions used for the baseline.

- [ ] Repeat launch, PDFKit open/render/search, workspace scan, tests, build, and memory measurements.
- [ ] Run `./scripts/verify.sh` with `/tmp` Cargo target isolation.
- [ ] Inspect the final diff and verify no path under `MASTER` was accessed or modified.
- [ ] Record PASS/WARN/FAIL, variability, trade-offs, and runtime-proof limitations.
