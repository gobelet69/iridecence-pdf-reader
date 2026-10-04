import re

with open('src/main.rs', 'r') as f:
    content = f.read()

old_open = """        fn application_openURLs(&self, _application: &NSApplication, urls: &NSArray<NSURL>) {
            if let Some(url) = urls.firstObject() {
                if self.ivars().window.get().is_none() {
                    *self.ivars().pending_url.borrow_mut() = Some(url.clone());
                    return;
                }
                
                let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                if let Some(doc) = doc {
                    if let Some(pdf_view) = self.ivars().pdf_view.get() {
                        unsafe { pdf_view.setDocument(Some(&doc)) };
                    }
                    if let Some(window) = self.ivars().window.get() {
                        if let Some(title) = url.lastPathComponent() {
                            window.setTitle(&title);
                        }
                    }
                } else {
                    // It's a folder or invalid PDF!
                    let path = url.path().unwrap().to_string();
                    let name = url.lastPathComponent().unwrap().to_string();
                    let root = crate::workspace::FileNode::new(path, name, true);
                    
                    if let Some(window) = self.ivars().window.get() {
                        if let Some(title) = url.lastPathComponent() {
                            window.setTitle(&title);
                        }
                        if let Some(split_vc) = self.ivars().split_vc.get() {
                            unsafe { window.setContentViewController(Some(&*self.ivars().split_vc.get().unwrap())) };
                        }
                    }"""

new_open = """        fn application_openURLs(&self, _application: &NSApplication, urls: &NSArray<NSURL>) {
            if let Some(url) = urls.firstObject() {
                if self.ivars().window.get().is_none() {
                    *self.ivars().pending_url.borrow_mut() = Some(url.clone());
                    return;
                }
                
                let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                if let Some(doc) = doc {
                    if let Some(pdf_view) = self.ivars().pdf_view.get() {
                        unsafe { pdf_view.setDocument(Some(&doc)) };
                    }
                    if let Some(window) = self.ivars().window.get() {
                        if let Some(title) = url.lastPathComponent() {
                            window.setTitle(&title);
                        }
                    }
                    if let Some(explorer_item) = self.ivars().explorer_item.get() {
                        unsafe { explorer_item.setCollapsed(true) };
                    }
                } else {
                    // It's a folder or invalid PDF!
                    let path = url.path().unwrap().to_string();
                    let name = url.lastPathComponent().unwrap().to_string();
                    let root = crate::workspace::FileNode::new(path, name, true);
                    
                    if let Some(explorer_item) = self.ivars().explorer_item.get() {
                        unsafe { explorer_item.setCollapsed(false) };
                    }
                    if let Some(window) = self.ivars().window.get() {
                        if let Some(title) = url.lastPathComponent() {
                            window.setTitle(&title);
                        }
                        if let Some(split_vc) = self.ivars().split_vc.get() {
                            unsafe { window.setContentViewController(Some(&*split_vc)) };
                        }
                    }"""

if old_open in content:
    content = content.replace(old_open, new_open)
    with open('src/main.rs', 'w') as f:
        f.write(content)
    print("Patched open urls!")
else:
    print("Not found open urls")
