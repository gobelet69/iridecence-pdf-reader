use objc2_foundation::NSInteger;
use std::cell::OnceCell;
use objc2::rc::Retained;
use objc2::{define_class, msg_send, sel, ClassType, DeclaredClass, MainThreadOnly, AnyThread};
use objc2::runtime::{ProtocolObject, AnyObject};

mod workspace;
mod workspace_delegate;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSBackingStoreType, 
    NSWindow, NSWindowStyleMask, NSOpenPanel, NSModalResponseOK,
    NSSplitViewController, NSSplitView, NSSplitViewItem, NSViewController,
    NSToolbar, NSToolbarDelegate, NSToolbarItem, NSWindowToolbarStyle,
    NSGlassEffectView, NSGlassEffectViewStyle, NSToolbarItemIdentifier,
    NSToolbarDisplayMode, NSMenu, NSMenuItem, NSStackView, NSView, NSButton,
    NSUserInterfaceLayoutOrientation, NSBezelStyle, NSSegmentedControl, NSSegmentSwitchTracking,
    NSScrollView, NSOutlineView, NSTableColumn, NSOutlineViewDelegate, NSOutlineViewDataSource
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
            
            let zoom_in_id = objc2_foundation::ns_string!("ZoomIn");
            let zoom_out_id = objc2_foundation::ns_string!("ZoomOut");
            let search_id = objc2_foundation::ns_string!("Search");
            let sidebar_tabs_id = objc2_foundation::ns_string!("SidebarTabs");
            
            if item_identifier.isEqualToString(sidebar_tabs_id) {
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
            }
            
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
                    objc2_foundation::ns_string!("SidebarTabs"),
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
                    objc2_foundation::ns_string!("SidebarTabs"),
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
                    unsafe { window.setContentViewController(Some(&*self.ivars().split_vc.get().unwrap())) };
                    
                    let path = url.path().unwrap().to_string();
                    let name = url.lastPathComponent().unwrap().to_string();
                    println!("Opening folder: {}", path); let root = crate::workspace::FileNode::new(path, name, true);
                    
                    if let Some(del) = self.ivars().workspace_delegate.get() {
                        del.ivars().root_node.borrow_mut().replace(root);
                        if let Some(scroll) = self.ivars().outline_scroll.get() {
                            if let Some(view) = unsafe { scroll.documentView() } {
                                if let Some(outline) = view.downcast_ref::<NSOutlineView>() { 
                                    println!("Reloading outline view!"); unsafe { outline.reloadData() };
                                    unsafe { outline.expandItem_expandChildren(None, true) };
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
                            unsafe { window.setContentViewController(Some(&*self.ivars().split_vc.get().unwrap())) };
                        }
                    }
                    
                    if let Some(del) = self.ivars().workspace_delegate.get() {
                        del.ivars().root_node.borrow_mut().replace(root);
                        if let Some(scroll) = self.ivars().outline_scroll.get() {
                            if let Some(view) = unsafe { scroll.documentView() } {
                                if let Some(outline) = view.downcast_ref::<NSOutlineView>() { 
                                    unsafe { outline.reloadData() };
                                    unsafe { outline.expandItem_expandChildren(None, true) };
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

        #[unsafe(method(workspaceClicked:))]
        fn workspace_clicked(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(outline) = sender.downcast_ref::<NSOutlineView>() {
                    let mtm = MainThreadMarker::new().unwrap();
                    let app = NSApplication::sharedApplication(mtm);
                    let mut is_enter = false;
                    if let Some(event) = app.currentEvent() {
                        let ev_type = event.r#type();
                        if ev_type == objc2_app_kit::NSEventType::KeyDown {
                            is_enter = true;
                        }
                    }
                    if !is_enter { return; }
                    
                    let clicked_row = outline.clickedRow();
                    let row = if clicked_row >= 0 { clicked_row } else { outline.selectedRow() };
                    
                    if row >= 0 {
                        if let Some(item) = unsafe { outline.itemAtRow(row) } {
                            let node: &crate::workspace::FileNode = unsafe { &*(&*item as *const AnyObject as *const crate::workspace::FileNode) };
                            if !node.ivars().is_dir.get() {
                                let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&node.ivars().path.borrow())) };
                                
                                let current_doc = unsafe { self.ivars().pdf_view.get().unwrap().document() };
                                let mut already_open = false;
                                if let Some(current_doc) = current_doc {
                                    if let Some(current_url) = unsafe { current_doc.documentURL() } {
                                        if current_url.path().unwrap().to_string() == url.path().unwrap().to_string() { already_open = true; }
                                    }
                                }
                                
                                if !already_open {
                                    let url_str = url.path().unwrap().to_string();
                                    *self.ivars().pending_pdf_path.borrow_mut() = url_str.clone();
                                    let app_ptr = self as *const AppDelegate as usize;
                                    let item_raw = Retained::into_raw(item.clone()) as usize;
                                    let outline_raw = outline as *const _ as *mut NSOutlineView as usize;
                                    let is_icloud = node.ivars().icloud_status.get() == 1 || node.ivars().icloud_status.get() == 2;
                                    let icloud_node_ptr = if is_icloud { node as *const crate::workspace::FileNode as usize } else { 0 };
                                    
                                    if node.ivars().icloud_status.get() == 1 {
                                        unsafe {
                                            let fm = objc2_foundation::NSFileManager::defaultManager();
                                            let mut err: *mut AnyObject = std::ptr::null_mut();
                                            let _: bool = msg_send![&fm, startDownloadingUbiquitousItemAtURL: &*url, error: &mut err];
                                        }
                                        node.ivars().icloud_status.set(2);
                                        node.ivars().download_progress.set(0.0);
                                    }
                                    
                                    std::thread::spawn(move || {
                                        let t0 = std::time::Instant::now();
                                        
                                        // 2. Filesystem access latency check
                                        let _meta = std::fs::metadata(&url_str);
                                        let t1 = std::time::Instant::now();
                                        
                                        // Polling loop for iCloud
                                        if icloud_node_ptr != 0 {
                                            let node_ptr = icloud_node_ptr as *const crate::workspace::FileNode;
                                            let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };
                                            loop {
                                                let mut is_dl_val: *mut AnyObject = std::ptr::null_mut();
                                                let mut error: *mut AnyObject = std::ptr::null_mut();
                                                let key = unsafe { NSString::from_str("NSURLUbiquitousItemIsDownloadingKey") };
                                                let ok: bool = unsafe { msg_send![&nsurl, getResourceValue: &mut is_dl_val, forKey: &*key, error: &mut error] };
                                                
                                                let mut still_dl = false;
                                                if ok && !is_dl_val.is_null() {
                                                    let is_dl: bool = unsafe { msg_send![is_dl_val, boolValue] };
                                                    still_dl = is_dl;
                                                }
                                                
                                                if still_dl {
                                                    let mut prog_val: *mut AnyObject = std::ptr::null_mut();
                                                    let key_prog = unsafe { NSString::from_str("NSURLUbiquitousItemPercentDownloadedKey") };
                                                    let mut error2: *mut AnyObject = std::ptr::null_mut();
                                                    let ok_prog: bool = unsafe { msg_send![&nsurl, getResourceValue: &mut prog_val, forKey: &*key_prog, error: &mut error2] };
                                                    if ok_prog && !prog_val.is_null() {
                                                        let p: f64 = unsafe { msg_send![prog_val, doubleValue] };
                                                        unsafe { (*node_ptr).ivars().download_progress.set(p) };
                                                    }
                                                    
                                                    // Trigger UI update
                                                    let o_ptr = outline_raw as *mut NSOutlineView;
                                                    let i_ptr = item_raw as *mut AnyObject;
                                                    unsafe {
                                                        let _: () = msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr, waitUntilDone: false];
                                                    }
                                                    
                                                    std::thread::sleep(std::time::Duration::from_millis(500));
                                                } else {
                                                    let mut stat_val: *mut AnyObject = std::ptr::null_mut();
                                                    let key_stat = unsafe { NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey") };
                                                    let mut error3: *mut AnyObject = std::ptr::null_mut();
                                                    let ok_stat: bool = unsafe { msg_send![&nsurl, getResourceValue: &mut stat_val, forKey: &*key_stat, error: &mut error3] };
                                                    if ok_stat && !stat_val.is_null() {
                                                        let stat_str = stat_val as *mut NSString;
                                                        let dl_id = unsafe { NSString::from_str("NSURLUbiquitousItemDownloadingStatusDownloaded") };
                                                        let current_id = unsafe { NSString::from_str("NSURLUbiquitousItemDownloadingStatusCurrent") };
                                                        let is_dl: bool = unsafe { msg_send![stat_str, isEqualToString: &*dl_id] };
                                                        let is_cur: bool = unsafe { msg_send![stat_str, isEqualToString: &*current_id] };
                                                        if is_dl || is_cur {
                                                            unsafe { (*node_ptr).ivars().icloud_status.set(0) };
                                                        }
                                                    }
                                                    
                                                    let o_ptr = outline_raw as *mut NSOutlineView;
                                                    let i_ptr = item_raw as *mut AnyObject;
                                                    unsafe {
                                                        let _: () = msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr, waitUntilDone: false];
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                        
                                        // 3. Init PDFDocument
                                        let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };
                                        
                                        // Check iCloud downloading status here if needed
                                        
                                        let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &nsurl) };
                                        let t2 = std::time::Instant::now();
                                        
                                        println!("[PDF TIMING] Click to Metadata: {:?}", t1 - t0);
                                        println!("[PDF TIMING] PDFDocument::initWithURL: {:?}", t2 - t1);
                                        
                                        if let Some(doc) = doc {
                                            let doc_any = unsafe { objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(doc) };
                                            let app_raw = app_ptr as *mut objc2::runtime::AnyObject;
                                            let _: () = unsafe {
                                                objc2::msg_send![app_raw, performSelectorOnMainThread: objc2::sel!(pdfDidLoad:), withObject: &*doc_any, waitUntilDone: false]
                                            };
                                        }
                                    });
                                }
                            } else {
                                let is_expanded = unsafe { outline.isItemExpanded(Some(&*item)) };
                                if is_expanded { unsafe { outline.collapseItem(Some(&*item)) }; }
                                else { unsafe { outline.expandItem(Some(&*item)) }; }
                            }
                        }
                    }
                }
            }
        }

        
        #[unsafe(method(pdfDidLoad:))]
        fn pdf_did_load(&self, doc: Option<&PDFDocument>) {
            if let Some(doc) = doc {
                if let Some(url) = unsafe { doc.documentURL() } {
                    let path = url.path().unwrap().to_string();
                    if path == *self.ivars().pending_pdf_path.borrow() {
                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(doc)) };
                        println!("[PDF TIMING] Async load complete and applied for: {}", path);
                    } else {
                        println!("[PDF TIMING] Discarded obsolete doc: {}", path);
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
                                let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&node.ivars().path.borrow())) };
                                
                                let current_doc = unsafe { self.ivars().pdf_view.get().unwrap().document() };
                                let mut already_open = false;
                                if let Some(current_doc) = current_doc {
                                    if let Some(current_url) = unsafe { current_doc.documentURL() } {
                                        if current_url.path().unwrap().to_string() == url.path().unwrap().to_string() { already_open = true; }
                                    }
                                }
                                
                                if !already_open {
                                    let url_str = url.path().unwrap().to_string();
                                    *self.ivars().pending_pdf_path.borrow_mut() = url_str.clone();
                                    let app_ptr = self as *const AppDelegate as usize;
                                    
                                    std::thread::spawn(move || {
                                        let t0 = std::time::Instant::now();
                                        
                                        // 2. Filesystem access latency check
                                        let _meta = std::fs::metadata(&url_str);
                                        let t1 = std::time::Instant::now();
                                        
                                        // 3. Init PDFDocument
                                        let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };
                                        
                                        // Check iCloud downloading status here if needed
                                        
                                        let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &nsurl) };
                                        let t2 = std::time::Instant::now();
                                        
                                        println!("[PDF TIMING] Click to Metadata: {:?}", t1 - t0);
                                        println!("[PDF TIMING] PDFDocument::initWithURL: {:?}", t2 - t1);
                                        
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
            let thumb_view = unsafe { PDFThumbnailView::initWithFrame(PDFThumbnailView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };
            unsafe {
                thumb_view.setPDFView(Some(&pdf_view));
                // Adaptive thumbnail sizes
                thumb_view.setThumbnailSize(NSSize::new(140.0, 198.0));
                
                // We use msg_send! to bypass missing bindings in objc2_pdf_kit
                let _: () = msg_send![&thumb_view, setMaximumNumberOfColumns: 1usize];
                
                // We can't reduce vertical spacing directly on PDFThumbnailView in standard AppKit easily, 
                // but setting a suitable thumbnail size helps.
                thumb_view.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
            }

            let workspace_del = WorkspaceDelegate::new(mtm);
            let _ = self.ivars().workspace_delegate.set(workspace_del.clone());

            let outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };
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

            let outline_scroll = unsafe { NSScrollView::initWithFrame(NSScrollView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };
            unsafe {
                outline_scroll.setHasVerticalScroller(true);
                outline_scroll.setDocumentView(Some(&outline_view));
                outline_scroll.setDrawsBackground(false);
                outline_scroll.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
            }

            let target_obj: &AnyObject = unsafe { &*(self as *const AppDelegate as *const AnyObject) };
            unsafe { thumb_view.setHidden(true) };

            let sidebar_stack = unsafe { NSStackView::stackViewWithViews(&objc2_foundation::NSArray::from_slice(&[outline_scroll.as_ref(), thumb_view.as_ref()]), mtm) };
            sidebar_stack.setOrientation(objc2_app_kit::NSUserInterfaceLayoutOrientation::Vertical);
            unsafe { sidebar_stack.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable) };
            unsafe { sidebar_stack.setSpacing(0.0) };

            let sidebar_vc = NSViewController::new(mtm);
            unsafe { sidebar_vc.setView(&sidebar_stack) };
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
