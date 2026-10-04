with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()

wd = wd.replace("unsafe { outline_view.expandItem(&node_any) };", "unsafe { outline_view.expandItem(Some(&node_any)) };")
wd = wd.replace("item as *const objc2::runtime::AnyObject", "&*item as *const objc2::runtime::AnyObject")

with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd)
