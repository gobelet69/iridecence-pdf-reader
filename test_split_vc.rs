use objc2::{define_class, msg_send, sel, ClassType, DeclaredClass};
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
            12.0
        }
    }
);

fn test(mtm: objc2::runtime::MainThreadMarker) {
    let split_vc = NSSplitViewController::new(mtm);
    let rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(100., 100.));
    let custom_split_view: Retained<CustomSplitView> = unsafe { msg_send![CustomSplitView::class(), alloc] };
    let custom_split_view: Retained<CustomSplitView> = unsafe { msg_send![custom_split_view, initWithFrame: rect] };
    unsafe { split_vc.setView(&custom_split_view) };
}
