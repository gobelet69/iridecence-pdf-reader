import re

with open("src/workspace_delegate.rs", "r") as f:
    code = f.read()

# Fix msg_send! -> msg_send_id! where it returns a Retained
code = re.sub(r"let node: objc2::rc::Retained<crate::workspace::FileNode> = unsafe \{ objc2::msg_send!\[notification, object\] \};",
              r"let node: Option<objc2::rc::Retained<crate::workspace::FileNode>> = unsafe { objc2::msg_send_id![notification, object] };\n                let node = node.unwrap();", code)
code = re.sub(r"let outline_view: objc2::rc::Retained<NSOutlineView> = unsafe \{ objc2::msg_send!\[notification, object\] \};",
              r"let outline_view: Option<objc2::rc::Retained<NSOutlineView>> = unsafe { objc2::msg_send_id![notification, object] };\n        let outline_view = outline_view.unwrap();", code)
code = re.sub(r"let target: Option<objc2::rc::Retained<objc2::runtime::AnyObject>> = unsafe \{ objc2::msg_send!\[&outline_view, target\] \};",
              r"let target: Option<objc2::rc::Retained<objc2::runtime::AnyObject>> = unsafe { objc2::msg_send_id![&outline_view, target] };", code)
code = re.sub(r"let cloud_iv: Option<Retained<objc2_app_kit::NSImageView>> = msg_send!\[&table_cell, viewWithTag: 101isize\];",
              r"let cloud_iv: Option<Retained<objc2_app_kit::NSImageView>> = msg_send_id![&table_cell, viewWithTag: 101isize];", code)
code = re.sub(r"let progress: Option<Retained<objc2_app_kit::NSProgressIndicator>> = msg_send!\[&table_cell, viewWithTag: 102isize\];",
              r"let progress: Option<Retained<objc2_app_kit::NSProgressIndicator>> = msg_send_id![&table_cell, viewWithTag: 102isize];", code)

# Init method should use msg_send_id! as well!
code = re.sub(r"let this: Retained<Self> = unsafe \{ msg_send!\[super\(this\), init\] \};",
              r"let this: Retained<Self> = unsafe { objc2::msg_send_id![super(this), init] };", code)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(code)
