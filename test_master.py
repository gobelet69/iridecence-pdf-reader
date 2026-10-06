with open("src/main.rs", "r") as f:
    main_rs = f.read()

# Replace the pending_url part to always open MASTER
injection = """
            let mtm = MainThreadMarker::new().unwrap();
            let url = unsafe { objc2_foundation::NSURL::fileURLWithPath(&objc2_foundation::NSString::from_str("/Users/theodeville/Documents/MASTER")) };
            let urls = objc2_foundation::NSArray::from_slice(&[&*url]);
            let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
            let _: () = unsafe { objc2::msg_send![self, application: &*app, openURLs: &*urls] };
"""

main_rs = main_rs.replace("if let Some(url) = self.ivars().pending_url.borrow_mut().take() {", injection + "\n            if let Some(url) = self.ivars().pending_url.borrow_mut().take() {")

with open("src/main.rs", "w") as f:
    f.write(main_rs)
