import re

with open("src/main.rs", "r") as f:
    content = f.read()

# Make application_openURLs set contentViewController
old = """                if let Some(wv) = self.ivars().welcome_view.get() {
                    unsafe { wv.setHidden(true) };
                }"""
new = """                if let Some(window) = self.ivars().window.get() {
                    if let Some(split_vc) = self.ivars().split_vc.get() {
                        unsafe { window.setContentViewController(Some(&**split_vc)) };
                    }
                }
                if let Some(wv) = self.ivars().welcome_view.get() {
                    unsafe { wv.setHidden(true) };
                }"""

content = content.replace(old, new)

with open("src/main.rs", "w") as f:
    f.write(content)
