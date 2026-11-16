# Stable reader window and restrained appearance

**Goal:** Keep the current window frame when opening documents and sidebars; fit the PDF into the remaining space. Use a restrained native macOS appearance approved in chat.

**Architecture:** Retain AppKit, PDFKit, existing controllers and features. Configure split panes to resize their siblings, bound their widths against the available content, and refit the PDF when a panel opens. Sanitize automatic placement against the actual visible screen, including small secondary displays. Keep user window dragging and manual zoom available.

**Constraints:** Preserve existing handoff changes and all reader capabilities. Never write to MASTER. Builds and fixtures live in /tmp. No publication.

- [x] Add regression tests for small-screen placement, document transitions and sidebar frame/zoom behavior; observe failures.
- [x] Fix window geometry and split-pane behavior; test left/right toggles, search, focus return and restored oversized frames.
- [x] Remove spectral gradients, tinted surfaces, decorative icon halos and colorful pills; retain semantic status/annotation colors and a discreet system selection.
- [x] Run full tests, release build and verification. Review real light/dark and compact captures using synthetic PDFs and isolated application state.
