use objc2::{define_class, msg_send, sel, ClassType, DeclaredClass, MainThreadMarker, rc::Retained};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString};
use objc2_app_kit::NSButton;
use objc2::runtime::AnyObject;

#[derive(Default)]
struct MyClassIvars;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "MyClass"]
    #[ivars = MyClassIvars]
    struct MyClass;

    unsafe impl NSObjectProtocol for MyClass {}

    unsafe impl MyClass {
        #[unsafe(method(test))]
        fn test(&self) {
            let mtm = MainThreadMarker::new().unwrap();
            let target: &AnyObject = unsafe { std::mem::transmute(self) };
            let btn = unsafe { NSButton::buttonWithTitle_target_action(objc2_foundation::ns_string!("Test"), Some(target), None) };
        }
    }
);
fn main() {}
