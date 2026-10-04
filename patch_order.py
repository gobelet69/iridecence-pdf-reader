import re

with open('src/main.rs', 'r') as f:
    content = f.read()

old_layout = """            let explorer_vc = NSViewController::new(mtm);
            unsafe { explorer_vc.setView(&outline_scroll) };
            let explorer_item = unsafe { NSSplitViewItem::sidebarWithViewController(&explorer_vc) };
            unsafe {
                explorer_item.setCanCollapse(true);
                explorer_item.setMinimumThickness(150.0);
                explorer_item.setMaximumThickness(400.0);
            }

            let navigation_vc = NSViewController::new(mtm);
            let nav_effect_view = unsafe { NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };
            unsafe {
                nav_effect_view.setMaterial(objc2_app_kit::NSVisualEffectMaterial::Sidebar);
                nav_effect_view.setBlendingMode(objc2_app_kit::NSVisualEffectBlendingMode::BehindWindow);
                nav_effect_view.setState(objc2_app_kit::NSVisualEffectState::Active);
                thumb_view.setFrame(nav_effect_view.bounds());
                thumb_view.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                nav_effect_view.addSubview(&thumb_view);
                navigation_vc.setView(&nav_effect_view);
            }
            
            let navigation_item = unsafe { NSSplitViewItem::splitViewItemWithViewController(&navigation_vc) };
            unsafe {
                navigation_item.setCanCollapse(true);
                navigation_item.setMinimumThickness(150.0);
                navigation_item.setMaximumThickness(300.0);
            }
            
            self.ivars().explorer_item.set(explorer_item.clone()).unwrap();
            self.ivars().navigation_item.set(navigation_item.clone()).unwrap();
            self.ivars().outline_scroll.set(outline_scroll).unwrap();
            self.ivars().thumb_view.set(thumb_view).unwrap();

            unsafe {
                split_vc.addSplitViewItem(&explorer_item);
                split_vc.addSplitViewItem(&navigation_item);
                split_vc.addSplitViewItem(&main_item);
            }"""

new_layout = """            let navigation_vc = NSViewController::new(mtm);
            unsafe { navigation_vc.setView(&thumb_view) };
            let navigation_item = unsafe { NSSplitViewItem::sidebarWithViewController(&navigation_vc) };
            unsafe {
                navigation_item.setCanCollapse(true);
                navigation_item.setMinimumThickness(150.0);
                navigation_item.setMaximumThickness(300.0);
            }

            let explorer_vc = NSViewController::new(mtm);
            let exp_effect_view = unsafe { NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };
            unsafe {
                exp_effect_view.setMaterial(objc2_app_kit::NSVisualEffectMaterial::Sidebar);
                exp_effect_view.setBlendingMode(objc2_app_kit::NSVisualEffectBlendingMode::BehindWindow);
                exp_effect_view.setState(objc2_app_kit::NSVisualEffectState::Active);
                outline_scroll.setFrame(exp_effect_view.bounds());
                outline_scroll.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                exp_effect_view.addSubview(&outline_scroll);
                explorer_vc.setView(&exp_effect_view);
            }
            let explorer_item = unsafe { NSSplitViewItem::splitViewItemWithViewController(&explorer_vc) };
            unsafe {
                explorer_item.setCanCollapse(true);
                explorer_item.setMinimumThickness(150.0);
                explorer_item.setMaximumThickness(400.0);
            }
            
            self.ivars().explorer_item.set(explorer_item.clone()).unwrap();
            self.ivars().navigation_item.set(navigation_item.clone()).unwrap();
            self.ivars().outline_scroll.set(outline_scroll).unwrap();
            self.ivars().thumb_view.set(thumb_view).unwrap();

            unsafe {
                split_vc.addSplitViewItem(&navigation_item);
                split_vc.addSplitViewItem(&explorer_item);
                split_vc.addSplitViewItem(&main_item);
            }"""

if old_layout in content:
    content = content.replace(old_layout, new_layout)
    with open('src/main.rs', 'w') as f:
        f.write(content)
    print("Patched successfully!")
else:
    print("Old layout not found!")
