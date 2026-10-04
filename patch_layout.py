import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# We want to replace everything from `let navigation_vc = NSViewController::new(mtm);` 
# up to `split_vc.addSplitViewItem(&main_item);`

start_marker = "let navigation_vc = NSViewController::new(mtm);"
end_marker = "split_vc.addSplitViewItem(&main_item);\n            }"

start_idx = content.find(start_marker)
end_idx = content.find(end_marker) + len(end_marker)

if start_idx != -1 and end_idx != -1:
    new_code = """let explorer_vc = NSViewController::new(mtm);
            unsafe { explorer_vc.setView(&outline_scroll) };
            let explorer_item = unsafe { NSSplitViewItem::sidebarWithViewController(&explorer_vc) };
            unsafe {
                explorer_item.setCanCollapse(true);
                explorer_item.setMinimumThickness(180.0);
                explorer_item.setMaximumThickness(420.0);
            }

            let navigation_vc = NSViewController::new(mtm);
            let nav_effect_view = unsafe { NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(190.0, 700.0))) };
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
                navigation_item.setMaximumThickness(320.0);
            }
            
            self.ivars().explorer_item.set(explorer_item.clone()).unwrap();
            self.ivars().navigation_item.set(navigation_item.clone()).unwrap();
            self.ivars().outline_scroll.set(outline_scroll).unwrap();
            self.ivars().thumb_view.set(thumb_view).unwrap();

            unsafe {
                split_vc.addSplitViewItem(&explorer_item);
                split_vc.addSplitViewItem(&main_item);
                split_vc.addSplitViewItem(&navigation_item);
            }"""
    
    content = content[:start_idx] + new_code + content[end_idx:]
    with open('src/main.rs', 'w') as f:
        f.write(content)
    print("Patched layout order successfully!")
else:
    print("Could not find markers.")

