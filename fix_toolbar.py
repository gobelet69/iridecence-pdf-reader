with open("src/main.rs", "r") as f:
    content = f.read()

old_items = """                    objc2_app_kit::NSToolbarToggleSidebarItemIdentifier,
                    objc2_app_kit::NSToolbarFlexibleSpaceItemIdentifier,
                    objc2_foundation::ns_string!("SidebarTabs"),
                    objc2_foundation::ns_string!("Search"),
                    objc2_foundation::ns_string!("ZoomOut"),
                    objc2_foundation::ns_string!("ZoomIn")"""

new_items = """                    objc2_app_kit::NSToolbarToggleSidebarItemIdentifier,
                    objc2_app_kit::NSToolbarFlexibleSpaceItemIdentifier,
                    objc2_foundation::ns_string!("SidebarTabs"),
                    objc2_app_kit::NSToolbarFlexibleSpaceItemIdentifier,
                    objc2_foundation::ns_string!("Search"),
                    objc2_foundation::ns_string!("ZoomOut"),
                    objc2_foundation::ns_string!("ZoomIn")"""

content = content.replace(old_items, new_items)

with open("src/main.rs", "w") as f:
    f.write(content)
