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


define_class!(
    #[unsafe(super(NSSplitView))]
    #[name = "CustomSplitView"]
    struct CustomSplitView;

    impl CustomSplitView {
        #[unsafe(method(dividerThickness))]
        fn divider_thickness(&self) -> f64 {
            12.0
        }
        
        #[unsafe(method(drawDividerInRect:))]
        fn draw_divider_in_rect(&self, mut rect: NSRect) {
            rect.origin.x += (rect.size.width - 1.0) / 2.0;
            rect.size.width = 1.0;
            unsafe {
                let color = objc2_app_kit::NSColor::separatorColor();
                color.set();
                objc2_app_kit::NSBezierPath::fillRect(rect);
            }
        }
    }
);

#[derive(Default)]
struct AppDelegateIvars {
    pending_url: std::cell::RefCell<Option<Retained<objc2_foundation::NSURL>>>,
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
                                    let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                                    if let Some(doc) = doc {
                                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(&doc)) };
                                    }
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
                                    let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                                    if let Some(doc) = doc {
                                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(&doc)) };
                                    }
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
            
            let custom_split_alloc: objc2::rc::Allocated<CustomSplitView> = unsafe { objc2::msg_send![CustomSplitView::class(), alloc] };
            let custom_split: Retained<CustomSplitView> = unsafe { objc2::msg_send![custom_split_alloc, initWithFrame: rect] };
            unsafe { split_vc.setView(&custom_split) };


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
                thumb_view.setThumbnailSize(NSSize::new(120.0, 160.0));
                thumb_view.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
            }

            let workspace_del = WorkspaceDelegate::new(mtm);
            let _ = self.ivars().workspace_delegate.set(workspace_del.clone());

            let outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };
            let column = unsafe { NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), objc2_foundation::ns_string!("MainColumn")) };
            unsafe {
                column.setWidth(200.0);
                outline_view.addTableColumn(&column);
                outline_view.setOutlineTableColumn(Some(&column));
                outline_view.setSelectionHighlightStyle(objc2_app_kit::NSTableViewSelectionHighlightStyle::Regular);
                outline_view.setAllowsMultipleSelection(false);
                outline_view.setUsesAlternatingRowBackgroundColors(false);
                outline_view.setHeaderView(None);
                
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
            unsafe { thumb_view.setHidden(true) };

            let sidebar_stack = unsafe { NSStackView::stackViewWithViews(&objc2_foundation::NSArray::from_slice(&[segmented_control.as_ref(), outline_scroll.as_ref(), thumb_view.as_ref()]), mtm) };
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
            self.ivars().split_vc.set(split_vc).unwrap();
            
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
        unsafe { msg_send![super(this), init] }
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
