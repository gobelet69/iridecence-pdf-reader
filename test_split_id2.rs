use objc2::{define_class, msg_send_id, ClassType};
use objc2_app_kit::{NSSplitViewController, NSSplitView, NSColor, NSBezierPath, NSViewController};
use objc2_foundation::{NSObject, NSRect, NSPoint, NSSize};
use objc2::rc::Retained;

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
    let custom_split_view: Retained<NSSplitView> = unsafe { msg_send_id![cls, alloc] };
    let custom_split_view: Retained<NSSplitView> = unsafe { msg_send_id![custom_split_view, initWithFrame: rect] };
    let split_vc = NSSplitViewController::new(mtm);
    unsafe { split_vc.setView(&custom_split_view) };
}
