use crate::window_geometry::{WindowGeometry, WindowKind};
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, msg_send_id, rc::Retained};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSColor, NSControl, NSControlTextEditingDelegate, NSFont,
    NSOutlineView, NSOutlineViewDataSource, NSOutlineViewDelegate, NSPanel, NSScrollView,
    NSTableCellView, NSTableColumn, NSTextField, NSTextFieldDelegate, NSTextView,
    NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
    NSWindowCollectionBehavior, NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{
    MainThreadMarker, NSInteger, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect,
    NSSize, NSString, NSUserDefaults, ns_string,
};
use std::cell::RefCell;
use std::path::Path;

#[derive(Clone, Debug)]
pub enum PaletteItemKind {
    Command { selector: &'static str },
    File { path: String },
    WorkspaceMatch { path: String, page_num: usize },
    PageJump { page_num: usize },
    OutlineJump { page_num: usize },
    BookmarkJump { path: String, page_num: usize },
}

#[derive(Clone, Debug)]
pub struct PaletteItemData {
    pub title: String,
    pub subtitle: Option<String>,
    pub badge: Option<String>,
    pub kind: PaletteItemKind,
}

#[derive(Default)]
pub struct PaletteIvars {
    pub panel: std::cell::OnceCell<Retained<NSPanel>>,
    pub search_field: std::cell::OnceCell<Retained<NSTextField>>,
    pub table_view: std::cell::OnceCell<Retained<NSOutlineView>>,
    pub items: RefCell<Vec<PaletteItemData>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "CommandPaletteDelegate"]
    #[ivars = PaletteIvars]
    pub struct CommandPaletteDelegate;

    unsafe impl NSObjectProtocol for CommandPaletteDelegate {}
    unsafe impl NSTextFieldDelegate for CommandPaletteDelegate {}

    unsafe impl NSWindowDelegate for CommandPaletteDelegate {
        #[unsafe(method(windowDidResignKey:))]
        fn windowDidResignKey(&self, _notification: &NSNotification) {
            if let Some(panel) = self.ivars().panel.get() {
                unsafe { panel.orderOut(None) };
                let app =
                    unsafe { NSApplication::sharedApplication(MainThreadMarker::new().unwrap()) };
                if let Some(main_win) = unsafe { app.mainWindow() } {
                    unsafe { main_win.makeKeyWindow() };
                }
            }
        }
    }

    unsafe impl NSOutlineViewDataSource for CommandPaletteDelegate {
        #[unsafe(method(outlineView:numberOfChildrenOfItem:))]
        fn outlineView_numberOfChildrenOfItem(
            &self,
            _outline_view: &NSOutlineView,
            item: Option<&objc2::runtime::AnyObject>,
        ) -> NSInteger {
            if item.is_some() {
                return 0;
            }
            self.ivars().items.borrow().len() as NSInteger
        }

        #[unsafe(method(outlineView:child:ofItem:))]
        fn outlineView_child_ofItem(
            &self,
            _outline_view: &NSOutlineView,
            index: NSInteger,
            item: Option<&objc2::runtime::AnyObject>,
        ) -> *mut objc2::runtime::AnyObject {
            if item.is_none() {
                let str_idx = format!("{}", index);
                return Retained::into_raw(NSString::from_str(&str_idx))
                    as *mut objc2::runtime::AnyObject;
            }
            std::ptr::null_mut()
        }

        #[unsafe(method(outlineView:isItemExpandable:))]
        fn outlineView_isItemExpandable(
            &self,
            _outline_view: &NSOutlineView,
            _item: &objc2::runtime::AnyObject,
        ) -> bool {
            false
        }
    }

