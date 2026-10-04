import re

with open("src/workspace_delegate.rs", "r") as f:
    content = f.read()

# Fix upgrade() -> load()
content = content.replace(".upgrade()", ".load()")

# Fix node.retain() -> use Retained::retain
# The line was: let node_ptr = objc2::rc::Retained::into_raw(node.retain()) as usize;
content = content.replace("objc2::rc::Retained::into_raw(node.retain())", "objc2::rc::Retained::into_raw(objc2::rc::Retained::retain(node as *const _ as *mut _).unwrap())")

# Add mtm = MainThreadMarker::new().unwrap(); before using mtm in the thread
if "let weak_ov2 = outline_view_weak.clone();" in content:
    content = content.replace("let weak_ov2 = outline_view_weak.clone();", "let weak_ov2 = outline_view_weak.clone();\n                                            let mtm = objc2_foundation::MainThreadMarker::new().unwrap();")

with open("src/workspace_delegate.rs", "w") as f:
    f.write(content)
