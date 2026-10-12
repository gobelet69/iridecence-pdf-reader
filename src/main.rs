#![allow(non_snake_case)] // Objective-C delegate methods and selector callbacks follow Cocoa camelCase naming conventions
#![allow(unused_unsafe)] // Nested unsafe blocks inside objc2 macros and FFI boundary wrappers

mod home_screen;

use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Mutex;
pub static ACTIVE_DOWNLOADS: Lazy<Mutex<HashSet<String>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));
use objc2::rc::Retained;
use std::cell::OnceCell;

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

use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{AnyThread, DeclaredClass, MainThreadOnly, Message, define_class, msg_send, sel};

pub mod annotations;
pub mod bookmarks;
pub mod bookmarks_view;
mod debug_console;
mod logger;
pub mod ocr;
mod palette;
pub mod pdf_outline;
pub mod pdf_search;
pub mod settings;
pub mod tabs;
pub mod ui_constants;
pub mod visual_qa;
pub mod window_geometry;
mod workspace;
pub mod workspace_index;
use crate::window_geometry::{WindowGeometry, WindowKind};
mod workspace_delegate;
use crate::workspace_delegate::WorkspaceDelegate;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSBackingStoreType,
    NSButton, NSColor, NSDragOperation, NSDraggingDestination, NSDraggingInfo, NSImage,
    NSImageSymbolConfiguration, NSImageSymbolScale, NSImageView, NSMenu, NSMenuItem, NSOpenPanel,
    NSOutlineView, NSPasteboardTypeFileURL, NSScrollView, NSSegmentSwitchTracking,
    NSSegmentedControl, NSSplitViewController, NSSplitViewItem, NSTableColumn, NSTextField,
    NSToolbar, NSToolbarDelegate, NSToolbarDisplayMode, NSToolbarItem, NSToolbarItemIdentifier,
    NSURLNSPasteboardSupport, NSView, NSViewController, NSVisualEffectBlendingMode,
    NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView, NSWindow, NSWindowDelegate,
    NSWindowStyleMask, NSWindowTitleVisibility, NSWindowToolbarStyle,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSEdgeInsets, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize,
    NSString, NSURL,
};
use objc2_pdf_kit::{PDFDisplayMode, PDFDocument, PDFThumbnailView, PDFView};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AppContentMode {
    Home,
    StandaloneDocument,
    Workspace,
}

impl Default for AppContentMode {
    fn default() -> Self {
        AppContentMode::Home
    }
}

#[derive(Default)]
pub struct AppDelegateIvars {
    pub content_mode: std::cell::Cell<AppContentMode>,
    pub pending_url: std::cell::RefCell<Option<Retained<objc2_foundation::NSURL>>>,
    pub pending_pdf_path: std::cell::RefCell<String>,
    pub window: OnceCell<Retained<NSWindow>>,
    pub pdf_view: OnceCell<Retained<PDFView>>,
    pub split_vc: OnceCell<Retained<NSSplitViewController>>,
    pub workspace_delegate: OnceCell<Retained<workspace_delegate::WorkspaceDelegate>>,
    pub outline_scroll: OnceCell<Retained<NSScrollView>>,
    pub thumb_view: OnceCell<Retained<PDFThumbnailView>>,
    pub explorer_item: OnceCell<Retained<NSSplitViewItem>>,
    pub navigation_item: OnceCell<Retained<NSSplitViewItem>>,
    pub right_sidebar_outline_scroll: OnceCell<Retained<NSScrollView>>,
    pub pdf_outline_delegate: OnceCell<Retained<crate::pdf_outline::PdfOutlineDelegate>>,
    pub right_sidebar_search_container: OnceCell<Retained<NSView>>,
    pub search_field: OnceCell<Retained<NSTextField>>,
    pub pdf_search_delegate: OnceCell<Retained<crate::pdf_search::PdfSearchDelegate>>,
    pub right_sidebar_bookmarks_scroll: OnceCell<Retained<NSScrollView>>,
    pub bookmarks_delegate: OnceCell<Retained<crate::bookmarks_view::BookmarksDelegate>>,
    pub right_sidebar_segmented: OnceCell<Retained<NSSegmentedControl>>,
    pub main_vc: OnceCell<Retained<NSViewController>>,
    pub welcome_view: OnceCell<Retained<NSView>>,
    pub home_screen: OnceCell<crate::home_screen::HomeScreen>,

    pub history: std::cell::RefCell<Vec<(String, usize)>>,
    pub history_index: std::cell::Cell<usize>,
    pub ignore_history_update: std::cell::Cell<bool>,

    pub palette_panel: std::cell::OnceCell<objc2::rc::Retained<objc2_app_kit::NSPanel>>,
    pub palette_delegate:
        std::cell::OnceCell<objc2::rc::Retained<crate::palette::CommandPaletteDelegate>>,