    unsafe impl NSOutlineViewDelegate for CommandPaletteDelegate {
        #[unsafe(method(outlineView:viewForTableColumn:item:))]
        fn outlineView_viewForTableColumn_item(
            &self,
            outline_view: &NSOutlineView,
            _table_column: Option<&NSTableColumn>,
            item: &objc2::runtime::AnyObject,
        ) -> *mut objc2_app_kit::NSView {
            let mtm = MainThreadMarker::new().unwrap();
            let identifier = ns_string!("PaletteRichCell");

            let str_val = unsafe {
                Retained::cast_unchecked::<NSString>(
                    Retained::retain(
                        item as *const objc2::runtime::AnyObject as *mut objc2::runtime::AnyObject,
                    )
                    .unwrap(),
                )
            };
            let row_idx: usize = str_val.to_string().parse().unwrap_or(0);

            let items = self.ivars().items.borrow();
            let item_data = items.get(row_idx).cloned();
            drop(items);

            let mut view: Option<Retained<NSTableCellView>> = unsafe {
                let v = outline_view.makeViewWithIdentifier_owner(identifier, None);
                v.map(|v| Retained::cast_unchecked::<NSTableCellView>(v))
            };

            if view.is_none() {
                let cell = unsafe {
                    NSTableCellView::initWithFrame(
                        NSTableCellView::alloc(mtm),
                        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(568.0, 44.0)),
                    )
                };

                let title_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                let subtitle_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                let badge_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };

                unsafe {
                    title_field.setTag(301);
                    title_field.setFont(Some(&NSFont::boldSystemFontOfSize(13.0)));
                    title_field.setTextColor(Some(&NSColor::labelColor()));
                    title_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    title_field.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingTail);

                    subtitle_field.setTag(302);
                    subtitle_field.setFont(Some(&NSFont::systemFontOfSize(11.0)));
                    subtitle_field.setTextColor(Some(&NSColor::secondaryLabelColor()));
                    subtitle_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    subtitle_field
                        .setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingTail);

                    badge_field.setTag(303);
                    badge_field.setFont(Some(&NSFont::systemFontOfSize(11.0)));
                    badge_field.setTextColor(Some(&NSColor::tertiaryLabelColor()));
                    badge_field.setTranslatesAutoresizingMaskIntoConstraints(false);

                    cell.setTextField(Some(&title_field));
                    cell.addSubview(&title_field);
                    cell.addSubview(&subtitle_field);
                    cell.addSubview(&badge_field);

                    badge_field
                        .trailingAnchor()
                        .constraintEqualToAnchor_constant(&cell.trailingAnchor(), -18.0)
                        .setActive(true);
                    badge_field
                        .centerYAnchor()
                        .constraintEqualToAnchor(&cell.centerYAnchor())
                        .setActive(true);

                    title_field
                        .leadingAnchor()
                        .constraintEqualToAnchor_constant(&cell.leadingAnchor(), 12.0)
                        .setActive(true);
                    title_field
                        .trailingAnchor()
                        .constraintLessThanOrEqualToAnchor_constant(
                            &badge_field.leadingAnchor(),
                            -8.0,
                        )
                        .setActive(true);
                    title_field
                        .topAnchor()
                        .constraintEqualToAnchor_constant(&cell.topAnchor(), 5.0)
                        .setActive(true);

