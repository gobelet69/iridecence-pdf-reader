import re

with open("src/main.rs", "r") as f:
    main = f.read()

# 1. Add the segmented control identifier to allowed and default items
main = main.replace(
    'objc2_app_kit::NSToolbarFlexibleSpaceItemIdentifier,\n                    objc2_foundation::ns_string!("Search"),',
    'objc2_app_kit::NSToolbarFlexibleSpaceItemIdentifier,\n                    objc2_foundation::ns_string!("SidebarTabs"),\n                    objc2_foundation::ns_string!("Search"),'
)

# 2. Add it to toolbar_item_for_item_identifier
toolbar_item_logic = """                let item = NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), &id);
                
                if id == *objc2_foundation::ns_string!("SidebarTabs") {
                    let segmented_control = NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action(
                        &NSArray::from_slice(&[
                            objc2_foundation::ns_string!("Workspace"),
                            objc2_foundation::ns_string!("Document")
                        ]),
                        objc2_app_kit::NSSegmentSwitchTracking::SelectOne,
                        Some(&*(self as *const AppDelegate as *const AnyObject)),
                        Some(objc2::sel!(switchSidebar:)),
                        mtm
                    );
                    segmented_control.setSelected_forSegment(true, 0);
                    item.setView(Some(&segmented_control));
                    
                    let min_size = NSSize::new(140.0, 24.0);
                    let max_size = NSSize::new(200.0, 24.0);
                    item.setMinSize(min_size);
                    item.setMaxSize(max_size);
                    return Retained::into_raw(item);
                }"""

main = re.sub(
    r"let item = NSToolbarItem::initWithItemIdentifier\(NSToolbarItem::alloc\(mtm\), &id\);",
    toolbar_item_logic,
    main
)

# 3. Remove the segmented_control from the sidebar stack
main = re.sub(
    r"let segmented_control = unsafe \{ NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action\([\s\S]*?unsafe \{ thumb_view\.setHidden\(true\) \};",
    "unsafe { thumb_view.setHidden(true) };",
    main
)

main = main.replace(
    "let sidebar_stack = unsafe { NSStackView::stackViewWithViews(&objc2_foundation::NSArray::from_slice(&[segmented_control.as_ref(), outline_scroll.as_ref(), thumb_view.as_ref()]), mtm) };",
    "let sidebar_stack = unsafe { NSStackView::stackViewWithViews(&objc2_foundation::NSArray::from_slice(&[outline_scroll.as_ref(), thumb_view.as_ref()]), mtm) };"
)

with open("src/main.rs", "w") as f:
    f.write(main)
