with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()

# Replace the last } with the new method
wd = wd[:-2] + """
    pub fn set_outline_view(&self, view: &NSOutlineView) {
        *self.ivars().outline_view.borrow_mut() = Some(view.downgrade());
        unsafe {
            let nc = objc2_foundation::NSNotificationCenter::defaultCenter();
            let name = objc2_foundation::ns_string!("FileNodeDidLoadChildren");
            nc.addObserver_selector_name_object(self, objc2::sel!(nodeDidLoadChildren:), Some(name), None);
        }
    }
}
"""

with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd)

with open("src/main.rs", "r") as f:
    main = f.read()

main = main.replace(
    "let outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };",
    "let outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };\n            workspace_del.set_outline_view(&outline_view);"
)

with open("src/main.rs", "w") as f:
    f.write(main)
