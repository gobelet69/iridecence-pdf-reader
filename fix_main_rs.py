with open("src/main.rs", "r") as f:
    main = f.read()

main = main.replace(
    "let outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };",
    "let outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };\n            workspace_del.set_outline_view(&outline_view);"
)

with open("src/main.rs", "w") as f:
    f.write(main)
