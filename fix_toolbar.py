import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# Replace toolbar items logic
old_toolbar = """        #[unsafe(method(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
        fn toolbar_itemForItemIdentifier_willBeInsertedIntoToolbar(
            &self,
            _toolbar: &NSToolbar,
            item_identifier: &NSToolbarItemIdentifier,
            _flag: bool,
        ) -> *mut NSToolbarItem {"""

new_toolbar = """        #[unsafe(method(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
        fn toolbar_itemForItemIdentifier_willBeInsertedIntoToolbar(
            &self,
            _toolbar: &NSToolbar,
            item_identifier: &NSToolbarItemIdentifier,
            _flag: bool,
        ) -> *mut NSToolbarItem {
            let mtm = MainThreadMarker::new().unwrap();
            
            // Intercept system items to set tooltips and icons
            unsafe {
                if item_identifier == objc2_app_kit::NSToolbarToggleSidebarItemIdentifier {
                    let item = NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier);
                    let img = NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("sidebar.left"), None);
                    item.setImage(img.as_deref());
                    item.setToolTip(Some(objc2_foundation::ns_string!("Toggle Workspace Sidebar")));
                    item.setAction(Some(objc2::sel!(toggleSidebar:)));
                    return Retained::into_raw(item);
                } else if item_identifier == objc2_app_kit::NSToolbarToggleInspectorItemIdentifier {
                    let item = NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier);
                    let img = NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("sidebar.right"), None);
                    item.setImage(img.as_deref());
                    item.setToolTip(Some(objc2_foundation::ns_string!("Toggle Document Sidebar")));
                    item.setAction(Some(objc2::sel!(toggleInspector:)));
                    // In some newer macOS, action should be toggleInspector: on NSApp, but NSSplitViewController handles it if we target firstResponder.
                    return Retained::into_raw(item);
                }
            }
"""

content = content.replace(old_toolbar, new_toolbar)

with open('src/main.rs', 'w') as f:
    f.write(content)
