use objc2::{define_class, msg_send, sel, ClassType};
use objc2_app_kit::NSOutlineView;
use objc2_foundation::{NSObject, NSNotification};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "TestDelegate"]
    struct TestDelegate;

    impl TestDelegate {
        #[unsafe(method(outlineViewSelectionDidChange:))]
        fn outlineViewSelectionDidChange(&self, notification: &NSNotification) {
            let outline_view: Retained<NSOutlineView> = unsafe { msg_send![notification, object] };
            let target: Option<Retained<AnyObject>> = unsafe { msg_send![&outline_view, target] };
            if let Some(target) = target {
                let _: () = unsafe { msg_send![&target, workspaceSelectionChanged: &*outline_view] };
            }
        }
    }
);
