use objc2::rc::Retained;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{NSColor, NSTableView, NSTableRowView};
use objc2_foundation::{MainThreadMarker, NSInteger, NSObjectProtocol, NSPoint, NSRect, NSSize};

#[derive(Default)]
pub struct GlassRowIvars {
    pub is_hovered: std::cell::Cell<bool>,
}

define_class!(
    #[unsafe(super(NSTableRowView))]
    #[thread_kind = MainThreadOnly]
    #[name = "GlassTableRowView"]
    #[ivars = GlassRowIvars]
    pub struct GlassTableRowView;

    unsafe impl NSObjectProtocol for GlassTableRowView {}

    impl GlassTableRowView {
        #[unsafe(method(prepareForReuse))]
        fn prepare_for_reuse(&self) {
            let _: () = unsafe { msg_send![super(self), prepareForReuse] };
            self.set_hovered(false);
        }

        #[unsafe(method(layout))]
        fn layout(&self) {
            let _: () = unsafe { msg_send![super(self), layout] };
            let subviews = unsafe { self.subviews() };
            for i in 0..subviews.count() {
                let sv = unsafe { subviews.objectAtIndex(i) };
                let is_vev: bool = unsafe {
                    msg_send![&sv, isKindOfClass: objc2::class!(NSVisualEffectView)]
                };
                if is_vev {
                    unsafe { sv.setHidden(true) };
                }
            }
        }

        #[unsafe(method(drawBackgroundInRect:))]
        fn draw_background_in_rect(&self, dirty_rect: NSRect) {
            let _: () = unsafe { msg_send![super(self), drawBackgroundInRect: dirty_rect] };
            if self.ivars().is_hovered.get() && unsafe { !self.isSelected() } {
                let color: Retained<NSColor> = unsafe {
                    msg_send![&NSColor::labelColor(), colorWithAlphaComponent: 0.065_f64]
                };
                let _: () = unsafe { msg_send![&color, setFill] };
                let bounds = unsafe { self.bounds() };
                let inset_bounds = NSRect::new(
                    NSPoint::new(bounds.origin.x + 6.0, bounds.origin.y + 1.5),
                    NSSize::new((bounds.size.width - 12.0).max(0.0), (bounds.size.height - 3.0).max(0.0)),
                );
                let path: *mut objc2::runtime::AnyObject = unsafe {
                    msg_send![objc2::class!(NSBezierPath), bezierPathWithRoundedRect: inset_bounds, xRadius: 6.0_f64, yRadius: 6.0_f64]
                };
                let _: () = unsafe { msg_send![path, fill] };
            }
        }

        #[unsafe(method(drawSelectionInRect:))]
        fn draw_selection_in_rect(&self, _dirty_rect: NSRect) {
            if unsafe { self.isSelected() } {
                let color: Retained<NSColor> = unsafe {
                    msg_send![&NSColor::controlAccentColor(), colorWithAlphaComponent: 0.20_f64]
                };
                let _: () = unsafe { msg_send![&color, setFill] };
                let bounds = unsafe { self.bounds() };
                let inset_bounds = NSRect::new(
                    NSPoint::new(bounds.origin.x + 6.0, bounds.origin.y + 1.5),
                    NSSize::new((bounds.size.width - 12.0).max(0.0), (bounds.size.height - 3.0).max(0.0)),
                );
                let path: *mut objc2::runtime::AnyObject = unsafe {
                    msg_send![objc2::class!(NSBezierPath), bezierPathWithRoundedRect: inset_bounds, xRadius: 6.0_f64, yRadius: 6.0_f64]
                };
                let _: () = unsafe { msg_send![path, fill] };
            }
        }
    }
);

impl GlassTableRowView {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(GlassRowIvars::default());
        unsafe { msg_send![super(this), init] }
    }

    pub fn set_hovered(&self, hovered: bool) {
        if self.ivars().is_hovered.get() != hovered {
            self.ivars().is_hovered.set(hovered);
            unsafe { self.setNeedsDisplay(true) };
        }
    }

    pub fn new_row(mtm: MainThreadMarker) -> *mut NSTableRowView {
        let view = Self::new(mtm);
        let row_view = unsafe { Retained::cast_unchecked::<NSTableRowView>(view) };
        Retained::autorelease_ptr(row_view)
    }
}

/// Centralized coordinator ensuring at most one row is hovered per table/outline.
/// Recalculates hovered row cleanly on mouse movements, live scrolling, content changes,
/// and window key transitions.
#[derive(Default, Debug)]
pub struct HoverCoordinator {
    hovered_row: std::cell::Cell<Option<NSInteger>>,
}

impl HoverCoordinator {
    pub const fn new() -> Self {
        Self {
            hovered_row: std::cell::Cell::new(None),
        }
    }

    pub fn hovered_row(&self) -> Option<NSInteger> {
        self.hovered_row.get()
    }

    pub fn clear_hover(&self, table: &NSTableView) {
        if let Some(r) = self.hovered_row.take() {
            if let Some(row_view) = unsafe { table.rowViewAtRow_makeIfNecessary(r, false) } {
                if let Some(glass_row) = row_view.downcast_ref::<GlassTableRowView>() {
                    glass_row.set_hovered(false);
                }
            }
        }
    }

    pub fn update_hover(&self, table: &NSTableView) {
        let window = match unsafe { table.window() } {
            Some(w) => w,
            None => {
                self.clear_hover(table);
                return;
            }
        };
        if unsafe { !window.isKeyWindow() } {
            self.clear_hover(table);
            return;
        }

        let mouse_win = unsafe { window.mouseLocationOutsideOfEventStream() };
        let mouse_table = unsafe { table.convertPoint_fromView(mouse_win, None) };
        let vis_rect = unsafe { table.visibleRect() };

        let target_row: Option<NSInteger> =
            if unsafe { objc2_foundation::NSPointInRect(mouse_table, vis_rect) } {
                let r = unsafe { table.rowAtPoint(mouse_table) };
                let count = unsafe { table.numberOfRows() };
                if r >= 0 && r < count {
                    Some(r)
                } else {
                    None
                }
            } else {
                None
            };

        let prev_row = self.hovered_row.get();
        if prev_row == target_row {
            return;
        }

        // Unhover previous row
        if let Some(old_r) = prev_row {
            if let Some(row_view) = unsafe { table.rowViewAtRow_makeIfNecessary(old_r, false) } {
                if let Some(glass_row) = row_view.downcast_ref::<GlassTableRowView>() {
                    glass_row.set_hovered(false);
                }
            }
        }

        // Hover new row
        if let Some(new_r) = target_row {
            if let Some(row_view) = unsafe { table.rowViewAtRow_makeIfNecessary(new_r, false) } {
                if let Some(glass_row) = row_view.downcast_ref::<GlassTableRowView>() {
                    glass_row.set_hovered(true);
                }
            }
        }

        self.hovered_row.set(target_row);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hover_coordinator_initial_state() {
        let coordinator = HoverCoordinator::new();
        assert_eq!(coordinator.hovered_row(), None);
    }

    #[test]
    fn test_hover_coordinator_single_hovered_row_invariant() {
        let coordinator = HoverCoordinator::new();
        coordinator.hovered_row.set(Some(5));
        assert_eq!(coordinator.hovered_row(), Some(5));

        coordinator.hovered_row.set(Some(12));
        assert_eq!(coordinator.hovered_row(), Some(12));

        coordinator.hovered_row.set(None);
        assert_eq!(coordinator.hovered_row(), None);
    }
}
