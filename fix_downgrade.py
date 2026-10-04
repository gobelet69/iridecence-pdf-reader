with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()
wd = wd.replace("pub fn set_outline_view(&self, view: &NSOutlineView) {", "pub fn set_outline_view(&self, view: &objc2::rc::Retained<NSOutlineView>) {")
with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd)