                    subtitle_field
                        .leadingAnchor()
                        .constraintEqualToAnchor_constant(&cell.leadingAnchor(), 12.0)
                        .setActive(true);
                    subtitle_field
                        .trailingAnchor()
                        .constraintLessThanOrEqualToAnchor_constant(
                            &badge_field.leadingAnchor(),
                            -8.0,
                        )
                        .setActive(true);
                    subtitle_field
                        .topAnchor()
                        .constraintEqualToAnchor_constant(&title_field.bottomAnchor(), 3.0)
                        .setActive(true);
                }
                view = Some(cell);
            }

            if let Some(cell) = &view {
                let is_selected = outline_view.selectedRow() == (row_idx as isize);
                if let Some(data) = item_data {
                    if let Some(tf) = unsafe { cell.viewWithTag(301) } {
                        if let Some(tf) = tf.downcast_ref::<NSTextField>() {
                            unsafe {
                                tf.setStringValue(&NSString::from_str(&data.title));
                                if is_selected {
                                    tf.setTextColor(Some(&NSColor::whiteColor()));
                                } else {
                                    tf.setTextColor(Some(&NSColor::labelColor()));
                                }
                            };
                        }
                    }
                    if let Some(sub) = unsafe { cell.viewWithTag(302) } {
                        if let Some(sub) = sub.downcast_ref::<NSTextField>() {
                            if let Some(ref text) = data.subtitle {
                                unsafe {
                                    sub.setStringValue(&NSString::from_str(text));
                                    if is_selected {
                                        sub.setTextColor(Some(&NSColor::colorWithWhite_alpha(
                                            0.9, 1.0,
                                        )));
                                    } else {
                                        sub.setTextColor(Some(&NSColor::secondaryLabelColor()));
                                    }
                                    sub.setHidden(false);
                                };
                            } else {
                                unsafe { sub.setHidden(true) };
                            }
                        }
                    }
                    if let Some(badge) = unsafe { cell.viewWithTag(303) } {
                        if let Some(badge) = badge.downcast_ref::<NSTextField>() {
                            if let Some(ref b_text) = data.badge {
                                unsafe {
                                    badge.setStringValue(&NSString::from_str(b_text));
                                    if is_selected {
                                        badge.setTextColor(Some(&NSColor::colorWithWhite_alpha(
                                            0.85, 1.0,
                                        )));
                                    } else {
                                        badge.setTextColor(Some(&NSColor::tertiaryLabelColor()));
                                    }
                                    badge.setHidden(false);
                                };
                            } else {
                                unsafe { badge.setHidden(true) };
                            }
                        }
                    }
                }
            }

            match view {
                Some(v) => Retained::into_raw(v) as *mut objc2_app_kit::NSView,
                None => std::ptr::null_mut(),
            }
        }
    }

    unsafe impl NSControlTextEditingDelegate for CommandPaletteDelegate {
        #[unsafe(method(controlTextDidChange:))]
        fn controlTextDidChange(&self, _notification: &NSNotification) {
            if let Some(tf) = self.ivars().search_field.get() {
                let s = unsafe { tf.stringValue().to_string() };
                self.update_search(&s);
            }
        }

        #[unsafe(method(control:textView:doCommandBySelector:))]
        fn control_textView_doCommandBySelector(
            &self,
            _control: &NSControl,
            _text_view: &NSTextView,
            command_selector: objc2::runtime::Sel,
        ) -> bool {
            let sel_name = command_selector.name();
            if sel_name.to_bytes() == b"moveDown:" {
                if let Some(tv) = self.ivars().table_view.get() {
                    let current_row = unsafe { tv.selectedRow() };
                    let next_row = if current_row < 0 { 0 } else { current_row + 1 };
                    let max_row = unsafe { tv.numberOfRows() } - 1;
                    if next_row <= max_row {
                        let index_set = unsafe {
                            objc2_foundation::NSIndexSet::indexSetWithIndex(next_row as usize)
                        };
                        unsafe { tv.selectRowIndexes_byExtendingSelection(&index_set, false) };
                        unsafe { tv.scrollRowToVisible(next_row) };
                    }
                }
                return true.into();
            } else if sel_name.to_bytes() == b"moveUp:" {
                if let Some(tv) = self.ivars().table_view.get() {
                    let current_row = unsafe { tv.selectedRow() };
                    if current_row > 0 {
                        let next_row = current_row - 1;
                        let index_set = unsafe {
                            objc2_foundation::NSIndexSet::indexSetWithIndex(next_row as usize)
                        };
                        unsafe { tv.selectRowIndexes_byExtendingSelection(&index_set, false) };
                        unsafe { tv.scrollRowToVisible(next_row) };
                    }
                }
                return true.into();
            } else if sel_name.to_bytes() == b"insertNewline:" {
                self.activate_selected_item();
                if let Some(panel) = self.ivars().panel.get() {
                    unsafe { panel.orderOut(None) };
                    let app = unsafe {
                        NSApplication::sharedApplication(MainThreadMarker::new().unwrap())
                    };
                    if let Some(main_win) = unsafe { app.mainWindow() } {
                        unsafe { main_win.makeKeyWindow() };
                    }
                }
                return true.into();
            } else if sel_name.to_bytes() == b"cancelOperation:" {
                if let Some(panel) = self.ivars().panel.get() {
                    unsafe { panel.orderOut(None) };
                    let app = unsafe {
                        NSApplication::sharedApplication(MainThreadMarker::new().unwrap())
                    };
                    if let Some(main_win) = unsafe { app.mainWindow() } {
                        unsafe { main_win.makeKeyWindow() };
                    }
                }
                return true.into();
            }
            false
        }
    }
);

