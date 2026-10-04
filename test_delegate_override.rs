use objc2_app_kit::{NSSplitViewController, NSSplitView};
use objc2_foundation::{MainThreadMarker, NSRect, NSInteger};
use objc2::{define_class, ClassType};
use objc2::rc::{Retained, Allocated};

define_class!(
    #[unsafe(super(NSSplitViewController))]
    #[name = "CustomSplitViewController"]
    struct CustomSplitViewController;
    
    impl CustomSplitViewController {
        #[unsafe(method(splitView:effectiveRect:forDrawnRect:ofDividerAtIndex:))]
        fn splitView_effectiveRect_forDrawnRect_ofDividerAtIndex(
            &self,
            _split_view: &NSSplitView,
            proposed_effective_rect: NSRect,
            _drawn_rect: NSRect,
            _divider_index: NSInteger,
        ) -> NSRect {
            let mut rect = proposed_effective_rect;
            rect.origin.x -= 5.0;
            rect.size.width += 10.0;
            rect
        }
    }
);

fn main() {
    let _mtm = MainThreadMarker::new().unwrap();
    let alloc: Allocated<CustomSplitViewController> = unsafe { objc2::msg_send![CustomSplitViewController::class(), alloc] };
    let _vc: Retained<CustomSplitViewController> = unsafe { objc2::msg_send![alloc, init] };
    println!("Successfully initialized CustomSplitViewController!");
}
