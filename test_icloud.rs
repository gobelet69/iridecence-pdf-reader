use objc2::{msg_send, rc::Retained};
use objc2_foundation::{NSString, NSURL};
use objc2::runtime::AnyObject;

fn main() {
    let path = "/Users/theodeville/Documents/MASTER";
    let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(path)) };
    
    let mut status_val: *mut AnyObject = std::ptr::null_mut();
    let mut error: *mut AnyObject = std::ptr::null_mut();
    
    // NSURLUbiquitousItemDownloadingStatusKey is available.
    // wait, what is the exact string value of NSURLUbiquitousItemDownloadingStatusKey?
    // It's "NSURLUbiquitousItemDownloadingStatusKey".
    let key = unsafe { NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey") };
    
    let ok: bool = unsafe { msg_send![&url, getResourceValue: &mut status_val, forKey: &*key, error: &mut error] };
    
    if ok && !status_val.is_null() {
        let status_str = status_val as *mut NSString;
        println!("Status: {:?}", unsafe { &*status_str });
    } else {
        println!("Failed or not ubiquitous");
    }
}
