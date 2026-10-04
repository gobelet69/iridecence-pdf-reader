use objc2::{define_class, msg_send, sel, ClassType, DeclaredClass};
use objc2_app_kit::{NSSplitView, NSColor, NSBezierPath};
use objc2_foundation::{NSObject, NSRect, CGFloat};

define_class!(
    #[unsafe(super(NSSplitView))]
    #[name = "CustomSplitView"]
    struct CustomSplitView;

    unsafe impl CustomSplitView {
        #[unsafe(method(dividerThickness))]
        fn dividerThickness(&self) -> CGFloat {
            10.0
        }
    }
);
