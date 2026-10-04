use objc2_app_kit::{NSSplitViewController, NSSplitView, NSViewController};
use objc2_foundation::{NSRect, NSPoint, NSSize};
use objc2::{define_class, msg_send};
use objc2::rc::{Retained, Allocated};

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

fn main() {
    let mtm = objc2::MainThreadMarker::new().unwrap();
    let split_vc = NSSplitViewController::new(mtm);
    
    let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(100.0, 100.0));
    let custom_split_alloc: Allocated<CustomSplitView> = unsafe { objc2::msg_send![CustomSplitView::class(), alloc] };
    let custom_split: Retained<CustomSplitView> = unsafe { objc2::msg_send![custom_split_alloc, initWithFrame: rect] };
    
    unsafe { split_vc.setSplitView(&custom_split) };
    println!("Successfully set split view");
}