impl CommandPaletteDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(PaletteIvars::default());
        let this: Retained<Self> = unsafe { msg_send_id![super(this), init] };
        this
    }

    pub fn setup(
        &self,
        panel: Retained<NSPanel>,
        text_field: Retained<NSTextField>,
        table_view: Retained<NSOutlineView>,
    ) {
        self.ivars().panel.set(panel).unwrap();
        self.ivars().search_field.set(text_field).unwrap();
        self.ivars().table_view.set(table_view).unwrap();
    }

    pub fn activate_selected_item(&self) {
        if let Some(tv) = self.ivars().table_view.get() {
            let row = unsafe { tv.selectedRow() };
            let items = self.ivars().items.borrow();
            let selected = if row >= 0 && (row as usize) < items.len() {
                Some(items[row as usize].kind.clone())
            } else if !items.is_empty() {
                Some(items[0].kind.clone())
            } else {
                None
            };
            drop(items);

            if let Some(kind) = selected {
                self.execute_item(kind);
            }
        }
    }

    pub fn execute_item(&self, kind: PaletteItemKind) {
        let mtm = MainThreadMarker::new().unwrap();
        let app = unsafe { NSApplication::sharedApplication(mtm) };
        if let Some(delegate) = unsafe { app.delegate() } {
            match kind {
                PaletteItemKind::Command { selector } => {
                    let ns_sel = NSString::from_str(selector);
                    let _: () = unsafe { msg_send![&*delegate, executePaletteCommand: &*ns_sel] };
                }
                PaletteItemKind::File { path } => {
                    let ns_path = NSString::from_str(&path);
                    let _: () = unsafe { msg_send![&*delegate, openPathString: &*ns_path] };
                }
                PaletteItemKind::WorkspaceMatch { path, page_num } => {
                    let ns_path = NSString::from_str(&path);
                    let _: () = unsafe {
                        msg_send![&*delegate, openPath: &*ns_path, atPage: page_num as NSInteger]
                    };
                }
                PaletteItemKind::PageJump { page_num } => {
                    let _: () = unsafe { msg_send![&*delegate, jumpToPage: page_num as NSInteger] };
                }
                PaletteItemKind::OutlineJump { page_num } => {
                    let _: () = unsafe { msg_send![&*delegate, jumpToPage: page_num as NSInteger] };
                }
                PaletteItemKind::BookmarkJump { path, page_num } => {
                    let ns_path = NSString::from_str(&path);
                    let _: () = unsafe {
                        msg_send![&*delegate, openPath: &*ns_path, atPage: page_num as NSInteger]
                    };
                }
            }
        }
    }

    pub fn update_search(&self, query: &str) {
        let mut items = Vec::new();

        if query.starts_with("//") {
            // Workspace-wide PDF content search
            let term = query[2..].trim();
            if term.is_empty() {
                items.push(PaletteItemData {
                    title: "Type to search workspace PDF text...".to_string(),
                    subtitle: Some(
                        "Fast incremental full-text search across current workspace".to_string(),
                    ),
                    badge: Some("Workspace Search".to_string()),
                    kind: PaletteItemKind::Command { selector: "noop" },
                });
            } else {
                let results = crate::workspace_index::search_workspace(term);
                if results.is_empty() {
                    items.push(PaletteItemData {
                        title: format!("No matches for '{}'", term),
                        subtitle: Some(
                            "Check spelling or verify workspace PDFs are indexed".to_string(),
                        ),
                        badge: Some("0 results".to_string()),
                        kind: PaletteItemKind::Command { selector: "noop" },
                    });
                } else {
                    for r in results {
                        items.push(PaletteItemData {
                            title: format!("{} — Page {}", r.filename, r.page_num),
                            subtitle: Some(r.snippet),
                            badge: Some("Match".to_string()),
                            kind: PaletteItemKind::WorkspaceMatch {
                                path: r.path,
                                page_num: r.page_num.saturating_sub(1),
                            },
                        });
                    }
                }
            }
        } else if query.starts_with('@') {
            // Current PDF Outline section search
            let term = query[1..].trim().to_lowercase();
            let mtm = MainThreadMarker::new().unwrap();
            let app = unsafe { NSApplication::sharedApplication(mtm) };
            if let Some(delegate) = unsafe { app.delegate() } {
                let sections: Vec<(String, usize)> = unsafe {
                    let ptr = Retained::as_ptr(&delegate) as *const crate::AppDelegate;
                    let del_ref = &*ptr;
                    del_ref.get_current_outline_sections()
                };

                for (sec_title, page_idx) in sections {
                    if term.is_empty() || sec_title.to_lowercase().contains(&term) {
                        items.push(PaletteItemData {
                            title: sec_title,
                            subtitle: Some(format!("Outline section at page {}", page_idx + 1)),
                            badge: Some("Outline".to_string()),
                            kind: PaletteItemKind::OutlineJump { page_num: page_idx },
                        });
                    }
                }
                if items.is_empty() {
                    items.push(PaletteItemData {
                        title: "No matching outline sections".to_string(),
                        subtitle: None,
                        badge: Some("Outline".to_string()),
                        kind: PaletteItemKind::Command { selector: "noop" },
                    });
                }
            }
        } else if query.starts_with('#') {
            // Bookmarks search
            let term = query[1..].trim().to_lowercase();
            let bookmarks = crate::bookmarks::load_bookmarks();
            for bm in bookmarks {
                let fname = Path::new(&bm.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if term.is_empty()
                    || bm.label.to_lowercase().contains(&term)
                    || fname.to_lowercase().contains(&term)
                {
                    items.push(PaletteItemData {
                        title: bm.label,
                        subtitle: Some(format!("{} — Page {}", fname, bm.page_idx + 1)),
                        badge: Some("Bookmark".to_string()),
                        kind: PaletteItemKind::BookmarkJump {
                            path: bm.path,
                            page_num: bm.page_idx,
                        },
                    });
                }
            }
            if items.is_empty() {
                items.push(PaletteItemData {
                    title: "No matching bookmarks".to_string(),
                    subtitle: Some("Press Cmd+D to bookmark current page".to_string()),
                    badge: Some("Bookmarks".to_string()),
                    kind: PaletteItemKind::Command { selector: "noop" },
                });
            }
        } else if query.starts_with('>') {
            // Commands
            let term = query[1..].trim().to_lowercase();
            let all_commands = vec![
                (
                    "New Tab (Cmd+T)",
                    "Open a PDF document in a new tab",
                    "new_tab",
                ),
                (
                    "Close Tab (Cmd+W)",
                    "Close the active document tab",
                    "close_tab",
                ),
                (
                    "Show Next Tab (Ctrl+Tab)",
                    "Switch to the next document tab",
                    "next_tab",
                ),
                (
                    "Show Previous Tab (Ctrl+Shift+Tab)",
                    "Switch to the previous document tab",
                    "prev_tab",
                ),
                (
                    "Save Document (Cmd+S)",
                    "Atomically save changes to current PDF",
                    "save_document",
                ),
                (
                    "Save Document As... (Cmd+Shift+S)",
                    "Save document to a new location",
                    "save_document_as",
                ),
                (
                    "Highlight Selection (Cmd+Shift+H)",
                    "Add yellow highlight annotation to selected text",
                    "highlight_selection",
                ),
                (
                    "Underline Selection (Cmd+Shift+U)",
                    "Add blue underline annotation to selected text",
                    "underline_selection",
                ),
                (
                    "Strikeout Selection (Cmd+Shift+X)",
                    "Add red strikeout annotation to selected text",
                    "strikeout_selection",
                ),
                (
                    "Add Note to Selection (Cmd+Shift+N)",
                    "Attach a text note / comment annotation",
                    "add_note",
                ),
                (
                    "Fit to Width",
                    "Zoom document to fill view width",
                    "fit_width",
                ),
                (
                    "Fit to Page",
                    "Zoom document to fit entire page",
                    "fit_page",
                ),
                (
                    "Actual Size (100%)",
                    "View document at original scale",
                    "actual_size",
                ),
                (
                    "Copy Page Reference",
                    "Copy citation e.g. document.pdf, p. 54",
                    "copy_page_ref",
                ),
                (
                    "Copy with Page Reference",
                    "Copy selected text with citation",
                    "copy_with_page_ref",
                ),
                (
                    "Copy as Markdown Quote",
                    "Copy selection as blockquote citation",
                    "copy_markdown_quote",
                ),
                (
                    "Search Selection in Document",
                    "Find selected text in current PDF",
                    "search_selection",
                ),
                (
                    "Enter Presentation Mode",
                    "Fullscreen lecture presentation with page controls",
                    "toggle_presentation",
                ),
                (
                    "Focus Reading Mode",
                    "Windowed reading without sidebars or toolbars",
                    "toggle_focus",
                ),
                (
                    "Toggle Bookmark (Cmd+D)",
                    "Bookmark or remove bookmark on current page",
                    "toggle_bookmark",
                ),
                (
                    "Get Document Info (Cmd+I)",
                    "View metadata, pages, author, security",
                    "get_info",
                ),
                (
                    "Export Bookmarks as Markdown",
                    "Save notes and bookmarks to Markdown file",
                    "export_notes",
                ),
                (
                    "Display: Single Page",
                    "View one page at a time",
                    "mode_single",
                ),
                (
                    "Display: Single Page Continuous",
                    "Continuous vertical page scroll",
                    "mode_single_cont",
                ),
                (
                    "Display: Two-Up",
                    "View two pages side by side",
                    "mode_two_up",
                ),
                (
                    "Display: Two-Up Continuous",
                    "Continuous two-page spread scroll",
                    "mode_two_up_cont",
                ),
                (
                    "OCR Current Page (Apple Vision)",
                    "Recognize text on current page using Apple Vision OCR",
                    "ocr_page",
                ),
                (
                    "OCR Entire Document",
                    "Run Apple Vision OCR across all pages in document",
                    "ocr_document",
                ),
                (
                    "Copy Recognized Text (OCR)",
                    "Run Vision OCR on page and copy directly to clipboard",
                    "copy_ocr",
                ),
                (
                    "Settings... (Cmd+,)",
                    "Open application settings and preferences",
                    "open_settings",
                ),
            ];

            for (title, desc, sel) in all_commands {
                if term.is_empty()
                    || title.to_lowercase().contains(&term)
                    || desc.to_lowercase().contains(&term)
                {
                    items.push(PaletteItemData {
                        title: title.to_string(),
                        subtitle: Some(desc.to_string()),
                        badge: Some("Command".to_string()),
                        kind: PaletteItemKind::Command { selector: sel },
                    });
                }
            }
        } else if query.starts_with('/') {
            // Current PDF text search
            let term = query[1..].trim();
            items.push(PaletteItemData {
                title: format!("Search current PDF for '{}'", term),
                subtitle: Some("Switches right sidebar to document search".to_string()),
                badge: Some("Search".to_string()),
                kind: PaletteItemKind::Command { selector: "search" },
            });
        } else if query.starts_with("p:") || query.starts_with(':') {
            let num_str = query.trim_start_matches(|c| c == 'p' || c == ':').trim();
            if let Ok(p) = num_str.parse::<usize>() {
                if p > 0 {
                    items.push(PaletteItemData {
                        title: format!("Go to Page {}", p),
                        subtitle: Some("Press Return to jump to page".to_string()),
                        badge: Some(format!("p. {}", p)),
                        kind: PaletteItemKind::PageJump { page_num: p - 1 },
                    });
                }
            }
        } else {
            // Default: search workspace files, recents, and favorites
            let term = query.trim().to_lowercase();
            let defaults = unsafe { NSUserDefaults::standardUserDefaults() };

            let mut seen_paths = std::collections::HashSet::new();

            // 1. Recents
            let recents = unsafe {
                defaults
                    .arrayForKey(&NSString::from_str("RecentFiles"))
                    .or_else(|| defaults.arrayForKey(&NSString::from_str("RecentItems")))
            };
            if let Some(recents) = recents {
                for i in 0..recents.count() {
                    let p = unsafe {
                        Retained::cast_unchecked::<NSString>(recents.objectAtIndex(i).clone())
                    }
                    .to_string();
                    let fname = Path::new(&p)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if (term.is_empty() || fname.to_lowercase().contains(&term))
                        && !seen_paths.contains(&p)
                    {
                        seen_paths.insert(p.clone());
                        items.push(PaletteItemData {
                            title: fname,
                            subtitle: Some(p.clone()),
                            badge: Some("Recent".to_string()),
                            kind: PaletteItemKind::File { path: p },
                        });
                        if items.len() >= 12 {
                            break;
                        }
                    }
                }
            }
        }

        *self.ivars().items.borrow_mut() = items.clone();

        if let Some(tv) = self.ivars().table_view.get() {
            unsafe {
                tv.reloadData();
                if !items.is_empty() {
                    let index_set = objc2_foundation::NSIndexSet::indexSetWithIndex(0);
                    tv.selectRowIndexes_byExtendingSelection(&index_set, false);
                }
            };

            // Dynamic panel sizing constrained by WindowGeometry
            if let Some(panel) = self.ivars().panel.get() {
                let mtm = MainThreadMarker::new().unwrap();
                let vis = WindowGeometry::screen_visible_frame(Some(panel.as_ref()), mtm);
                let max_palette_h = WindowGeometry::max_height(vis, WindowKind::Palette);

                let row_count = items.len() as f64;
                let max_rows = 7.0;
                let display_rows = row_count.min(max_rows);
                let new_height = if display_rows == 0.0 {
                    72.0
                } else {
                    (72.0 + (display_rows * 44.0) + 16.0).min(max_palette_h)
                };

                let mut frame = unsafe { panel.frame() };
                frame.origin.y += frame.size.height - new_height;
                frame.size.height = new_height;
                let sanitized = WindowGeometry::sanitize_frame(frame, vis, WindowKind::Palette);
                unsafe { panel.setFrame_display(sanitized, true) };
            }
        }
    }
}

