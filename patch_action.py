import re

with open('src/workspace_delegate.rs', 'r') as f:
    content = f.read()

action_method = """    #[unsafe(method(downloadItem:))]
    fn download_item(&self, sender: &objc2_app_kit::NSButton) {
        unsafe {
            use objc2::msg_send;
            use objc2::runtime::AnyObject;
            use objc2_app_kit::{NSOutlineView, NSView};
            
            // Get cell view
            let mut view: *mut NSView = sender as *const _ as *mut NSView;
            view = msg_send![view, superview];
            
            // Assuming we stored outline view in self.ivars() or we can find it
            // Actually, we can get it from workspace_delegate? No, we don't have outline_view reference.
            // But we can get it from the cell view's superview's superview etc.
            let mut outline_view: *mut NSOutlineView = std::ptr::null_mut();
            let mut current = view;
            while !current.is_null() {
                let is_outline: bool = msg_send![current, isKindOfClass: class!(NSOutlineView)];
                if is_outline {
                    outline_view = current as *mut NSOutlineView;
                    break;
                }
                current = msg_send![current, superview];
            }
            
            if !outline_view.is_null() {
                let row: isize = msg_send![outline_view, rowForView: view];
                if row >= 0 {
                    let item: *mut AnyObject = msg_send![outline_view, itemAtRow: row];
                    if !item.is_null() {
                        let node: &crate::workspace::FileNode = &*(item as *const AnyObject as *const crate::workspace::FileNode);
                        if node.ivars().icloud_status.get() == 1 || node.ivars().icloud_status.get() == 4 {
                            node.ivars().icloud_status.set(2);
                            node.ivars().download_progress.set(0.0);
                            let _: () = msg_send![outline_view, reloadItem: item, reloadChildren: false];
                            
                            let url_str = node.ivars().path.borrow().clone();
                            let item_retained = objc2::rc::Retained::retain(item).unwrap();
                            let item_raw = objc2::rc::Retained::into_raw(item_retained) as usize;
                            let outline_raw = outline_view as usize;
                            let node_ptr = node as *const crate::workspace::FileNode as usize;
                            
                            std::thread::spawn(move || {
                                crate::download_file(url_str, item_raw, outline_raw, node_ptr);
                            });
                        }
                    }
                }
            }
        }
    }
}"""

content = content.replace("}\n}", "}\n" + action_method)

with open('src/workspace_delegate.rs', 'w') as f:
    f.write(content)
