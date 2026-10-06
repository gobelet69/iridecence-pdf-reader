use objc2::{define_class, msg_send, sel, ClassType, DeclaredClass};
use objc2_app_kit::{NSSplitView, NSColor, NSBezierPath};
use objc2_foundation::{NSObject, NSRect};

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
