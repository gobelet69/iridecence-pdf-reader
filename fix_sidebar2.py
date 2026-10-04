import re

with open("src/main.rs", "r") as f:
    content = f.read()

old = """            let sidebar_stack = unsafe { NSStackView::stackViewWithViews(&objc2_foundation::NSArray::from_slice(&[outline_scroll.as_ref(), thumb_view.as_ref()]), mtm) };
            sidebar_stack.setOrientation(objc2_app_kit::NSUserInterfaceLayoutOrientation::Vertical);
            unsafe { sidebar_stack.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable) };
            unsafe { sidebar_stack.setSpacing(0.0) };

            let sidebar_vc = NSViewController::new(mtm);
            unsafe { sidebar_vc.setView(&sidebar_stack) };"""

new = """            let sidebar_view = unsafe { NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 600.0))) };
            unsafe {
                outline_scroll.setFrame(sidebar_view.bounds());
                outline_scroll.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                sidebar_view.addSubview(&outline_scroll);
                
                thumb_view.setFrame(sidebar_view.bounds());
                thumb_view.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                sidebar_view.addSubview(&thumb_view);
            }

            let sidebar_vc = NSViewController::new(mtm);
            unsafe { sidebar_vc.setView(&sidebar_view) };"""

content = content.replace(old, new)

with open("src/main.rs", "w") as f:
    f.write(content)
