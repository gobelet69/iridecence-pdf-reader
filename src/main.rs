use std::sync::Mutex;
use std::collections::HashSet;
use once_cell::sync::Lazy;
pub static ACTIVE_DOWNLOADS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
use objc2_foundation::NSInteger;
use std::cell::OnceCell;
use objc2::rc::Retained;

#[derive(Debug, PartialEq, Eq)]
pub enum ICloudState {
    Local = 0,
    CloudOnly = 1,
    DownloadRequested = 2, // Also Downloading for now
    Failed = 4,
    Unknown = 99,
}
impl ICloudState {
    pub fn from_int(v: u32) -> Self {
        match v {
            0 => ICloudState::Local,
            1 => ICloudState::CloudOnly,
            2 => ICloudState::DownloadRequested,
            4 => ICloudState::Failed,
            _ => ICloudState::Unknown,
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            ICloudState::Local => "Local",
            ICloudState::CloudOnly => "CloudOnly",
            ICloudState::DownloadRequested => "Downloading",
            ICloudState::Failed => "Failed",
            ICloudState::Unknown => "Unknown",
        }
    }
}

use objc2::{define_class, msg_send, sel, ClassType, DeclaredClass, MainThreadOnly, AnyThread};
use objc2::runtime::{ProtocolObject, AnyObject};

mod workspace;
mod workspace_delegate;
mod logger;
mod debug_console;
use objc2_app_kit::{NSVisualEffectView, 
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSBackingStoreType, 
    NSWindow, NSWindowStyleMask, NSOpenPanel, NSModalResponseOK,
    NSSplitViewController, NSSplitView, NSSplitViewItem, NSViewController,
    NSToolbar, NSToolbarDelegate, NSToolbarItem, NSWindowToolbarStyle,
    NSGlassEffectView, NSGlassEffectViewStyle, NSToolbarItemIdentifier,
    NSToolbarDisplayMode, NSMenu, NSMenuItem, NSStackView, NSView, NSButton,
    NSUserInterfaceLayoutOrientation, NSBezelStyle, NSSegmentedControl, NSSegmentSwitchTracking,
    NSScrollView, NSOutlineView, NSTableColumn, NSOutlineViewDelegate, NSOutlineViewDataSource, NSImage, NSImageView, NSColor, NSTextField, NSImageSymbolConfiguration, NSImageSymbolScale
};
use objc2_foundation::{
    MainThreadMarker, NSObject, NSObjectProtocol, NSRect, NSPoint, NSSize, NSArray, NSString, NSURL, NSEdgeInsets
};
use objc2_pdf_kit::{PDFView, PDFDocument, PDFThumbnailView, PDFDisplayMode};
use crate::workspace_delegate::WorkspaceDelegate;




