import re
with open('src/main.rs', 'r') as f:
    content = f.read()

# I will replace MainSplitViewController::new to use initWithNibName
new_func = """
    fn new(mtm: objc2::MainThreadMarker) -> objc2::rc::Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        unsafe {
            let vc: objc2::rc::Retained<Self> = objc2::msg_send_id![super(this), initWithNibName: None::<&objc2_foundation::NSString>, bundle: None::<&objc2_foundation::NSBundle>];
            vc
        }
    }
"""

content = re.sub(r'fn new\(mtm: objc2::MainThreadMarker\).*?\}', new_func, content, flags=re.DOTALL)

with open('src/main.rs', 'w') as f:
    f.write(content)
