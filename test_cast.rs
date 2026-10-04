use objc2::{define_class, msg_send, sel, ClassType, DeclaredClass};
use objc2_foundation::{NSObject, NSObjectProtocol};
use objc2::runtime::AnyObject;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "AppDelegate"]
    struct AppDelegate;
    unsafe impl NSObjectProtocol for AppDelegate {}
);

fn test(app: &AppDelegate) -> &AnyObject {
    use objc2::Message;
    // let obj: &AnyObject = app; // doesn't work
    let obj: *const AppDelegate = app;
    unsafe { &*(obj as *const AnyObject) }
}
