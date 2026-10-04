use objc2::{msg_send, rc::Retained};
use objc2_app_kit::NSOutlineView;
use objc2_foundation::NSObject;

fn set_delegate(outline: &NSOutlineView, delegate: &NSObject) {
    unsafe { msg_send![outline, setDelegate: delegate] }
}
