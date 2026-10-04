with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()

import re

# We will just rewrite the methods.
methods_str = """
    #[unsafe(method(nodeDidLoadChildren:))]
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
                    unsafe { outline_view.expandItem(&node_any) }; // Keep expanded if it was expanded
                }
            }
        }
    }

    #[unsafe(method(outlineViewSelectionDidChange:))]
    fn outlineViewSelectionDidChange(&self, notification: &objc2_foundation::NSNotification) {
        let outline_view: objc2::rc::Retained<NSOutlineView> = unsafe { objc2::msg_send![notification, object] };
        let selected_row = unsafe { outline_view.selectedRow() };
        if selected_row >= 0 {
            if let Some(item) = unsafe { outline_view.itemAtRow(selected_row) } {
                let node: &crate::workspace::FileNode = unsafe { &*(item as *const objc2::runtime::AnyObject as *const crate::workspace::FileNode) };
                if node.ivars().is_loading_placeholder.get() {
                    return;
                }
            }
        }
        
        let target: Option<objc2::rc::Retained<objc2::runtime::AnyObject>> = unsafe { objc2::msg_send![&outline_view, target] };
        if let Some(target) = target {
            let _: () = unsafe { objc2::msg_send![&target, workspaceSelectionChanged: &*outline_view] };
        }
    }
"""

start_idx = wd.find("    #[unsafe(method(nodeDidLoadChildren:))]")
end_idx = wd.find("        #[unsafe(method(outlineView:numberOfChildrenOfItem:))]")

if start_idx != -1 and end_idx != -1:
    wd = wd[:start_idx] + methods_str + "\n" + wd[end_idx:]
    with open("src/workspace_delegate.rs", "w") as f:
        f.write(wd)
else:
    print("Could not find boundaries")
