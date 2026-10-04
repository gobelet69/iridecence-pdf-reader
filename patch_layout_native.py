import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# Replace the layout logic
start_marker = "let explorer_vc = NSViewController::new(mtm);"
end_marker = "split_vc.addSplitViewItem(&navigation_item);\n            }"

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
            unsafe { navigation_vc.setView(&thumb_view) };
            let navigation_item = unsafe { NSSplitViewItem::inspectorWithViewController(&navigation_vc) };
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
    print("Patched layout order using inspector successfully!")
else:
    print("Could not find markers.")

