use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};
use objc2::{define_class, msg_send, sel, ClassType};
use objc2::rc::Retained;
use std::thread;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "TestObj"]
    struct TestObj;

    unsafe impl NSObjectProtocol for TestObj {}

    impl TestObj {
        #[unsafe(method(update))]
        fn update(&self) {
            println!("Update called on main thread!");
            std::process::exit(0);
        }
    }
);

fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let obj: Retained<TestObj> = unsafe { msg_send![TestObj::class(), new] };
    let obj_ptr = Retained::into_raw(obj) as usize;

    thread::spawn(move || {
        println!("Background thread running");
        thread::sleep(std::time::Duration::from_millis(500));
        let obj_raw = obj_ptr as *mut objc2::runtime::AnyObject;
        unsafe {
            let _: () = msg_send![obj_raw, performSelectorOnMainThread: sel!(update), withObject: std::ptr::null_mut::<objc2::runtime::AnyObject>(), waitUntilDone: false];
        }
    });

    let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
    app.run();
}
