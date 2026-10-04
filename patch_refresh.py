with open('src/workspace_delegate.rs', 'r') as f:
    content = f.read()

import re

# We will inject the async refresh block in `outlineView_viewForTableColumn_item`
# Right after:
# let is_dir = node.ivars().is_dir.get();
# let icloud_status = node.ivars().icloud_status.get();

insert_point = """let is_dir = node.ivars().is_dir.get();
                let icloud_status = node.ivars().icloud_status.get();"""

refresh_logic = """let is_dir = node.ivars().is_dir.get();
                let icloud_status = node.ivars().icloud_status.get();
                
                let now = unsafe { objc2_foundation::NSDate::date().timeIntervalSince1970() };
                let last = node.ivars().last_status_check.get();
                // If cloud status is not local, or if it hasn't been checked in 3 seconds
                if now - last > 3.0 && !is_dir {
                    node.ivars().last_status_check.set(now);
                    let url_str = node.ivars().path.borrow().clone();
                    
                    let item_retained = objc2::rc::Retained::retain(item).unwrap();
                    let item_raw = objc2::rc::Retained::into_raw(item_retained) as usize;
                    let outline_raw = outline_view as *const objc2_app_kit::NSOutlineView as usize;
                    let node_ptr = node as *const crate::workspace::FileNode as usize;
                    
                    std::thread::spawn(move || {
                        crate::refresh_file_status(url_str, item_raw, outline_raw, node_ptr);
                    });
                }"""

if refresh_logic not in content:
    content = content.replace(insert_point, refresh_logic)

with open('src/workspace_delegate.rs', 'w') as f:
    f.write(content)
