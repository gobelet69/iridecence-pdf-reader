import re

# Fix workspace_delegate.rs
with open("src/workspace_delegate.rs", "r") as f:
    code = f.read()

code = code.replace(
    "if let img = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!(\"icloud.and.arrow.down\"), None) {",
    "if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!(\"icloud.and.arrow.down\"), None) {"
)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(code)

# Fix main.rs
with open("src/main.rs", "r") as f:
    code = f.read()

code = code.replace(
    "let outline_raw = Retained::into_raw(outline.retain()) as usize;",
    "let outline_raw = Retained::into_raw(objc2::rc::Retained::retain(outline)) as usize;"
)

with open("src/main.rs", "w") as f:
    f.write(code)