pub fn create_palette(
    mtm: MainThreadMarker,
) -> (Retained<NSPanel>, Retained<CommandPaletteDelegate>) {
    let vis = WindowGeometry::screen_visible_frame(None, mtm);
    let init_frame = WindowGeometry::default_frame(vis, WindowKind::Palette);

    let panel = unsafe {
        NSPanel::initWithContentRect_styleMask_backing_defer(
            NSPanel::alloc(mtm),
            init_frame,
            NSWindowStyleMask::Titled
                | NSWindowStyleMask::Resizable
                | NSWindowStyleMask::FullSizeContentView,
            NSBackingStoreType::Buffered,
            false,
        )
    };

    unsafe {
        panel.setTitleVisibility(objc2_app_kit::NSWindowTitleVisibility::Hidden);
        panel.setTitlebarAppearsTransparent(true);
        if let Some(btn) = panel.standardWindowButton(objc2_app_kit::NSWindowButton::CloseButton) {
            btn.setHidden(true);
        }
        if let Some(btn) =
            panel.standardWindowButton(objc2_app_kit::NSWindowButton::MiniaturizeButton)
        {
            btn.setHidden(true);
        }
        if let Some(btn) = panel.standardWindowButton(objc2_app_kit::NSWindowButton::ZoomButton) {
            btn.setHidden(true);
        }

        panel.setHasShadow(true);
        panel.setLevel(3);
        panel.setCollectionBehavior(
            NSWindowCollectionBehavior::Transient | NSWindowCollectionBehavior::IgnoresCycle,
        );
        panel.setBackgroundColor(Some(&NSColor::clearColor()));
        panel.setOpaque(false);
    }
    WindowGeometry::sanitize_window(&panel, WindowKind::Palette, mtm);

    let delegate = CommandPaletteDelegate::new(mtm);
    let del_ptr: *const CommandPaletteDelegate = &*delegate;
    unsafe {
        let _: () = msg_send![&panel, setDelegate: del_ptr];
    }

    let visual_effect = unsafe {
        NSVisualEffectView::initWithFrame(
            NSVisualEffectView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), init_frame.size),
        )
    };
    unsafe {
        visual_effect.setMaterial(NSVisualEffectMaterial::HUDWindow);
        visual_effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        visual_effect.setState(NSVisualEffectState::Active);
        visual_effect.setWantsLayer(true);
        let layer: Retained<objc2::runtime::AnyObject> = msg_send_id![&visual_effect, layer];
        let _: () = msg_send![&layer, setCornerRadius: 18.0_f64];
        let _: () = msg_send![&layer, setMasksToBounds: true];
        let cg_border = NSColor::separatorColor().CGColor();
        let ptr_border = Retained::as_ptr(&cg_border);
        let _: () = msg_send![&layer, setBorderWidth: 1.0_f64];
        let _: () = msg_send![&layer, setBorderColor: ptr_border];
        visual_effect.setTranslatesAutoresizingMaskIntoConstraints(false);
    }

    let search_field = unsafe { NSTextField::textFieldWithString(ns_string!(""), mtm) };
    let search_icon = unsafe {
        objc2_app_kit::NSImageView::imageViewWithImage(
            &objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(
                ns_string!("magnifyingglass"),
                None,
            )
            .unwrap(),
            mtm,
        )
    };
    unsafe {
        let config =
            objc2_app_kit::NSImageSymbolConfiguration::configurationWithPointSize_weight_scale(
                18.0,
                0.0,
                objc2_app_kit::NSImageSymbolScale::Large,
            );
        let _: () = msg_send![&search_icon, setSymbolConfiguration: &*config];
        search_icon.setTranslatesAutoresizingMaskIntoConstraints(false);

        search_field.setFocusRingType(objc2_app_kit::NSFocusRingType::None);
        search_field.setBezeled(false);
        search_field.setDrawsBackground(false);
        search_field.setPlaceholderString(Some(ns_string!(
            "Search files, recents, commands (>), workspace (//), outline (@)..."
        )));

        let font = objc2_app_kit::NSFont::systemFontOfSize_weight(18.0, 0.0);
        let _: () = msg_send![&search_field, setFont: &*font];
        let _: () = msg_send![&search_field, setDelegate: del_ptr];
        search_field.setTranslatesAutoresizingMaskIntoConstraints(false);
    }

    let scroll = unsafe {
        NSScrollView::initWithFrame(
            NSScrollView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(620.0, 0.0)),
        )
    };
    let table_view = unsafe {
        NSOutlineView::initWithFrame(
            NSOutlineView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(620.0, 0.0)),
        )
    };
    let col =
        unsafe { NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), ns_string!("Col")) };
    unsafe {
        col.setWidth(590.0);
        table_view.addTableColumn(&col);
        table_view.setOutlineTableColumn(Some(&col));
        table_view.setHeaderView(None);
        table_view.setRowHeight(44.0);
        table_view.setBackgroundColor(&NSColor::clearColor());
        table_view.setSelectionHighlightStyle(
            objc2_app_kit::NSTableViewSelectionHighlightStyle::SourceList,
        );

        let _: () = msg_send![&table_view, setDelegate: del_ptr];
        let _: () = msg_send![&table_view, setDataSource: del_ptr];

        scroll.setDocumentView(Some(&table_view));
        scroll.setDrawsBackground(false);
        scroll.setTranslatesAutoresizingMaskIntoConstraints(false);
    }

    unsafe {
        visual_effect.addSubview(&search_icon);
        visual_effect.addSubview(&search_field);
        visual_effect.addSubview(&scroll);

        search_icon
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.leadingAnchor(), 16.0)
            .setActive(true);
        search_icon
            .topAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.topAnchor(), 24.0)
            .setActive(true);
        search_icon
            .widthAnchor()
            .constraintEqualToConstant(24.0)
            .setActive(true);
        search_icon
            .heightAnchor()
            .constraintEqualToConstant(24.0)
            .setActive(true);

        search_field
            .centerYAnchor()
            .constraintEqualToAnchor(&search_icon.centerYAnchor())
            .setActive(true);
        search_field
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&search_icon.trailingAnchor(), 10.0)
            .setActive(true);
        search_field
            .trailingAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.trailingAnchor(), -16.0)
            .setActive(true);
        search_field
            .heightAnchor()
            .constraintEqualToConstant(32.0)
            .setActive(true);

        scroll
            .topAnchor()
            .constraintEqualToAnchor_constant(&search_field.bottomAnchor(), 16.0)
            .setActive(true);
        scroll
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.leadingAnchor(), 8.0)
            .setActive(true);
        scroll
            .trailingAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.trailingAnchor(), -8.0)
            .setActive(true);
        scroll
            .bottomAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.bottomAnchor(), -8.0)
            .setActive(true);

        panel.setContentView(Some(&visual_effect));
    }

    delegate.setup(panel.clone(), search_field.clone(), table_view.clone());

    (panel, delegate)
}
