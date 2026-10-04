with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()

wd = wd.replace(
    "fn nodeDidLoadChildren(&self, notification: &objc2_foundation::NSNotification) {",
    "fn nodeDidLoadChildren(&self, notification: &objc2_foundation::NSNotification) {\n        println!(\"nodeDidLoadChildren received!\");"
)
wd = wd.replace(
    "if let Some(outline_view) = weak_view.load() {",
    "if let Some(outline_view) = weak_view.load() {\n                println!(\"outline_view loaded!\");"
)
wd = wd.replace(
    "if is_root {",
    "println!(\"is_root: {}\", is_root);\n                if is_root {"
)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd)
