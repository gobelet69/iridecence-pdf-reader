import re

with open('src/workspace_delegate.rs', 'r') as f:
    content = f.read()

freshness_code = """
            if !is_dir {
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64();
                if now - node.ivars().last_status_check.get() > 2.0 {
                    node.ivars().last_status_check.set(now);
                    let url_str = node.ivars().path.borrow().clone();
                    let outline_raw = outline_view as *const _ as usize;
                    
                    let item_raw = unsafe { 
                        let ptr = item as *const _ as *mut objc2::runtime::AnyObject;
                        objc2::rc::Retained::into_raw(objc2::rc::Retained::retain(ptr).unwrap()) as usize 
                    };
                    
                    let node_ptr = node as *const _ as usize;
                    std::thread::spawn(move || {
                        crate::refresh_file_status(url_str, item_raw, outline_raw, node_ptr);
                    });
                }
            }
            
            Retained::into_raw(cell)
"""

content = content.replace("            Retained::into_raw(cell)", freshness_code)

with open('src/workspace_delegate.rs', 'w') as f:
    f.write(content)
