import re
with open('src/main.rs', 'r') as f:
    content = f.read()

divider_code = """
            let split_view = unsafe { split_vc.splitView() };
            unsafe { split_view.setDividerStyle(objc2_app_kit::NSSplitViewDividerStyle::Thin) };
            
            unsafe { window.setContentViewController(Some(&*self.ivars().split_vc.get().unwrap())) };
"""

content = content.replace("unsafe { window.setContentViewController(Some(&*self.ivars().split_vc.get().unwrap())) };", divider_code)

with open('src/main.rs', 'w') as f:
    f.write(content)