#[derive(Default)]
struct AppDelegateIvars {
    pending_url: std::cell::RefCell<Option<Retained<objc2_foundation::NSURL>>>,
    pending_pdf_path: std::cell::RefCell<String>,
    window: OnceCell<Retained<NSWindow>>,
    pdf_view: OnceCell<Retained<PDFView>>,
    split_vc: OnceCell<Retained<NSSplitViewController>>,
    workspace_delegate: OnceCell<Retained<workspace_delegate::WorkspaceDelegate>>,
    outline_scroll: OnceCell<Retained<NSScrollView>>,
    thumb_view: OnceCell<Retained<PDFThumbnailView>>,
    explorer_item: OnceCell<Retained<NSSplitViewItem>>,
    navigation_item: OnceCell<Retained<NSSplitViewItem>>,
    right_sidebar_outline_scroll: OnceCell<Retained<NSScrollView>>,
    pdf_outline_delegate: OnceCell<Retained<crate::pdf_outline::PdfOutlineDelegate>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "AppDelegate"]
    #[ivars = AppDelegateIvars]
    struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl NSToolbarDelegate for AppDelegate {
        #[unsafe(method(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
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

            let mtm = MainThreadMarker::new().unwrap();
            
            let zoom_in_id = objc2_foundation::ns_string!("ZoomIn");
            let zoom_out_id = objc2_foundation::ns_string!("ZoomOut");
            let search_id = objc2_foundation::ns_string!("Search");
            let sidebar_tabs_id = objc2_foundation::ns_string!("SidebarTabs");
            
            
            
            if item_identifier.isEqualToString(zoom_in_id) {
                let item = unsafe { NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier) };
                unsafe {
                    if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("plus.magnifyingglass"), None) {
                        item.setImage(Some(&img));
                    }
                    item.setLabel(objc2_foundation::ns_string!("Zoom In"));
                    item.setAction(Some(objc2::sel!(zoomIn:)));
                }
                return Retained::into_raw(item);
            }
            if item_identifier.isEqualToString(zoom_out_id) {
                let item = unsafe { NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier) };
                unsafe {
                    if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("minus.magnifyingglass"), None) {
                        item.setImage(Some(&img));
                    }
                    item.setLabel(objc2_foundation::ns_string!("Zoom Out"));
                    item.setAction(Some(objc2::sel!(zoomOut:)));
                }
                return Retained::into_raw(item);
            }
            if item_identifier.isEqualToString(search_id) {
                let item = unsafe { NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier) };
                unsafe {
                    if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("magnifyingglass"), None) {
                        item.setImage(Some(&img));
                    }
                    item.setLabel(objc2_foundation::ns_string!("Search"));
                    item.setAction(Some(objc2::sel!(performFindPanelAction:)));
                }
                return Retained::into_raw(item);
            }
            
            std::ptr::null_mut()
        }

        #[unsafe(method(toolbarDefaultItemIdentifiers:))]
        fn toolbarDefaultItemIdentifiers(&self, _toolbar: &NSToolbar) -> *mut NSArray<NSToolbarItemIdentifier> {
            unsafe {
                let array = NSArray::from_slice(&[
                    objc2_app_kit::NSToolbarToggleSidebarItemIdentifier,
                    objc2_app_kit::NSToolbarFlexibleSpaceItemIdentifier,
                    objc2_app_kit::NSToolbarToggleInspectorItemIdentifier,
                    objc2_foundation::ns_string!("Search"),
                    objc2_foundation::ns_string!("ZoomOut"),
                    objc2_foundation::ns_string!("ZoomIn")
                ]);
                Retained::into_raw(array)
            }
        }

        #[unsafe(method(toolbarAllowedItemIdentifiers:))]
        fn toolbarAllowedItemIdentifiers(&self, _toolbar: &NSToolbar) -> *mut NSArray<NSToolbarItemIdentifier> {
            unsafe {
                let array = NSArray::from_slice(&[
                    objc2_app_kit::NSToolbarToggleSidebarItemIdentifier,
                    objc2_app_kit::NSToolbarFlexibleSpaceItemIdentifier,
                    objc2_app_kit::NSToolbarToggleInspectorItemIdentifier,
                    objc2_foundation::ns_string!("Search"),
                    objc2_foundation::ns_string!("ZoomOut"),
                    objc2_foundation::ns_string!("ZoomIn")
                ]);
                Retained::into_raw(array)
            }
        }
    }

    unsafe impl NSApplicationDelegate for AppDelegate {}

    impl AppDelegate {
        #[unsafe(method(openPdf:))]
        fn open_pdf(&self, _sender: Option<&AnyObject>) {
            let mtm = MainThreadMarker::new().unwrap();
            let panel = NSOpenPanel::openPanel(mtm);
            unsafe {
                panel.setCanChooseFiles(true);
                panel.setCanChooseDirectories(false);
            }
            if unsafe { panel.runModal() } == objc2_app_kit::NSModalResponseOK {
                if let Some(url) = unsafe { panel.URL() } {
                    let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                    if let Some(doc) = doc {
                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(&doc)) };
                        if let Some(title) = url.lastPathComponent() {
                            self.ivars().window.get().unwrap().setTitle(&title);
                        }
                        
                        let window = self.ivars().window.get().unwrap();
                        let split_vc = self.ivars().split_vc.get().unwrap();
                        
            let split_view = unsafe { split_vc.splitView() };
            unsafe { split_view.setDividerStyle(objc2_app_kit::NSSplitViewDividerStyle::Thin) };
            
            unsafe { window.setContentViewController(Some(&*self.ivars().split_vc.get().unwrap())) };

                    }
                }
            }
        }

        #[unsafe(method(openFolder:))]
        fn open_folder(&self, _sender: Option<&AnyObject>) {
            let mtm = MainThreadMarker::new().unwrap();
            let panel = NSOpenPanel::openPanel(mtm);
            unsafe {
                panel.setCanChooseFiles(false);
                panel.setCanChooseDirectories(true);
            }
            if unsafe { panel.runModal() } == objc2_app_kit::NSModalResponseOK {
                if let Some(url) = unsafe { panel.URL() } {
                    if let Some(title) = url.lastPathComponent() {
                        self.ivars().window.get().unwrap().setTitle(&title);
                    }
                    
                    let window = self.ivars().window.get().unwrap();
                    let split_vc = self.ivars().split_vc.get().unwrap();
                    
            let split_view = unsafe { split_vc.splitView() };
            unsafe { split_view.setDividerStyle(objc2_app_kit::NSSplitViewDividerStyle::Thin) };
            
            unsafe { window.setContentViewController(Some(&*self.ivars().split_vc.get().unwrap())) };

                    
                    let path = url.path().unwrap().to_string();
                    let name = url.lastPathComponent().unwrap().to_string();
                    crate::log_workspace!("Opening folder: {}", path); let root = crate::workspace::FileNode::new(path, name, true);
                    
                    if let Some(del) = self.ivars().workspace_delegate.get() {
                        del.ivars().root_node.borrow_mut().replace(root);
                        if let Some(scroll) = self.ivars().outline_scroll.get() {
                            if let Some(view) = unsafe { scroll.documentView() } {
                                if let Some(outline) = view.downcast_ref::<NSOutlineView>() { 
                                    crate::log_workspace!("Reloading outline view!"); unsafe { outline.reloadData() };
                                    unsafe { outline.expandItem(None) };
                                }
                            }
                        }
                    }
                }
            }
        }

        #[unsafe(method(application:openURLs:))]
        fn application_openURLs(&self, _application: &NSApplication, urls: &NSArray<NSURL>) {
            if let Some(url) = urls.firstObject() {
                if self.ivars().window.get().is_none() {
                    *self.ivars().pending_url.borrow_mut() = Some(url.clone());
                    return;
                }
                
                let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                if let Some(doc) = doc {
                    if let Some(pdf_view) = self.ivars().pdf_view.get() {
                        unsafe { pdf_view.setDocument(Some(&doc)) };
                    }
                    if let Some(window) = self.ivars().window.get() {
                        if let Some(title) = url.lastPathComponent() {
                            window.setTitle(&title);
                        }
                    }
                } else {
                    // It's a folder or invalid PDF!
                    let path = url.path().unwrap().to_string();
                    let name = url.lastPathComponent().unwrap().to_string();
                    let root = crate::workspace::FileNode::new(path, name, true);
                    
                    if let Some(window) = self.ivars().window.get() {
                        if let Some(title) = url.lastPathComponent() {
                            window.setTitle(&title);
                        }
                        if let Some(split_vc) = self.ivars().split_vc.get() {
                            
            let split_view = unsafe { split_vc.splitView() };
            unsafe { split_view.setDividerStyle(objc2_app_kit::NSSplitViewDividerStyle::Thin) };
            
            unsafe { window.setContentViewController(Some(&*self.ivars().split_vc.get().unwrap())) };

                        }
                    }
                    
                    if let Some(del) = self.ivars().workspace_delegate.get() {
                        del.ivars().root_node.borrow_mut().replace(root);
                        if let Some(scroll) = self.ivars().outline_scroll.get() {
                            if let Some(view) = unsafe { scroll.documentView() } {
                                if let Some(outline) = view.downcast_ref::<NSOutlineView>() { 
                                    unsafe { outline.reloadData() };
                                    unsafe { outline.expandItem(None) };
                                }
                            }
                        }
                    }
                }
            }
        }

        #[unsafe(method(switchSidebar:))]
        fn switch_sidebar(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(segmented) = sender.downcast_ref::<NSSegmentedControl>() {
                    let index = unsafe { segmented.selectedSegment() };
                    unsafe {
                        let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
                        defaults.setInteger_forKey(index, objc2_foundation::ns_string!("RightSidebarMode"));
                    }
                    if let Some(outline_scroll) = self.ivars().right_sidebar_outline_scroll.get() {
                        let thumb_view = self.ivars().thumb_view.get().unwrap();
                        if index == 0 {
                            unsafe { outline_scroll.setHidden(true) };
                            unsafe { thumb_view.setHidden(false) };
                        } else {
                            unsafe { outline_scroll.setHidden(false) };
                            unsafe { thumb_view.setHidden(true) };
                        }
                        let _: () = unsafe { objc2::msg_send![self, updateOutlineEmptyState] };
                    }
                }
            }
        }

        #[unsafe(method(workspaceDoubleClicked:))]
        fn workspace_double_clicked(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(outline) = sender.downcast_ref::<NSOutlineView>() {
                    let clicked_row = unsafe { outline.clickedRow() };
                    if clicked_row >= 0 {
                        if let Some(item) = unsafe { outline.itemAtRow(clicked_row) } {
                            let node: &crate::workspace::FileNode = unsafe { &*(&*item as *const AnyObject as *const crate::workspace::FileNode) };
                            if node.ivars().is_dir.get() {
                                let is_expanded = unsafe { outline.isItemExpanded(Some(&*item)) };
                                if is_expanded {
                                    unsafe { outline.collapseItem(Some(&*item)) };
                                } else {
                                    unsafe { outline.expandItem(Some(&*item)) };
                                }
                            }
                        }
                    }
                }
            }
        }

        
        #[unsafe(method(pdfDidLoad:))]
        fn pdf_did_load(&self, doc: Option<&objc2_pdf_kit::PDFDocument>) {
            crate::log_pdf!("[DIAGNOSTIC] PDF DID LOAD ENTER");
            if let Some(doc) = doc {
                let main_thread = std::thread::current().name() == Some("main");
                crate::log_pdf!("[DIAGNOSTIC] main thread = {}", main_thread);
                
                if let Some(url) = unsafe { doc.documentURL() } {
                    let path = url.path().unwrap().to_string();
                    crate::log_pdf!("[DIAGNOSTIC] path = {}", path);
                    if path == *self.ivars().pending_pdf_path.borrow() {
                        crate::log_pdf!("[DIAGNOSTIC] PDFVIEW BEFORE setDocument");
                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(doc)) };
                        
                        let page_count = unsafe { doc.pageCount() };
                        crate::log_pdf!("[DIAGNOSTIC] PDFVIEW AFTER setDocument: pageCount = {}", page_count);
                        
                        let _: () = unsafe { objc2::msg_send![self, updateOutlineEmptyState] };
                    } else {
                        crate::log_pdf!("[DIAGNOSTIC] Discarded obsolete doc: {}", path);
                    }
                }
            } else {
                crate::log_pdf!("[DIAGNOSTIC] doc is None");
            }
        }

        #[unsafe(method(updateOutlineEmptyState))]
        fn update_outline_empty_state(&self) {
            unsafe {
                if let Some(scroll) = self.ivars().right_sidebar_outline_scroll.get() {
                    if let Some(view) = scroll.documentView() {
                        if let Some(outline_view) = view.downcast_ref::<NSOutlineView>() {
                            outline_view.reloadData();
                            
                            // Check for outline root
                            let mut has_outline = false;
                            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                                if let Some(doc) = pdf_view.document() {
                                    if let Some(root) = doc.outlineRoot() {
                                        if root.numberOfChildren() > 0 {
                                            has_outline = true;
                                        }
                                    }
                                }
                            }
                            
                            // Let's find or create the empty state view
                            let empty_tag = 999;
                            let superview = scroll.superview().unwrap();
                            let mut existing_empty = None;
                            let subviews = superview.subviews();
                            for i in 0..subviews.count() {
                                let sub = subviews.objectAtIndex(i);
                                if sub.tag() == empty_tag {
                                    existing_empty = Some(sub);
                                    break;
                                }
                            }
                            
                            if has_outline {
                                if let Some(empty) = existing_empty.as_ref() {
                                    empty.removeFromSuperview();
                                }
                            } else {
                                if existing_empty.is_none() {
                                    let mtm = MainThreadMarker::new().unwrap();
                                    let empty_view = NSView::initWithFrame(NSView::alloc(mtm), scroll.frame());
                                    empty_view.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                                    let _: () = objc2::msg_send![&empty_view, setTag: empty_tag];
                                    
                                    // SF Symbol list.bullet.indent
                                    let img = NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("list.bullet.indent"), None).unwrap();
                                    let img_view = NSImageView::imageViewWithImage(&img, mtm);
                                    img_view.setContentTintColor(Some(&NSColor::tertiaryLabelColor()));
                                    img_view.setTranslatesAutoresizingMaskIntoConstraints(false);
                                    
                                    let label = NSTextField::labelWithString(objc2_foundation::ns_string!("No document outline available"), mtm);
                                    label.setTextColor(Some(&NSColor::secondaryLabelColor()));
                                    label.setTranslatesAutoresizingMaskIntoConstraints(false);
                                    
                                    empty_view.addSubview(&img_view);
                                    empty_view.addSubview(&label);
                                    
                                    img_view.centerXAnchor().constraintEqualToAnchor(&empty_view.centerXAnchor()).setActive(true);
                                    img_view.centerYAnchor().constraintEqualToAnchor_constant(&empty_view.centerYAnchor(), -20.0).setActive(true);
                                    
                                    
                                    label.centerXAnchor().constraintEqualToAnchor(&empty_view.centerXAnchor()).setActive(true);
                                    label.topAnchor().constraintEqualToAnchor_constant(&img_view.bottomAnchor(), 16.0).setActive(true);
                                    
                                    // Make sure it doesn't block hits if we need it to, but it's an empty state so it's fine.
                                    
                                    // Hidden initially if outline isn't currently selected, but actually it sits on top of scroll, so we want it to show/hide with scroll.
                                    // A better way: just add it to scroll's superview, and sync its hidden state with scroll's hidden state.
                                    // Or better yet, we just add it to the content_wrapper, but toggle its visibility in switchSidebar.
                                    superview.addSubview(&empty_view);
                                }
                            }
                            
                            // We need to make sure the empty view's hidden state matches the outline scroll view's hidden state.
                            if let Some(empty) = existing_empty.as_ref() {
                                empty.setHidden(scroll.isHidden());
                            } else {
                                // If we just created it:
                                let subviews = superview.subviews();
                                for i in 0..subviews.count() {
                                    let sub = subviews.objectAtIndex(i);
                                    if sub.tag() == empty_tag {
                                        sub.setHidden(scroll.isHidden());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        #[unsafe(method(workspaceSelectionChanged:))]
        fn workspace_selection_changed(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(outline) = sender.downcast_ref::<NSOutlineView>() {
                    let mtm = MainThreadMarker::new().unwrap();
                    let app = NSApplication::sharedApplication(mtm);
                    let mut is_click = false;
                    if let Some(event) = app.currentEvent() {
                        let ev_type = event.r#type();
                        if ev_type == objc2_app_kit::NSEventType::LeftMouseDown || ev_type == objc2_app_kit::NSEventType::LeftMouseUp {
                            is_click = true;
                        }
                    }
                    if !is_click { return; }
                    
                    let selected_row = outline.selectedRow();
                    if selected_row >= 0 {
                        if let Some(item) = unsafe { outline.itemAtRow(selected_row) } {
                            let node: &crate::workspace::FileNode = unsafe { &*(&*item as *const AnyObject as *const crate::workspace::FileNode) };
                            if !node.ivars().is_dir.get() {
                                let url_str = node.ivars().path.borrow().clone();
                                let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };
                                
                                let current_doc = unsafe { self.ivars().pdf_view.get().unwrap().document() };
                                let mut already_open = false;
                                if let Some(current_doc) = current_doc {
                                    if let Some(current_url) = unsafe { current_doc.documentURL() } {
                                        if current_url.path().unwrap().to_string() == url_str { already_open = true; }
                                    }
                                }
                                
                                if !already_open {
                                    let icloud_status = node.ivars().icloud_status.get();
                                    
                                    crate::log_workspace!("[DIAGNOSTIC] CLICK: path = {}, old_state = {}", url_str, crate::ICloudState::from_int(icloud_status as u32).to_str());
                                    
                                    if icloud_status == 2 {
                                        crate::log_workspace!("[DIAGNOSTIC] FORCE REFRESH: {}", url_str);
                                        if crate::is_file_locally_usable(&url_str) {
                                            crate::log_workspace!("[DIAGNOSTIC] status=Current, readable=true");
                                            node.ivars().icloud_status.set(0);
                                            crate::log_ui!("[DIAGNOSTIC] MODEL UPDATE: Downloading -> Local");
                                            let _: () = unsafe { objc2::msg_send![outline, reloadItem: &*item, reloadChildren: false] };
                                            crate::log_ui!("[DIAGNOSTIC] ROW RELOAD: row = {}", selected_row);
                                        }
                                    }
                                    
                                    if icloud_status == 1 || icloud_status == 4 || node.ivars().icloud_status.get() == 2 { // CloudOnly, Failed, or still Downloading
                                        if node.ivars().icloud_status.get() != 2 {
                                            // 1. Set to DownloadRequested
                                            node.ivars().icloud_status.set(2);
                                            crate::log_ui!("[DIAGNOSTIC] MODEL UPDATE: {} -> {}", crate::ICloudState::from_int(icloud_status as u32).to_str(), "Downloading");
                                            
                                            // 2. Reload row immediately
                                            let _: () = unsafe { objc2::msg_send![outline, reloadItem: &*item, reloadChildren: false] };
                                            crate::log_ui!("[DIAGNOSTIC] ROW RELOAD: row = {}", selected_row);
                                            
                                            // 4. Spawn download
                                            let outline_raw = outline as *const _ as usize;
                                            let app_ptr = self as *const AppDelegate as usize;
                                            
                                            let mut should_spawn = false;
                                            if let Ok(mut guard) = crate::ACTIVE_DOWNLOADS.lock() {
                                                if !guard.contains(&url_str) {
                                                    guard.insert(url_str.clone());
                                                    should_spawn = true;
                                                }
                                            }
                                            if should_spawn {
                                                let url_clone = url_str.clone();
                                                std::thread::spawn(move || {
                                                    crate::download_file(url_clone, outline_raw, app_ptr);
                                                });
                                            }
                                        }
                                        
                                        // 3. Mark as pending open
                                        *self.ivars().pending_pdf_path.borrow_mut() = url_str.clone();
                                        return;
                                    }
                                    
                                    // Local file
                                    *self.ivars().pending_pdf_path.borrow_mut() = url_str.clone();
                                    let app_ptr = self as *const AppDelegate as usize;
                                    
                                    crate::log_pdf!("[DIAGNOSTIC] PDF LOAD START");
                                    std::thread::spawn(move || {
                                        let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };
                                        let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &nsurl) };
                                        if let Some(doc) = doc {
                                            let doc_any = unsafe { objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(doc) };
                                            let app_raw = app_ptr as *mut objc2::runtime::AnyObject;
                                            let _: () = unsafe {
                                                objc2::msg_send![app_raw, performSelectorOnMainThread: objc2::sel!(pdfDidLoad:), withObject: &*doc_any, waitUntilDone: false]
                                            };
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }


        #[unsafe(method(downloadDidComplete:))]
        fn download_did_complete(&self, path: &objc2_foundation::NSString) {
            let path_str = path.to_string();
            crate::log_pdf!("[DIAGNOSTIC] MODEL LOOKUP: canonical path = {}", path_str);
            if let Some(wd) = self.ivars().workspace_delegate.get() {
                if let Some(node) = wd.find_node_by_path(&path_str) {
                    crate::log_ui!("[DIAGNOSTIC] MODEL LOOKUP: FileNode identity = {:p}", &*node);
                    let old = node.ivars().icloud_status.get();
                    node.ivars().icloud_status.set(0);
                    crate::log_ui!("[DIAGNOSTIC] MODEL UPDATE: {} -> Local", crate::ICloudState::from_int(old as u32).to_str());
                    
                    if let Some(weak_view) = &*wd.ivars().outline_view.borrow() {
                        if let Some(outline) = weak_view.load() {
                            let row = unsafe { outline.rowForItem(Some(&*(&*node as *const _ as *const objc2::runtime::AnyObject))) };
                            if row >= 0 {
                                crate::log_ui!("[DIAGNOSTIC] ROW RELOAD: item identity = {:p}, row = {}", &*node, row);
                                let _: () = unsafe { objc2::msg_send![&outline, reloadItem: &*(&*node as *const _ as *const objc2::runtime::AnyObject), reloadChildren: false] };
                            }
                        }
                    }
                }
            }
            
            let pending = self.ivars().pending_pdf_path.borrow().clone();
            if let Ok(mut guard) = crate::ACTIVE_DOWNLOADS.lock() {
                guard.remove(&path_str);
            }
            crate::log_pdf!("[DIAGNOSTIC] PENDING OPEN LOOKUP: pending path = {}, matches = {}", pending, pending == path_str);
            
            if pending == path_str {
                crate::log_pdf!("[DIAGNOSTIC] PDF LOAD START");
                let nsurl = unsafe { objc2_foundation::NSURL::fileURLWithPath(path) };
                let doc = unsafe { objc2_pdf_kit::PDFDocument::initWithURL(objc2_pdf_kit::PDFDocument::alloc(), &nsurl) };
                if let Some(doc) = doc {
                    crate::log_pdf!("[DIAGNOSTIC] PDF LOAD COMPLETE");
                    let doc_any = unsafe { objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(doc) };
                    println!("[DIAGNOSTIC] PDF DISPLAY DISPATCH");
                    let _: () = unsafe { objc2::msg_send![self, pdfDidLoad: &*doc_any] };
                    *self.ivars().pending_pdf_path.borrow_mut() = String::new();
                } else {
                    crate::log_pdf!("[DIAGNOSTIC] PDF LOAD FAILED");
                }
            }
        }
        
        #[unsafe(method(downloadDidFail:))]
        fn download_did_fail(&self, path: &objc2_foundation::NSString) {
            let path_str = path.to_string();
            if let Ok(mut guard) = crate::ACTIVE_DOWNLOADS.lock() {
                guard.remove(&path_str);
            }
            if let Some(wd) = self.ivars().workspace_delegate.get() {
                if let Some(node) = wd.find_node_by_path(&path_str) {
                    node.ivars().icloud_status.set(4);
                    if let Some(weak_view) = &*wd.ivars().outline_view.borrow() {
                        if let Some(outline) = weak_view.load() {
                            let _: () = unsafe { objc2::msg_send![&outline, reloadItem: &*(&*node as *const _ as *const objc2::runtime::AnyObject), reloadChildren: false] };
                        }
                    }
                }
            }
        }
        
                #[unsafe(method(downloadDidTimeout:))]
        fn download_did_timeout(&self, path: &objc2_foundation::NSString) {
            let path_str = path.to_string();
            if let Ok(mut guard) = crate::ACTIVE_DOWNLOADS.lock() {
                guard.remove(&path_str);
            }
            
            let nsurl = unsafe { objc2_foundation::NSURL::fileURLWithPath(path) };
            let mut stat_val: *mut objc2::runtime::AnyObject = std::ptr::null_mut();
            let key_stat = objc2_foundation::NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");
            let ok_stat: bool = unsafe { objc2::msg_send![&nsurl, getResourceValue: &mut stat_val, forKey: &*key_stat, error: std::ptr::null_mut::<objc2::runtime::AnyObject>()] };
            
            let mut is_local = false;
            if ok_stat && !stat_val.is_null() {
                let stat_str = stat_val as *mut objc2_foundation::NSString;
                let dl_id = objc2_foundation::NSString::from_str("NSURLUbiquitousItemDownloadingStatusDownloaded");
                let current_id = objc2_foundation::NSString::from_str("NSURLUbiquitousItemDownloadingStatusCurrent");
                let is_downloaded: bool = unsafe { objc2::msg_send![stat_str, isEqualToString: &*dl_id] };
                let is_current: bool = unsafe { objc2::msg_send![stat_str, isEqualToString: &*current_id] };
                is_local = is_downloaded || is_current;
            }
            
            if is_local {
                crate::log_icloud!("[DIAGNOSTIC] TIMEOUT RECOVERY: File is local, treating as complete.");
                let _: () = unsafe { objc2::msg_send![self, downloadDidComplete: path] };
                return;
            }
            
            if let Some(wd) = self.ivars().workspace_delegate.get() {
                if let Some(node) = wd.find_node_by_path(&path_str) {
                    node.ivars().icloud_status.set(4);
                    if let Some(weak_view) = &*wd.ivars().outline_view.borrow() {
                        if let Some(outline) = weak_view.load() {
                            let _: () = unsafe { objc2::msg_send![&outline, reloadItem: &*(&*node as *const _ as *const objc2::runtime::AnyObject), reloadChildren: false] };
                        }
                    }
                }
            }
        }


#[unsafe(method(applicationDidBecomeActive:))]
        fn application_did_become_active(&self, _notification: &objc2_foundation::NSNotification) {
            if let Some(wd) = self.ivars().workspace_delegate.get() {
                if let Some(outline_weak) = &*wd.ivars().outline_view.borrow() {
                    if let Some(outline) = outline_weak.load() {
                        unsafe {
                            let _: () = objc2::msg_send![&outline, reloadData];
                        }
                    }
                }
            }
        }

        
                #[unsafe(method(thumbnailFrameChanged:))]
        fn thumbnail_frame_changed(&self, notification: &objc2_foundation::NSNotification) {
            if let Some(obj) = unsafe { notification.object() } {
                if let Some(view) = obj.downcast_ref::<NSView>() {
                    let frame = view.frame();
                    let width = frame.size.width;
                    
                    if let Some(thumb_view) = self.ivars().thumb_view.get() {
                        let old_frame = thumb_view.frame();
                        let new_thumb_width = (width * 0.75).max(50.0);
                        let new_thumb_height = new_thumb_width * 1.414;
                        
                        // Throttle updates: only resize if width changed by > 5pt
                        if (old_frame.size.width - new_thumb_width).abs() > 5.0 {
                            unsafe { 
                                thumb_view.setFrame(NSRect::new(NSPoint::new((width - new_thumb_width) / 2.0, 0.0), NSSize::new(new_thumb_width, frame.size.height)));
                                thumb_view.setThumbnailSize(NSSize::new(new_thumb_width, new_thumb_height));
                            }
                        }
                    }
                }
            }
        }
        #[unsafe(method(fitWidth:))]
        fn fit_width(&self, _sender: Option<&AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                unsafe { pdf_view.setAutoScales(true) };
                // There is no explicit Fit Width in PDFView's public property except setting scale manually,
                // but setting autoScales typically behaves as fit width/page depending on display mode.
                // We will just call setDisplayBox if needed, but PDFDisplayMode is enough.
                // Better yet: PDFView has `setAutoScales` which does Fit Page.
                // Let's rely on standard actions or just `setAutoScales`.
            }
        }
        #[unsafe(method(fitPage:))]
        fn fit_page(&self, _sender: Option<&AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                unsafe { pdf_view.setAutoScales(true) };
            }
        }
        #[unsafe(method(actualSize:))]
        fn actual_size(&self, _sender: Option<&AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                unsafe { 
                    pdf_view.setAutoScales(false);
                    pdf_view.setScaleFactor(1.0); 
                };
            }
        }
        #[unsafe(method(openDebugConsole:))]
        fn open_debug_console(&self, _sender: Option<&AnyObject>) {
            crate::debug_console::show_debug_console();
        }
        
        #[unsafe(method(applicationDidFinishLaunching:))]

        fn application_did_finish_launching(&self, _notification: *mut objc2_foundation::NSNotification) {
            let mtm = MainThreadMarker::new().unwrap();
            
            // Set up main menu
            let menubar = NSMenu::new(mtm);
            let app_menu_item = NSMenuItem::new(mtm);
            unsafe { menubar.addItem(&app_menu_item) };
            
            let app_menu = NSMenu::new(mtm);
            let quit_title = objc2_foundation::ns_string!("Quit OxiReader");
            let quit_action = sel!(terminate:);
            let quit_key = objc2_foundation::ns_string!("q");
            let quit_item = unsafe { NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), quit_title, Some(quit_action), quit_key) };
            
            unsafe {
                app_menu.addItem(&quit_item);
                app_menu_item.setSubmenu(Some(&app_menu));
                
                // Window menu
                let window_menu_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Window"), None, &objc2_foundation::NSString::from_str(""));
                let window_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &objc2_foundation::NSString::from_str("Window"));
                let debug_title = objc2_foundation::NSString::from_str("Debug Console");
                let debug_key = objc2_foundation::NSString::from_str("l");
                let debug_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &debug_title, Some(objc2::sel!(openDebugConsole:)), &debug_key);
                                window_menu.addItem(&debug_item);
                window_menu_item.setSubmenu(Some(&window_menu));
                
                menubar.addItem(&window_menu_item);
                
                let view_menu_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("View"), None, &objc2_foundation::NSString::from_str(""));
                let view_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &objc2_foundation::NSString::from_str("View"));
                
                let fit_width_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Fit Width"), Some(objc2::sel!(fitWidth:)), &objc2_foundation::NSString::from_str("2"));
                let fit_page_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Fit Page"), Some(objc2::sel!(fitPage:)), &objc2_foundation::NSString::from_str("3"));
                let actual_size_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Actual Size"), Some(objc2::sel!(actualSize:)), &objc2_foundation::NSString::from_str("0"));
                
                unsafe {
                    view_menu.addItem(&actual_size_item);
                    view_menu.addItem(&fit_width_item);
                    view_menu.addItem(&fit_page_item);
                    view_menu_item.setSubmenu(Some(&view_menu));
                    menubar.addItem(&view_menu_item);
                }

                
                NSApplication::sharedApplication(mtm).setMainMenu(Some(&menubar));
            }

            
            let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1000.0, 700.0));
            let style = NSWindowStyleMask::Titled 
                      | NSWindowStyleMask::Closable 
                      | NSWindowStyleMask::Resizable 
                      | NSWindowStyleMask::Miniaturizable;
            
            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(mtm),
                    rect,
                    style,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };
            
            window.setTitle(objc2_foundation::ns_string!("OxiReader"));
            window.center();
            
            // Toolbar (Liquid Glass Native)
            let toolbar = unsafe { NSToolbar::initWithIdentifier(NSToolbar::alloc(mtm), objc2_foundation::ns_string!("MainToolbar")) };
            unsafe {
                toolbar.setDelegate(Some(ProtocolObject::from_ref(self)));
                toolbar.setDisplayMode(NSToolbarDisplayMode::IconOnly);
                window.setToolbar(Some(&toolbar));
                window.setTitlebarAppearsTransparent(true);
                window.setToolbarStyle(NSWindowToolbarStyle::Unified);
            }
            

            // Split View Controller
            let split_vc = NSSplitViewController::new(mtm);


            // PDF View
            let pdf_view = unsafe { PDFView::initWithFrame(PDFView::alloc(mtm), rect) };
            unsafe {
                pdf_view.setDisplayMode(PDFDisplayMode::SinglePageContinuous);
                pdf_view.setAutoScales(true);
                // Fix margins and spacing
                pdf_view.setBackgroundColor(&objc2_app_kit::NSColor::underPageBackgroundColor());
                pdf_view.setPageBreakMargins(NSEdgeInsets { top: 20.0, left: 20.0, bottom: 20.0, right: 20.0 });
                pdf_view.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
            }
            let main_vc = NSViewController::new(mtm);
            unsafe { main_vc.setView(&pdf_view) };

            let main_item = unsafe { NSSplitViewItem::splitViewItemWithViewController(&main_vc) };

            // Sidebar View
            let thumb_view = unsafe { PDFThumbnailView::initWithFrame(PDFThumbnailView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                thumb_view.setPDFView(Some(&pdf_view));
                // Adaptive thumbnail sizes
                thumb_view.setThumbnailSize(NSSize::new(100.0, 140.0));
                
                // We use msg_send! to bypass missing bindings in objc2_pdf_kit
                let _: () = msg_send![&thumb_view, setMaximumNumberOfColumns: 1usize];
                
                // We can't reduce vertical spacing directly on PDFThumbnailView in standard AppKit easily, 
                // but setting a suitable thumbnail size helps.
                thumb_view.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
            }

            let workspace_del = WorkspaceDelegate::new(mtm);
            let _ = self.ivars().workspace_delegate.set(workspace_del.clone());

            let outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            workspace_del.set_outline_view(&outline_view);
            let column = unsafe { NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), objc2_foundation::ns_string!("MainColumn")) };
            unsafe {
                column.setWidth(200.0);
                outline_view.addTableColumn(&column);
                outline_view.setOutlineTableColumn(Some(&column));
                outline_view.setSelectionHighlightStyle(objc2_app_kit::NSTableViewSelectionHighlightStyle::SourceList);
                outline_view.setAllowsMultipleSelection(false);
                outline_view.setUsesAlternatingRowBackgroundColors(false);
                outline_view.setHeaderView(None);
                outline_view.setRowHeight(24.0);
                outline_view.setIntercellSpacing(NSSize::new(4.0, 0.0));

                
                let delegate_ptr: *const WorkspaceDelegate = &*workspace_del;
                let _: () = unsafe { msg_send![&outline_view, setDelegate: delegate_ptr] };
                let _: () = unsafe { msg_send![&outline_view, setDataSource: delegate_ptr] };

                let target_ptr: *const AppDelegate = self;
                let _: () = unsafe { msg_send![&outline_view, setTarget: target_ptr] };
                outline_view.setDoubleAction(Some(objc2::sel!(workspaceDoubleClicked:)));
            }

            let outline_scroll = unsafe { NSScrollView::initWithFrame(NSScrollView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                outline_scroll.setHasVerticalScroller(true);
                outline_scroll.setDocumentView(Some(&outline_view));
                outline_scroll.setDrawsBackground(false);
                outline_scroll.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
            }

            let explorer_vc = NSViewController::new(mtm);
            unsafe { explorer_vc.setView(&outline_scroll) };
            let explorer_item = unsafe { NSSplitViewItem::sidebarWithViewController(&explorer_vc) };
            unsafe {
                explorer_item.setCanCollapse(true);
                explorer_item.setMinimumThickness(180.0);
                explorer_item.setMaximumThickness(420.0);
            }


            // RIGHT SIDEBAR (Outline + Pages)
            let pdf_outline_del = crate::pdf_outline::PdfOutlineDelegate::new(mtm, pdf_view.clone());
            let right_outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            let right_col = unsafe { NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), objc2_foundation::ns_string!("OutlineColumn")) };
            unsafe {
                right_col.setWidth(230.0);
                right_outline_view.addTableColumn(&right_col);
                right_outline_view.setOutlineTableColumn(Some(&right_col));
                right_outline_view.setHeaderView(None);
                
                let del_ptr: *const crate::pdf_outline::PdfOutlineDelegate = &*pdf_outline_del;
                let _: () = msg_send![&right_outline_view, setDelegate: del_ptr];
                let _: () = msg_send![&right_outline_view, setDataSource: del_ptr];
            }
            
            let right_outline_scroll = unsafe { NSScrollView::initWithFrame(NSScrollView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                right_outline_scroll.setHasVerticalScroller(true);
                right_outline_scroll.setDocumentView(Some(&right_outline_view));
                right_outline_scroll.setDrawsBackground(false);
                right_outline_scroll.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                right_outline_scroll.setHidden(true); // default to Pages
            }
            
            let right_container = unsafe { NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            
            let segmented = unsafe { NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action(
                &NSArray::from_slice(&[objc2_foundation::ns_string!("Pages"), objc2_foundation::ns_string!("Outline")]),
                NSSegmentSwitchTracking::SelectOne,
                Some(unsafe { &*(self as *const AppDelegate as *const AnyObject) }),
                Some(objc2::sel!(switchSidebar:)),
                mtm
            ) };
            unsafe {
                let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
                let selected_idx = defaults.integerForKey(objc2_foundation::ns_string!("RightSidebarMode"));
                segmented.setSelected_forSegment(true, selected_idx);
                
                if selected_idx == 1 {
                    right_outline_scroll.setHidden(false);
                    thumb_view.setHidden(true);
                } else {
                    right_outline_scroll.setHidden(true);
                    thumb_view.setHidden(false);
                }
                
                let seg_wrapper = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 41.0)));
                seg_wrapper.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewMinYMargin);
                seg_wrapper.addSubview(&segmented);
                
                // Add 8-10pt top padding, 10-12pt horizontal padding, 8pt bottom padding.
                // 41pt height - 8pt top - 8pt bottom = 25pt.
                segmented.setFrame(NSRect::new(NSPoint::new(12.0, 8.0), NSSize::new(230.0 - 24.0, 25.0)));
                segmented.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable);
                
                let separator = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 1.0)));
                separator.setWantsLayer(true);
                if let Some(layer) = separator.layer() {
                    let cg = objc2_app_kit::NSColor::separatorColor().CGColor(); let ptr = Retained::as_ptr(&cg); let _: () = msg_send![&layer, setBackgroundColor: ptr];
                }
                separator.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewMinYMargin);
                
                let content_wrapper = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 660.0)));
                content_wrapper.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                content_wrapper.addSubview(&thumb_view);
                content_wrapper.addSubview(&right_outline_scroll);
                
                right_container.addSubview(&content_wrapper);
                right_container.addSubview(&separator);
                right_container.addSubview(&seg_wrapper);
            unsafe {
                let nc = objc2_foundation::NSNotificationCenter::defaultCenter();
                nc.addObserver_selector_name_object(
                    &*(self as *const AppDelegate as *const AnyObject),
                    objc2::sel!(thumbnailFrameChanged:),
                    Some(objc2_app_kit::NSViewFrameDidChangeNotification),
                    Some(&*content_wrapper as &AnyObject)
                );
                content_wrapper.setPostsFrameChangedNotifications(true);
            }

                
                // Layout adjustment
                seg_wrapper.setFrame(NSRect::new(NSPoint::new(0.0, 700.0 - 40.0), NSSize::new(230.0, 41.0)));
                separator.setFrame(NSRect::new(NSPoint::new(0.0, 700.0 - 41.0), NSSize::new(230.0, 1.0)));
                content_wrapper.setFrame(NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0 - 41.0)));
            }

            let navigation_vc = NSViewController::new(mtm);
            unsafe { navigation_vc.setView(&right_container) };
            let navigation_item = unsafe { NSSplitViewItem::inspectorWithViewController(&navigation_vc) };
            unsafe {
                navigation_item.setCanCollapse(true);
                navigation_item.setMinimumThickness(180.0);
                navigation_item.setMaximumThickness(360.0);
            }

            
            self.ivars().explorer_item.set(explorer_item.clone()).unwrap();
            self.ivars().navigation_item.set(navigation_item.clone()).unwrap();
            self.ivars().outline_scroll.set(outline_scroll).unwrap();
            self.ivars().thumb_view.set(thumb_view).unwrap();
            self.ivars().right_sidebar_outline_scroll.set(right_outline_scroll).unwrap();
            let _ = self.ivars().pdf_outline_delegate.set(pdf_outline_del);

            unsafe {
                split_vc.addSplitViewItem(&explorer_item);
                split_vc.addSplitViewItem(&main_item);
                split_vc.addSplitViewItem(&navigation_item);
            }
            self.ivars().window.set(window).unwrap();
            self.ivars().pdf_view.set(pdf_view).unwrap();
            let split_vc_base = unsafe { Retained::cast_unchecked::<NSSplitViewController>(split_vc) };
            self.ivars().split_vc.set(split_vc_base).unwrap();
            
            let has_doc = unsafe { self.ivars().pdf_view.get().unwrap().document().is_some() };
            if has_doc {
                self.ivars().window.get().unwrap().setContentViewController(Some(&*self.ivars().split_vc.get().unwrap()));
            } else {
                let welcome_vc = NSViewController::new(mtm);
                let welcome_view = unsafe { NSView::initWithFrame(NSView::alloc(mtm), rect) };
                
                let target: &objc2::runtime::AnyObject = unsafe { &*(self as *const AppDelegate as *const AnyObject) };
                let open_pdf_btn = unsafe { NSButton::buttonWithTitle_target_action(objc2_foundation::ns_string!("Open PDF..."), Some(target), Some(objc2::sel!(openPdf:)), mtm) };
                let open_folder_btn = unsafe { NSButton::buttonWithTitle_target_action(objc2_foundation::ns_string!("Open Folder..."), Some(target), Some(objc2::sel!(openFolder:)), mtm) };
                
                unsafe {
                    open_pdf_btn.setBezelStyle(objc2_app_kit::NSBezelStyle::Push);
                    open_folder_btn.setBezelStyle(objc2_app_kit::NSBezelStyle::Push);
                    
                    let stack = NSStackView::stackViewWithViews(&NSArray::from_slice(&[open_pdf_btn.as_ref(), open_folder_btn.as_ref()]), mtm);
                    stack.setOrientation(objc2_app_kit::NSUserInterfaceLayoutOrientation::Vertical);
                    stack.setSpacing(20.0);
                    
                    let stack_rect = NSRect::new(NSPoint::new(1000.0/2.0 - 100.0, 700.0/2.0 - 50.0), NSSize::new(200.0, 100.0));
                    stack.setFrame(stack_rect);
                    stack.setAutoresizingMask(
                        objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin | 
                        objc2_app_kit::NSAutoresizingMaskOptions::ViewMaxXMargin | 
                        objc2_app_kit::NSAutoresizingMaskOptions::ViewMinYMargin | 
                        objc2_app_kit::NSAutoresizingMaskOptions::ViewMaxYMargin
                    );
                    
                    welcome_view.addSubview(&stack);
                    welcome_vc.setView(&welcome_view);
                }
                
                self.ivars().window.get().unwrap().setContentViewController(Some(&welcome_vc));
            }
            
            self.ivars().window.get().unwrap().makeKeyAndOrderFront(None);
            
            if let Some(url) = self.ivars().pending_url.borrow_mut().take() {
                let urls = NSArray::from_slice(&[&*url]);
                let app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
                let _: () = unsafe { msg_send![self, application: &*app, openURLs: &*urls] };
            }
        }
    }
);

