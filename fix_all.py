import re

# 1. Fix workspace.rs
with open("src/workspace.rs", "r") as f:
    ws = f.read()

ws = ws.replace("node.ivars().path.borrow_mut() = path;", "*node.ivars().path.borrow_mut() = path;")
ws = ws.replace("node.ivars().name.borrow_mut() = name;", "*node.ivars().name.borrow_mut() = name;")
ws = ws.replace("use objc2::{define_class, msg_send, ClassType, DefinedClass};", "use objc2::{define_class, msg_send, ClassType, DefinedClass, Message};")
with open("src/workspace.rs", "w") as f:
    f.write(ws)

# 2. Fix workspace_delegate.rs duplicate nodeDidLoadChildren
with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()

# I added nodeDidLoadChildren to the second impl WorkspaceDelegate. I need to move it inside define_class!
import_str = """    #[unsafe(method(nodeDidLoadChildren:))]
    fn nodeDidLoadChildren(&self, notification: &objc2_foundation::NSNotification) {
        if let Some(weak_view) = self.ivars().outline_view.borrow().as_ref() {
            if let Some(outline_view) = weak_view.load() {
                let node: Retained<FileNode> = unsafe { msg_send![notification, object] };
                let node_any = unsafe { objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(node) };
                unsafe { outline_view.reloadItem_reloadChildren(Some(&node_any), true) };
            }
        }
    }"""

# Remove it from the second impl WorkspaceDelegate
wd = wd.replace(import_str, "")

# Insert it inside define_class! right before } );
# But wait, it might be easier to just add it inside define_class! manually.
wd = wd.replace("        #[unsafe(method(outlineViewSelectionDidChange:))]", import_str + "\n        #[unsafe(method(outlineViewSelectionDidChange:))]")

with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd)

