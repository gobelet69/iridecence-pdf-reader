import re

with open("src/workspace_delegate.rs", "r") as f:
    code = f.read()

replacement = """
        if let Some(view_weak) = self.ivars().outline_view.borrow().as_ref() {
            if let Some(view) = view_weak.load() {
                let is_root = if let Some(root) = self.ivars().root_node.borrow().as_ref() {
                    let root_ptr: *const _ = &**root;
                    let node_ptr: *const _ = &*node;
                    root_ptr == node_ptr
                } else {
                    false
                };
                
                if is_root {
                    unsafe { view.reloadItem_reloadChildren(None, true) };
                } else {
                    unsafe { view.reloadItem_reloadChildren(Some(&node), true) };
                }
            }
        }
"""

code = re.sub(r"        if let Some\(view_weak\) = self\.ivars\(\)\.outline_view\.borrow\(\)\.as_ref\(\) \{\n            if let Some\(view\) = view_weak\.load\(\) \{\n                unsafe \{ view\.reloadItem_reloadChildren\(Some\(&node\), true\) \};\n            \}\n        \}", replacement.strip('\n'), code)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(code)
