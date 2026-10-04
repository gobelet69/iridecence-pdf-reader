with open('src/workspace_delegate.rs', 'r') as f:
    content = f.read()

content = content.replace(
    'crate::download_file(url_str, item_raw, outline_raw, node_ptr);',
    'crate::download_file(url_str, item_raw, outline_raw, node_ptr, 0);'
)

with open('src/workspace_delegate.rs', 'w') as f:
    f.write(content)

with open('src/main.rs', 'r') as f:
    content2 = f.read()

content2 = content2.replace(
    'let item_retained = objc2::rc::Retained::retain(item).unwrap();',
    'let item_retained = unsafe { objc2::rc::Retained::retain(item as *const _ as *mut _) }.unwrap();'
)

with open('src/main.rs', 'w') as f:
    f.write(content2)
