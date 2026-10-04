import re
with open('src/main.rs', 'r') as f:
    content = f.read()

loop_code = """
        let mut iterations = 0;
        loop {
            iterations += 1;
            if iterations > 120 {
                node.ivars().icloud_status.set(1); // Revert to cloud if timeout
                let o_ptr = outline_raw as *mut NSOutlineView;
                let i_ptr = item_raw as *mut AnyObject;
                let _: () = objc2::msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr, waitUntilDone: false];
                let _: objc2::rc::Retained<AnyObject> = objc2::rc::Retained::from_raw(i_ptr as *mut AnyObject).unwrap();
                break;
            }
"""

content = content.replace("        loop {", loop_code)
with open('src/main.rs', 'w') as f:
    f.write(content)
