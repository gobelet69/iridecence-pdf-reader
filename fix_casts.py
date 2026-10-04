with open('src/main.rs', 'r') as f:
    content = f.read()

content = content.replace(
    'let item_retained = unsafe { objc2::rc::Retained::retain(item as *const _ as *mut _) }.unwrap();',
    'let item_retained = unsafe { objc2::rc::Retained::retain(&*item as *const _ as *mut _) }.unwrap();'
)

content = content.replace(
    'let item_raw = objc2::rc::Retained::into_raw(item_retained) as usize;',
    'let item_raw = objc2::rc::Retained::into_raw(item_retained) as *mut objc2::runtime::AnyObject as usize;'
)

with open('src/main.rs', 'w') as f:
    f.write(content)
