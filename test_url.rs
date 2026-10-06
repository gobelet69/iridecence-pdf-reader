use objc2_foundation::{NSURL, NSString, NSFileManager};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;

fn main() {
    let url = NSURL::fileURLWithPath(&NSString::from_str("/tmp"));
    let mut val: Option<Retained<AnyObject>> = None;
    let key = NSString::from_str("NSURLUbiquitousItemIsDownloadingKey");
    unsafe {
        let ok = url.getResourceValue_forKey_error(&mut val, &key);
    }
}
