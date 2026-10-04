import re
with open('src/main.rs', 'r') as f:
    content = f.read()

content = content.replace("struct MainSplitViewController;", "#[ivars = ()]\n    struct MainSplitViewController;")
content = content.replace("objc2::rc::MainThreadMarker", "objc2::MainThreadMarker")
content = content.replace("let this = Self::alloc(mtm);", "let this = Self::alloc(mtm).set_ivars(());")

with open('src/main.rs', 'w') as f:
    f.write(content)
