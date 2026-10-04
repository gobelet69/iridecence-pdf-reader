import re
import sys

def main():
    with open('src/main.rs', 'r') as f:
        content = f.read()

    # 1. Ivars
    ivars_pattern = r"(pub struct AppDelegateIvars \{[^\}]+)thumb_view: OnceCell<Retained<PDFThumbnailView>>,"
    ivars_replacement = r"\1thumb_view: OnceCell<Retained<PDFThumbnailView>>,\n    explorer_item: OnceCell<Retained<NSSplitViewItem>>,\n    navigation_item: OnceCell<Retained<NSSplitViewItem>>,"
    content = re.sub(ivars_pattern, ivars_replacement, content)

    # 2. Add visual effect import
    if "NSVisualEffectView" not in content[:500]:
        content = content.replace("use objc2_app_kit::{", "use objc2_app_kit::{NSVisualEffectView, ")

    # 3. Replace SidebarTabs with ToggleExplorer and ToggleNavigation in toolbar arrays
    content = content.replace('objc2_foundation::ns_string!("SidebarTabs")', 'objc2_foundation::ns_string!("ToggleExplorer"), objc2_foundation::ns_string!("ToggleNavigation")')
    
    # 4. Remove sidebar_tabs_id definition
    content = content.replace('let sidebar_tabs_id = objc2_foundation::ns_string!("SidebarTabs");', '')

    # 5. toolbar_itemForItemIdentifier
    toolbar_item_pattern = r"(if item_identifier\.isEqualToString\(search_id\) \{.*?return Some\(item\);\s*\})\s*if item_identifier\.isEqualToString.*?segmentedControlWithLabels_trackingMode_target_action.*?return Some\(item\);\s*\}"
    
    toolbar_item_replacement = r"""\1
            if item_identifier.isEqualToString(objc2_foundation::ns_string!("ToggleExplorer")) {
                let target: &objc2::runtime::AnyObject = unsafe { &*(self as *const AppDelegate as *const AnyObject) };
                let image = unsafe { NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("sidebar.left"), None) };
                let item = unsafe { NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier) };
                unsafe {
                    let button = NSButton::buttonWithImage_target_action(image.as_deref().unwrap(), Some(target), Some(objc2::sel!(toggleExplorer:)), mtm);
                    button.setBezelStyle(objc2_app_kit::NSBezelStyle::TexturedRounded);
                    item.setLabel(objc2_foundation::ns_string!("Explorer"));
                    item.setPaletteLabel(objc2_foundation::ns_string!("Toggle Explorer"));
                    item.setToolTip(objc2_foundation::ns_string!("Toggle Workspace Explorer"));
                    item.setView(Some(&button));
                    item.setNavigational(true);
                }
                return Retained::into_raw(item);
            }
            if item_identifier.isEqualToString(objc2_foundation::ns_string!("ToggleNavigation")) {
                let target: &objc2::runtime::AnyObject = unsafe { &*(self as *const AppDelegate as *const AnyObject) };
                let image = unsafe { NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("sidebar.squares.left"), None) };
                let item = unsafe { NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier) };
                unsafe {
                    let button = NSButton::buttonWithImage_target_action(image.as_deref().unwrap(), Some(target), Some(objc2::sel!(toggleNavigation:)), mtm);
                    button.setBezelStyle(objc2_app_kit::NSBezelStyle::TexturedRounded);
                    item.setLabel(objc2_foundation::ns_string!("Navigation"));
                    item.setPaletteLabel(objc2_foundation::ns_string!("Toggle Navigation"));
                    item.setToolTip(objc2_foundation::ns_string!("Toggle Document Navigation"));
                    item.setView(Some(&button));
                    item.setNavigational(true);
                }
                return Retained::into_raw(item);
            }"""
    content = re.sub(toolbar_item_pattern, toolbar_item_replacement, content, flags=re.DOTALL)

    # 6. Replace toggleSidebar: and viewModeChanged: methods
    methods_pattern = r"\#\[unsafe\(method\(toggleSidebar:\)\)\].*?\#\[unsafe\(method\(viewModeChanged:\)\)\].*?\}\n\s*\}\n\s*\}"
    methods_replacement = """#[unsafe(method(toggleExplorer:))]
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
    content = re.sub(methods_pattern, methods_replacement, content, flags=re.DOTALL)

    # 7. Split view layout
    split_creation_pattern = r"let target_obj.*?split_vc\.addSplitViewItem\(&main_item\);\s*\}"
    split_creation_replacement = """
            let explorer_vc = NSViewController::new(mtm);
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
    content = re.sub(split_creation_pattern, split_creation_replacement, content, flags=re.DOTALL)

    with open('src/main.rs', 'w') as f:
        f.write(content)

main()