    pub last_file_mtime: std::cell::Cell<Option<std::time::SystemTime>>,
    pub is_presentation_mode: std::cell::Cell<bool>,
    pub is_focus_mode: std::cell::Cell<bool>,
    pub pre_zen_explorer_collapsed: std::cell::Cell<bool>,
    pub pre_zen_inspector_collapsed: std::cell::Cell<bool>,
    pub live_watcher_running: std::cell::Cell<bool>,
    pub is_document_edited: std::cell::Cell<bool>,
    pub tab_manager: std::cell::RefCell<crate::tabs::TabManager>,
    pub tab_bar_view: std::cell::OnceCell<Retained<NSView>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "AppDelegate"]
    #[ivars = AppDelegateIvars]
    pub struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}
    unsafe impl NSWindowDelegate for AppDelegate {
        #[unsafe(method(windowDidBecomeKey:))]
        fn window_did_become_key(&self, _notification: &objc2_foundation::NSNotification) {
            self.check_external_file_change();
        }
    }
    unsafe impl NSDraggingDestination for AppDelegate {
        #[unsafe(method(draggingEntered:))]
        fn dragging_entered(&self, sender: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
            let pboard = sender.draggingPasteboard();
            if unsafe { NSURL::URLFromPasteboard(&pboard).is_some() } {
                NSDragOperation::Copy
            } else {
                NSDragOperation::None
            }
        }

        #[unsafe(method(performDragOperation:))]
        fn perform_drag_operation(&self, sender: &ProtocolObject<dyn NSDraggingInfo>) -> objc2::runtime::Bool {
            let pboard = sender.draggingPasteboard();
            if let Some(url) = unsafe { NSURL::URLFromPasteboard(&pboard) } {
                let mtm = MainThreadMarker::from(self);
                let app = NSApplication::sharedApplication(mtm);
                let urls = NSArray::from_slice(&[&*url]);
                let _: () = unsafe { msg_send![self, application: &*app, openURLs: &*urls] };
                return true.into();
            }
            false.into()
        }
    }

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
                    item.setTarget(Some(&*self));
                    item.setAction(Some(objc2::sel!(toggleSidebar:)));
                    return Retained::into_raw(item);
                } else if item_identifier == objc2_app_kit::NSToolbarToggleInspectorItemIdentifier {
                    let item = NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), item_identifier);
                    let img = NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("sidebar.right"), None);
                    item.setImage(img.as_deref());
                    item.setToolTip(Some(objc2_foundation::ns_string!("Toggle Document Sidebar")));
                    item.setTarget(Some(&*self));
                    item.setAction(Some(objc2::sel!(toggleInspector:)));
                    return Retained::into_raw(item);
                }
            }

            let mtm = MainThreadMarker::new().unwrap();

            let zoom_in_id = objc2_foundation::ns_string!("ZoomIn");
            let zoom_out_id = objc2_foundation::ns_string!("ZoomOut");
            let search_id = objc2_foundation::ns_string!("Search");
            let _sidebar_tabs_id = objc2_foundation::ns_string!("SidebarTabs");



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
        #[unsafe(method(toggleSidebar:))]
        fn toggle_sidebar(&self, _sender: Option<&AnyObject>) {
            if self.ivars().content_mode.get() == AppContentMode::Workspace {
                if let Some(explorer_item) = self.ivars().explorer_item.get() {
                    let saved_frame = self.ivars().window.get().map(|w| unsafe { w.frame() });
                    let is_collapsed = unsafe { explorer_item.isCollapsed() };
                    unsafe { explorer_item.setCollapsed(!is_collapsed) };
                    if let (Some(win), Some(frame)) = (self.ivars().window.get(), saved_frame) {
                        unsafe { win.setFrame_display(frame, true) };
                    }
                }
            }
        }

        #[unsafe(method(toggleInspector:))]
        fn toggle_inspector(&self, _sender: Option<&AnyObject>) {
            if let Some(navigation_item) = self.ivars().navigation_item.get() {
                let saved_frame = self.ivars().window.get().map(|w| unsafe { w.frame() });
                let is_collapsed = unsafe { navigation_item.isCollapsed() };
                unsafe { navigation_item.setCollapsed(!is_collapsed) };
                if let (Some(win), Some(frame)) = (self.ivars().window.get(), saved_frame) {
                    unsafe { win.setFrame_display(frame, true) };
                }
            }
        }

        #[unsafe(method(validateToolbarItem:))]
        fn validate_toolbar_item(&self, item: &NSToolbarItem) -> objc2::runtime::Bool {
            let ident = unsafe { item.itemIdentifier() };
            unsafe {
                if &*ident == objc2_app_kit::NSToolbarToggleSidebarItemIdentifier {
                    return (self.ivars().content_mode.get() == AppContentMode::Workspace).into();
                } else if &*ident == objc2_app_kit::NSToolbarToggleInspectorItemIdentifier {
                    return (self.ivars().content_mode.get() != AppContentMode::Home).into();
                }
            }
            true.into()
        }

        #[unsafe(method(validateMenuItem:))]
        fn validate_menu_item(&self, item: &NSMenuItem) -> objc2::runtime::Bool {
            let sel = unsafe { item.action() };
            if sel == Some(objc2::sel!(toggleSidebar:)) {
                return (self.ivars().content_mode.get() == AppContentMode::Workspace).into();
            }
            if sel == Some(objc2::sel!(closeDocument:)) || sel == Some(objc2::sel!(closeTab:)) {
                return (self.ivars().content_mode.get() != AppContentMode::Home).into();
            }
            if sel == Some(objc2::sel!(selectNextTab:)) || sel == Some(objc2::sel!(selectPreviousTab:)) {
                return (self.ivars().content_mode.get() != AppContentMode::Home && self.ivars().tab_manager.borrow().tab_count() > 1).into();
            }
            if sel == Some(objc2::sel!(search:)) || sel == Some(objc2::sel!(findNext:)) || sel == Some(objc2::sel!(findPrevious:)) {
                return (self.ivars().content_mode.get() != AppContentMode::Home).into();
            }
            if sel == Some(objc2::sel!(saveDocument:)) || sel == Some(objc2::sel!(saveDocumentAs:)) {
                return (self.ivars().content_mode.get() != AppContentMode::Home).into();
            }
            if sel == Some(objc2::sel!(highlightSelection:))
                || sel == Some(objc2::sel!(underlineSelection:))
                || sel == Some(objc2::sel!(strikeoutSelection:))
                || sel == Some(objc2::sel!(addNoteSelection:))
            {
                return (self.ivars().content_mode.get() != AppContentMode::Home).into();
            }
            true.into()
        }

        #[unsafe(method(openPdf:))]
        fn open_pdf(&self, _sender: Option<&AnyObject>) {
            let mtm = MainThreadMarker::new().unwrap();
            let panel = NSOpenPanel::openPanel(mtm);
            WindowGeometry::sanitize_window(&panel, WindowKind::Utility, mtm);
            unsafe {
                panel.setCanChooseFiles(true);
                panel.setCanChooseDirectories(false);
            }
            if unsafe { panel.runModal() } == objc2_app_kit::NSModalResponseOK {
                if let Some(url) = unsafe { panel.URL() } {
                    self.open_file_url(&url);
                }
            }
        }

        #[unsafe(method(openFolder:))]
        fn open_folder(&self, _sender: Option<&AnyObject>) {
            let mtm = MainThreadMarker::new().unwrap();
            let panel = NSOpenPanel::openPanel(mtm);
            WindowGeometry::sanitize_window(&panel, WindowKind::Utility, mtm);
            unsafe {
                panel.setCanChooseFiles(false);
                panel.setCanChooseDirectories(true);
            }
            if unsafe { panel.runModal() } == objc2_app_kit::NSModalResponseOK {
                if let Some(url) = unsafe { panel.URL() } {
                    self.open_folder_url(&url);
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

                if let Some(path) = unsafe { url.path() } {
                    let path_str = path.to_string();
                    if std::path::Path::new(&path_str).is_dir() {
                        self.open_folder_url(&url);
                    } else {
                        self.open_file_url(&url);
                    }
                }
            }
        }

        #[unsafe(method(openPathString:))]
        fn open_path_string(&self, path: &NSString) {
            let path_str = path.to_string();
            let url = unsafe { NSURL::fileURLWithPath(path) };
            if std::path::Path::new(&path_str).is_dir() {
                self.open_folder_url(&url);
            } else {
                self.open_file_url(&url);
            }
        }

        #[unsafe(method(showHomeScreen:))]
        fn show_home_screen(&self, _sender: Option<&AnyObject>) {
            self.ivars().content_mode.set(AppContentMode::Home);
            if let Some(home_screen) = self.ivars().home_screen.get() {
                home_screen.refresh();
                if let Some(window) = self.ivars().window.get() {
                    let mtm = MainThreadMarker::from(self);
                    unsafe {
                        window.setContentViewController(Some(&*home_screen.view_controller()));
                        window.setTitle(objc2_foundation::ns_string!("Iridecence PDF Reader"));
                        window.setTitleVisibility(NSWindowTitleVisibility::Hidden);
                        window.setFrameAutosaveName(objc2_foundation::ns_string!("HomeWindowGeometry"));
                        if let Some(tb) = window.toolbar() {
                            tb.setVisible(false);
                        }
                    }
                    WindowGeometry::sanitize_window(window, WindowKind::Home, mtm);
                }
            }
        }

        #[unsafe(method(newTab:))]
        fn new_tab(&self, sender: Option<&AnyObject>) {
            let _: () = unsafe { objc2::msg_send![self, openPdf: sender] };
        }

        #[unsafe(method(closeTab:))]
        fn close_tab(&self, _sender: Option<&AnyObject>) {
            let active_idx = self.ivars().tab_manager.borrow().active_index;
            self.close_tab_at_index(active_idx);
        }

        #[unsafe(method(selectNextTab:))]
        fn select_next_tab_action(&self, _sender: Option<&AnyObject>) {
            self.select_next_tab();
        }

        #[unsafe(method(selectPreviousTab:))]
        fn select_previous_tab_action(&self, _sender: Option<&AnyObject>) {
            self.select_previous_tab();
        }

        #[unsafe(method(selectTabWithTag:))]
        fn select_tab_with_tag(&self, sender: Option<&AnyObject>) {
            if let Some(btn) = sender.and_then(|s| s.downcast_ref::<NSButton>()) {
                let tag = unsafe { btn.tag() } as usize;
                self.switch_to_tab_index(tag);
            }
        }

        #[unsafe(method(closeTabWithTag:))]
        fn close_tab_with_tag(&self, sender: Option<&AnyObject>) {
            if let Some(btn) = sender.and_then(|s| s.downcast_ref::<NSButton>()) {
                let tag = unsafe { btn.tag() } as usize;
                self.close_tab_at_index(tag);
            }
        }

        #[unsafe(method(closeDocument:))]
        fn close_document(&self, _sender: Option<&AnyObject>) {
            let active_idx = self.ivars().tab_manager.borrow().active_index;
            self.close_tab_at_index(active_idx);
        }

        #[unsafe(method(ocrCurrentPage:))]
        fn ocr_current_page(&self, _sender: Option<&AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    if let Some(page) = unsafe { pdf_view.currentPage() } {
                        let page_idx = unsafe { doc.indexForPage(&page) };
                        let total_pages = unsafe { doc.pageCount() };
                        if let Some(tiff_data) = crate::ocr::extract_page_tiff_data(&page) {
                            let mtm = MainThreadMarker::from(self);
                            match crate::ocr::perform_vision_ocr(&tiff_data) {
                                Ok(text) => {
                                    let pboard = unsafe { objc2_app_kit::NSPasteboard::generalPasteboard() };
                                    unsafe {
                                        pboard.clearContents();
                                        pboard.setString_forType(&NSString::from_str(&text), objc2_app_kit::NSPasteboardTypeString);
                                    }
                                    crate::ocr::show_ocr_result_panel(&text, page_idx, total_pages, mtm);
                                }
                                Err(err) => {
                                    let alert = objc2_app_kit::NSAlert::new(mtm);
                                    unsafe {
                                        alert.setMessageText(&NSString::from_str("OCR Recognition Failed"));
                                        alert.setInformativeText(&NSString::from_str(&err));
                                        alert.addButtonWithTitle(&NSString::from_str("OK"));
                                        alert.runModal();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        #[unsafe(method(ocrEntireDocument:))]
        fn ocr_entire_document(&self, _sender: Option<&AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    let total_pages = unsafe { doc.pageCount() };
                    let mut all_text = String::new();
                    let mtm = MainThreadMarker::from(self);

                    for i in 0..total_pages {
                        if let Some(page) = unsafe { doc.pageAtIndex(i) } {
                            if let Some(tiff_data) = crate::ocr::extract_page_tiff_data(&page) {
                                if let Ok(text) = crate::ocr::perform_vision_ocr(&tiff_data) {
                                    if !text.is_empty() {
                                        if !all_text.is_empty() {
                                            all_text.push_str("\n\n");
                                        }
                                        all_text.push_str(&format!("--- Page {} of {} ---\n{}", i + 1, total_pages, text));
                                    }
                                }
                            }
                        }
                    }

                    if all_text.is_empty() {
                        all_text = "No text recognized in document.".to_string();
                    }

                    let pboard = unsafe { objc2_app_kit::NSPasteboard::generalPasteboard() };
                    unsafe {
                        pboard.clearContents();
                        pboard.setString_forType(&NSString::from_str(&all_text), objc2_app_kit::NSPasteboardTypeString);
                    }
                    crate::ocr::show_ocr_result_panel(&all_text, 0, total_pages, mtm);
                }
            }
        }

        #[unsafe(method(copyOcrCurrentPage:))]
        fn copy_ocr_current_page(&self, _sender: Option<&AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    if let Some(page) = unsafe { pdf_view.currentPage() } {
                        let page_idx = unsafe { doc.indexForPage(&page) };
                        if let Some(tiff_data) = crate::ocr::extract_page_tiff_data(&page) {
                            let mtm = MainThreadMarker::from(self);
                            if let Ok(text) = crate::ocr::perform_vision_ocr(&tiff_data) {
                                let pboard = unsafe { objc2_app_kit::NSPasteboard::generalPasteboard() };
                                unsafe {
                                    pboard.clearContents();
                                    pboard.setString_forType(&NSString::from_str(&text), objc2_app_kit::NSPasteboardTypeString);
                                }
                                let alert = objc2_app_kit::NSAlert::new(mtm);
                                unsafe {
                                    alert.setMessageText(&NSString::from_str("OCR Text Copied"));
                                    alert.setInformativeText(&NSString::from_str(&format!("Recognized text from Page {} has been copied to the clipboard.", page_idx + 1)));
                                    alert.addButtonWithTitle(&NSString::from_str("OK"));
                                    alert.runModal();
                                }
                            }
                        }
                    }
                }
            }
        }

        #[unsafe(method(openSettings:))]
        fn open_settings(&self, _sender: Option<&AnyObject>) {
            let mtm = MainThreadMarker::from(self);
            crate::settings::show_settings_window(mtm);
        }

        #[unsafe(method(highlightSelection:))]
        fn highlight_selection(&self, _sender: Option<&AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if crate::annotations::add_annotation_to_selection(pdf_view, crate::annotations::AnnotationType::Highlight, None) {
                    self.mark_current_tab_edited(true);
                    self.refresh_bookmarks_view();
                }
            }
        }

        #[unsafe(method(underlineSelection:))]
        fn underline_selection(&self, _sender: Option<&AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if crate::annotations::add_annotation_to_selection(pdf_view, crate::annotations::AnnotationType::Underline, None) {
                    self.mark_current_tab_edited(true);
                    self.refresh_bookmarks_view();
                }
            }
        }

        #[unsafe(method(strikeoutSelection:))]
        fn strikeout_selection(&self, _sender: Option<&AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if crate::annotations::add_annotation_to_selection(pdf_view, crate::annotations::AnnotationType::Strikeout, None) {
                    self.mark_current_tab_edited(true);
                    self.refresh_bookmarks_view();
                }
            }
        }

        #[unsafe(method(addNoteSelection:))]
        fn add_note_selection(&self, _sender: Option<&AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                let mtm = MainThreadMarker::from(self);
                let alert = objc2_app_kit::NSAlert::new(mtm);
                let input = unsafe {
                    NSTextField::initWithFrame(
                        NSTextField::alloc(mtm),
                        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(260.0, 24.0)),
                    )
                };
                unsafe {
                    alert.setMessageText(&NSString::from_str("Add Note Annotation"));
                    alert.setInformativeText(&NSString::from_str("Enter note text for this selection:"));
                    alert.addButtonWithTitle(&NSString::from_str("Add Note"));
                    alert.addButtonWithTitle(&NSString::from_str("Cancel"));
                    alert.setAccessoryView(Some(&input));
                }
                if unsafe { alert.runModal() } == objc2_app_kit::NSAlertFirstButtonReturn {
                    let note_text = unsafe { input.stringValue() }.to_string();
                    let trimmed = note_text.trim();
                    if !trimmed.is_empty() {
                        if crate::annotations::add_annotation_to_selection(pdf_view, crate::annotations::AnnotationType::TextNote, Some(trimmed)) {
                            self.mark_current_tab_edited(true);
                            self.refresh_bookmarks_view();
                        }
                    }
                }
            }
        }

        #[unsafe(method(saveDocument:))]
        fn save_document(&self, sender: Option<&AnyObject>) {
            let path = self.ivars().pending_pdf_path.borrow().clone();
            if path.contains("/Documents/MASTER") {
                let mtm = MainThreadMarker::from(self);
                let alert = objc2_app_kit::NSAlert::new(mtm);
                unsafe {
                    alert.setMessageText(&NSString::from_str("Read-Only Protection"));
                    alert.setInformativeText(&NSString::from_str("The MASTER directory is strictly read-only. Please use 'Save As...' to save to a different location."));
                    alert.addButtonWithTitle(&NSString::from_str("Save As..."));
                    alert.addButtonWithTitle(&NSString::from_str("Cancel"));
                }
                if unsafe { alert.runModal() } == objc2_app_kit::NSAlertFirstButtonReturn {
                    let _: () = unsafe { objc2::msg_send![self, saveDocumentAs: sender] };
                }
                return;
            }

            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    match crate::annotations::save_document_safely(&doc, &path) {
                        Ok(_) => {
                            self.mark_current_tab_edited(false);
                        }
                        Err(err) => {
                            let mtm = MainThreadMarker::from(self);
                            let alert = objc2_app_kit::NSAlert::new(mtm);
                            unsafe {
                                alert.setMessageText(&NSString::from_str("Error Saving Document"));
                                alert.setInformativeText(&NSString::from_str(&err));
                                alert.addButtonWithTitle(&NSString::from_str("OK"));
                                alert.runModal();
                            }
                        }
                    }
                }
            }
        }

        #[unsafe(method(saveDocumentAs:))]
        fn save_document_as(&self, _sender: Option<&AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    if let Some(window) = self.ivars().window.get() {
                        let mtm = MainThreadMarker::from(self);
                        let path = self.ivars().pending_pdf_path.borrow().clone();
                        let default_name = std::path::Path::new(&path)
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("document.pdf");
                        if let Some(saved_path) = crate::annotations::show_save_panel(window, &doc, default_name, mtm) {
                            *self.ivars().pending_pdf_path.borrow_mut() = saved_path.clone();
                            let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&saved_path)) };
                            if let Some(last_pc) = unsafe { nsurl.lastPathComponent() } {
                                window.setTitle(&last_pc);
                            }
                            self.mark_current_tab_edited(false);
                            self.refresh_bookmarks_view();
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
                        let search_container = self.ivars().right_sidebar_search_container.get();
                        let bookmarks_scroll = self.ivars().right_sidebar_bookmarks_scroll.get();

                        unsafe {
                            thumb_view.setHidden(index != 0);
                            outline_scroll.setHidden(index != 1);
                            if let Some(search_container) = search_container {
                                search_container.setHidden(index != 2);
                            }
                            if let Some(bookmarks_scroll) = bookmarks_scroll {
                                bookmarks_scroll.setHidden(index != 3);
                            }
                        }

                        if index == 3 {
                            self.refresh_bookmarks_view();
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
                if let Some(url) = unsafe { doc.documentURL() } {
                    let path = url.path().unwrap().to_string();
                    let pending = self.ivars().pending_pdf_path.borrow().clone();
                    if path == pending || pending.is_empty() {
                        let mode = if self.ivars().content_mode.get() == AppContentMode::Workspace {
                            AppContentMode::Workspace
                        } else {
                            AppContentMode::StandaloneDocument
                        };
                        self.load_pdf_document(doc, &path, mode);
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
                                let _url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };

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

        #[unsafe(method(search:))]
    fn search(&self, _sender: Option<&AnyObject>) {
        if let Some(segmented) = self.ivars().right_sidebar_segmented.get() {
            unsafe { segmented.setSelected_forSegment(true, 2) };
            let _: () = unsafe { objc2::msg_send![self, switchSidebar: &**segmented] };
        }

        if let Some(nav_item) = self.ivars().navigation_item.get() {
            unsafe { nav_item.setCollapsed(false) };
        }

        if let Some(window) = self.ivars().window.get() {
            if let Some(search_field) = self.ivars().search_field.get() {
                unsafe { window.makeFirstResponder(Some(search_field.as_ref())) };
            }
        }
    }

    #[unsafe(method(findNext:))]
    fn find_next_action(&self, _sender: Option<&AnyObject>) {
        if let Some(del) = self.ivars().pdf_search_delegate.get() {
            del.find_next();
        }
    }

    #[unsafe(method(findPrevious:))]
    fn find_previous_action(&self, _sender: Option<&AnyObject>) {
        if let Some(del) = self.ivars().pdf_search_delegate.get() {
            del.find_previous();
        }
    }
    #[unsafe(method(gotoPage:))]
    fn goto_page(&self, sender: Option<&AnyObject>) {
        if let Some(panel) = self.ivars().palette_panel.get() {
            if let Some(del) = self.ivars().palette_delegate.get() {
                if let Some(tf) = del.ivars().search_field.get() {
                    unsafe { tf.setStringValue(&objc2_foundation::NSString::from_str("p: ")) };
                    let notif = unsafe { objc2_foundation::NSNotification::notificationWithName_object(&objc2_foundation::NSString::from_str(""), None) };
                    unsafe { let _: () = objc2::msg_send![del, controlTextDidChange: &*notif]; }
                }
            }
            if let Some(window) = self.ivars().window.get() {
                unsafe {
                    window.addChildWindow_ordered(panel, objc2_app_kit::NSWindowOrderingMode::Above);
                    panel.makeKeyAndOrderFront(None);
                    if let Some(tf) = self.ivars().palette_delegate.get().unwrap().ivars().search_field.get() {
                        let field_editor = panel.fieldEditor_forObject(true, Some(tf.as_ref()));
                        if let Some(editor) = field_editor {
                            let text_len = tf.stringValue().length();
                            editor.setSelectedRange(objc2_foundation::NSRange::new(text_len, 0));
                        }
                    }
                }
            }
        }
    }


        #[unsafe(method(goBack:))]
        fn go_back(&self, _sender: Option<&AnyObject>) {
            let idx = self.ivars().history_index.get();
            if idx > 1 {
                self.ivars().history_index.set(idx - 1);
                let hist = self.ivars().history.borrow();
                let entry = &hist[idx - 2];
                self.ivars().ignore_history_update.set(true);

                let path = &entry.0;
                let page_idx = entry.1;

                let current_path = self.ivars().pending_pdf_path.borrow().clone();
                if current_path != *path {
                    *self.ivars().pending_pdf_path.borrow_mut() = path.clone();
                    let nsurl = unsafe { objc2_foundation::NSURL::fileURLWithPath(&objc2_foundation::NSString::from_str(path)) };
                    let doc = unsafe { objc2_pdf_kit::PDFDocument::initWithURL(objc2_pdf_kit::PDFDocument::alloc(), &nsurl) };
                    if let Some(doc) = doc {
                        let mode = self.ivars().content_mode.get();
                        self.load_pdf_document(&doc, path, mode);
                        if let Some(pdf_view) = self.ivars().pdf_view.get() {
                            if let Some(page) = unsafe { doc.pageAtIndex(page_idx) } {
                                unsafe { pdf_view.goToPage(&page) };
                            }
                        }
                    }
                } else {
                    if let Some(pdf_view) = self.ivars().pdf_view.get() {
                        if let Some(doc) = unsafe { pdf_view.document() } {
                            if let Some(page) = unsafe { doc.pageAtIndex(page_idx) } {
                                unsafe { pdf_view.goToPage(&page) };
                            }
                        }
                    }
                }

                self.ivars().ignore_history_update.set(false);
            }
        }

        #[unsafe(method(goForward:))]
        fn go_forward(&self, _sender: Option<&AnyObject>) {
            let idx = self.ivars().history_index.get();
            let hist = self.ivars().history.borrow();
            if idx < hist.len() {
                self.ivars().history_index.set(idx + 1);
                let entry = &hist[idx];
                self.ivars().ignore_history_update.set(true);

                let path = &entry.0;
                let page_idx = entry.1;

                let current_path = self.ivars().pending_pdf_path.borrow().clone();
                if current_path != *path {
                    *self.ivars().pending_pdf_path.borrow_mut() = path.clone();
                    let nsurl = unsafe { objc2_foundation::NSURL::fileURLWithPath(&objc2_foundation::NSString::from_str(path)) };
                    let doc = unsafe { objc2_pdf_kit::PDFDocument::initWithURL(objc2_pdf_kit::PDFDocument::alloc(), &nsurl) };
                    if let Some(doc) = doc {
                        let mode = self.ivars().content_mode.get();
                        self.load_pdf_document(&doc, path, mode);
                        if let Some(pdf_view) = self.ivars().pdf_view.get() {
                            if let Some(page) = unsafe { doc.pageAtIndex(page_idx) } {
                                unsafe { pdf_view.goToPage(&page) };
                            }
                        }
                    }
                } else {
                    if let Some(pdf_view) = self.ivars().pdf_view.get() {
                        if let Some(doc) = unsafe { pdf_view.document() } {
                            if let Some(page) = unsafe { doc.pageAtIndex(page_idx) } {
                                unsafe { pdf_view.goToPage(&page) };
                            }
                        }
                    }
                }

                self.ivars().ignore_history_update.set(false);
            }
        }

        #[unsafe(method(restoreReadingPosition:))]
        fn restore_reading_position(&self, _sender: Option<&AnyObject>) {}

        #[unsafe(method(goToPage:))]
        fn go_to_page(&self, _sender: Option<&AnyObject>) {
            if let Some(panel) = self.ivars().palette_panel.get() {
                if let Some(del) = self.ivars().palette_delegate.get() {
                    if let Some(tf) = del.ivars().search_field.get() {
                        unsafe { tf.setStringValue(&objc2_foundation::NSString::from_str("p:")) };
                        del.update_search("p:");
                        unsafe { panel.makeKeyAndOrderFront(None) };
                        if let Some(_window) = self.ivars().window.get() {
                            unsafe { panel.center() };
                        }
                    }
                }
            }
        }

        #[unsafe(method(quickOpen:))]
        fn quick_open(&self, _sender: Option<&AnyObject>) {
            if let Some(panel) = self.ivars().palette_panel.get() {
                if let Some(del) = self.ivars().palette_delegate.get() {
                    if let Some(tf) = del.ivars().search_field.get() {
                        unsafe { tf.setStringValue(&objc2_foundation::NSString::from_str("")) };
                        del.update_search("");
                        unsafe { panel.makeKeyAndOrderFront(None) };
                        if let Some(_window) = self.ivars().window.get() {
                            unsafe { panel.center() };
                        }
                    }
                }
            }
        }

        #[unsafe(method(commandPalette:))]
        fn command_palette(&self, _sender: Option<&AnyObject>) {
            if let Some(panel) = self.ivars().palette_panel.get() {
                if let Some(del) = self.ivars().palette_delegate.get() {
                    if let Some(tf) = del.ivars().search_field.get() {
                        unsafe { tf.setStringValue(&objc2_foundation::NSString::from_str(">")) };
                        del.update_search(">");
                        unsafe { panel.makeKeyAndOrderFront(None) };
                        if let Some(_window) = self.ivars().window.get() {
                            unsafe { panel.center() };
                        }
                    }
                }
            }
        }

        #[unsafe(method(openPath:atPage:))]
        fn open_path_at_page(&self, path: &objc2_foundation::NSString, page: objc2_foundation::NSInteger) {
            let path_str = path.to_string();
            let current = self.ivars().pending_pdf_path.borrow().clone();
            if current != path_str {
                let _: () = unsafe { objc2::msg_send![self, openPathString: path] };
            }
            let _: () = unsafe { objc2::msg_send![self, jumpToPage: page] };
        }

        #[unsafe(method(jumpToPage:))]
        fn jump_to_page(&self, page: objc2_foundation::NSInteger) {
            let _: () = unsafe { objc2::msg_send![self, recordHistory] };
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    let page_idx = page.max(0) as usize;
                    let count = unsafe { doc.pageCount() };
                    if page_idx < count {
                        if let Some(pdf_page) = unsafe { doc.pageAtIndex(page_idx) } {
                            unsafe { pdf_view.goToPage(&pdf_page) };
                        }
                    }
                }
            }
        }

        #[unsafe(method(executePaletteCommand:))]
        fn execute_palette_command(&self, cmd: &objc2_foundation::NSString) {
            let s = cmd.to_string();
            match s.as_str() {
                "openDocument:" => { let _: () = unsafe { objc2::msg_send![self, openPdf: None::<&objc2::runtime::AnyObject>] }; }
                "openFolder:" => { let _: () = unsafe { objc2::msg_send![self, openFolder: None::<&objc2::runtime::AnyObject>] }; }
                "search:" => { let _: () = unsafe { objc2::msg_send![self, search: None::<&objc2::runtime::AnyObject>] }; }
                "toggleSidebar:" => { let _: () = unsafe { objc2::msg_send![self, toggleSidebar: None::<&objc2::runtime::AnyObject>] }; }
                "toggleInspector:" => { let _: () = unsafe { objc2::msg_send![self, toggleInspector: None::<&objc2::runtime::AnyObject>] }; }
                "fitPage:" | "fit_page" => { let _: () = unsafe { objc2::msg_send![self, fitPage: None::<&objc2::runtime::AnyObject>] }; }
                "fitWidth:" | "fit_width" => { let _: () = unsafe { objc2::msg_send![self, fitWidth: None::<&objc2::runtime::AnyObject>] }; }
                "actualSize:" | "actual_size" => { let _: () = unsafe { msg_send![self, actualSize: None::<&objc2::runtime::AnyObject>] }; }
                "toggleBookmark:" | "toggle_bookmark" => { let _: () = unsafe { msg_send![self, toggleBookmark: None::<&objc2::runtime::AnyObject>] }; }
                "togglePresentationMode:" | "toggle_presentation" => { let _: () = unsafe { msg_send![self, togglePresentationMode: None::<&objc2::runtime::AnyObject>] }; }
                "toggleFocusMode:" | "toggle_focus" => { let _: () = unsafe { msg_send![self, toggleFocusMode: None::<&objc2::runtime::AnyObject>] }; }
                "copyPageReference:" | "copy_page_ref" => { let _: () = unsafe { msg_send![self, copyPageReference: None::<&objc2::runtime::AnyObject>] }; }
                "copyWithPageReference:" | "copy_with_page_ref" => { let _: () = unsafe { msg_send![self, copyWithPageReference: None::<&objc2::runtime::AnyObject>] }; }
                "copyAsMarkdownQuote:" | "copy_markdown_quote" => { let _: () = unsafe { msg_send![self, copyAsMarkdownQuote: None::<&objc2::runtime::AnyObject>] }; }
                "searchSelection:" | "search_selection" => { let _: () = unsafe { msg_send![self, searchSelection: None::<&objc2::runtime::AnyObject>] }; }
                "exportNotesMarkdown:" | "export_notes" => { let _: () = unsafe { msg_send![self, exportNotesMarkdown: None::<&objc2::runtime::AnyObject>] }; }
                "showDocumentInfo:" | "get_info" => { let _: () = unsafe { msg_send![self, showDocumentInfo: None::<&objc2::runtime::AnyObject>] }; }
                "setDisplayModeSingle:" | "mode_single" => { let _: () = unsafe { msg_send![self, setDisplayModeSingle: None::<&objc2::runtime::AnyObject>] }; }
                "setDisplayModeSingleContinuous:" | "mode_single_cont" => { let _: () = unsafe { msg_send![self, setDisplayModeSingleContinuous: None::<&objc2::runtime::AnyObject>] }; }
                "setDisplayModeTwoUp:" | "mode_two_up" => { let _: () = unsafe { msg_send![self, setDisplayModeTwoUp: None::<&objc2::runtime::AnyObject>] }; }
                "setDisplayModeTwoUpContinuous:" | "mode_two_up_cont" => { let _: () = unsafe { msg_send![self, setDisplayModeTwoUpContinuous: None::<&objc2::runtime::AnyObject>] }; }
                "highlightSelection:" | "highlight_selection" => { let _: () = unsafe { msg_send![self, highlightSelection: None::<&objc2::runtime::AnyObject>] }; }
                "underlineSelection:" | "underline_selection" => { let _: () = unsafe { msg_send![self, underlineSelection: None::<&objc2::runtime::AnyObject>] }; }
                "strikeoutSelection:" | "strikeout_selection" => { let _: () = unsafe { msg_send![self, strikeoutSelection: None::<&objc2::runtime::AnyObject>] }; }
                "addNoteSelection:" | "add_note_selection" | "add_note" => { let _: () = unsafe { msg_send![self, addNoteSelection: None::<&objc2::runtime::AnyObject>] }; }
                "saveDocument:" | "save_document" => { let _: () = unsafe { msg_send![self, saveDocument: None::<&objc2::runtime::AnyObject>] }; }
                "saveDocumentAs:" | "save_document_as" => { let _: () = unsafe { msg_send![self, saveDocumentAs: None::<&objc2::runtime::AnyObject>] }; }
                "closeDocument:" => { let _: () = unsafe { msg_send![self, closeDocument: None::<&objc2::runtime::AnyObject>] }; }
                "newTab:" | "new_tab" => { let _: () = unsafe { msg_send![self, newTab: None::<&objc2::runtime::AnyObject>] }; }
                "closeTab:" | "close_tab" => { let _: () = unsafe { msg_send![self, closeTab: None::<&objc2::runtime::AnyObject>] }; }
                "selectNextTab:" | "next_tab" => { let _: () = unsafe { msg_send![self, selectNextTab: None::<&objc2::runtime::AnyObject>] }; }
                "selectPreviousTab:" | "prev_tab" => { let _: () = unsafe { msg_send![self, selectPreviousTab: None::<&objc2::runtime::AnyObject>] }; }
                "ocrCurrentPage:" | "ocr_page" | "ocr" => { let _: () = unsafe { msg_send![self, ocrCurrentPage: None::<&objc2::runtime::AnyObject>] }; }
                "ocrEntireDocument:" | "ocr_document" => { let _: () = unsafe { msg_send![self, ocrEntireDocument: None::<&objc2::runtime::AnyObject>] }; }
                "copyOcrCurrentPage:" | "copy_ocr" => { let _: () = unsafe { msg_send![self, copyOcrCurrentPage: None::<&objc2::runtime::AnyObject>] }; }
                "openSettings:" | "open_settings" | "settings" | "preferences" => { let _: () = unsafe { msg_send![self, openSettings: None::<&objc2::runtime::AnyObject>] }; }
                _ => {}
            }
        }

        #[unsafe(method(copyPageReference:))]
        fn copy_page_reference(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    if let Some(page) = unsafe { pdf_view.currentPage() } {
                        let page_idx = unsafe { doc.indexForPage(&page) };
                        let doc_title = unsafe { doc.documentURL() }
                            .and_then(|u| unsafe { u.lastPathComponent() }.map(|s| s.to_string()))
                            .unwrap_or_else(|| "Document".to_string());
                        let text = format!("{}, p. {}", doc_title, page_idx + 1);
                        let pboard = unsafe { objc2_app_kit::NSPasteboard::generalPasteboard() };
                        unsafe {
                            pboard.clearContents();
                            pboard.setString_forType(&objc2_foundation::NSString::from_str(&text), objc2_app_kit::NSPasteboardTypeString);
                        }
                    }
                }
            }
        }

        #[unsafe(method(copyWithPageReference:))]
        fn copy_with_page_reference(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    if let Some(page) = unsafe { pdf_view.currentPage() } {
                        let page_idx = unsafe { doc.indexForPage(&page) };
                        let doc_title = unsafe { doc.documentURL() }
                            .and_then(|u| unsafe { u.lastPathComponent() }.map(|s| s.to_string()))
                            .unwrap_or_else(|| "Document".to_string());
                        let sel_text = unsafe { pdf_view.currentSelection() }
                            .and_then(|s| unsafe { s.string() }.map(|t| t.to_string()))
                            .unwrap_or_default();
                        let text = if !sel_text.is_empty() {
                            format!("\"{}\" ({}, p. {})", sel_text.trim(), doc_title, page_idx + 1)
                        } else {
                            format!("{}, p. {}", doc_title, page_idx + 1)
                        };
                        let pboard = unsafe { objc2_app_kit::NSPasteboard::generalPasteboard() };
                        unsafe {
                            pboard.clearContents();
                            pboard.setString_forType(&objc2_foundation::NSString::from_str(&text), objc2_app_kit::NSPasteboardTypeString);
                        }
                    }
                }
            }
        }

        #[unsafe(method(copyAsMarkdownQuote:))]
        fn copy_as_markdown_quote(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    if let Some(page) = unsafe { pdf_view.currentPage() } {
                        let page_idx = unsafe { doc.indexForPage(&page) };
                        let doc_title = unsafe { doc.documentURL() }
                            .and_then(|u| unsafe { u.lastPathComponent() }.map(|s| s.to_string()))
                            .unwrap_or_else(|| "Document".to_string());
                        let sel_text = unsafe { pdf_view.currentSelection() }
                            .and_then(|s| unsafe { s.string() }.map(|t| t.to_string()))
                            .unwrap_or_default();
                        let quoted = sel_text.lines().map(|l| format!("> {}", l)).collect::<Vec<_>>().join("\n");
                        let text = format!("{}\n> — *{}*, page {}", quoted, doc_title, page_idx + 1);
                        let pboard = unsafe { objc2_app_kit::NSPasteboard::generalPasteboard() };
                        unsafe {
                            pboard.clearContents();
                            pboard.setString_forType(&objc2_foundation::NSString::from_str(&text), objc2_app_kit::NSPasteboardTypeString);
                        }
                    }
                }
            }
        }

        #[unsafe(method(searchSelection:))]
        fn search_selection(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(sel) = unsafe { pdf_view.currentSelection() } {
                    if let Some(s) = unsafe { sel.string() } {
                        let query = s.to_string();
                        if !query.trim().is_empty() {
                            let _: () = unsafe { objc2::msg_send![self, search: None::<&objc2::runtime::AnyObject>] };
                            if let Some(del) = self.ivars().pdf_search_delegate.get() {
                                let ns_q = objc2_foundation::NSString::from_str(&query);
                                if let Some(tf) = del.ivars().search_field.get() {
                                    unsafe { tf.setStringValue(&ns_q) };
                                }
                                del.start_search(&ns_q);
                            }
                        }
                    }
                }
            }
        }

        #[unsafe(method(toggleBookmark:))]
        fn toggle_bookmark(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    if let Some(page) = unsafe { pdf_view.currentPage() } {
                        let page_idx = unsafe { doc.indexForPage(&page) };
                        let path = self.ivars().pending_pdf_path.borrow().clone();
                        if !path.is_empty() {
                            let label = format!("Page {}", page_idx + 1);
                            let _ = crate::bookmarks::toggle_bookmark(&path, page_idx, &label);
                            self.refresh_bookmarks_view();
                        }
                    }
                }
            }
        }

        #[unsafe(method(showDocumentInfo:))]
        fn show_document_info(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    let page_count = unsafe { doc.pageCount() };
                    let path = self.ivars().pending_pdf_path.borrow().clone();
                    let filename = std::path::Path::new(&path)
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("Document");

                    let meta = std::fs::metadata(&path).ok();
                    let size_str = meta.map(|m| {
                        let bytes = m.len();
                        if bytes < 1024 { format!("{} B", bytes) }
                        else if bytes < 1024 * 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
                        else { format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)) }
                    }).unwrap_or_else(|| "Unknown".to_string());

                    let mtm = MainThreadMarker::from(self);
                    let alert = objc2_app_kit::NSAlert::new(mtm);
                    unsafe {
                        alert.setMessageText(&objc2_foundation::NSString::from_str(filename));
                        alert.setInformativeText(&objc2_foundation::NSString::from_str(&format!(
                            "Pages: {}\nFile Size: {}\nLocation: {}\nReading Position: Page {}",
                            page_count,
                            size_str,
                            path,
                            pdf_view.currentPage().map(|p| doc.indexForPage(&p) + 1).unwrap_or(1)
                        )));
                        alert.addButtonWithTitle(&objc2_foundation::NSString::from_str("OK"));
                        let alert_win = alert.window();
                        WindowGeometry::sanitize_window(&alert_win, WindowKind::Utility, mtm);
                        alert.runModal();
                    }
                }
            }
        }

        #[unsafe(method(exportNotesMarkdown:))]
        fn export_notes_markdown(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            let path = self.ivars().pending_pdf_path.borrow().clone();
            if path.is_empty() { return; }
            let filename = std::path::Path::new(&path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("Document");
            let md = crate::bookmarks::export_notes_markdown(&path, filename);
            let pboard = unsafe { objc2_app_kit::NSPasteboard::generalPasteboard() };
            unsafe {
                pboard.clearContents();
                pboard.setString_forType(&objc2_foundation::NSString::from_str(&md), objc2_app_kit::NSPasteboardTypeString);
            }
            let mtm = MainThreadMarker::from(self);
            let alert = objc2_app_kit::NSAlert::new(mtm);
            unsafe {
                alert.setMessageText(&objc2_foundation::NSString::from_str("Notes Exported"));
                alert.setInformativeText(&objc2_foundation::NSString::from_str("Exported Markdown notes and bookmarks have been copied to the Clipboard."));
                alert.addButtonWithTitle(&objc2_foundation::NSString::from_str("OK"));
                alert.runModal();
            }
        }

        #[unsafe(method(setDisplayModeSingle:))]
        fn set_display_mode_single(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                unsafe { pdf_view.setDisplayMode(PDFDisplayMode::SinglePage) };
            }
        }

        #[unsafe(method(setDisplayModeSingleContinuous:))]
        fn set_display_mode_single_continuous(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                unsafe { pdf_view.setDisplayMode(PDFDisplayMode::SinglePageContinuous) };
            }
        }

        #[unsafe(method(setDisplayModeTwoUp:))]
        fn set_display_mode_two_up(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                unsafe { pdf_view.setDisplayMode(PDFDisplayMode::TwoUp) };
            }
        }

        #[unsafe(method(setDisplayModeTwoUpContinuous:))]
        fn set_display_mode_two_up_continuous(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                unsafe { pdf_view.setDisplayMode(PDFDisplayMode::TwoUpContinuous) };
            }
        }

        #[unsafe(method(togglePresentationMode:))]
        fn toggle_presentation_mode(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(window) = self.ivars().window.get() {
                let is_pres = self.ivars().is_presentation_mode.get();
                if !is_pres {
                    if let Some(exp) = self.ivars().explorer_item.get() {
                        self.ivars().pre_zen_explorer_collapsed.set(unsafe { exp.isCollapsed() });
                        unsafe { exp.setCollapsed(true) };
                    }
                    if let Some(nav) = self.ivars().navigation_item.get() {
                        self.ivars().pre_zen_inspector_collapsed.set(unsafe { nav.isCollapsed() });
                        unsafe { nav.setCollapsed(true) };
                    }
                    unsafe { window.toggleFullScreen(None) };
                    self.ivars().is_presentation_mode.set(true);
                } else {
                    if unsafe { window.styleMask().contains(NSWindowStyleMask::FullScreen) } {
                        unsafe { window.toggleFullScreen(None) };
                    }
                    if let Some(exp) = self.ivars().explorer_item.get() {
                        unsafe { exp.setCollapsed(self.ivars().pre_zen_explorer_collapsed.get()) };
                    }
                    if let Some(nav) = self.ivars().navigation_item.get() {
                        unsafe { nav.setCollapsed(self.ivars().pre_zen_inspector_collapsed.get()) };
                    }
                    self.ivars().is_presentation_mode.set(false);
                    let mtm = MainThreadMarker::from(self);
                    WindowGeometry::sanitize_window(window, WindowKind::Reader, mtm);
                }
            }
        }

        #[unsafe(method(toggleFocusMode:))]
        fn toggle_focus_mode(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            let is_focus = self.ivars().is_focus_mode.get();
            if !is_focus {
                if let Some(exp) = self.ivars().explorer_item.get() {
                    self.ivars().pre_zen_explorer_collapsed.set(unsafe { exp.isCollapsed() });
                    unsafe { exp.setCollapsed(true) };
                }
                if let Some(nav) = self.ivars().navigation_item.get() {
                    self.ivars().pre_zen_inspector_collapsed.set(unsafe { nav.isCollapsed() });
                    unsafe { nav.setCollapsed(true) };
                }
                self.ivars().is_focus_mode.set(true);
            } else {
                if let Some(exp) = self.ivars().explorer_item.get() {
                    unsafe { exp.setCollapsed(self.ivars().pre_zen_explorer_collapsed.get()) };
                }
                if let Some(nav) = self.ivars().navigation_item.get() {
                    unsafe { nav.setCollapsed(self.ivars().pre_zen_inspector_collapsed.get()) };
                }
                self.ivars().is_focus_mode.set(false);
            }
        }

    #[unsafe(method(recordHistory))]
    fn record_history(&self) {
        if self.ivars().ignore_history_update.get() {
            return;
        }
        if let Some(pdf_view) = self.ivars().pdf_view.get() {
            if let Some(doc) = unsafe { pdf_view.document() } {
            if let Some(url) = unsafe { doc.documentURL() } {
                if let Some(page) = unsafe { pdf_view.currentPage() } {
                    let idx = unsafe { doc.indexForPage(&page) };
                    let path = url.path().unwrap().to_string();
                    let mut hist = self.ivars().history.borrow_mut();
                    let h_idx = self.ivars().history_index.get();

                    if h_idx > 0 && h_idx <= hist.len() {
                        let last = &hist[h_idx - 1];
                        if last.0 == path && last.1 == idx {
                            return; // Don't record same page
                        }
                    }
                    if h_idx < hist.len() {
                        hist.truncate(h_idx);
                    }
                    hist.push((path, idx));
                    self.ivars().history_index.set(hist.len());
                }
            }
        }
    }
    }

    #[unsafe(method(pdfPageChanged:))]
        fn pdf_page_changed(&self, _notification: &objc2_foundation::NSNotification) {
            let pdf_view = self.ivars().pdf_view.get().unwrap();
            unsafe {
                if let Some(doc) = pdf_view.document() {
                    if let Some(url) = doc.documentURL() {
                        if let Some(page) = pdf_view.currentPage() {
                            let idx = doc.indexForPage(&page);
                            let path = url.path().unwrap().to_string();

                            let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
                            let key = objc2_foundation::NSString::from_str(&format!("PDFPos_{}", path));

                            let dict = objc2_foundation::NSMutableDictionary::<objc2_foundation::NSString, objc2::runtime::AnyObject>::new();
                            let page_idx_key = objc2_foundation::NSString::from_str("PageIndex");
                            let zoom_key = objc2_foundation::NSString::from_str("Zoom");

                            let idx_num: *mut AnyObject = objc2::msg_send![objc2::class!(NSNumber), numberWithUnsignedInteger: idx];
                            dict.setObject_forKey(std::mem::transmute(idx_num), ProtocolObject::from_ref(&*page_idx_key));

                            let zoom_num: *mut AnyObject = objc2::msg_send![objc2::class!(NSNumber), numberWithDouble: pdf_view.scaleFactor()];
                            dict.setObject_forKey(std::mem::transmute(zoom_num), ProtocolObject::from_ref(&*zoom_key));

                            let total_key = objc2_foundation::NSString::from_str("TotalPages");
                            let total_num: *mut AnyObject = objc2::msg_send![objc2::class!(NSNumber), numberWithUnsignedInteger: doc.pageCount()];
                            dict.setObject_forKey(std::mem::transmute(total_num), ProtocolObject::from_ref(&*total_key));

                            let time_key = objc2_foundation::NSString::from_str("LastReadTime");
                            let now_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as f64;
                            let time_num: *mut AnyObject = objc2::msg_send![objc2::class!(NSNumber), numberWithDouble: now_ts];
                            dict.setObject_forKey(std::mem::transmute(time_num), ProtocolObject::from_ref(&*time_key));

                            defaults.setObject_forKey(Some(&dict), &key);


                        }
                    }
                }
            }
        }

        #[unsafe(method(pdfScaleChanged:))]
        fn pdf_scale_changed(&self, _notification: &objc2_foundation::NSNotification) {
            let pdf_view = self.ivars().pdf_view.get().unwrap();
            unsafe {
                if let Some(doc) = pdf_view.document() {
                    if let Some(url) = doc.documentURL() {
                        if let Some(page) = pdf_view.currentPage() {
                            let idx = doc.indexForPage(&page);
                            let path = url.path().unwrap().to_string();

                            let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
                            let key = objc2_foundation::NSString::from_str(&format!("PDFPos_{}", path));

                            let dict = objc2_foundation::NSMutableDictionary::<objc2_foundation::NSString, objc2::runtime::AnyObject>::new();
                            let page_idx_key = objc2_foundation::NSString::from_str("PageIndex");
                            let zoom_key = objc2_foundation::NSString::from_str("Zoom");

                            let idx_num: *mut AnyObject = objc2::msg_send![objc2::class!(NSNumber), numberWithUnsignedInteger: idx];
                            dict.setObject_forKey(std::mem::transmute(idx_num), ProtocolObject::from_ref(&*page_idx_key));

                            let zoom_num: *mut AnyObject = objc2::msg_send![objc2::class!(NSNumber), numberWithDouble: pdf_view.scaleFactor()];
                            dict.setObject_forKey(std::mem::transmute(zoom_num), ProtocolObject::from_ref(&*zoom_key));

                            defaults.setObject_forKey(Some(&dict), &key);
                        }
                    }
                }
            }
        }

        #[unsafe(method(pdfDocumentChanged:))]
        fn pdf_document_changed(&self, _notification: &objc2_foundation::NSNotification) {
            // Already handling in pdfDidLoad
        }

        #[unsafe(method(applicationDidFinishLaunching:))]

        fn application_did_finish_launching(&self, _notification: *mut objc2_foundation::NSNotification) {
            let mtm = MainThreadMarker::new().unwrap();

            // Set up main menu
            let menubar = NSMenu::new(mtm);
            let app_menu_item = NSMenuItem::new(mtm);
            unsafe { menubar.addItem(&app_menu_item) };

            let app_menu = NSMenu::new(mtm);
            let settings_title = objc2_foundation::ns_string!("Settings...");
            let settings_action = sel!(openSettings:);
            let settings_key = objc2_foundation::ns_string!(",");
            let settings_item = unsafe { NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), settings_title, Some(settings_action), settings_key) };

            let quit_title = objc2_foundation::ns_string!("Quit Iridecence PDF Reader");
            let quit_action = sel!(terminate:);
            let quit_key = objc2_foundation::ns_string!("q");
            let quit_item = unsafe { NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), quit_title, Some(quit_action), quit_key) };

            unsafe {
                app_menu.addItem(&settings_item);
                app_menu.addItem(&NSMenuItem::separatorItem(mtm));
                app_menu.addItem(&quit_item);
                app_menu_item.setSubmenu(Some(&app_menu));

                // File menu
                let file_menu_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("File"), None, &objc2_foundation::NSString::from_str(""));
                let file_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &objc2_foundation::NSString::from_str("File"));

                let new_tab_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("New Tab"), Some(objc2::sel!(newTab:)), &objc2_foundation::NSString::from_str("t"));
                let open_pdf_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Open PDF..."), Some(objc2::sel!(openPdf:)), &objc2_foundation::NSString::from_str("o"));
                let open_folder_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Open Folder..."), Some(objc2::sel!(openFolder:)), &objc2_foundation::NSString::from_str("O"));
                let save_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Save"), Some(objc2::sel!(saveDocument:)), &objc2_foundation::NSString::from_str("s"));
                let save_as_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Save As..."), Some(objc2::sel!(saveDocumentAs:)), &objc2_foundation::NSString::from_str("S"));
                let home_screen_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Home Screen"), Some(objc2::sel!(showHomeScreen:)), &objc2_foundation::NSString::from_str("H"));
                let close_tab_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Close Tab"), Some(objc2::sel!(closeTab:)), &objc2_foundation::NSString::from_str("w"));

                file_menu.addItem(&new_tab_item);
                file_menu.addItem(&open_pdf_item);
                file_menu.addItem(&open_folder_item);
                file_menu.addItem(&NSMenuItem::separatorItem(mtm));
                file_menu.addItem(&save_item);
                file_menu.addItem(&save_as_item);
                file_menu.addItem(&NSMenuItem::separatorItem(mtm));
                file_menu.addItem(&home_screen_item);
                file_menu.addItem(&close_tab_item);
                file_menu_item.setSubmenu(Some(&file_menu));
                menubar.addItem(&file_menu_item);

                // Window menu
                let window_menu_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Window"), None, &objc2_foundation::NSString::from_str(""));
                let window_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &objc2_foundation::NSString::from_str("Window"));
                let next_tab_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Show Next Tab"), Some(objc2::sel!(selectNextTab:)), &objc2_foundation::NSString::from_str("}"));
                let prev_tab_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Show Previous Tab"), Some(objc2::sel!(selectPreviousTab:)), &objc2_foundation::NSString::from_str("{"));
                let debug_title = objc2_foundation::NSString::from_str("Debug Console");
                let debug_key = objc2_foundation::NSString::from_str("l");
                let debug_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &debug_title, Some(objc2::sel!(openDebugConsole:)), &debug_key);

                window_menu.addItem(&next_tab_item);
                window_menu.addItem(&prev_tab_item);
                window_menu.addItem(&NSMenuItem::separatorItem(mtm));
                window_menu.addItem(&debug_item);
                window_menu_item.setSubmenu(Some(&window_menu));

                menubar.addItem(&window_menu_item);

                // Edit menu
                let edit_menu_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Edit"), None, &objc2_foundation::NSString::from_str(""));
                let edit_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &objc2_foundation::NSString::from_str("Edit"));
                let highlight_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Highlight Selection"), Some(objc2::sel!(highlightSelection:)), &objc2_foundation::NSString::from_str("H"));
                let underline_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Underline Selection"), Some(objc2::sel!(underlineSelection:)), &objc2_foundation::NSString::from_str("U"));
                let strikeout_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Strikeout Selection"), Some(objc2::sel!(strikeoutSelection:)), &objc2_foundation::NSString::from_str("X"));
                let add_note_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Add Note to Selection..."), Some(objc2::sel!(addNoteSelection:)), &objc2_foundation::NSString::from_str("N"));
                let copy_ref_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Copy Page Reference"), Some(objc2::sel!(copyPageReference:)), &objc2_foundation::NSString::from_str("C"));
                let copy_with_ref_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Copy with Page Reference"), Some(objc2::sel!(copyWithPageReference:)), &objc2_foundation::NSString::from_str("c"));
                let copy_quote_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Copy as Markdown Quote"), Some(objc2::sel!(copyAsMarkdownQuote:)), &objc2_foundation::NSString::from_str("M"));
                let search_sel_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Search Selection in Document"), Some(objc2::sel!(searchSelection:)), &objc2_foundation::NSString::from_str("e"));
                let export_notes_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Export Notes to Markdown"), Some(objc2::sel!(exportNotesMarkdown:)), &objc2_foundation::NSString::from_str(""));

                unsafe {
                    edit_menu.addItem(&highlight_item);
                    edit_menu.addItem(&underline_item);
                    edit_menu.addItem(&strikeout_item);
                    edit_menu.addItem(&add_note_item);
                    edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
                    edit_menu.addItem(&copy_ref_item);
                    edit_menu.addItem(&copy_with_ref_item);
                    edit_menu.addItem(&copy_quote_item);
                    edit_menu.addItem(&search_sel_item);
                    edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
                    let ocr_page_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("OCR Current Page (Apple Vision)"), Some(objc2::sel!(ocrCurrentPage:)), &objc2_foundation::NSString::from_str("R"));
                    let ocr_doc_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("OCR Entire Document..."), Some(objc2::sel!(ocrEntireDocument:)), &objc2_foundation::NSString::from_str(""));
                    let copy_ocr_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Copy Recognized Text (OCR)"), Some(objc2::sel!(copyOcrCurrentPage:)), &objc2_foundation::NSString::from_str(""));
                    edit_menu.addItem(&ocr_page_item);
                    edit_menu.addItem(&ocr_doc_item);
                    edit_menu.addItem(&copy_ocr_item);
                    edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
                    edit_menu.addItem(&export_notes_item);
                    edit_menu_item.setSubmenu(Some(&edit_menu));
                    menubar.addItem(&edit_menu_item);
                }

                let view_menu_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("View"), None, &objc2_foundation::NSString::from_str(""));
                let view_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &objc2_foundation::NSString::from_str("View"));

                let fit_width_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Fit Width"), Some(objc2::sel!(fitWidth:)), &objc2_foundation::NSString::from_str("2"));
                let fit_page_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Fit Page"), Some(objc2::sel!(fitPage:)), &objc2_foundation::NSString::from_str("3"));
                let actual_size_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Actual Size"), Some(objc2::sel!(actualSize:)), &objc2_foundation::NSString::from_str("0"));

                let single_page_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Single Page"), Some(objc2::sel!(setDisplayModeSingle:)), &objc2_foundation::NSString::from_str(""));
                let continuous_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Single Page Continuous"), Some(objc2::sel!(setDisplayModeSingleContinuous:)), &objc2_foundation::NSString::from_str(""));
                let two_up_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Two Pages"), Some(objc2::sel!(setDisplayModeTwoUp:)), &objc2_foundation::NSString::from_str(""));
                let two_up_cont_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Two Pages Continuous"), Some(objc2::sel!(setDisplayModeTwoUpContinuous:)), &objc2_foundation::NSString::from_str(""));

                let presentation_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Presentation Mode"), Some(objc2::sel!(togglePresentationMode:)), &objc2_foundation::NSString::from_str("\r"));
                let focus_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Focus Reading Mode"), Some(objc2::sel!(toggleFocusMode:)), &objc2_foundation::NSString::from_str("F"));

                unsafe {
                    view_menu.addItem(&actual_size_item);
                    view_menu.addItem(&fit_width_item);
                    view_menu.addItem(&fit_page_item);
                    view_menu.addItem(&NSMenuItem::separatorItem(mtm));
                    view_menu.addItem(&single_page_item);
                    view_menu.addItem(&continuous_item);
                    view_menu.addItem(&two_up_item);
                    view_menu.addItem(&two_up_cont_item);
                    view_menu.addItem(&NSMenuItem::separatorItem(mtm));
                    view_menu.addItem(&presentation_item);
                    view_menu.addItem(&focus_item);
                    view_menu_item.setSubmenu(Some(&view_menu));
                    menubar.addItem(&view_menu_item);
                }

                let go_menu_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Go"), None, &objc2_foundation::NSString::from_str(""));
                let go_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &objc2_foundation::NSString::from_str("Go"));

                let search_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Search..."), Some(objc2::sel!(search:)), &objc2_foundation::NSString::from_str("f"));
                let go_back_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Back"), Some(objc2::sel!(goBack:)), &objc2_foundation::NSString::from_str("["));
                let go_forward_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Forward"), Some(objc2::sel!(goForward:)), &objc2_foundation::NSString::from_str("]"));
                let go_to_page_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Go to Page..."), Some(objc2::sel!(goToPage:)), &objc2_foundation::NSString::from_str("l"));
                let quick_open_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Quick Open..."), Some(objc2::sel!(quickOpen:)), &objc2_foundation::NSString::from_str("p"));
                let cmd_palette_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Command Palette..."), Some(objc2::sel!(commandPalette:)), &objc2_foundation::NSString::from_str("P"));
                let bookmark_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Bookmark Page"), Some(objc2::sel!(toggleBookmark:)), &objc2_foundation::NSString::from_str("d"));
                let doc_info_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Document Info..."), Some(objc2::sel!(showDocumentInfo:)), &objc2_foundation::NSString::from_str("i"));

                let find_next_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Find Next"), Some(objc2::sel!(findNext:)), &objc2_foundation::NSString::from_str("g"));
                let find_prev_item = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Find Previous"), Some(objc2::sel!(findPrevious:)), &objc2_foundation::NSString::from_str("G"));

                unsafe {
                    go_menu.addItem(&search_item);
                    go_menu.addItem(&find_next_item);
                    go_menu.addItem(&find_prev_item);
                    go_menu.addItem(&NSMenuItem::separatorItem(mtm));
                    go_menu.addItem(&go_back_item);
                    go_menu.addItem(&go_forward_item);
                    go_menu.addItem(&go_to_page_item);
                    go_menu.addItem(&quick_open_item);
                    go_menu.addItem(&cmd_palette_item);
                    go_menu.addItem(&NSMenuItem::separatorItem(mtm));
                    go_menu.addItem(&bookmark_item);
                    go_menu.addItem(&doc_info_item);
                    go_menu_item.setSubmenu(Some(&go_menu));
                    menubar.addItem(&go_menu_item);
                    NSApplication::sharedApplication(mtm).setMainMenu(Some(&menubar));
                }

            }


            let vis = WindowGeometry::screen_visible_frame(None, mtm);
            let rect = WindowGeometry::default_frame(vis, WindowKind::Home);
            let style = NSWindowStyleMask::Titled
                      | NSWindowStyleMask::Closable
                      | NSWindowStyleMask::Resizable
                      | NSWindowStyleMask::Miniaturizable
                      | NSWindowStyleMask::FullSizeContentView;

            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(mtm),
                    rect,
                    style,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };

            window.setTitle(objc2_foundation::ns_string!("Iridecence PDF Reader"));
            unsafe {
                window.setTitlebarAppearsTransparent(true);
                window.setTitleVisibility(NSWindowTitleVisibility::Hidden);
                window.setDelegate(Some(ProtocolObject::from_ref(self)));
                window.registerForDraggedTypes(&NSArray::from_slice(&[NSPasteboardTypeFileURL]));
                window.setTabbingMode(objc2_app_kit::NSWindowTabbingMode::Preferred);
                window.setTabbingIdentifier(objc2_foundation::ns_string!("com.iridecence.pdfreader.tab"));
            }

            // Restore position but automatically sanitize against visibleFrame
            window.setFrameAutosaveName(objc2_foundation::ns_string!("HomeWindowGeometry"));
            WindowGeometry::sanitize_window(&window, WindowKind::Home, mtm);

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

                // Contextual Menu on PDFView
                let ctx_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &objc2_foundation::NSString::from_str("PDFContext"));
                let ctx_hl = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Highlight Selection"), Some(objc2::sel!(highlightSelection:)), &objc2_foundation::NSString::from_str(""));
                let ctx_ul = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Underline Selection"), Some(objc2::sel!(underlineSelection:)), &objc2_foundation::NSString::from_str(""));
                let ctx_so = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Strikeout Selection"), Some(objc2::sel!(strikeoutSelection:)), &objc2_foundation::NSString::from_str(""));
                let ctx_note = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Add Note to Selection..."), Some(objc2::sel!(addNoteSelection:)), &objc2_foundation::NSString::from_str(""));
                let ctx_c1 = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Copy Page Reference"), Some(objc2::sel!(copyPageReference:)), &objc2_foundation::NSString::from_str(""));
                let ctx_c2 = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Copy with Page Reference"), Some(objc2::sel!(copyWithPageReference:)), &objc2_foundation::NSString::from_str(""));
                let ctx_c3 = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Copy as Markdown Quote"), Some(objc2::sel!(copyAsMarkdownQuote:)), &objc2_foundation::NSString::from_str(""));
                let ctx_srch = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("Search Selection in Document"), Some(objc2::sel!(searchSelection:)), &objc2_foundation::NSString::from_str(""));

                ctx_menu.addItem(&ctx_hl);
                ctx_menu.addItem(&ctx_ul);
                ctx_menu.addItem(&ctx_so);
                ctx_menu.addItem(&ctx_note);
                ctx_menu.addItem(&NSMenuItem::separatorItem(mtm));
                ctx_menu.addItem(&ctx_c1);
                ctx_menu.addItem(&ctx_c2);
                ctx_menu.addItem(&ctx_c3);
                ctx_menu.addItem(&NSMenuItem::separatorItem(mtm));
                ctx_menu.addItem(&ctx_srch);
                let ctx_ocr = NSMenuItem::initWithTitle_action_keyEquivalent(NSMenuItem::alloc(mtm), &objc2_foundation::NSString::from_str("OCR Current Page (Apple Vision)"), Some(objc2::sel!(ocrCurrentPage:)), &objc2_foundation::NSString::from_str(""));
                ctx_menu.addItem(&ctx_ocr);
                pdf_view.setMenu(Some(&ctx_menu));
            }
            let center_container = unsafe { NSView::initWithFrame(NSView::alloc(mtm), rect) };
            center_container.setTranslatesAutoresizingMaskIntoConstraints(false);

            let tab_bar = crate::tabs::create_tab_bar_view(mtm, unsafe { &*(self as *const AppDelegate as *const AnyObject) });
            tab_bar.setTranslatesAutoresizingMaskIntoConstraints(false);
            pdf_view.setTranslatesAutoresizingMaskIntoConstraints(false);

            unsafe {
                center_container.addSubview(&tab_bar);
                center_container.addSubview(&pdf_view);

                tab_bar.topAnchor().constraintEqualToAnchor_constant(&center_container.topAnchor(), 52.0).setActive(true);
                tab_bar.leadingAnchor().constraintEqualToAnchor(&center_container.leadingAnchor()).setActive(true);
                tab_bar.trailingAnchor().constraintEqualToAnchor(&center_container.trailingAnchor()).setActive(true);
                tab_bar.heightAnchor().constraintEqualToConstant(32.0).setActive(true);

                pdf_view.topAnchor().constraintEqualToAnchor(&tab_bar.bottomAnchor()).setActive(true);
                pdf_view.leadingAnchor().constraintEqualToAnchor(&center_container.leadingAnchor()).setActive(true);
                pdf_view.trailingAnchor().constraintEqualToAnchor(&center_container.trailingAnchor()).setActive(true);
                pdf_view.bottomAnchor().constraintEqualToAnchor(&center_container.bottomAnchor()).setActive(true);
            }

            let _ = self.ivars().tab_bar_view.set(tab_bar);

            let main_vc = NSViewController::new(mtm);
            self.ivars().main_vc.set(main_vc.clone()).unwrap();
            unsafe { main_vc.setView(&center_container) };

            let main_item = unsafe { NSSplitViewItem::splitViewItemWithViewController(&main_vc) };
            unsafe { main_item.setHoldingPriority(250.0); }

            // Sidebar View
            let thumb_view = unsafe { PDFThumbnailView::initWithFrame(PDFThumbnailView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                thumb_view.setPDFView(Some(&pdf_view));
                // Adaptive thumbnail sizes
                thumb_view.setThumbnailSize(NSSize::new(100.0, 140.0));

                // We use msg_send! to bypass missing bindings in objc2_pdf_kit
                let _: () = msg_send![&thumb_view, setMaximumNumberOfColumns: 1usize];

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
                outline_view.setBackgroundColor(&NSColor::clearColor());

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

            // Wrap in Liquid Glass NSVisualEffectView
            let explorer_effect = unsafe { NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                explorer_effect.setMaterial(NSVisualEffectMaterial::Sidebar);
                explorer_effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
                explorer_effect.setState(NSVisualEffectState::FollowsWindowActiveState);
                explorer_effect.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                explorer_effect.addSubview(&outline_scroll);
            }

            let explorer_vc = NSViewController::new(mtm);
            unsafe { explorer_vc.setView(&explorer_effect) };
            let explorer_item = unsafe { NSSplitViewItem::sidebarWithViewController(&explorer_vc) };
            unsafe {
                explorer_item.setCanCollapse(true);
                explorer_item.setMinimumThickness(180.0);
                explorer_item.setMaximumThickness(360.0);
                explorer_item.setHoldingPriority(750.0);
            }


            // RIGHT SIDEBAR (Outline + Pages + Search)
            let pdf_outline_del = crate::pdf_outline::PdfOutlineDelegate::new(mtm, pdf_view.clone());
            let right_outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            let right_col = unsafe { NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), objc2_foundation::ns_string!("OutlineColumn")) };
            unsafe {
                right_col.setWidth(230.0);
                right_outline_view.addTableColumn(&right_col);
                right_outline_view.setOutlineTableColumn(Some(&right_col));
                right_outline_view.setHeaderView(None);
                right_outline_view.setBackgroundColor(&NSColor::clearColor());

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

            // Search UI
            let pdf_search_del = crate::pdf_search::PdfSearchDelegate::new(mtm, pdf_view.clone());
            let search_outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            let search_col = unsafe { NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), objc2_foundation::ns_string!("SearchColumn")) };
            unsafe {
                search_col.setWidth(230.0);
                search_outline_view.addTableColumn(&search_col);
                search_outline_view.setOutlineTableColumn(Some(&search_col));
                search_outline_view.setHeaderView(None);
                search_outline_view.setBackgroundColor(&NSColor::clearColor());

                let s_del_ptr: *const crate::pdf_search::PdfSearchDelegate = &*pdf_search_del;
                let _: () = msg_send![&search_outline_view, setDelegate: s_del_ptr];
                let _: () = msg_send![&search_outline_view, setDataSource: s_del_ptr];
            }
            pdf_search_del.ivars().outline_view.set(search_outline_view.clone()).unwrap();

            let search_scroll = unsafe { NSScrollView::initWithFrame(NSScrollView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                search_scroll.setHasVerticalScroller(true);
                search_scroll.setDocumentView(Some(&search_outline_view));
                search_scroll.setDrawsBackground(false);
                search_scroll.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                search_scroll.setHidden(true);
            }

            let search_field = unsafe { NSTextField::textFieldWithString(objc2_foundation::ns_string!(""), mtm) };
            unsafe {
                let s_del_ptr: *const crate::pdf_search::PdfSearchDelegate = &*pdf_search_del;
                let _: () = msg_send![&search_field, setDelegate: s_del_ptr];
                search_field.setPlaceholderString(Some(objc2_foundation::ns_string!("Search...")));
                search_field.setTranslatesAutoresizingMaskIntoConstraints(false);
            }

            let search_container = unsafe { NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                search_container.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                search_container.addSubview(&search_field);
                search_container.addSubview(&search_scroll);
                search_scroll.setTranslatesAutoresizingMaskIntoConstraints(false);

                search_field.topAnchor().constraintEqualToAnchor_constant(&search_container.topAnchor(), 8.0).setActive(true);
                search_field.leadingAnchor().constraintEqualToAnchor_constant(&search_container.leadingAnchor(), 8.0).setActive(true);
                search_field.trailingAnchor().constraintEqualToAnchor_constant(&search_container.trailingAnchor(), -8.0).setActive(true);

                search_scroll.topAnchor().constraintEqualToAnchor_constant(&search_field.bottomAnchor(), 8.0).setActive(true);
                search_scroll.leadingAnchor().constraintEqualToAnchor(&search_container.leadingAnchor()).setActive(true);
                search_scroll.trailingAnchor().constraintEqualToAnchor(&search_container.trailingAnchor()).setActive(true);
                search_scroll.bottomAnchor().constraintEqualToAnchor(&search_container.bottomAnchor()).setActive(true);

                search_container.setHidden(true);
            }

            let bookmarks_del = crate::bookmarks_view::BookmarksDelegate::new(mtm, pdf_view.clone());
            let bookmarks_outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            let bookmarks_col = unsafe { NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), objc2_foundation::ns_string!("BookmarksColumn")) };
            unsafe {
                bookmarks_col.setWidth(230.0);
                bookmarks_outline_view.addTableColumn(&bookmarks_col);
                bookmarks_outline_view.setOutlineTableColumn(Some(&bookmarks_col));
                bookmarks_outline_view.setHeaderView(None);
                bookmarks_outline_view.setBackgroundColor(&NSColor::clearColor());

                let bm_del_ptr: *const crate::bookmarks_view::BookmarksDelegate = &*bookmarks_del;
                let _: () = msg_send![&bookmarks_outline_view, setDelegate: bm_del_ptr];
                let _: () = msg_send![&bookmarks_outline_view, setDataSource: bm_del_ptr];
            }
            bookmarks_del.set_outline_view(bookmarks_outline_view.clone());

            let bookmarks_scroll = unsafe { NSScrollView::initWithFrame(NSScrollView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                bookmarks_scroll.setHasVerticalScroller(true);
                bookmarks_scroll.setDocumentView(Some(&bookmarks_outline_view));
                bookmarks_scroll.setDrawsBackground(false);
                bookmarks_scroll.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                bookmarks_scroll.setHidden(true);
            }

            // Using pure NSView with direct Auto Layout for right_container
            let right_container = unsafe { NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };

            let segmented = unsafe {
                let seg = NSSegmentedControl::new(mtm);
                seg.setSegmentCount(4);
                seg.setTrackingMode(NSSegmentSwitchTracking::SelectOne);

                let icon_config = NSImageSymbolConfiguration::configurationWithPointSize_weight_scale(12.0, 0.0, NSImageSymbolScale::Medium);

                if let Some(img) = NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("doc.on.doc"), None) {
                    let configured = img.imageWithSymbolConfiguration(&icon_config);
                    seg.setImage_forSegment(configured.as_deref().or(Some(&img)), 0);
                }
                seg.setToolTip_forSegment(Some(objc2_foundation::ns_string!("Pages")), 0);

                if let Some(img) = NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("list.bullet.indent"), None) {
                    let configured = img.imageWithSymbolConfiguration(&icon_config);
                    seg.setImage_forSegment(configured.as_deref().or(Some(&img)), 1);
                }
                seg.setToolTip_forSegment(Some(objc2_foundation::ns_string!("Outline")), 1);

                if let Some(img) = NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("magnifyingglass"), None) {
                    let configured = img.imageWithSymbolConfiguration(&icon_config);
                    seg.setImage_forSegment(configured.as_deref().or(Some(&img)), 2);
                }
                seg.setToolTip_forSegment(Some(objc2_foundation::ns_string!("Search")), 2);

                if let Some(img) = NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("bookmark"), None) {
                    let configured = img.imageWithSymbolConfiguration(&icon_config);
                    seg.setImage_forSegment(configured.as_deref().or(Some(&img)), 3);
                }
                seg.setToolTip_forSegment(Some(objc2_foundation::ns_string!("Bookmarks")), 3);

                seg.setTarget(Some(&*(self as *const AppDelegate as *const AnyObject)));
                seg.setAction(Some(objc2::sel!(switchSidebar:)));
                seg
            };
            unsafe {
                let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
                let selected_idx = defaults.integerForKey(objc2_foundation::ns_string!("RightSidebarMode"));
                segmented.setSelected_forSegment(true, selected_idx);

                if selected_idx == 1 {
                    right_outline_scroll.setHidden(false);
                    thumb_view.setHidden(true);
                    search_container.setHidden(true);
                    bookmarks_scroll.setHidden(true);
                } else if selected_idx == 2 {
                    right_outline_scroll.setHidden(true);
                    thumb_view.setHidden(true);
                    search_container.setHidden(false);
                    bookmarks_scroll.setHidden(true);
                } else if selected_idx == 3 {
                    right_outline_scroll.setHidden(true);
                    thumb_view.setHidden(true);
                    search_container.setHidden(true);
                    bookmarks_scroll.setHidden(false);
                } else {
                    right_outline_scroll.setHidden(true);
                    thumb_view.setHidden(false);
                    search_container.setHidden(true);
                    bookmarks_scroll.setHidden(true);
                }

                let seg_wrapper = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 44.0)));
                seg_wrapper.addSubview(&segmented);
                seg_wrapper.setTranslatesAutoresizingMaskIntoConstraints(false);

                segmented.setTranslatesAutoresizingMaskIntoConstraints(false);
                segmented.centerYAnchor().constraintEqualToAnchor(&seg_wrapper.centerYAnchor()).setActive(true);
                segmented.leadingAnchor().constraintEqualToAnchor_constant(&seg_wrapper.leadingAnchor(), 12.0).setActive(true);
                segmented.trailingAnchor().constraintEqualToAnchor_constant(&seg_wrapper.trailingAnchor(), -12.0).setActive(true);

                let separator = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 1.0)));
                separator.setWantsLayer(true);
                if let Some(layer) = separator.layer() {
                    let cg = objc2_app_kit::NSColor::separatorColor().CGColor(); let ptr = Retained::as_ptr(&cg); let _: () = msg_send![&layer, setBackgroundColor: ptr];
                }
                separator.setTranslatesAutoresizingMaskIntoConstraints(false);
                separator.heightAnchor().constraintEqualToConstant(1.0).setActive(true);

                let content_wrapper = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 660.0)));
                content_wrapper.setTranslatesAutoresizingMaskIntoConstraints(false);
                content_wrapper.addSubview(&thumb_view);
                content_wrapper.addSubview(&right_outline_scroll);
                content_wrapper.addSubview(&search_container);
                content_wrapper.addSubview(&bookmarks_scroll);

                thumb_view.setTranslatesAutoresizingMaskIntoConstraints(false);
                right_outline_scroll.setTranslatesAutoresizingMaskIntoConstraints(false);
                search_container.setTranslatesAutoresizingMaskIntoConstraints(false);
                bookmarks_scroll.setTranslatesAutoresizingMaskIntoConstraints(false);

                thumb_view.topAnchor().constraintEqualToAnchor(&content_wrapper.topAnchor()).setActive(true);
                thumb_view.bottomAnchor().constraintEqualToAnchor(&content_wrapper.bottomAnchor()).setActive(true);
                thumb_view.leadingAnchor().constraintEqualToAnchor(&content_wrapper.leadingAnchor()).setActive(true);
                thumb_view.trailingAnchor().constraintEqualToAnchor(&content_wrapper.trailingAnchor()).setActive(true);

                right_outline_scroll.topAnchor().constraintEqualToAnchor(&content_wrapper.topAnchor()).setActive(true);
                right_outline_scroll.bottomAnchor().constraintEqualToAnchor(&content_wrapper.bottomAnchor()).setActive(true);
                right_outline_scroll.leadingAnchor().constraintEqualToAnchor(&content_wrapper.leadingAnchor()).setActive(true);
                right_outline_scroll.trailingAnchor().constraintEqualToAnchor(&content_wrapper.trailingAnchor()).setActive(true);

                search_container.topAnchor().constraintEqualToAnchor(&content_wrapper.topAnchor()).setActive(true);
                search_container.bottomAnchor().constraintEqualToAnchor(&content_wrapper.bottomAnchor()).setActive(true);
                search_container.leadingAnchor().constraintEqualToAnchor(&content_wrapper.leadingAnchor()).setActive(true);
                search_container.trailingAnchor().constraintEqualToAnchor(&content_wrapper.trailingAnchor()).setActive(true);

                bookmarks_scroll.topAnchor().constraintEqualToAnchor(&content_wrapper.topAnchor()).setActive(true);
                bookmarks_scroll.bottomAnchor().constraintEqualToAnchor(&content_wrapper.bottomAnchor()).setActive(true);
                bookmarks_scroll.leadingAnchor().constraintEqualToAnchor(&content_wrapper.leadingAnchor()).setActive(true);
                bookmarks_scroll.trailingAnchor().constraintEqualToAnchor(&content_wrapper.trailingAnchor()).setActive(true);

                right_container.addSubview(&seg_wrapper);
                right_container.addSubview(&separator);
                right_container.addSubview(&content_wrapper);

                seg_wrapper.topAnchor().constraintEqualToAnchor(&right_container.topAnchor()).setActive(true);
                seg_wrapper.leadingAnchor().constraintEqualToAnchor(&right_container.leadingAnchor()).setActive(true);
                seg_wrapper.trailingAnchor().constraintEqualToAnchor(&right_container.trailingAnchor()).setActive(true);
                seg_wrapper.heightAnchor().constraintEqualToConstant(44.0).setActive(true);

                separator.topAnchor().constraintEqualToAnchor(&seg_wrapper.bottomAnchor()).setActive(true);
                separator.leadingAnchor().constraintEqualToAnchor(&right_container.leadingAnchor()).setActive(true);
                separator.trailingAnchor().constraintEqualToAnchor(&right_container.trailingAnchor()).setActive(true);
                separator.heightAnchor().constraintEqualToConstant(1.0).setActive(true);

                content_wrapper.topAnchor().constraintEqualToAnchor(&separator.bottomAnchor()).setActive(true);
                content_wrapper.leadingAnchor().constraintEqualToAnchor(&right_container.leadingAnchor()).setActive(true);
                content_wrapper.trailingAnchor().constraintEqualToAnchor(&right_container.trailingAnchor()).setActive(true);
                content_wrapper.bottomAnchor().constraintEqualToAnchor(&right_container.bottomAnchor()).setActive(true);
            }

            unsafe {
                let nc = objc2_foundation::NSNotificationCenter::defaultCenter();
                nc.addObserver_selector_name_object(
                    &*(self as *const AppDelegate as *const AnyObject),
                    objc2::sel!(thumbnailFrameChanged:),
                    Some(objc2_app_kit::NSViewFrameDidChangeNotification),
                    Some(&*right_container as &AnyObject) // Observe container instead
                );
                right_container.setPostsFrameChangedNotifications(true);
            }

            let _ = self.ivars().right_sidebar_search_container.set(search_container);
            let _ = self.ivars().search_field.set(search_field.clone());
            let _ = self.ivars().right_sidebar_bookmarks_scroll.set(bookmarks_scroll);
            let _ = self.ivars().bookmarks_delegate.set(bookmarks_del);
            let _ = pdf_search_del.ivars().search_field.set(search_field);
            let _ = self.ivars().pdf_search_delegate.set(pdf_search_del);
            let _ = self.ivars().right_sidebar_segmented.set(segmented.clone());


            let right_effect = unsafe { NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                right_effect.setMaterial(NSVisualEffectMaterial::Sidebar);
                right_effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
                right_effect.setState(NSVisualEffectState::FollowsWindowActiveState);
                right_effect.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                right_container.setTranslatesAutoresizingMaskIntoConstraints(false);
                right_effect.addSubview(&right_container);
                right_container.topAnchor().constraintEqualToAnchor_constant(&right_effect.topAnchor(), 52.0).setActive(true);
                right_container.leadingAnchor().constraintEqualToAnchor(&right_effect.leadingAnchor()).setActive(true);
                right_container.trailingAnchor().constraintEqualToAnchor(&right_effect.trailingAnchor()).setActive(true);
                right_container.bottomAnchor().constraintEqualToAnchor(&right_effect.bottomAnchor()).setActive(true);
            }

            let navigation_vc = NSViewController::new(mtm);
            unsafe { navigation_vc.setView(&right_effect) };
            let navigation_item = unsafe { NSSplitViewItem::inspectorWithViewController(&navigation_vc) };
            unsafe {
                navigation_item.setCanCollapse(true);
                navigation_item.setMinimumThickness(180.0);
                navigation_item.setMaximumThickness(360.0);
                navigation_item.setHoldingPriority(750.0);
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

            let (palette_panel, palette_delegate) = crate::palette::create_palette(mtm);
            let _ = self.ivars().palette_panel.set(palette_panel);
            let _ = self.ivars().palette_delegate.set(palette_delegate);


            unsafe {
                let nc = objc2_foundation::NSNotificationCenter::defaultCenter();
                nc.addObserver_selector_name_object(
                    &*(self as *const AppDelegate as *const AnyObject),
                    objc2::sel!(pdfPageChanged:),
                    Some(&*objc2_foundation::NSString::from_str("PDFViewPageChangedNotification")),
                    None
                );
                nc.addObserver_selector_name_object(
                    &*(self as *const AppDelegate as *const AnyObject),
                    objc2::sel!(pdfScaleChanged:),
                    Some(&*objc2_foundation::NSString::from_str("PDFViewScaleChangedNotification")),
                    None
                );
                nc.addObserver_selector_name_object(
                    &*(self as *const AppDelegate as *const AnyObject),
                    objc2::sel!(pdfDocumentChanged:),
                    Some(&*objc2_foundation::NSString::from_str("PDFViewDocumentChangedNotification")),
                    None
                );
            }

            let split_vc_base = unsafe { Retained::cast_unchecked::<NSSplitViewController>(split_vc) };
            self.ivars().split_vc.set(split_vc_base).unwrap();

            let home_screen = crate::home_screen::HomeScreen::new(mtm);
            let home_vc = home_screen.view_controller();
            let _ = self.ivars().home_screen.set(home_screen);

            let has_doc = unsafe { self.ivars().pdf_view.get().unwrap().document().is_some() };
            if has_doc {
                self.ivars().content_mode.set(AppContentMode::StandaloneDocument);
                self.ivars().window.get().unwrap().setContentViewController(Some(&*self.ivars().split_vc.get().unwrap()));
                self.ivars().window.get().unwrap().setFrameAutosaveName(objc2_foundation::ns_string!("ReaderWindowGeometry"));
                self.ensure_reader_window_size();
                WindowGeometry::sanitize_window(self.ivars().window.get().unwrap(), WindowKind::Reader, mtm);
                if let Some(tb) = unsafe { self.ivars().window.get().unwrap().toolbar() } {
                    unsafe { tb.setVisible(true) };
                }
            } else {
                self.ivars().content_mode.set(AppContentMode::Home);
                self.ivars().window.get().unwrap().setContentViewController(Some(&home_vc));
                self.ivars().window.get().unwrap().setTitle(objc2_foundation::ns_string!("Iridecence PDF Reader"));
                self.ivars().window.get().unwrap().setFrameAutosaveName(objc2_foundation::ns_string!("HomeWindowGeometry"));
                WindowGeometry::sanitize_window(self.ivars().window.get().unwrap(), WindowKind::Home, mtm);
                if let Some(tb) = unsafe { self.ivars().window.get().unwrap().toolbar() } {
                    unsafe { tb.setVisible(false) };
                }
            }

            self.ivars().window.get().unwrap().makeKeyAndOrderFront(None);

            if let Some(url) = self.ivars().pending_url.borrow_mut().take() {
                let urls = NSArray::from_slice(&[&*url]);
                let app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
                let _: () = unsafe { msg_send![self, application: &*app, openURLs: &*urls] };
            }

            if std::env::var("IRIDESCENCE_VISUAL_QA").is_ok() {
                crate::visual_qa::VisualQA::run_qa_sequence(self, mtm);
            }
            if std::env::var("IRIDESCENCE_VISUAL_QA_ANNOTATIONS").is_ok() {
                crate::visual_qa::VisualQA::run_annotations_qa(self, mtm);
            }
            if std::env::var("IRIDESCENCE_VISUAL_QA_TABS").is_ok() {
                crate::visual_qa::VisualQA::run_tabs_qa(self, mtm);
            }
            if std::env::var("IRIDESCENCE_VISUAL_QA_OCR").is_ok() {
                crate::visual_qa::VisualQA::run_ocr_qa(self, mtm);
            }
            if std::env::var("IRIDESCENCE_VISUAL_QA_SETTINGS").is_ok() {
                crate::visual_qa::VisualQA::run_settings_qa(self, mtm);
            }
        }
    }
);

