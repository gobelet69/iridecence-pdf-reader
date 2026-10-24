# Iridescence PDF Reader — Liquid Glass Design System

## Vision: macOS 27 Native Liquid Glass Aesthetic

Iridescence PDF Reader embodies a native, lightweight, ultra-modern macOS visual identity:
- **True Transparency**: Surfaces composite directly against the Quartz Compositor, capturing live wallpaper and desktop vibrancy without artificial pseudo-blur.
- **Hierarchical Depth**: Stratified translucent layers creating effortless depth separation between background, sidebars, controls, and floating panels.
- **Document Sanctity**: The PDF is the star. Surrounding surfaces remain light and secondary, providing breathing room without competing for attention.

---

## 1. Material & Vibrancy Matrix

| Component | AppKit Material | Blending Mode | Purpose & Visual Quality |
|---|---|---|---|
| **Main Window Base** | None (`clearColor`) | Transparent Backing Store | Enables compositor to project desktop vibrancy through full-size content view |
| **Workspace Sidebar (Left)** | `NSVisualEffectMaterial::Sidebar` | `BehindWindow` | Standard Apple source-list translucency with dynamic accent tints |
| **Document Sidebar (Right)** | `NSVisualEffectMaterial::Sidebar` | `BehindWindow` | Symmetrical glass surface hosting Pages, Outline, Search, and Bookmarks |
| **Home Screen Canvas** | `NSVisualEffectMaterial::UnderWindowBackground` | `BehindWindow` | Deep translucent atmosphere for welcoming document selection |
| **Document Canvas (Center)** | Dynamic neutral canvas | `WithinWindow` | Gentle neutral matte with breathing margins around the document |
| **Tab Bar** | `NSVisualEffectMaterial::HeaderView` | `BehindWindow` | Translucent glass bar with subtle specular boundary |
| **Command Palette (`Cmd+P`)** | `NSVisualEffectMaterial::Popover` | `BehindWindow` | Spotlight-grade floating glass capsule with smooth 18pt continuous corners |

---

## 2. Geometry, Spacing & Corner Radii

