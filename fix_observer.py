import re

# 1. Update workspace_delegate.rs
with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()

new_impl = """impl WorkspaceDelegate {

    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let mut this = Self::alloc(mtm).set_ivars(WorkspaceDelegateIvars::default());
        let mut this: Retained<Self> = unsafe { msg_send![super(this), init] };
        this
    }

    pub fn set_outline_view(&self, view: &NSOutlineView) {
        *self.ivars().outline_view.borrow_mut() = Some(view.downgrade());
        unsafe {
            let nc = objc2_foundation::NSNotificationCenter::defaultCenter();
            let name = objc2_foundation::ns_string!("FileNodeDidLoadChildren");
            nc.addObserver_selector_name_object(self, objc2::sel!(nodeDidLoadChildren:), Some(name), None);
        }
    }
}"""
wd = re.sub(r"impl WorkspaceDelegate \{.*?\}", new_impl, wd, flags=re.DOTALL)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd)

# 2. Update main.rs
with open("src/main.rs", "r") as f:
    main = f.read()

main = main.replace(
    "let outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };",
    "let outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };\n            workspace_del.set_outline_view(&outline_view);"
)

with open("src/main.rs", "w") as f:
    f.write(main)

