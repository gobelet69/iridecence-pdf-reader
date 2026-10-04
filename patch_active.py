import re

with open('src/main.rs', 'r') as f:
    content = f.read()

active_code = """
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
"""

# Insert before application_did_finish_launching
idx = content.find("fn application_did_finish_launching")
content = content[:idx] + active_code + "\n        " + content[idx:]

with open('src/main.rs', 'w') as f:
    f.write(content)
