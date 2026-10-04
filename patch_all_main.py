import re
with open('src/main.rs', 'r') as f:
    content = f.read()

loop_code = """
        let mut iterations = 0;
        loop {
            iterations += 1;
            if iterations > 120 {
                node.ivars().icloud_status.set(1); // Revert to cloud if timeout
                let o_ptr = outline_raw as *mut NSOutlineView;
                let i_ptr = item_raw as *mut AnyObject;
                let _: () = objc2::msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr, waitUntilDone: false];
                let _: objc2::rc::Retained<AnyObject> = objc2::rc::Retained::from_raw(i_ptr as *mut AnyObject).unwrap();
                break;
            }
"""
content = content.replace("        loop {", loop_code)

active_code = """
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn application_did_finish_launching(&self, _notification: *mut objc2_foundation::NSNotification) {
"""
new_active_code = """
        #[unsafe(method(applicationDidBecomeActive:))]
        fn application_did_become_active(&self, _notification: &objc2_foundation::NSNotification) {
            if let Some(wd) = self.ivars().workspace_delegate.get() {
                if let Some(outline_weak) = &*wd.ivars().outline_view.borrow() {
                    if let Some(outline) = outline_weak.load() {
                        unsafe {
                            let _: () = objc2::msg_send![&outline, reloadData];
                        }
                    }
                }
            }
        }

        #[unsafe(method(applicationDidFinishLaunching:))]
        fn application_did_finish_launching(&self, _notification: *mut objc2_foundation::NSNotification) {
"""
content = content.replace(active_code, new_active_code)

# Increase thumbnail size
content = content.replace("unsafe { thumb_view.setThumbnailSize(NSSize::new(100.0, 140.0)) };", "unsafe { thumb_view.setThumbnailSize(NSSize::new(120.0, 168.0)) };")

with open('src/main.rs', 'w') as f:
    f.write(content)
