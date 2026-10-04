import sys

def main():
    with open('src/main.rs', 'r') as f:
        content = f.read()
        
    # 1. Add missing import
    if "NSVisualEffectView" not in content[:500]:
        content = content.replace("use objc2_app_kit::{", "use objc2_app_kit::{NSVisualEffectView, ")

    # 2. Update ivars
    old_ivars = """pub struct AppDelegateIvars {
    window: OnceCell<Retained<NSWindow>>,
    split_vc: OnceCell<Retained<NSSplitViewController>>,
    pdf_view: OnceCell<Retained<PDFView>>,
    workspace_delegate: OnceCell<Retained<WorkspaceDelegate>>,
    outline_scroll: OnceCell<Retained<NSScrollView>>,
    thumb_view: OnceCell<Retained<PDFThumbnailView>>,
}"""
    new_ivars = """pub struct AppDelegateIvars {
    window: OnceCell<Retained<NSWindow>>,
    split_vc: OnceCell<Retained<NSSplitViewController>>,
    pdf_view: OnceCell<Retained<PDFView>>,
    workspace_delegate: OnceCell<Retained<WorkspaceDelegate>>,
    outline_scroll: OnceCell<Retained<NSScrollView>>,
    thumb_view: OnceCell<Retained<PDFThumbnailView>>,
    explorer_item: OnceCell<Retained<NSSplitViewItem>>,
    navigation_item: OnceCell<Retained<NSSplitViewItem>>,
}"""
    content = content.replace(old_ivars, new_ivars)
    
    # 3. Replace toolbar items
    old_toolbar_item = """            let sidebar_tabs_id = objc2_foundation::ns_string!("SidebarTabs");
            
            if item_identifier.isEqualToString(sidebar_tabs_id) {
                let item = unsafe { NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier) };
                
                let target_obj: &AnyObject = unsafe { &*(self as *const AppDelegate as *const AnyObject) };
                let segmented_control = unsafe { NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action(
                    &objc2_foundation::NSArray::from_slice(&[
                        objc2_foundation::ns_string!("Workspace"),
                        objc2_foundation::ns_string!("Document")
                    ]),
                    objc2_app_kit::NSSegmentSwitchTrackingSelectOne,
                    Some(target_obj),
                    Some(objc2::sel!(viewModeChanged:)),
                    mtm
                ) };
                unsafe {
                    segmented_control.setSelected_forSegment(true, 0);
                    item.setLabel(objc2_foundation::ns_string!("View Mode"));
                    item.setPaletteLabel(objc2_foundation::ns_string!("View Mode"));
                    item.setToolTip(objc2_foundation::ns_string!("Switch between Workspace and Document views"));
                    item.setView(Some(&segmented_control));
                }
                return Retained::into_raw(item);
            }"""
    
    new_toolbar_item = """            let toggle_explorer_id = objc2_foundation::ns_string!("ToggleExplorer");
            let toggle_navigation_id = objc2_foundation::ns_string!("ToggleNavigation");
            
            if item_identifier.isEqualToString(toggle_explorer_id) {
                let item = unsafe { NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier) };
                let target_obj: &AnyObject = unsafe { &*(self as *const AppDelegate as *const AnyObject) };
                let image = unsafe { NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("sidebar.left"), None) };
                unsafe {
                    let button = NSButton::buttonWithImage_target_action(image.as_deref().unwrap(), Some(target_obj), Some(objc2::sel!(toggleExplorer:)), mtm);
                    button.setBezelStyle(objc2_app_kit::NSBezelStyle::TexturedRounded);
                    item.setLabel(objc2_foundation::ns_string!("Explorer"));
                    item.setPaletteLabel(objc2_foundation::ns_string!("Toggle Explorer"));
                    item.setToolTip(objc2_foundation::ns_string!("Toggle Workspace Explorer"));
                    item.setView(Some(&button));
                    item.setNavigational(true);
                }
                return Retained::into_raw(item);
            }
            
            if item_identifier.isEqualToString(toggle_navigation_id) {
                let item = unsafe { NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier) };
                let target_obj: &AnyObject = unsafe { &*(self as *const AppDelegate as *const AnyObject) };
                let image = unsafe { NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("sidebar.squares.left"), None) };
                unsafe {
                    let button = NSButton::buttonWithImage_target_action(image.as_deref().unwrap(), Some(target_obj), Some(objc2::sel!(toggleNavigation:)), mtm);
                    button.setBezelStyle(objc2_app_kit::NSBezelStyle::TexturedRounded);
                    item.setLabel(objc2_foundation::ns_string!("Navigation"));
                    item.setPaletteLabel(objc2_foundation::ns_string!("Toggle Navigation"));
                    item.setToolTip(objc2_foundation::ns_string!("Toggle Document Navigation"));
                    item.setView(Some(&button));
                    item.setNavigational(true);
                }
                return Retained::into_raw(item);
            }"""
    content = content.replace(old_toolbar_item, new_toolbar_item)
    
    # 4. Replace arrays
    content = content.replace(
        'objc2_foundation::ns_string!("SidebarTabs"),',
        'objc2_foundation::ns_string!("ToggleExplorer"), objc2_foundation::ns_string!("ToggleNavigation"),'
    )
    
    # 5. Replace methods
    old_methods = """        #[unsafe(method(toggleSidebar:))]
        fn toggle_sidebar(&self, _sender: Option<&AnyObject>) {
            if let Some(split_vc) = self.ivars().split_vc.get() {
                let items = unsafe { split_vc.splitViewItems() };
                if let Some(sidebar_item) = items.first() {
                    let current_collapsed = unsafe { sidebar_item.isCollapsed() };
                    unsafe { sidebar_item.setCollapsed(!current_collapsed) };
                }
            }
        }
        
        #[unsafe(method(viewModeChanged:))]
        fn view_mode_changed(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(seg) = sender.downcast_ref::<objc2_app_kit::NSSegmentedControl>() {
                    let index = unsafe { seg.selectedSegment() };
                    let outline_scroll = self.ivars().outline_scroll.get().unwrap();
                    let thumb_view = self.ivars().thumb_view.get().unwrap();
                    if index == 0 {
                        unsafe { outline_scroll.setHidden(false) };
                        unsafe { thumb_view.setHidden(true) };
                    } else {
                        unsafe { outline_scroll.setHidden(true) };
                        unsafe { thumb_view.setHidden(false) };
                    }
                }
            }
        }"""
    new_methods = """        #[unsafe(method(toggleExplorer:))]
        fn toggle_explorer(&self, _sender: Option<&AnyObject>) {
            if let Some(item) = self.ivars().explorer_item.get() {
                let current_collapsed = unsafe { item.isCollapsed() };
                unsafe { item.setCollapsed(!current_collapsed) };
            }
        }
        
        #[unsafe(method(toggleNavigation:))]
        fn toggle_navigation(&self, _sender: Option<&AnyObject>) {
            if let Some(item) = self.ivars().navigation_item.get() {
                let current_collapsed = unsafe { item.isCollapsed() };
                unsafe { item.setCollapsed(!current_collapsed) };
            }
        }"""
    content = content.replace(old_methods, new_methods)
    
    # 6. Replace layout setup
    old_layout = """            let target_obj: &AnyObject = unsafe { &*(self as *const AppDelegate as *const AnyObject) };
            unsafe { thumb_view.setHidden(true) };

            let sidebar_view = unsafe { NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 600.0))) };
            unsafe {
                outline_scroll.setFrame(sidebar_view.bounds());
                outline_scroll.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                sidebar_view.addSubview(&outline_scroll);
                
                thumb_view.setFrame(sidebar_view.bounds());
                thumb_view.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                sidebar_view.addSubview(&thumb_view);
            }

            let sidebar_vc = NSViewController::new(mtm);
            unsafe { sidebar_vc.setView(&sidebar_view) };
            let sidebar_item = unsafe { NSSplitViewItem::sidebarWithViewController(&sidebar_vc) };
            unsafe {
                sidebar_item.setCanCollapse(true);
                sidebar_item.setMinimumThickness(150.0);
                sidebar_item.setMaximumThickness(400.0);
            }
            
            self.ivars().outline_scroll.set(outline_scroll).unwrap();
            self.ivars().thumb_view.set(thumb_view).unwrap();

            unsafe {
                split_vc.addSplitViewItem(&sidebar_item);
                split_vc.addSplitViewItem(&main_item);
            }"""
    
    new_layout = """            let explorer_vc = NSViewController::new(mtm);
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
    content = content.replace(old_layout, new_layout)
    
    with open('src/main.rs', 'w') as f:
        f.write(content)

main()