impl AppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppDelegateIvars::default());
        unsafe { objc2::msg_send_id![super(this), init] }
    }

    pub fn mark_document_edited(&self, edited: bool) {
        self.ivars().is_document_edited.set(edited);
        if let Some(win) = self.ivars().window.get() {
            unsafe { win.setDocumentEdited(edited) };
        }
    }

    pub fn refresh_tab_bar(&self) {
        if let Some(tab_bar) = self.ivars().tab_bar_view.get() {
            let mtm = MainThreadMarker::from(self);
            let any_self: &objc2::runtime::AnyObject =
                unsafe { &*(self as *const AppDelegate as *const objc2::runtime::AnyObject) };
            crate::tabs::update_tab_bar(tab_bar, &self.ivars().tab_manager.borrow(), any_self, mtm);
        }
    }

    pub fn mark_current_tab_edited(&self, edited: bool) {
        let active_idx = self.ivars().tab_manager.borrow().active_index;
        if let Some(tab) = self
            .ivars()
            .tab_manager
            .borrow_mut()
            .tabs
            .get_mut(active_idx)
        {
            tab.is_edited = edited;
        }
        self.mark_document_edited(edited);
        self.refresh_tab_bar();
    }

    pub fn select_next_tab(&self) {
        let count = self.ivars().tab_manager.borrow().tab_count();
        if count > 1 {
            let next = self.ivars().tab_manager.borrow().next_index();
            self.switch_to_tab_index(next);
        }
    }

    pub fn select_previous_tab(&self) {
        let count = self.ivars().tab_manager.borrow().tab_count();
        if count > 1 {
            let prev = self.ivars().tab_manager.borrow().prev_index();
            self.switch_to_tab_index(prev);
        }
    }

    pub fn switch_to_tab_index(&self, index: usize) {
        let count = self.ivars().tab_manager.borrow().tab_count();
        if index >= count {
            return;
        }

        let pdf_view = match self.ivars().pdf_view.get() {
            Some(v) => v,
            None => return,
        };

        // Save reading position of current active tab
        let cur_idx = self.ivars().tab_manager.borrow().active_index;
        if let Some(tab) = self.ivars().tab_manager.borrow_mut().tabs.get_mut(cur_idx) {
            if let Some(doc) = unsafe { pdf_view.document() } {
                if let Some(page) = unsafe { pdf_view.currentPage() } {
                    tab.page_index = unsafe { doc.indexForPage(&page) };
                    tab.scale_factor = unsafe { pdf_view.scaleFactor() };
                }
            }
        }

        self.ivars().tab_manager.borrow_mut().active_index = index;
        let target_tab = match self.ivars().tab_manager.borrow().tabs.get(index).cloned() {
            Some(t) => t,
            None => return,
        };

        *self.ivars().pending_pdf_path.borrow_mut() = target_tab.path.clone();
        self.ivars().content_mode.set(target_tab.mode);

        unsafe {
            pdf_view.setDocument(Some(&target_tab.document));
            if let Some(page) = target_tab.document.pageAtIndex(target_tab.page_index) {
                pdf_view.goToPage(&page);
            }
            if target_tab.scale_factor > 0.0 {
                pdf_view.setScaleFactor(target_tab.scale_factor);
            }
        }

        if let Some(window) = self.ivars().window.get() {
            window.setTitle(&NSString::from_str(&target_tab.title));
            unsafe {
                window.setTitleVisibility(NSWindowTitleVisibility::Hidden);
                window.setDocumentEdited(target_tab.is_edited);
            }
        }
        self.ivars().is_document_edited.set(target_tab.is_edited);

        self.refresh_outline_for_doc(&target_tab.document);
        self.refresh_bookmarks_view();

        if let Some(search_del) = self.ivars().pdf_search_delegate.get() {
            search_del.clear();
        }

        self.refresh_tab_bar();
    }

    pub fn close_tab_at_index(&self, index: usize) {
        let tab_opt = self.ivars().tab_manager.borrow().tabs.get(index).cloned();
        let tab = match tab_opt {
            Some(t) => t,
            None => return,
        };

        if tab.is_edited {
            let mtm = MainThreadMarker::from(self);
            let alert = objc2_app_kit::NSAlert::new(mtm);
            unsafe {
                alert.setMessageText(&NSString::from_str(&format!(
                    "Do you want to save the changes made to the document \"{}\"?",
                    tab.title
                )));
                alert.setInformativeText(&NSString::from_str(
                    "Your annotations will be lost if you don't save them.",
                ));
                alert.addButtonWithTitle(&NSString::from_str("Save"));
                alert.addButtonWithTitle(&NSString::from_str("Don't Save"));
                alert.addButtonWithTitle(&NSString::from_str("Cancel"));
            }
            let response = unsafe { alert.runModal() };
            if response == objc2_app_kit::NSAlertFirstButtonReturn {
                // Save
                if let Err(err) = crate::annotations::save_document_safely(&tab.document, &tab.path)
                {
                    let mtm = MainThreadMarker::from(self);
                    let err_alert = objc2_app_kit::NSAlert::new(mtm);
                    unsafe {
                        err_alert.setMessageText(&NSString::from_str("Error Saving Document"));
                        err_alert.setInformativeText(&NSString::from_str(&err));
                        err_alert.addButtonWithTitle(&NSString::from_str("OK"));
                        err_alert.runModal();
                    }
                    return;
                }
            } else if response == objc2_app_kit::NSAlertSecondButtonReturn {
                // Don't save
            } else {
                // Cancel
                return;
            }
        }

        let _removed = self.ivars().tab_manager.borrow_mut().remove_tab(index);
        let count = self.ivars().tab_manager.borrow().tab_count();
        if count == 0 {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                unsafe { pdf_view.setDocument(None) };
            }
            *self.ivars().pending_pdf_path.borrow_mut() = String::new();
            self.mark_document_edited(false);
            let _: () = unsafe { objc2::msg_send![self, showHomeScreen: None::<&AnyObject>] };
        } else {
            let new_idx = self.ivars().tab_manager.borrow().active_index;
            self.switch_to_tab_index(new_idx);
        }
    }

    pub fn switch_sidebar_to_index(&self, index: isize) {
        unsafe {
            let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
            defaults.setInteger_forKey(index, objc2_foundation::ns_string!("RightSidebarMode"));
            if let Some(segmented) = self.ivars().right_sidebar_segmented.get() {
                segmented.setSelected_forSegment(true, index);
            }
            if let Some(outline_scroll) = self.ivars().right_sidebar_outline_scroll.get() {
                if let Some(thumb_view) = self.ivars().thumb_view.get() {
                    let search_container = self.ivars().right_sidebar_search_container.get();
                    let bookmarks_scroll = self.ivars().right_sidebar_bookmarks_scroll.get();

                    thumb_view.setHidden(index != 0);
                    outline_scroll.setHidden(index != 1);
                    if let Some(sc) = search_container {
                        sc.setHidden(index != 2);
                    }
                    if let Some(bs) = bookmarks_scroll {
                        bs.setHidden(index != 3);
                    }
                }
                if index == 3 {
                    self.refresh_bookmarks_view();
                }
                let _: () = objc2::msg_send![self, updateOutlineEmptyState];
            }
        }
    }

    pub fn ensure_reader_window_size(&self) {
        if let Some(window) = self.ivars().window.get() {
            let mtm = MainThreadMarker::from(self);
            WindowGeometry::ensure_reader_window_size(window, mtm);
        }
    }

    pub fn refresh_outline_for_doc(&self, _doc: &PDFDocument) {
        if let Some(scroll) = self.ivars().right_sidebar_outline_scroll.get() {
            if let Some(view) = unsafe { scroll.documentView() } {
                if let Some(outline_view) = view.downcast_ref::<NSOutlineView>() {
                    unsafe { outline_view.reloadData() };
                }
            }
        }
        let _: () = unsafe { objc2::msg_send![self, updateOutlineEmptyState] };
    }

    pub fn get_current_outline_sections(&self) -> Vec<(String, usize)> {
        let mut sections = Vec::new();
        if let Some(pdf_view) = self.ivars().pdf_view.get() {
            if let Some(doc) = unsafe { pdf_view.document() } {
                if let Some(root) = unsafe { doc.outlineRoot() } {
                    fn traverse(
                        outline: &objc2_pdf_kit::PDFOutline,
                        doc: &PDFDocument,
                        out: &mut Vec<(String, usize)>,
                    ) {
                        let count = unsafe { outline.numberOfChildren() };
                        for i in 0..count {
                            if let Some(child) = unsafe { outline.childAtIndex(i) } {
                                let label = unsafe { child.label() }
                                    .map(|s| s.to_string())
                                    .unwrap_or_default();
                                let mut page_num = 0;
                                if let Some(dest) = unsafe { child.destination() } {
                                    if let Some(page) = unsafe { dest.page() } {
                                        page_num = unsafe { doc.indexForPage(&page) };
                                    }
                                }
                                if !label.is_empty() {
                                    out.push((label, page_num));
                                }
                                traverse(&child, doc, out);
                            }
                        }
                    }
                    traverse(&root, &doc, &mut sections);
                }
            }
        }
        sections
    }

    pub fn refresh_bookmarks_view(&self) {
        if let Some(bm_del) = self.ivars().bookmarks_delegate.get() {
            let path = self.ivars().pending_pdf_path.borrow().clone();
            bm_del.set_current_path(&path);
        }
    }

    pub fn restore_position_for_pdf(&self, doc: &PDFDocument, path: &str) {
        let pdf_view = match self.ivars().pdf_view.get() {
            Some(v) => v,
            None => return,
        };
        let page_count = unsafe { doc.pageCount() };
        unsafe {
            let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
            let key = objc2_foundation::NSString::from_str(&format!("PDFPos_{}", path));
            if let Some(dict) = defaults.dictionaryForKey(&key) {
                let page_idx_key = objc2_foundation::NSString::from_str("PageIndex");
                let zoom_key = objc2_foundation::NSString::from_str("Zoom");

                let mut page_idx = 0;
                let mut zoom = 0.0;

                if let Some(idx_obj) = dict.objectForKey(&page_idx_key) {
                    page_idx = objc2::msg_send![&*idx_obj, unsignedIntegerValue];
                }
                if let Some(zoom_obj) = dict.objectForKey(&zoom_key) {
                    let z: f64 = objc2::msg_send![&*zoom_obj, doubleValue];
                    zoom = z;
                }

                if page_idx < page_count {
                    if let Some(page) = doc.pageAtIndex(page_idx) {
                        pdf_view.goToPage(&page);
                    }
                }
                if zoom > 0.0 {
                    pdf_view.setScaleFactor(zoom);
                }
            }
        }
    }

    pub fn load_pdf_document(&self, doc: &PDFDocument, path: &str, target_mode: AppContentMode) {
        self.ivars().content_mode.set(target_mode);

        if let Some(window) = self.ivars().window.get() {
            let mtm = MainThreadMarker::from(self);
            window.setFrameAutosaveName(objc2_foundation::ns_string!("ReaderWindowGeometry"));

            let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(path)) };
            if let Some(last_pc) = unsafe { nsurl.lastPathComponent() } {
                window.setTitle(&last_pc);
                unsafe { window.setTitleVisibility(NSWindowTitleVisibility::Hidden) };
            }

            if let Some(split_vc) = self.ivars().split_vc.get() {
                unsafe {
                    let split_view = split_vc.splitView();
                    split_view.setDividerStyle(objc2_app_kit::NSSplitViewDividerStyle::Thin);
                    window.setContentViewController(Some(&*split_vc));
                }
            }

            self.ensure_reader_window_size();
            WindowGeometry::sanitize_window(window, WindowKind::Reader, mtm);

            if let Some(tb) = unsafe { window.toolbar() } {
                unsafe {
                    tb.setVisible(true);
                    tb.validateVisibleItems();
                };
            }
        }

        if let Some(explorer_item) = self.ivars().explorer_item.get() {
            unsafe {
                if target_mode == AppContentMode::StandaloneDocument {
                    explorer_item.setCollapsed(true);
                } else if target_mode == AppContentMode::Workspace {
                    explorer_item.setCollapsed(false);
                }
            }
        }

        if let Some(pdf_view) = self.ivars().pdf_view.get() {
            unsafe { pdf_view.setDocument(Some(doc)) };
        }

        self.refresh_outline_for_doc(doc);

        if let Some(search_del) = self.ivars().pdf_search_delegate.get() {
            search_del.clear();
        }

        self.restore_position_for_pdf(doc, path);
        self.refresh_bookmarks_view();

        if target_mode == AppContentMode::Workspace {
            if let Some(del) = self.ivars().workspace_delegate.get() {
                if let Some(root) = del.ivars().root_node.borrow().as_ref() {
                    let root_path = root.ivars().path.borrow().clone();
                    crate::workspace_index::start_indexing_workspace(
                        root_path,
                        Some(path.to_string()),
                    );
                }
            }
        }

        if !self.ivars().ignore_history_update.get() {
            let mut hist = self.ivars().history.borrow_mut();
            let idx = self.ivars().history_index.get();
            if idx < hist.len() {
                hist.truncate(idx);
            }
            hist.push((path.to_string(), 0));
            self.ivars().history_index.set(hist.len());
        }

        if let Ok(meta) = std::fs::metadata(path) {
            if let Ok(mtime) = meta.modified() {
                self.ivars().last_file_mtime.set(Some(mtime));
            }
        }

        crate::add_recent_item(path);

        // Synchronize with tab manager
        let has_tab = self.ivars().tab_manager.borrow().find_tab_by_path(path);
        if has_tab.is_none() {
            let filename = std::path::Path::new(path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("Document.pdf")
                .to_string();
            let idx = self.ivars().tab_manager.borrow_mut().add_tab(
                path.to_string(),
                filename,
                doc.retain(),
                target_mode,
            );
            self.ivars().tab_manager.borrow_mut().active_index = idx;
        } else if let Some(idx) = has_tab {
            self.ivars().tab_manager.borrow_mut().active_index = idx;
        }

        self.refresh_tab_bar();
    }

    pub fn check_external_file_change(&self) {
        if self.ivars().content_mode.get() == AppContentMode::Home {
            return;
        }
        let path = self.ivars().pending_pdf_path.borrow().clone();
        if path.is_empty() {
            return;
        }
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                let last = self.ivars().last_file_mtime.get();
                if let Some(last_time) = last {
                    if mtime > last_time {
                        self.ivars().last_file_mtime.set(Some(mtime));
                        let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&path)) };
                        if let Some(new_doc) =
                            unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &nsurl) }
                        {
                            let cur_page_idx = if let Some(pv) = self.ivars().pdf_view.get() {
                                if let Some(d) = unsafe { pv.document() } {
                                    if let Some(p) = unsafe { pv.currentPage() } {
                                        unsafe { d.indexForPage(&p) }
                                    } else {
                                        0
                                    }
                                } else {
                                    0
                                }
                            } else {
                                0
                            };

                            let mode = self.ivars().content_mode.get();
                            self.ivars().ignore_history_update.set(true);
                            self.load_pdf_document(&new_doc, &path, mode);
                            if let Some(pv) = self.ivars().pdf_view.get() {
                                if let Some(page) = unsafe { new_doc.pageAtIndex(cur_page_idx) } {
                                    unsafe { pv.goToPage(&page) };
                                }
                            }
                            self.ivars().ignore_history_update.set(false);
                        }
                    }
                } else {
                    self.ivars().last_file_mtime.set(Some(mtime));
                }
            }
        }
    }

    pub fn open_file_url(&self, url: &NSURL) {
        if let Some(path) = unsafe { url.path() } {
            let path_str = path.to_string();
            if let Some(existing_idx) = self
                .ivars()
                .tab_manager
                .borrow()
                .find_tab_by_path(&path_str)
            {
                self.switch_to_tab_index(existing_idx);
                return;
            }
            let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), url) };
            if let Some(doc) = doc {
                *self.ivars().pending_pdf_path.borrow_mut() = path_str.clone();
                self.load_pdf_document(&doc, &path_str, AppContentMode::StandaloneDocument);
            }
        }
    }

    pub fn open_folder_url(&self, url: &NSURL) {
        let path = match unsafe { url.path() } {
            Some(p) => p.to_string(),
            None => return,
        };
        let name = url
            .lastPathComponent()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Workspace".to_string());

        self.ivars().content_mode.set(AppContentMode::Workspace);

        if let Some(window) = self.ivars().window.get() {
            let mtm = MainThreadMarker::from(self);
            window.setFrameAutosaveName(objc2_foundation::ns_string!("ReaderWindowGeometry"));
            window.setTitle(&NSString::from_str(&name));

            if let Some(split_vc) = self.ivars().split_vc.get() {
                unsafe {
                    let split_view = split_vc.splitView();
                    split_view.setDividerStyle(objc2_app_kit::NSSplitViewDividerStyle::Thin);
                    window.setContentViewController(Some(&*split_vc));
                }
            }

            self.ensure_reader_window_size();
            WindowGeometry::sanitize_window(window, WindowKind::Reader, mtm);

            if let Some(tb) = unsafe { window.toolbar() } {
                unsafe {
                    tb.setVisible(true);
                    tb.validateVisibleItems();
                };
            }
        }

        if let Some(explorer_item) = self.ivars().explorer_item.get() {
            unsafe { explorer_item.setCollapsed(false) };
        }

        let root = crate::workspace::FileNode::new(path.clone(), name, true);
        if let Some(del) = self.ivars().workspace_delegate.get() {
            del.ivars().root_node.borrow_mut().replace(root);
            if let Some(scroll) = self.ivars().outline_scroll.get() {
                if let Some(view) = unsafe { scroll.documentView() } {
                    if let Some(outline) = view.downcast_ref::<NSOutlineView>() {
                        unsafe {
                            outline.reloadData();
                            outline.expandItem(None);
                        }
                    }
                }
            }
        }

        crate::add_recent_item(&path);
        crate::add_recent_workspace(&path);
        crate::workspace_index::start_indexing_workspace(path.clone(), None);
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

pub fn download_file(url_str: String, _outline_raw: usize, app_ptr: usize) {
    let result = std::panic::catch_unwind(move || unsafe {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;

        use objc2_foundation::{NSFileManager, NSString, NSURL};

        crate::log_icloud!("[DIAGNOSTIC] DOWNLOAD THREAD ENTER: path = {}", url_str);
        crate::log_icloud!("[DIAGNOSTIC] START DOWNLOAD API BEFORE");

        let nsurl = NSURL::fileURLWithPath(&NSString::from_str(&url_str));
        let fm = NSFileManager::defaultManager();
        let mut err: *mut AnyObject = std::ptr::null_mut();
        let ok: bool =
            msg_send![&fm, startDownloadingUbiquitousItemAtURL: &*nsurl, error: &mut err];

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
            let ok1 = nsurl
                .getResourceValue_forKey_error(&mut is_dl_val, &key)
                .is_ok();

            let mut still_dl = false;
            if ok1 {
                if let Some(val) = is_dl_val {
                    still_dl = msg_send![&val, boolValue];
                }
            }

            let mut stat_val: Option<objc2::rc::Retained<AnyObject>> = None;
            let key_stat = NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");
            let ok_stat = nsurl
                .getResourceValue_forKey_error(&mut stat_val, &key_stat)
                .is_ok();

            let mut is_current = false;
            let mut is_downloaded = false;
            let mut stat_string = "Unknown".to_string();
            if ok_stat {
                if let Some(stat_obj) = stat_val {
                    let utf8: *const std::ffi::c_char = msg_send![&stat_obj, UTF8String];
                    if !utf8.is_null() {
                        stat_string = std::ffi::CStr::from_ptr(utf8)
                            .to_string_lossy()
                            .into_owned();
                    }
                    let dl_id =
                        NSString::from_str("NSURLUbiquitousItemDownloadingStatusDownloaded");
                    let current_id =
                        NSString::from_str("NSURLUbiquitousItemDownloadingStatusCurrent");
                    is_downloaded = msg_send![&stat_obj, isEqualToString: &*dl_id];
                    is_current = msg_send![&stat_obj, isEqualToString: &*current_id];
                }
            }

            let mut prog_val: Option<objc2::rc::Retained<AnyObject>> = None;
            let key_prog = NSString::from_str("NSURLUbiquitousItemPercentDownloadedKey");
            let ok_prog = nsurl
                .getResourceValue_forKey_error(&mut prog_val, &key_prog)
                .is_ok();
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
            crate::log_icloud!(
                "[DIAGNOSTIC] POLL ITERATION #{} RESULT: status={}, isDownloading={}, isDownloaded={}, percent={}, exists={}, readable={}, size={}",
                iterations,
                stat_string,
                still_dl,
                is_downloaded,
                percent,
                exists,
                is_readable,
                size
            );

            if is_downloaded
                || is_current
                || (exists && is_readable && size > 0 && percent == 100.0)
                || (exists
                    && is_readable
                    && size > 0
                    && stat_string == "NSURLUbiquitousItemDownloadingStatusNotDownloaded")
            {
                crate::log_pdf!(
                    "[DIAGNOSTIC] DOWNLOAD COMPLETE DETECTED: path = {}",
                    url_str
                );
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
        use objc2_foundation::{NSFileManager, NSString, NSURL};

        let nsurl = NSURL::fileURLWithPath(&NSString::from_str(url_str));
        let fm = NSFileManager::defaultManager();

        let mut stat_val: Option<objc2::rc::Retained<AnyObject>> = None;
        let key_stat = NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");
        let ok_stat = nsurl
            .getResourceValue_forKey_error(&mut stat_val, &key_stat)
            .is_ok();

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
        use objc2_app_kit::NSOutlineView;
        use objc2_foundation::{NSString, NSURL};

        let nsurl = NSURL::fileURLWithPath(&NSString::from_str(&url_str));
        let node = &*(node_ptr as *const crate::workspace::FileNode);

        let mut new_status = node.ivars().icloud_status.get();
        let mut new_progress = node.ivars().download_progress.get();
        let old_status = new_status;
        let old_progress = new_progress;

        let mut is_dl_val: Option<objc2::rc::Retained<AnyObject>> = None;
        let key = NSString::from_str("NSURLUbiquitousItemIsDownloadingKey");
        let ok = nsurl
            .getResourceValue_forKey_error(&mut is_dl_val, &key)
            .is_ok();

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
                let not_dl_id =
                    NSString::from_str("NSURLUbiquitousItemDownloadingStatusNotDownloaded");

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
        let retained_item: objc2::rc::Retained<AnyObject> =
            objc2::rc::Retained::from_raw(i_ptr).unwrap();

        if new_status != old_status
            || (new_status == 2 && (new_progress - old_progress).abs() > 0.05)
        {
            node.ivars().icloud_status.set(new_status);
            node.ivars().download_progress.set(new_progress);

            let o_ptr = outline_raw as *mut NSOutlineView;
            let i_ptr2 = &*retained_item as *const AnyObject as *mut AnyObject;
            let _: () = msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr2, waitUntilDone: false];
        }
    }
}

pub fn add_recent_item(path: &str) {
    unsafe {
        let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
        let key1 = objc2_foundation::NSString::from_str("RecentFiles");
        let key2 = objc2_foundation::NSString::from_str("RecentItems");
        let mut items = std::vec::Vec::new();
        if let Some(arr) = defaults
            .arrayForKey(&key1)
            .or_else(|| defaults.arrayForKey(&key2))
        {
            for i in 0..arr.count() {
                if let Some(obj) = arr
                    .objectAtIndex(i)
                    .downcast_ref::<objc2_foundation::NSString>()
                {
                    items.push(obj.to_string());
                }
            }
        }
        items.retain(|x| x != path);
        items.insert(0, path.to_string());
        if items.len() > 20 {
            items.truncate(20);
        }

        let mut ns_strs = std::vec::Vec::new();
        for item in &items {
            ns_strs.push(objc2_foundation::NSString::from_str(item));
        }
        let mut refs = std::vec::Vec::new();
        for ns in &ns_strs {
            refs.push(&**ns);
        }
        let ns_items = objc2_foundation::NSArray::from_slice(&refs);
        defaults.setObject_forKey(Some(&ns_items), &key1);
        defaults.setObject_forKey(Some(&ns_items), &key2);
    }
}

pub fn add_recent_workspace(path: &str) {
    unsafe {
        let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
        let key = objc2_foundation::NSString::from_str("RecentWorkspaces");
        let mut items = std::vec::Vec::new();
        if let Some(arr) = defaults.arrayForKey(&key) {
            for i in 0..arr.count() {
                if let Some(obj) = arr
                    .objectAtIndex(i)
                    .downcast_ref::<objc2_foundation::NSString>()
                {
                    items.push(obj.to_string());
                }
            }
        }
        items.retain(|x| x != path);
        items.insert(0, path.to_string());
        if items.len() > 10 {
            items.truncate(10);
        }

        let mut ns_strs = std::vec::Vec::new();
        for item in &items {
            ns_strs.push(objc2_foundation::NSString::from_str(item));
        }
        let mut refs = std::vec::Vec::new();
        for ns in &ns_strs {
            refs.push(&**ns);
        }
        let ns_items = objc2_foundation::NSArray::from_slice(&refs);
        defaults.setObject_forKey(Some(&ns_items), &key);
    }
}
