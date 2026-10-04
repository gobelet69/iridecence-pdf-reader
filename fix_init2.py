with open('src/main.rs', 'r') as f:
    content = f.read()

import re
pattern = r"fn new\(mtm: objc2::MainThreadMarker\) -> objc2::rc::Retained<Self> \{.*?    \}"
new_func = """fn new(mtm: objc2::MainThreadMarker) -> objc2::rc::Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        unsafe { objc2::msg_send_id![super(this), initWithNibName: None::<&objc2_foundation::NSString>, bundle: None::<&objc2_foundation::NSBundle>] }
    }"""
content = re.sub(pattern, new_func, content, flags=re.DOTALL)

with open('src/main.rs', 'w') as f:
    f.write(content)
