with open('src/main.rs', 'r') as f:
    content = f.read()

import re

# We remove the custom segmented control logic
custom_logic = """if item_identifier.isEqualToString(sidebar_tabs_id) {
                let item = unsafe { NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier) };
                
                let target_obj: &AnyObject = unsafe { &*(self as *const AppDelegate as *const AnyObject) };
                let segmented_control = unsafe { NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action(
                    &objc2_foundation::NSArray::from_slice(&[
                        objc2_foundation::ns_string!("Workspace"),
                        objc2_foundation::ns_string!("Document")
                    ]),
                    objc2_app_kit::NSSegmentSwitchTracking::SelectOne,
                    Some(target_obj),
                    Some(objc2::sel!(switchSidebar:)),
                    mtm
                ) };
                unsafe { segmented_control.setSelected_forSegment(true, 0) };
                unsafe { item.setView(Some(&segmented_control)) };
                
                let min_size = NSSize::new(140.0, 24.0);
                let max_size = NSSize::new(200.0, 24.0);
                unsafe { item.setMinSize(min_size) };
                unsafe { item.setMaxSize(max_size) };
                
                return Retained::into_raw(item);
            }"""

content = content.replace(custom_logic, "")

# We update the arrays
old_array = """let array = NSArray::from_slice(&[
                    objc2_app_kit::NSToolbarToggleSidebarItemIdentifier,
                    objc2_app_kit::NSToolbarFlexibleSpaceItemIdentifier,
                    objc2_foundation::ns_string!("ToggleExplorer"), objc2_foundation::ns_string!("ToggleNavigation"),
                    objc2_foundation::ns_string!("Search"),
                    objc2_foundation::ns_string!("ZoomOut"),
                    objc2_foundation::ns_string!("ZoomIn")
                ]);"""

new_array = """let array = NSArray::from_slice(&[
                    objc2_app_kit::NSToolbarToggleSidebarItemIdentifier,
                    objc2_app_kit::NSToolbarFlexibleSpaceItemIdentifier,
                    objc2_app_kit::NSToolbarToggleInspectorItemIdentifier,
                    objc2_foundation::ns_string!("Search"),
                    objc2_foundation::ns_string!("ZoomOut"),
                    objc2_foundation::ns_string!("ZoomIn")
                ]);"""

content = content.replace(old_array, new_array)

with open('src/main.rs', 'w') as f:
    f.write(content)
