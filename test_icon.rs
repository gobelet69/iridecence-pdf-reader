use objc2::{msg_send, rc::Retained};
use objc2_foundation::NSString;
use objc2_app_kit::NSWorkspace;
use objc2::runtime::AnyObject;

fn main() {
    let mtm = objc2_app_kit::MainThreadMarker::new().unwrap();
    let ws = unsafe { NSWorkspace::sharedWorkspace() };
    let folder_img: *mut AnyObject = unsafe { msg_send![&ws, iconForFileType: objc2_foundation::ns_string!("public.folder")] };
    println!("folder_img: {:?}", folder_img);
}
