use objc2::{define_class, msg_send, msg_send_id, sel, ClassType, DeclaredClass};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_foundation::{NSRect, NSPoint, NSSize};
use objc2_app_kit::{NSSplitViewController, NSSplitView};

define_class!(
    #[unsafe(super(NSSplitViewController))]
    #[name = "MainSplitViewController"]
    struct MainSplitViewController;

    impl MainSplitViewController {
        #[unsafe(method(splitView:effectiveRect:forDrawnRect:ofDividerAtIndex:))]
        fn effective_rect(&self, split_view: &NSSplitView, proposed_effective_rect: NSRect, drawn_rect: NSRect, divider_index: isize) -> NSRect {
            let mut rect = proposed_effective_rect;
            // The right divider is index 1 (between main and right)
            if divider_index == 1 {
                // Expand the hit area to the left and right
                rect.origin.x -= 6.0;
                rect.size.width += 12.0;
            }
            rect
        }
    }
);
