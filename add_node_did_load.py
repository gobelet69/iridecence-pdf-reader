import re
with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()

method = """
    #[unsafe(method(nodeDidLoadChildren:))]
    fn nodeDidLoadChildren(&self, notification: &objc2_foundation::NSNotification) {
        if let Some(weak_view) = self.ivars().outline_view.borrow().as_ref() {
            if let Some(outline_view) = weak_view.load() {
                let node: objc2::rc::Retained<crate::workspace::FileNode> = unsafe { objc2::msg_send![notification, object] };
                let node_any = unsafe { objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(node) };
                unsafe { outline_view.reloadItem_reloadChildren(Some(&node_any), true) };
            }
        }
    }
"""

wd = wd.replace("    #[unsafe(method(outlineViewSelectionDidChange:))]", method + "\n    #[unsafe(method(outlineViewSelectionDidChange:))]")
with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd)