impl AppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppDelegateIvars::default());
        unsafe { objc2::msg_send_id![super(this), init] }
    }
}

fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    
    let delegate = AppDelegate::new(mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    
    app.run();
}

pub fn download_file(url_str: String, outline_raw: usize, app_ptr: usize) {
    let result = std::panic::catch_unwind(move || {
        unsafe {
            use objc2::msg_send;
            use objc2::runtime::AnyObject;
            use objc2_foundation::{NSURL, NSString, NSFileManager};
            use objc2_app_kit::NSOutlineView;
            
            crate::log_icloud!("[DIAGNOSTIC] DOWNLOAD THREAD ENTER: path = {}", url_str);
            crate::log_icloud!("[DIAGNOSTIC] START DOWNLOAD API BEFORE");
            
            let nsurl = NSURL::fileURLWithPath(&NSString::from_str(&url_str));
            let fm = NSFileManager::defaultManager();
            let mut err: *mut AnyObject = std::ptr::null_mut();
            let ok: bool = msg_send![&fm, startDownloadingUbiquitousItemAtURL: &*nsurl, error: &mut err];
            
            crate::log_icloud!("[DIAGNOSTIC] START DOWNLOAD API AFTER: success = {}", ok);
            crate::log_icloud!("[DIAGNOSTIC] POLL LOOP ENTER");
            
            let mut iterations = 0;
            loop {
                iterations += 1;
                crate::log_icloud!("[DIAGNOSTIC] POLL ITERATION #{} START", iterations);
                
                if iterations > 120 {
                    crate::log_icloud!("[DIAGNOSTIC] POLL LOOP EXIT reason=timeout");
                    if app_ptr != 0 {
                        let app_raw = app_ptr as *mut AnyObject;
                        let path_str = NSString::from_str(&url_str);
                        let _: () = msg_send![app_raw, performSelectorOnMainThread: objc2::sel!(downloadDidTimeout:), withObject: &*path_str, waitUntilDone: false];
                    }
                    break;
                }

                crate::log_icloud!("[DIAGNOSTIC] METADATA QUERY START");
                let mut is_dl_val: Option<objc2::rc::Retained<AnyObject>> = None;
                let key = NSString::from_str("NSURLUbiquitousItemIsDownloadingKey");
                let ok1 = nsurl.getResourceValue_forKey_error(&mut is_dl_val, &key).is_ok();
                
                let mut still_dl = false;
                if ok1 {
                    if let Some(val) = is_dl_val {
                        still_dl = msg_send![&val, boolValue];
                    }
                }
                
                let mut stat_val: Option<objc2::rc::Retained<AnyObject>> = None;
                let key_stat = NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");
                let ok_stat = nsurl.getResourceValue_forKey_error(&mut stat_val, &key_stat).is_ok();
                
                let mut is_current = false;
                let mut is_downloaded = false;
                let mut stat_string = "Unknown".to_string();
                if ok_stat {
                    if let Some(stat_obj) = stat_val {
                        let utf8: *const std::ffi::c_char = msg_send![&stat_obj, UTF8String];
                        if !utf8.is_null() {
                            stat_string = std::ffi::CStr::from_ptr(utf8).to_string_lossy().into_owned();
                        }
                        let dl_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusDownloaded");
                        let current_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusCurrent");
                        is_downloaded = msg_send![&stat_obj, isEqualToString: &*dl_id];
                        is_current = msg_send![&stat_obj, isEqualToString: &*current_id];
                    }
                }
                
                let mut prog_val: Option<objc2::rc::Retained<AnyObject>> = None;
                let key_prog = NSString::from_str("NSURLUbiquitousItemPercentDownloadedKey");
                let ok_prog = nsurl.getResourceValue_forKey_error(&mut prog_val, &key_prog).is_ok();
                let mut percent = 0.0;
                if ok_prog {
                    if let Some(val) = prog_val {
                        percent = msg_send![&val, doubleValue];
                    }
                }
                
                let is_readable = fm.isReadableFileAtPath(&NSString::from_str(&url_str));
                let exists = fm.fileExistsAtPath(&NSString::from_str(&url_str));
                
                let mut size: u64 = 0;
                if let Ok(meta) = std::fs::metadata(&url_str) {
                    size = meta.len();
                }
                
                crate::log_icloud!("[DIAGNOSTIC] METADATA QUERY END");
                crate::log_icloud!("[DIAGNOSTIC] POLL ITERATION #{} RESULT: status={}, isDownloading={}, isDownloaded={}, percent={}, exists={}, readable={}, size={}", 
                    iterations, stat_string, still_dl, is_downloaded, percent, exists, is_readable, size);
                
                if is_downloaded || is_current || (exists && is_readable && size > 0 && percent == 100.0) || (exists && is_readable && size > 0 && stat_string == "NSURLUbiquitousItemDownloadingStatusNotDownloaded") {
                    crate::log_pdf!("[DIAGNOSTIC] DOWNLOAD COMPLETE DETECTED: path = {}", url_str);
                    crate::log_icloud!("[DIAGNOSTIC] POLL LOOP EXIT reason=completed");
                    if app_ptr != 0 {
                        let app_raw = app_ptr as *mut AnyObject;
                        let path_str = NSString::from_str(&url_str);
                        let _: () = msg_send![app_raw, performSelectorOnMainThread: objc2::sel!(downloadDidComplete:), withObject: &*path_str, waitUntilDone: false];
                    }
                    break;
                } else {
                    crate::log_icloud!("[DIAGNOSTIC] SLEEP");
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        }
    });
    
    if let Err(err) = result {
        if let Some(s) = err.downcast_ref::<String>() {
            crate::log_error!("[DIAGNOSTIC] DOWNLOAD THREAD PANICKED: {}", s);
        } else if let Some(s) = err.downcast_ref::<&str>() {
            crate::log_error!("[DIAGNOSTIC] DOWNLOAD THREAD PANICKED: {}", s);
        } else {
            crate::log_error!("[DIAGNOSTIC] DOWNLOAD THREAD PANICKED: {:?}", err);
        }
    }
}
pub fn is_file_locally_usable(url_str: &str) -> bool {
    unsafe {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use objc2_foundation::{NSURL, NSString, NSFileManager};
        
        let nsurl = NSURL::fileURLWithPath(&NSString::from_str(url_str));
        let fm = NSFileManager::defaultManager();
        
        let mut stat_val: Option<objc2::rc::Retained<AnyObject>> = None;
        let key_stat = NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");
        let ok_stat = nsurl.getResourceValue_forKey_error(&mut stat_val, &key_stat).is_ok();
        
        if ok_stat {
            if let Some(stat_obj) = stat_val {
                let dl_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusDownloaded");
                let current_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusCurrent");
                let is_downloaded: bool = msg_send![&stat_obj, isEqualToString: &*dl_id];
                let is_current: bool = msg_send![&stat_obj, isEqualToString: &*current_id];
                if is_downloaded || is_current {
                    return true;
                }
            }
        }
        
        // Fallback: check if we can read it and size > 0
        if fm.isReadableFileAtPath(&NSString::from_str(url_str)) {
            if let Ok(meta) = std::fs::metadata(url_str) {
                if meta.len() > 0 {
                    // Try to instantiate a PDFDocument as ultimate fallback?
                    // Might be too slow for general UI, but if it's readable and size > 0 it's likely local.
                    return true;
                }
            }
        }
        false
    }
}

pub fn refresh_file_status(url_str: String, item_raw: usize, outline_raw: usize, node_ptr: usize) {

    unsafe {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use objc2_foundation::{NSURL, NSString};
        use objc2_app_kit::NSOutlineView;
        
        let nsurl = NSURL::fileURLWithPath(&NSString::from_str(&url_str));
        let node = &*(node_ptr as *const crate::workspace::FileNode);
        
        let mut new_status = node.ivars().icloud_status.get();
        let mut new_progress = node.ivars().download_progress.get();
        let old_status = new_status;
        let old_progress = new_progress;
        
        let mut is_dl_val: Option<objc2::rc::Retained<AnyObject>> = None;
        let key = NSString::from_str("NSURLUbiquitousItemIsDownloadingKey");
        let ok = nsurl.getResourceValue_forKey_error(&mut is_dl_val, &key).is_ok();
        
        let mut is_downloading = false;
        if ok {
            if let Some(val) = is_dl_val {
                is_downloading = msg_send![&val, boolValue];
            }
        }
        
        if is_downloading {
            new_status = 2; // Downloading
            let mut prog_val: *mut AnyObject = std::ptr::null_mut();
            let key_prog = NSString::from_str("NSURLUbiquitousItemPercentDownloadedKey");
            let ok_prog: bool = msg_send![&nsurl, getResourceValue: &mut prog_val, forKey: &*key_prog, error: std::ptr::null_mut::<AnyObject>()];
            if ok_prog && !prog_val.is_null() {
                new_progress = msg_send![prog_val, doubleValue];
            }
        } else {
            let mut stat_val: *mut AnyObject = std::ptr::null_mut();
            let key_stat = NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");
            let ok_stat: bool = msg_send![&nsurl, getResourceValue: &mut stat_val, forKey: &*key_stat, error: std::ptr::null_mut::<AnyObject>()];
            if ok_stat && !stat_val.is_null() {
                let stat_str = stat_val as *mut NSString;
                let dl_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusDownloaded");
                let current_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusCurrent");
                let not_dl_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusNotDownloaded");
                
                let is_dl: bool = msg_send![stat_str, isEqualToString: &*dl_id];
                let is_cur: bool = msg_send![stat_str, isEqualToString: &*current_id];
                let is_not_dl: bool = msg_send![stat_str, isEqualToString: &*not_dl_id];
                
                if is_dl || is_cur {
                    new_status = 0; // Local
                } else if is_not_dl {
                    new_status = 1; // Cloud-only
                }
            } else {
                new_status = 0; // Fallback to local if no iCloud
            }
        }
        
        // Ensure we properly release the retained item whether we update or not
        let i_ptr = item_raw as *mut AnyObject;
        let retained_item: objc2::rc::Retained<AnyObject> = objc2::rc::Retained::from_raw(i_ptr).unwrap();
        
        if new_status != old_status || (new_status == 2 && (new_progress - old_progress).abs() > 0.05) {
            node.ivars().icloud_status.set(new_status);
            node.ivars().download_progress.set(new_progress);
            
            let o_ptr = outline_raw as *mut NSOutlineView;
            let i_ptr2 = &*retained_item as *const AnyObject as *mut AnyObject;
            let _: () = msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr2, waitUntilDone: false];
        }
    }
}
mod test_outline;
mod pdf_outline;
