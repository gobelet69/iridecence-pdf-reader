import re

with open('src/debug_console.rs', 'r') as f:
    content = f.read()

pattern = r'(\s*fn new\(mtm: objc2::MainThreadMarker\) -> Retained<Self> \{.*?\s+this\n\s*\})'
match = re.search(pattern, content, re.DOTALL)
if match:
    new_method = match.group(1)
    content = content.replace(new_method, "")
    
    # insert after define_class!
    pattern2 = r'(\);\nunsafe impl objc2_foundation::NSObjectProtocol for DebugConsoleController \{\})'
    replacement = r'\1\n\nimpl DebugConsoleController {' + new_method + '\n}'
    content = re.sub(pattern2, replacement, content)
    
    with open('src/debug_console.rs', 'w') as f:
        f.write(content)

