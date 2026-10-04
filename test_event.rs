use objc2::{define_class, msg_send, sel, ClassType};
use objc2_app_kit::{NSOutlineView, NSApplication, NSEventType};
use objc2_foundation::{NSObject, NSNotification};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::MainThreadMarker;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "TestDelegate"]
    struct TestDelegate;

    impl TestDelegate {
        #[unsafe(method(outlineViewSelectionDidChange:))]
        fn outlineViewSelectionDidChange(&self, notification: &NSNotification) {
            let mtm = MainThreadMarker::new().unwrap();
            let app = NSApplication::sharedApplication(mtm);
            let event = app.currentEvent();
            if let Some(event) = event {
                let event_type = event.r#type();
                if event_type == NSEventType::LeftMouseDown || event_type == NSEventType::LeftMouseUp {
                    // It was a click
                }
            }
        }
    }
);
