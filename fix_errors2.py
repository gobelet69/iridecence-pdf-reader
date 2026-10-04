import re

with open("src/workspace_delegate.rs", "r") as f:
    content = f.read()

content = content.replace("let weak_ov = outline_view_weak.clone();", "let mtm = objc2_foundation::MainThreadMarker::new().unwrap();\n                                            let weak_ov = outline_view_weak.clone();")

with open("src/workspace_delegate.rs", "w") as f:
    f.write(content)
