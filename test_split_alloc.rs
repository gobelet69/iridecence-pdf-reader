use objc2::{define_class, msg_send, ClassType};
use objc2_app_kit::{NSSplitViewController, NSSplitView};
use objc2_foundation::{NSObject, NSRect, NSPoint, NSSize};
use objc2::rc::{Retained, Allocated};

define_class!(
    #[unsafe(super(NSSplitView))]
    #[name = "CustomSplitView"]
    struct CustomSplitView;

    impl CustomSplitView {
        #[unsafe(method(dividerThickness))]
        fn dividerThickness(&self) -> f64 {
            10.0
        }
    }
);

fn test(mtm: objc2::MainThreadMarker) {
    let rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(100., 100.));
    let cls = CustomSplitView::class();
    let custom_split_view: Allocated<CustomSplitView> = unsafe { msg_send![cls, alloc] };
    let custom_split_view: Retained<CustomSplitView> = unsafe { msg_send![custom_split_view, initWithFrame: rect] };
}
