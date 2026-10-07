use objc2_app_kit::{NSAlert, NSTextField};
use objc2_foundation::MainThreadMarker;
use objc2_foundation::ns_string;
pub fn test(mtm: MainThreadMarker) {
    let alert = NSAlert::new(mtm);
    alert.setMessageText(ns_string!("Go to Page"));
    let text_field = NSTextField::textFieldWithString(ns_string!(""), mtm);
    unsafe { alert.setAccessoryView(Some(&text_field)) };
}