Defined in [`src/ui_constants.rs`](file:///Users/theodeville/Documents/OxiReader/src/ui_constants.rs):

- **Corner Radii**:
  - Command Palette: `18.0 pt`
  - Window Frame: `14.0 pt`
  - Cards & Recent Rows: `10.0 pt`
  - Buttons & Inputs: `6.0 pt`
  - Tabs: `6.0 pt`
- **Specular Borders**:
  - Width: `0.5 pt` (retina hairline)
  - Color: Dynamic `separatorColor` / subtle glass specular reflection
- **Margins & Spacing**:
  - PDF Canvas Margin: `24.0 pt`
  - Header Horizontal Padding: `24.0 pt`
  - Header Vertical Padding: `16.0 pt`
  - Inter-item Spacing: `8.0 pt`

---

## 3. Typography & Hierarchy

Utilizes Apple's native San Francisco typography:
- **Hero Title**: System Bold `22.0 pt` (`NSFont::boldSystemFontOfSize(22.0)`)
- **Section Headers**: System Medium `11.0 pt` uppercase (`tertiaryLabelColor`)
- **Card Primary Text**: System Semi-Bold / Medium `13.5 pt` (`labelColor`)
- **Metadata / Subtitles**: System Regular `12.0 pt` (`secondaryLabelColor`)
- **Badges / Compact Labels**: System Regular `10.5 pt` (`tertiaryLabelColor`)

---

## 4. Light Mode & Dark Mode Harmony

- In **Dark Mode**: Translucent surfaces absorb ambient dark desktop tones; text retains crisp contrast via `labelColor` and `secondaryLabelColor`.
- In **Light Mode**: Translucent surfaces reflect ambient light desktop tones; PDF pages remain distinct from the soft neutral canvas without harsh boundaries.
- **Accessibility**:
  - Respects *Reduce Transparency* by falling back gracefully to system opaque sheet colors.
  - Respects *Increase Contrast* with automatic AppKit dynamic border enhancement.

---

## 5. Spotlight-Grade Command Palette (`Cmd+P`)

The Command Palette is designed as a true macOS 27 Spotlight-style floating glass capsule:

### 5.1 Window & Geometry
- **Style Mask**: `NSWindowStyleMask::Borderless` (`0`). No legacy titlebars, window chrome, or close buttons.
- **Window Level**: Floating Level 3 (`kCGFloatingWindowLevel`), hovering gracefully above the main window and menus.
- **Corner Radius**: `16.0 pt` continuous rounded corners (`layer.setCornerRadius(16.0)` with `masksToBounds = true`).
- **Specular Border**: Retina hairline `0.5 pt` with dynamic specular reflection (`NSColor::separatorColor()` / subtle white tint).
- **Material**: `NSVisualEffectMaterial::Popover` with `NSVisualEffectBlendingMode::BehindWindow` and `state: Active`.
- **Width**: Responsive `580.0 pt` (default/max width `640.0 pt`).
- **Dynamic Height**:
  - **Empty State**: Exactly `48.0 pt` (compact Spotlight pill). Separator and table scroll view are hidden.
  - **Populated State**: Dynamically sized to `min(48.5 + display_rows * 36.0 + 10.0, max_palette_h) pt`.

### 5.2 Header & Row Metrics
- **Header Height**: `48.0 pt` compact search bar.
- **Search Icon**: 16pt SF Symbol (`magnifyingglass`) in `secondaryLabelColor`.
- **Search Field**: 15pt text field (`NSFont::systemFontOfSize(15.0)`), borderless (`setBezeled(false)`), no focus ring.
- **Separator**: Native 0.5pt `NSBoxType::Separator` dividing search header from results.
- **Row Height**: `36.0 pt` per result item.
- **Row Typography**:
  - Primary Title: San Francisco Regular `13.0 pt` (`NSFont::systemFontOfSize(13.0)`).
  - Secondary Subtitle: San Francisco Regular `11.0 pt` (`secondaryLabelColor`).
- **Selection Pill**: Translucent rounded capsule (`GlassTableRowView`) filled with `controlAccentColor().colorWithAlphaComponent(0.22)`.
- **Shortcut Badges**: Trailing compact badges displaying actual system shortcuts (`⌘T`, `⌘W`, `⌘S`, `⇧⌘S`, `⌘D`, `⌘I`, `⌘,`, etc.) styled with `tertiaryLabelColor` and rounded pill background, replacing repetitive "Command" badges.

### 5.3 Prefix Routing & Cursor Management
- `>` Prefix: Command execution mode (defaults via `Cmd+Shift+P`).
- `/` Prefix: In-document full-text search mode.
- `p:` or `:` Prefix: Go to page mode (defaults via `Cmd+L`).
- Default: Workspace file filtering and recents search.
- **Cursor Placement**: When prefilling prefix strings (e.g. `>` or `p:`), the text view's selected range is explicitly positioned at the end (`NSMakeRange(prefix.len(), 0)`), completely eliminating raw text selection highlights and blue boxes.

---

## 6. Keyboard Shortcuts Matrix

| Shortcut | Action | Scope / Context |
|---|---|---|
| `⌘P` | Open Command Palette / Quick Open | Global |
| `⇧⌘P` | Open Command Palette with `>` prefix | Global |
| `⌘L` | Go to Page (Palette with `p:` prefix) | Document Mode |
| `⌘O` | Open PDF Document | Global |
| `⇧⌘O` | Open Workspace Folder | Global |
| `⌘F` | Focus Search Bar | Document Mode |
| `⌘T` | New Tab | Document Mode |
| `⌘W` | Close Tab / Document | Document Mode |
| `⌘S` | Safe Save Annotations | Document Mode |
| `⇧⌘S` | Save As Copy | Document Mode |
| `⌘D` | Add Bookmark | Document Mode |
| `⌘I` | Toggle Document Inspector / Outline | Document Mode |
| `⌘,` | Open Settings | Global |
| `⌘[` | Navigate Back in History | Document Mode |
| `⌘]` | Navigate Forward in History | Document Mode |
| `⌘0` | Actual Size (100% Zoom) | Document Mode |
| `⌘1` | Fit Width | Document Mode |
| `⌘2` | Fit Page | Document Mode |
| `⌘+` / `⌘=` | Zoom In | Document Mode |
| `⌘-` | Zoom Out | Document Mode |
| `⎋` (Escape) | Dismiss Command Palette / Floating Panels | Modal / Floating |

