use objc2::{define_class, msg_send_id, msg_send, ClassType};
use objc2_app_kit::{NSSplitViewController, NSSplitView, NSColor, NSBezierPath, NSViewController};
use objc2_foundation::{NSObject, NSRect, NSPoint, NSSize};
use objc2::{rc::Retained, MainThreadMarker};

define_class!(
    #[unsafe(super(NSSplitView))]
    #[name = "CustomSplitView"]
    struct CustomSplitView;

    impl CustomSplitView {
        #[unsafe(method(dividerThickness))]
        fn dividerThickness(&self) -> f64 {
            12.0
        }
        
        #[unsafe(method(drawDividerInRect:))]
        fn drawDividerInRect(&self, mut rect: NSRect) {
            rect.origin.x += (rect.size.width - 1.0) / 2.0;
            rect.size.width = 1.0;
            unsafe {
                let color = NSColor::separatorColor();
                color.set();
                NSBezierPath::fillRect(rect);
            }
        }
    }
);

fn test(mtm: MainThreadMarker) {
    let split_vc = NSSplitViewController::new(mtm);
    let rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(100., 100.));
    let custom_split_view: Retained<CustomSplitView> = unsafe { msg_send_id![CustomSplitView::class(), alloc] };
    let custom_split_view: Retained<CustomSplitView> = unsafe { msg_send_id![custom_split_view, initWithFrame: rect] };
    unsafe { split_vc.setView(&custom_split_view) };
}
