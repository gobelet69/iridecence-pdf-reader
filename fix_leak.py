with open('src/main.rs', 'r') as f:
    content = f.read()

leak_logic = """            // Re-retain for the main thread block
            let i_ptr2 = objc2::rc::Retained::into_raw(retained_item.clone()) as *mut AnyObject;
            let o_ptr = outline_raw as *mut NSOutlineView;
            let _: () = msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr2, waitUntilDone: false];"""

fixed_logic = """            let o_ptr = outline_raw as *mut NSOutlineView;
            let i_ptr2 = &*retained_item as *const AnyObject as *mut AnyObject;
            let _: () = msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr2, waitUntilDone: false];"""

content = content.replace(leak_logic, fixed_logic)

with open('src/main.rs', 'w') as f:
    f.write(content)
