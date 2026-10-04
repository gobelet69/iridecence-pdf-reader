import re

with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()

correct_method = """    #[unsafe(method(nodeDidLoadChildren:))]
    fn nodeDidLoadChildren(&self, notification: &objc2_foundation::NSNotification) {
        if let Some(weak_view) = self.ivars().outline_view.borrow().as_ref() {
            if let Some(outline_view) = weak_view.load() {
                let node: objc2::rc::Retained<crate::workspace::FileNode> = unsafe { objc2::msg_send![notification, object] };
                
                let mut is_root = false;
                if let Some(root) = self.ivars().root_node.borrow().as_ref() {
                    if std::ptr::eq(&**root, &*node) {
                        is_root = true;
                    }
                }
                
                if is_root {
                    unsafe { outline_view.reloadItem_reloadChildren(None, true) };
                } else {
                    let node_any = unsafe { objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(node) };
                    unsafe { outline_view.reloadItem_reloadChildren(Some(&node_any), true) };
                    
                    // We also want to expand the node if it was previously loading.
                    // Wait, NSOutlineView might not automatically expand if we just reload.
                    // We can tell it to expand. But let's just do reloadItem_reloadChildren for now.
                }
            }
        }
    }"""

wd = re.sub(r"    #\[unsafe\(method\(nodeDidLoadChildren:\)\)\].*?    \}", correct_method, wd, flags=re.DOTALL)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd)
