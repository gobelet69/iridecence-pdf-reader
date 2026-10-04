use objc2::rc::Retained;
use objc2_foundation::NSObject;
fn test(obj: Retained<NSObject>) -> *mut NSObject {
    Retained::autorelease(obj)
}
