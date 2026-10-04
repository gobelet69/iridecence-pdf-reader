import re

with open("src/main.rs", "r") as f:
    content = f.read()

old = """            let sidebar_stack = unsafe { NSStackView::stackViewWithViews(&objc2_foundation::NSArray::from_slice(&[
                &*top_bar,
                &*outline_scroll
            ])) };
            unsafe { sidebar_stack.setOrientation(objc2_app_kit::NSUserInterfaceLayoutOrientation::Vertical) };
            unsafe { sidebar_stack.setSpacing(0.0) };
            
            let sidebar_vc = unsafe { NSViewController::initWithNibName_bundle(NSViewController::alloc(mtm), None, None) };
            unsafe { sidebar_vc.setView(&sidebar_stack) };"""

new = """            let sidebar_view = unsafe { NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 600.0))) };
            unsafe {
                sidebar_view.addSubview(&top_bar);
                sidebar_view.addSubview(&outline_scroll);
                top_bar.setFrame(NSRect::new(NSPoint::new(0.0, 556.0), NSSize::new(200.0, 44.0)));
                top_bar.setAutoresizingMask(objc2_app_kit::NSViewAutoresizing::MinYMargin | objc2_app_kit::NSViewAutoresizing::WidthSizable);
                outline_scroll.setFrame(NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 556.0)));
                outline_scroll.setAutoresizingMask(objc2_app_kit::NSViewAutoresizing::WidthSizable | objc2_app_kit::NSViewAutoresizing::HeightSizable);
            }
            
            let sidebar_vc = unsafe { NSViewController::initWithNibName_bundle(NSViewController::alloc(mtm), None, None) };
            unsafe { sidebar_vc.setView(&sidebar_view) };"""

content = content.replace(old, new)
with open("src/main.rs", "w") as f:
    f.write(content)
