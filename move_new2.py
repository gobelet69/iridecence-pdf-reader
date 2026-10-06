import re

with open('src/debug_console.rs', 'r') as f:
    content = f.read()

pattern = r'(\s*fn new\(mtm: objc2::MainThreadMarker\) -> Retained<Self> \{.*?\s+this\n\s*\})'
match = re.search(pattern, content, re.DOTALL)
if match:
    new_method = match.group(1)
    content = content.replace(new_method, "")
    
    content = content.replace("unsafe impl objc2_foundation::NSObjectProtocol for DebugConsoleController {}\n", "unsafe impl objc2_foundation::NSObjectProtocol for DebugConsoleController {}\n\nimpl DebugConsoleController {" + new_method + "\n}")

    with open('src/debug_console.rs', 'w') as f:
        f.write(content)
