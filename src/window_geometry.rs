use objc2_app_kit::{NSScreen, NSWindow};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowKind {
    Home,
    Reader,
    Palette,
    DebugConsole,
    Utility,
}

/// Authoritative window sizing and positioning constraints for Iridecence PDF Reader.
/// Guarantees that windows NEVER exceed visible screen bounds (NSScreen.visibleFrame),
/// fit comfortably on a 14-inch MacBook baseline (~1512x982 down to 1280x800),
/// and never place titlebars under the menu bar or bottoms behind the Dock.
pub struct WindowGeometry;

impl WindowGeometry {
    /// Retrieve the authoritative visible frame for a window, or fallback to the main screen.
    pub fn screen_visible_frame(window: Option<&NSWindow>, mtm: MainThreadMarker) -> NSRect {
        if let Some(win) = window {
            if let Some(screen) = win.screen() {
                return screen.visibleFrame();
            }
        }
        if let Some(screen) = NSScreen::mainScreen(mtm) {
            return screen.visibleFrame();
        }
        // Fallback default for 14-inch MacBook baseline
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1512.0, 982.0))
    }

    /// Hard maximum width allowed for kind on the given visible frame.
    pub fn max_width(vis: NSRect, kind: WindowKind) -> f64 {
        match kind {
            WindowKind::Home => (vis.size.width * 0.90).min(1100.0),
            WindowKind::Reader => vis.size.width * 0.92,
            WindowKind::Palette => (vis.size.width * 0.65).min(640.0),
            WindowKind::DebugConsole => (vis.size.width * 0.75).min(820.0),
            WindowKind::Utility => (vis.size.width * 0.75).min(700.0),
        }
    }

    /// Hard maximum height allowed for kind on the given visible frame.
    pub fn max_height(vis: NSRect, kind: WindowKind) -> f64 {
        match kind {
            WindowKind::Home => (vis.size.height * 0.88).min(750.0),
            WindowKind::Reader => vis.size.height * 0.90,
            WindowKind::Palette => (vis.size.height * 0.65).min(460.0),
            WindowKind::DebugConsole => (vis.size.height * 0.70).min(560.0),
            WindowKind::Utility => (vis.size.height * 0.70).min(500.0),
        }
    }

    /// Absolute minimum width for kind.
    pub fn min_width(vis: NSRect, kind: WindowKind) -> f64 {
        let max_w = Self::max_width(vis, kind);
        let desired_min: f64 = match kind {
            WindowKind::Home => 740.0,
            WindowKind::Reader => 760.0,
            WindowKind::Palette => 520.0,
            WindowKind::DebugConsole => 480.0,
            WindowKind::Utility => 400.0,
        };
        desired_min.min(max_w)
    }

    /// Absolute minimum height for kind.
    pub fn min_height(vis: NSRect, kind: WindowKind) -> f64 {
        let max_h = Self::max_height(vis, kind);
        let desired_min: f64 = match kind {
            WindowKind::Home => 480.0,
            WindowKind::Reader => 520.0,
            WindowKind::Palette => 48.0,
            WindowKind::DebugConsole => 360.0,
            WindowKind::Utility => 300.0,
        };
        desired_min.min(max_h)
    }

    /// Comfortable default/ideal dimensions for a fresh window.
    pub fn default_size(vis: NSRect, kind: WindowKind) -> (f64, f64) {
        let min_w = Self::min_width(vis, kind);
        let max_w = Self::max_width(vis, kind);
        let min_h = Self::min_height(vis, kind);
        let max_h = Self::max_height(vis, kind);

        let (target_w, target_h) = match kind {
            WindowKind::Home => {
                let w = (920.0f64).min(vis.size.width * 0.80);
                let h = (600.0f64).min(vis.size.height * 0.76);
                (w, h)
            }
            WindowKind::Reader => {
                let w = (1000.0f64).min(vis.size.width * 0.84);
                let h = (680.0f64).min(vis.size.height * 0.80);
                (w, h)
            }
            WindowKind::Palette => {
                let w = (620.0f64).min(vis.size.width * 0.65);
                (w, 48.0)
            }
            WindowKind::DebugConsole => {
                let w = (720.0f64).min(vis.size.width * 0.72);
                let h = (480.0f64).min(vis.size.height * 0.64);
                (w, h)
            }
            WindowKind::Utility => {
                let w = (560.0f64).min(vis.size.width * 0.60);
                let h = (380.0f64).min(vis.size.height * 0.50);
                (w, h)
            }
        };

        (target_w.clamp(min_w, max_w), target_h.clamp(min_h, max_h))
    }

    /// Compute default centered frame for kind within visible bounds.
    pub fn default_frame(vis: NSRect, kind: WindowKind) -> NSRect {
        let (w, h) = Self::default_size(vis, kind);
        let x = vis.origin.x + (vis.size.width - w) / 2.0;
        let y = match kind {
            WindowKind::Palette => {
                // Spotlight-style positioning: top-third of display
                vis.origin.y + (vis.size.height * 0.70) - h
            }
            _ => vis.origin.y + (vis.size.height - h) / 2.0,
        };
        NSRect::new(NSPoint::new(x, y), NSSize::new(w, h))
    }

    /// Sanitize an arbitrary frame (saved geometry, restored frame, or resized frame)
    /// to guarantee it fits 100% within `vis` and respects the hard policy limits.
    pub fn sanitize_frame(mut frame: NSRect, vis: NSRect, kind: WindowKind) -> NSRect {
        // Guard against NaN or infinite coordinates
        if frame.size.width.is_nan()
            || frame.size.height.is_nan()
            || frame.origin.x.is_nan()
            || frame.origin.y.is_nan()
            || !frame.size.width.is_finite()
            || !frame.size.height.is_finite()
        {
            return Self::default_frame(vis, kind);
        }

        let min_w = Self::min_width(vis, kind);
        let max_w = Self::max_width(vis, kind);
        let min_h = Self::min_height(vis, kind);
        let max_h = Self::max_height(vis, kind);

        // Auto-repair absurdly tiny or negative frames
        if frame.size.width < min_w {
            frame.size.width = min_w;
        }
        if frame.size.height < min_h {
            frame.size.height = min_h;
        }

        // Clamp width and height to maximum allowed bounds
        if frame.size.width > max_w {
            frame.size.width = max_w;
        }
        if frame.size.height > max_h {
            frame.size.height = max_h;
        }

        // Ensure window dimension never strictly exceeds screen visible bounds
        if frame.size.width > vis.size.width {
            frame.size.width = vis.size.width;
        }
        if frame.size.height > vis.size.height {
            frame.size.height = vis.size.height;
        }

        let min_x = vis.origin.x;
        let max_x = vis.origin.x + vis.size.width - frame.size.width;
        let min_y = vis.origin.y;
        let max_y = vis.origin.y + vis.size.height - frame.size.height;

        // If completely off-screen, re-center
        let is_completely_offscreen = (frame.origin.x + frame.size.width) <= min_x
            || frame.origin.x >= (vis.origin.x + vis.size.width)
            || (frame.origin.y + frame.size.height) <= min_y
            || frame.origin.y >= (vis.origin.y + vis.size.height);

        if is_completely_offscreen || max_x < min_x || max_y < min_y {
            frame.origin.x = min_x + (vis.size.width - frame.size.width).max(0.0) / 2.0;
            frame.origin.y = min_y + (vis.size.height - frame.size.height).max(0.0) / 2.0;
        } else {
            // Nudge frame so it is completely within visibleFrame
            frame.origin.x = frame.origin.x.clamp(min_x, max_x);
            frame.origin.y = frame.origin.y.clamp(min_y, max_y);
        }

        frame
    }

    /// Apply sanitization directly to an NSWindow.
    pub fn sanitize_window(window: &NSWindow, kind: WindowKind, mtm: MainThreadMarker) {
        let vis = Self::screen_visible_frame(Some(window), mtm);
        let cur = unsafe { window.frame() };
        let sanitized = Self::sanitize_frame(cur, vis, kind);

        if (cur.origin.x - sanitized.origin.x).abs() > 0.5
            || (cur.origin.y - sanitized.origin.y).abs() > 0.5
            || (cur.size.width - sanitized.size.width).abs() > 0.5
            || (cur.size.height - sanitized.size.height).abs() > 0.5
        {
            unsafe { window.setFrame_display(sanitized, true) };
        }

        // Also enforce NSWindow minimum and maximum sizes
        let min_w = Self::min_width(vis, kind);
        let min_h = Self::min_height(vis, kind);
        let max_w = Self::max_width(vis, kind);
        let max_h = Self::max_height(vis, kind);

        unsafe {
            window.setMinSize(NSSize::new(min_w, min_h));
            window.setMaxSize(NSSize::new(max_w, max_h));
        }
    }

    /// Smoothly expand to Reader size when transitioning from a small Home window,
    /// while strictly clamping within visibleFrame and never jumping offscreen.
    pub fn ensure_reader_window_size(window: &NSWindow, mtm: MainThreadMarker) {
        let vis = Self::screen_visible_frame(Some(window), mtm);
        let mut frame = unsafe { window.frame() };

        let comfortable_w = (1000.0f64).min(vis.size.width * 0.84);
        let comfortable_h = (680.0f64).min(vis.size.height * 0.80);

        let min_reader_w = (840.0f64).min(vis.size.width * 0.80);
        let min_reader_h = (580.0f64).min(vis.size.height * 0.75);

        // If saved size from previous buggy versions was excessively large (near screen width)
        let max_reasonable_saved_w = (vis.size.width * 0.88).min(1150.0);
        let max_reasonable_saved_h = (vis.size.height * 0.88).min(820.0);

        let mut resized = false;
        if frame.size.width < min_reader_w || frame.size.width > max_reasonable_saved_w {
            frame.size.width = comfortable_w;
            resized = true;
        }
        if frame.size.height < min_reader_h || frame.size.height > max_reasonable_saved_h {
            frame.size.height = comfortable_h;
            resized = true;
        }

        if resized {
            // Keep window centered when expanding from small size or repairing oversized window
            frame.origin.x = vis.origin.x + (vis.size.width - frame.size.width) / 2.0;
            frame.origin.y = vis.origin.y + (vis.size.height - frame.size.height) / 2.0;
        }

        let sanitized = Self::sanitize_frame(frame, vis, WindowKind::Reader);
        unsafe { window.setFrame_display(sanitized, true) };

        let min_w = Self::min_width(vis, WindowKind::Reader);
        let min_h = Self::min_height(vis, WindowKind::Reader);
        let max_w = Self::max_width(vis, WindowKind::Reader);
        let max_h = Self::max_height(vis, WindowKind::Reader);
        unsafe {
            window.setMinSize(NSSize::new(min_w, min_h));
            window.setMaxSize(NSSize::new(max_w, max_h));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_geometry_min_max_bounds() {
        let vis = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1440.0, 900.0));
        for kind in [
            WindowKind::Home,
            WindowKind::Reader,
            WindowKind::Palette,
            WindowKind::DebugConsole,
            WindowKind::Utility,
        ] {
            let min_w = WindowGeometry::min_width(vis, kind);
            let max_w = WindowGeometry::max_width(vis, kind);
            let min_h = WindowGeometry::min_height(vis, kind);
            let max_h = WindowGeometry::max_height(vis, kind);

            assert!(min_w <= max_w, "min_w must be <= max_w for {:?}", kind);
            assert!(min_h <= max_h, "min_h must be <= max_h for {:?}", kind);

            let (def_w, def_h) = WindowGeometry::default_size(vis, kind);
            assert!(
                def_w >= min_w && def_w <= max_w,
                "default width in bounds for {:?}",
                kind
            );
            assert!(
                def_h >= min_h && def_h <= max_h,
                "default height in bounds for {:?}",
                kind
            );
        }
    }

    #[test]
    fn test_window_geometry_sanitize_nan_and_inf() {
        let vis = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1440.0, 900.0));
        let bad_frame = NSRect::new(
            NSPoint::new(f64::NAN, 100.0),
            NSSize::new(800.0, f64::INFINITY),
        );
        let sanitized = WindowGeometry::sanitize_frame(bad_frame, vis, WindowKind::Reader);

        assert!(sanitized.origin.x.is_finite());
        assert!(sanitized.origin.y.is_finite());
        assert!(sanitized.size.width.is_finite());
        assert!(sanitized.size.height.is_finite());
    }

    #[test]
    fn test_window_geometry_sanitize_offscreen_frame() {
        let vis = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1440.0, 900.0));
        // Window placed far out to the right and above visible screen
        let offscreen = NSRect::new(NSPoint::new(5000.0, 5000.0), NSSize::new(1200.0, 800.0));
        let sanitized = WindowGeometry::sanitize_frame(offscreen, vis, WindowKind::Reader);

        // Sanity check: must be inside visible bounds
        assert!(sanitized.origin.x >= vis.origin.x);
        assert!(sanitized.origin.x + sanitized.size.width <= vis.origin.x + vis.size.width + 0.01);
        assert!(sanitized.origin.y >= vis.origin.y);
        assert!(
            sanitized.origin.y + sanitized.size.height <= vis.origin.y + vis.size.height + 0.01
        );
    }
}
