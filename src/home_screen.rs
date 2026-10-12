use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{DeclaredClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSBezelStyle, NSButton, NSColor, NSFont, NSImage, NSImageView, NSLayoutConstraint, NSMenu,
    NSMenuItem, NSResponder, NSScrollView, NSSearchField, NSSplitViewController, NSSplitViewItem,
    NSTableCellView, NSTableColumn, NSTableView, NSTableViewDataSource, NSTableViewDelegate,
    NSTextField, NSUserInterfaceItemIdentification, NSView, NSViewController,
    NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
    NSWorkspace,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSIndexSet, NSInteger, NSObjectProtocol, NSPoint, NSRect, NSSize,
    NSString, NSURL, NSUserDefaults,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub enum SidebarItem {
    Header(String),
    Item(String, String, String), // Title, SF Symbol Icon, Identifier
}

#[derive(Clone, Debug)]
pub struct MainAreaItem {
    pub name: String,
    pub path: String,
    pub is_folder: bool,
}

#[derive(Default)]
pub struct HomeDelegateIvars {
    pub sidebar_items: RefCell<Vec<SidebarItem>>,
    pub main_items: RefCell<Vec<MainAreaItem>>,
    pub all_main_items: RefCell<Vec<MainAreaItem>>,
    pub current_sidebar_id: RefCell<String>,
    pub sidebar_table: RefCell<Option<Retained<NSTableView>>>,
    pub main_table: RefCell<Option<Retained<NSTableView>>>,
    pub search_field: RefCell<Option<Retained<NSSearchField>>>,
    pub main_title_label: RefCell<Option<Retained<NSTextField>>>,
    pub main_subtitle_label: RefCell<Option<Retained<NSTextField>>>,
    pub empty_state_view: RefCell<Option<Retained<NSView>>>,
}

define_class!(
    #[unsafe(super(NSResponder))]
    #[thread_kind = MainThreadOnly]
    #[name = "HomeDelegate"]
    #[ivars = HomeDelegateIvars]
    pub struct HomeDelegate;

    unsafe impl NSObjectProtocol for HomeDelegate {}

    impl HomeDelegate {
        #[unsafe(method(numberOfRowsInTableView:))]
        fn number_of_rows_in_table_view(&self, table_view: &NSTableView) -> NSInteger {
            let identifier = unsafe { table_view.identifier() };
            if identifier.map(|s| s.to_string()).unwrap_or_default() == "SidebarTable" {
                self.ivars().sidebar_items.borrow().len() as NSInteger
            } else {
                self.ivars().main_items.borrow().len() as NSInteger
            }
        }

        #[unsafe(method(tableView:viewForTableColumn:row:))]
        fn table_view_view_for_table_column_row(
            &self,
            table_view: &NSTableView,
            _table_column: Option<&NSTableColumn>,
            row: NSInteger,
        ) -> *mut AnyObject {
            let mtm = MainThreadMarker::from(self);
            let identifier = unsafe { table_view.identifier() };

            if identifier.map(|s| s.to_string()).unwrap_or_default() == "SidebarTable" {
                let items = self.ivars().sidebar_items.borrow();
                let item = match items.get(row as usize) {
                    Some(it) => it.clone(),
                    None => return std::ptr::null_mut(),
                };

                match item {
                    SidebarItem::Header(title) => {
                        let view_id = NSString::from_str("SidebarHeaderCell");
                        let cell = unsafe { table_view.makeViewWithIdentifier_owner(&view_id, Some(self)) };
                        let cell_view = if let Some(cell) = cell {
                            unsafe { Retained::cast_unchecked::<NSTableCellView>(cell) }
                        } else {
                            let new_cell = NSTableCellView::new(mtm);
                            unsafe { new_cell.setIdentifier(Some(&view_id)) };
                            let tf = NSTextField::labelWithString(&NSString::from_str(""), mtm);
                            unsafe {
                                tf.setTag(201);
                                tf.setFont(Some(&NSFont::boldSystemFontOfSize(11.0)));
                                tf.setTextColor(Some(&NSColor::tertiaryLabelColor()));
                                tf.setTranslatesAutoresizingMaskIntoConstraints(false);
                                new_cell.addSubview(&tf);
                                NSLayoutConstraint::activateConstraints(&NSArray::from_slice(&[
                                    &*tf.leadingAnchor().constraintEqualToAnchor_constant(&new_cell.leadingAnchor(), 10.0),
                                    &*tf.trailingAnchor().constraintEqualToAnchor_constant(&new_cell.trailingAnchor(), -8.0),
                                    &*tf.centerYAnchor().constraintEqualToAnchor(&new_cell.centerYAnchor()),
                                ]));
                            }
                            new_cell
                        };

                        if let Some(tf) = unsafe { cell_view.viewWithTag(201) } {
                            let tf = unsafe { Retained::cast_unchecked::<NSTextField>(tf) };
                            unsafe { tf.setStringValue(&NSString::from_str(&title)) };
                        }

                        Retained::autorelease_ptr(unsafe { Retained::cast_unchecked::<AnyObject>(cell_view) })
                    }
                    SidebarItem::Item(title, icon_name, _) => {
                        let view_id = NSString::from_str("SidebarItemCell");
                        let cell = unsafe { table_view.makeViewWithIdentifier_owner(&view_id, Some(self)) };
                        let cell_view = if let Some(cell) = cell {
                            unsafe { Retained::cast_unchecked::<NSTableCellView>(cell) }
                        } else {
                            let new_cell = NSTableCellView::new(mtm);
                            unsafe { new_cell.setIdentifier(Some(&view_id)) };

                            let img_view = NSImageView::new(mtm);
                            img_view.setTag(202);
                            img_view.setTranslatesAutoresizingMaskIntoConstraints(false);

                            let tf = NSTextField::labelWithString(&NSString::from_str(""), mtm);
                            tf.setTag(203);
                            tf.setFont(Some(&NSFont::systemFontOfSize(13.0)));
                            tf.setTextColor(Some(&NSColor::labelColor()));
                            tf.setTranslatesAutoresizingMaskIntoConstraints(false);

                            unsafe {
                                new_cell.addSubview(&img_view);
                                new_cell.addSubview(&tf);

                                NSLayoutConstraint::activateConstraints(&NSArray::from_slice(&[
                                    &*img_view.leadingAnchor().constraintEqualToAnchor_constant(&new_cell.leadingAnchor(), 8.0),
                                    &*img_view.centerYAnchor().constraintEqualToAnchor(&new_cell.centerYAnchor()),
                                    &*img_view.widthAnchor().constraintEqualToConstant(18.0),
                                    &*img_view.heightAnchor().constraintEqualToConstant(18.0),

                                    &*tf.leadingAnchor().constraintEqualToAnchor_constant(&img_view.trailingAnchor(), 8.0),
                                    &*tf.trailingAnchor().constraintEqualToAnchor_constant(&new_cell.trailingAnchor(), -8.0),
                                    &*tf.centerYAnchor().constraintEqualToAnchor(&new_cell.centerYAnchor()),
                                ]));
                            }
                            new_cell
                        };

                        if let Some(img_view) = unsafe { cell_view.viewWithTag(202) } {
                            let img_view = unsafe { Retained::cast_unchecked::<NSImageView>(img_view) };
                            if let Some(img) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
                                &NSString::from_str(&icon_name),
                                None,
                            ) {
                                unsafe {
                                    img_view.setImage(Some(&img));
                                    img_view.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
                                }
                            }
                        }

                        if let Some(tf) = unsafe { cell_view.viewWithTag(203) } {
                            let tf = unsafe { Retained::cast_unchecked::<NSTextField>(tf) };
                            unsafe { tf.setStringValue(&NSString::from_str(&title)) };
                        }

                        Retained::autorelease_ptr(unsafe { Retained::cast_unchecked::<AnyObject>(cell_view) })
                    }
                }
            } else {
                // Main Table Cell
                let items = self.ivars().main_items.borrow();
                let item = match items.get(row as usize) {
                    Some(it) => it.clone(),
                    None => return std::ptr::null_mut(),
                };

                let view_id = NSString::from_str("MainCell");
                let cell = unsafe { table_view.makeViewWithIdentifier_owner(&view_id, Some(self)) };
                let cell_view = if let Some(cell) = cell {
                    unsafe { Retained::cast_unchecked::<NSTableCellView>(cell) }
                } else {
                    let new_cell = NSTableCellView::new(mtm);
                    unsafe { new_cell.setIdentifier(Some(&view_id)) };

                    let img_view = NSImageView::new(mtm);
                    img_view.setTag(101);
                    img_view.setTranslatesAutoresizingMaskIntoConstraints(false);

                    let title_tf = NSTextField::labelWithString(&NSString::from_str(""), mtm);
                    title_tf.setTag(102);
                    title_tf.setFont(Some(&NSFont::boldSystemFontOfSize(13.5)));
                    title_tf.setTextColor(Some(&NSColor::labelColor()));
                    title_tf.setTranslatesAutoresizingMaskIntoConstraints(false);

                    let sub_tf = NSTextField::labelWithString(&NSString::from_str(""), mtm);
                    sub_tf.setTag(103);
                    sub_tf.setFont(Some(&NSFont::systemFontOfSize(11.0)));
                    sub_tf.setTextColor(Some(&NSColor::secondaryLabelColor()));
                    sub_tf.setTranslatesAutoresizingMaskIntoConstraints(false);

                    let badge_tf = NSTextField::labelWithString(&NSString::from_str(""), mtm);
                    badge_tf.setTag(104);
                    badge_tf.setFont(Some(&NSFont::systemFontOfSize(11.0)));
                    badge_tf.setTextColor(Some(&NSColor::tertiaryLabelColor()));
                    badge_tf.setTranslatesAutoresizingMaskIntoConstraints(false);

                    unsafe {
                        new_cell.addSubview(&img_view);
                        new_cell.addSubview(&title_tf);
                        new_cell.addSubview(&sub_tf);
                        new_cell.addSubview(&badge_tf);

                        NSLayoutConstraint::activateConstraints(&NSArray::from_slice(&[
                            &*img_view.leadingAnchor().constraintEqualToAnchor_constant(&new_cell.leadingAnchor(), 16.0),
                            &*img_view.centerYAnchor().constraintEqualToAnchor(&new_cell.centerYAnchor()),
                            &*img_view.widthAnchor().constraintEqualToConstant(32.0),
                            &*img_view.heightAnchor().constraintEqualToConstant(32.0),

                            &*badge_tf.trailingAnchor().constraintEqualToAnchor_constant(&new_cell.trailingAnchor(), -20.0),
                            &*badge_tf.centerYAnchor().constraintEqualToAnchor(&new_cell.centerYAnchor()),

                            &*title_tf.leadingAnchor().constraintEqualToAnchor_constant(&img_view.trailingAnchor(), 12.0),
                            &*title_tf.topAnchor().constraintEqualToAnchor_constant(&new_cell.topAnchor(), 9.0),
                            &*title_tf.trailingAnchor().constraintLessThanOrEqualToAnchor_constant(&badge_tf.leadingAnchor(), -12.0),

                            &*sub_tf.leadingAnchor().constraintEqualToAnchor_constant(&img_view.trailingAnchor(), 12.0),
                            &*sub_tf.topAnchor().constraintEqualToAnchor_constant(&title_tf.bottomAnchor(), 3.0),
                            &*sub_tf.trailingAnchor().constraintLessThanOrEqualToAnchor_constant(&badge_tf.leadingAnchor(), -12.0),
                        ]));
                    }
                    new_cell
                };

                let exists = Path::new(&item.path).exists();

                // Icon
                if let Some(img_view) = unsafe { cell_view.viewWithTag(101) } {
                    let img_view = unsafe { Retained::cast_unchecked::<NSImageView>(img_view) };
                    let icon_name = if item.is_folder { "folder.fill" } else { "doc.richtext.fill" };
                    if let Some(img) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
                        &NSString::from_str(icon_name),
                        None,
                    ) {
                        unsafe {
                            img_view.setImage(Some(&img));
                            let tint = if item.is_folder {
                                NSColor::systemBlueColor()
                            } else {
                                NSColor::systemRedColor()
                            };
                            img_view.setContentTintColor(Some(&tint));
                        }
                    }
                }

                // Title
                if let Some(tf) = unsafe { cell_view.viewWithTag(102) } {
                    let tf = unsafe { Retained::cast_unchecked::<NSTextField>(tf) };
                    unsafe {
                        tf.setStringValue(&NSString::from_str(&item.name));
                        if exists {
                            tf.setTextColor(Some(&NSColor::labelColor()));
                        } else {
                            tf.setTextColor(Some(&NSColor::secondaryLabelColor()));
                        }
                    }
                }

                // Subtitle
                if let Some(tf) = unsafe { cell_view.viewWithTag(103) } {
                    let tf = unsafe { Retained::cast_unchecked::<NSTextField>(tf) };
                    let home_dir = unsafe { objc2_foundation::NSHomeDirectory() }.to_string();
                    let display_path = if item.path.starts_with(&home_dir) {
                        format!("~{}", &item.path[home_dir.len()..])
                    } else {
                        item.path.clone()
                    };
                    let text = if exists {
                        display_path
                    } else {
                        format!("{} • Missing", display_path)
                    };
                    unsafe {
                        tf.setStringValue(&NSString::from_str(&text));
                        tf.setTextColor(Some(&NSColor::secondaryLabelColor()));
                    }
                }

                // Badge
                if let Some(tf) = unsafe { cell_view.viewWithTag(104) } {
                    let tf = unsafe { Retained::cast_unchecked::<NSTextField>(tf) };
                    let is_fav = Self::is_favorite(&item.path);

                    let mut progress_info = None;
                    if !item.is_folder {
                        let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
                        let key = NSString::from_str(&format!("PDFPos_{}", item.path));
                        if let Some(dict) = unsafe { defaults.dictionaryForKey(&key) } {
                            let page_key = NSString::from_str("PageIndex");
                            let total_key = NSString::from_str("TotalPages");
                            let page_idx: usize = if let Some(p) = unsafe { dict.objectForKey(&page_key) } {
                                unsafe { msg_send![&*p, unsignedIntegerValue] }
                            } else { 0 };
                            let total_pages: usize = if let Some(t) = unsafe { dict.objectForKey(&total_key) } {
                                unsafe { msg_send![&*t, unsignedIntegerValue] }
                            } else { 0 };
                            if total_pages > 0 {
                                let pct = ((page_idx + 1) as f64 / total_pages as f64 * 100.0).round() as usize;
                                progress_info = Some((page_idx + 1, total_pages, pct));
                            }
                        }
                    }

                    let badge_text = if let Some((page, total, pct)) = progress_info {
                        if is_fav {
                            format!("★ p. {}/{} ({}%)", page, total, pct)
                        } else {
                            format!("p. {}/{} ({}%)", page, total, pct)
                        }
                    } else if is_fav {
                        if item.is_folder { "★ Folder".to_string() } else { "★ PDF".to_string() }
                    } else {
                        if item.is_folder { "Folder".to_string() } else { "PDF".to_string() }
                    };

                    unsafe {
                        tf.setStringValue(&NSString::from_str(&badge_text));
                        if is_fav {
                            tf.setTextColor(Some(&NSColor::systemYellowColor()));
                        } else if progress_info.is_some() {
                            tf.setTextColor(Some(&NSColor::systemBlueColor()));
                        } else {
                            tf.setTextColor(Some(&NSColor::tertiaryLabelColor()));
                        }
                    }
                }

                Retained::autorelease_ptr(unsafe { Retained::cast_unchecked::<AnyObject>(cell_view) })
            }
        }

        #[unsafe(method(tableView:isGroupRow:))]
        fn table_view_is_group_row(&self, table_view: &NSTableView, row: NSInteger) -> objc2::runtime::Bool {
            let identifier = unsafe { table_view.identifier() };
            if identifier.map(|s| s.to_string()).unwrap_or_default() == "SidebarTable" {
                let items = self.ivars().sidebar_items.borrow();
                if let Some(SidebarItem::Header(_)) = items.get(row as usize) {
                    return true.into();
                }
            }
            false.into()
        }

        #[unsafe(method(tableView:shouldSelectRow:))]
        fn table_view_should_select_row(&self, table_view: &NSTableView, row: NSInteger) -> objc2::runtime::Bool {
            let identifier = unsafe { table_view.identifier() };
            if identifier.map(|s| s.to_string()).unwrap_or_default() == "SidebarTable" {
                let items = self.ivars().sidebar_items.borrow();
                if let Some(SidebarItem::Header(_)) = items.get(row as usize) {
                    return false.into();
                }
            }
            true.into()
        }

        #[unsafe(method(tableViewSelectionDidChange:))]
        fn table_view_selection_did_change(&self, notification: &objc2_foundation::NSNotification) {
            let table_view: Retained<NSTableView> = unsafe { msg_send![notification, object] };
            let identifier = unsafe { table_view.identifier() };

            if identifier.map(|s| s.to_string()).unwrap_or_default() == "SidebarTable" {
                let row = unsafe { table_view.selectedRow() };
                if row >= 0 {
                    let items = self.ivars().sidebar_items.borrow();
                    if let Some(item) = items.get(row as usize) {
                        let item_clone = item.clone();
                        drop(items);
                        self.update_main_area_for_sidebar_selection(&item_clone);
                    }
                }
            }
        }

        #[unsafe(method(controlTextDidChange:))]
        fn control_text_did_change(&self, _notification: &objc2_foundation::NSNotification) {
            if let Some(search_field) = self.ivars().search_field.borrow().as_ref() {
                let query = unsafe { search_field.stringValue() }.to_string().to_lowercase();
                let all_items = self.ivars().all_main_items.borrow();
                let mut filtered = Vec::new();

                for item in all_items.iter() {
                    if query.is_empty()
                        || item.name.to_lowercase().contains(&query)
                        || item.path.to_lowercase().contains(&query)
                    {
                        filtered.push(item.clone());
                    }
                }

                *self.ivars().main_items.borrow_mut() = filtered.clone();

                if let Some(table) = self.ivars().main_table.borrow().as_ref() {
                    unsafe { table.reloadData() };
                }

                if let Some(empty) = self.ivars().empty_state_view.borrow().as_ref() {
                    unsafe { empty.setHidden(!filtered.is_empty()) };
                }
            }
        }

        #[unsafe(method(openPdf:))]
        fn open_pdf(&self, _sender: Option<&AnyObject>) {
            let mtm = MainThreadMarker::from(self);
            if let Some(app) = unsafe { objc2_app_kit::NSApplication::sharedApplication(mtm).delegate() } {
                let _: () = unsafe { msg_send![&*app, openPdf: std::ptr::null::<AnyObject>()] };
            }
        }

        #[unsafe(method(openFolder:))]
        fn open_folder(&self, _sender: Option<&AnyObject>) {
            let mtm = MainThreadMarker::from(self);
            if let Some(app) = unsafe { objc2_app_kit::NSApplication::sharedApplication(mtm).delegate() } {
                let _: () = unsafe { msg_send![&*app, openFolder: std::ptr::null::<AnyObject>()] };
            }
        }

        #[unsafe(method(mainTableDoubleClicked:))]
        fn main_table_double_clicked(&self, sender: &AnyObject) {
            let table_view: *mut NSTableView = sender as *const AnyObject as *mut NSTableView;
            let row = unsafe { (*table_view).clickedRow() };
            if row >= 0 {
                let items = self.ivars().main_items.borrow();
                if let Some(item) = items.get(row as usize) {
                    let path = item.path.clone();
                    drop(items);

                    let mtm = MainThreadMarker::from(self);
                    if let Some(app) = unsafe { objc2_app_kit::NSApplication::sharedApplication(mtm).delegate() } {
                        let ns_path = NSString::from_str(&path);
                        let _: () = unsafe { msg_send![&*app, openPathString: &*ns_path] };
                    }
                }
            }
        }

        #[unsafe(method(contextOpen:))]
        fn context_open(&self, _sender: &AnyObject) {
            if let Some(table) = self.ivars().main_table.borrow().as_ref() {
                let row = unsafe { table.clickedRow() };
                if row >= 0 {
                    let items = self.ivars().main_items.borrow();
                    if let Some(item) = items.get(row as usize) {
                        let path = item.path.clone();
                        drop(items);
                        let mtm = MainThreadMarker::from(self);
                        if let Some(app) = unsafe { objc2_app_kit::NSApplication::sharedApplication(mtm).delegate() } {
                            let ns_path = NSString::from_str(&path);
                            let _: () = unsafe { msg_send![&*app, openPathString: &*ns_path] };
                        }
                    }
                }
            }
        }

        #[unsafe(method(contextShowInFinder:))]
        fn context_show_in_finder(&self, _sender: &AnyObject) {
            if let Some(table) = self.ivars().main_table.borrow().as_ref() {
                let row = unsafe { table.clickedRow() };
                if row >= 0 {
                    let items = self.ivars().main_items.borrow();
                    if let Some(item) = items.get(row as usize) {
                        let ws = unsafe { NSWorkspace::sharedWorkspace() };
                        let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&item.path)) };
                        let urls = NSArray::from_slice(&[&*url]);
                        unsafe { ws.activateFileViewerSelectingURLs(&urls) };
                    }
                }
            }
        }

        #[unsafe(method(contextToggleFavorite:))]
        fn context_toggle_favorite(&self, _sender: &AnyObject) {
            if let Some(table) = self.ivars().main_table.borrow().as_ref() {
                let row = unsafe { table.clickedRow() };
                if row >= 0 {
                    let items = self.ivars().main_items.borrow();
                    if let Some(item) = items.get(row as usize) {
                        let path = item.path.clone();
                        drop(items);
                        Self::toggle_favorite(&path);
                        self.refresh_current_view();
                    }
                }
            }
        }

        #[unsafe(method(contextRemoveRecent:))]
        fn context_remove_recent(&self, _sender: &AnyObject) {
            if let Some(table) = self.ivars().main_table.borrow().as_ref() {
                let row = unsafe { table.clickedRow() };
                if row >= 0 {
                    let items = self.ivars().main_items.borrow();
                    if let Some(item) = items.get(row as usize) {
                        let path = item.path.clone();
                        drop(items);
                        Self::remove_from_recents(&path);
                        self.refresh_current_view();
                    }
                }
            }
        }

        #[unsafe(method(contextClearRecents:))]
        fn context_clear_recents(&self, _sender: &AnyObject) {
            Self::clear_recents();
            self.refresh_current_view();
        }
    }
);

unsafe impl NSTableViewDataSource for HomeDelegate {}
unsafe impl NSTableViewDelegate for HomeDelegate {}
unsafe impl objc2_app_kit::NSControlTextEditingDelegate for HomeDelegate {}
unsafe impl objc2_app_kit::NSTextFieldDelegate for HomeDelegate {}
unsafe impl objc2_app_kit::NSSearchFieldDelegate for HomeDelegate {}

impl HomeDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(HomeDelegateIvars::default());
        unsafe { msg_send![super(this), init] }
    }

    pub fn update_main_area_for_sidebar_selection(&self, item: &SidebarItem) {
        let mut new_items = Vec::new();
        let defaults = unsafe { NSUserDefaults::standardUserDefaults() };

        match item {
            SidebarItem::Item(title, _, id) => {
                *self.ivars().current_sidebar_id.borrow_mut() = id.clone();

                if id == "home" || id == "recent" {
                    if let Some(title_lbl) = self.ivars().main_title_label.borrow().as_ref() {
                        unsafe {
                            title_lbl.setStringValue(&NSString::from_str("Recent Documents"))
                        };
                    }
                    if let Some(sub_lbl) = self.ivars().main_subtitle_label.borrow().as_ref() {
                        unsafe {
                            sub_lbl
                                .setStringValue(&NSString::from_str("Pick up where you left off"))
                        };
                    }

                    let key1 = NSString::from_str("RecentFiles");
                    let key2 = NSString::from_str("RecentItems");
                    let recents = unsafe {
                        defaults
                            .arrayForKey(&key1)
                            .or_else(|| defaults.arrayForKey(&key2))
                    };
                    if let Some(recents) = recents {
                        for i in 0..recents.count() {
                            let path = unsafe {
                                Retained::cast_unchecked::<NSString>(
                                    recents.objectAtIndex(i).clone(),
                                )
                            }
                            .to_string();
                            let is_folder = Path::new(&path).is_dir();
                            let name = PathBuf::from(&path)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            new_items.push(MainAreaItem {
                                name,
                                path,
                                is_folder,
                            });
                        }
                    }
                } else if id == "continue_reading" {
                    if let Some(title_lbl) = self.ivars().main_title_label.borrow().as_ref() {
                        unsafe {
                            title_lbl.setStringValue(&NSString::from_str("Continue Reading"))
                        };
                    }
                    if let Some(sub_lbl) = self.ivars().main_subtitle_label.borrow().as_ref() {
                        unsafe {
                            sub_lbl.setStringValue(&NSString::from_str(
                                "Jump right back into your active books",
                            ))
                        };
                    }

                    let key1 = NSString::from_str("RecentFiles");
                    let key2 = NSString::from_str("RecentItems");
                    let recents = unsafe {
                        defaults
                            .arrayForKey(&key1)
                            .or_else(|| defaults.arrayForKey(&key2))
                    };
                    if let Some(recents) = recents {
                        for i in 0..recents.count() {
                            let path = unsafe {
                                Retained::cast_unchecked::<NSString>(
                                    recents.objectAtIndex(i).clone(),
                                )
                            }
                            .to_string();
                            let is_folder = Path::new(&path).is_dir();
                            if !is_folder {
                                let key_pos = NSString::from_str(&format!("PDFPos_{}", path));
                                if unsafe { defaults.dictionaryForKey(&key_pos) }.is_some() {
                                    let name = PathBuf::from(&path)
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string();
                                    new_items.push(MainAreaItem {
                                        name,
                                        path,
                                        is_folder,
                                    });
                                }
                            }
                        }
                    }
                } else if id == "favorites" {
                    if let Some(title_lbl) = self.ivars().main_title_label.borrow().as_ref() {
                        unsafe { title_lbl.setStringValue(&NSString::from_str("Favorites")) };
                    }
                    if let Some(sub_lbl) = self.ivars().main_subtitle_label.borrow().as_ref() {
                        unsafe {
                            sub_lbl
                                .setStringValue(&NSString::from_str("Pinned documents and folders"))
                        };
                    }

                    let favs =
                        unsafe { defaults.arrayForKey(&NSString::from_str("FavoriteFiles")) };
                    if let Some(favs) = favs {
                        for i in 0..favs.count() {
                            let path = unsafe {
                                Retained::cast_unchecked::<NSString>(favs.objectAtIndex(i).clone())
                            }
                            .to_string();
                            let is_folder = Path::new(&path).is_dir();
                            let name = PathBuf::from(&path)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            new_items.push(MainAreaItem {
                                name,
                                path,
                                is_folder,
                            });
                        }
                    }
                } else if id == "local" {
                    if let Some(title_lbl) = self.ivars().main_title_label.borrow().as_ref() {
                        unsafe { title_lbl.setStringValue(&NSString::from_str("Local Documents")) };
                    }
                    let home = unsafe { objc2_foundation::NSHomeDirectory() }.to_string();
                    let docs_path = format!("{}/Documents", home);
                    if let Some(sub_lbl) = self.ivars().main_subtitle_label.borrow().as_ref() {
                        unsafe { sub_lbl.setStringValue(&NSString::from_str(&docs_path)) };
                    }
                    Self::load_directory_items(&docs_path, &mut new_items);
                } else if id == "icloud" {
                    if let Some(title_lbl) = self.ivars().main_title_label.borrow().as_ref() {
                        unsafe { title_lbl.setStringValue(&NSString::from_str("iCloud Drive")) };
                    }
                    let home = unsafe { objc2_foundation::NSHomeDirectory() }.to_string();
                    let icloud_path =
                        format!("{}/Library/Mobile Documents/com~apple~CloudDocs", home);
                    if let Some(sub_lbl) = self.ivars().main_subtitle_label.borrow().as_ref() {
                        unsafe { sub_lbl.setStringValue(&NSString::from_str("iCloud Documents")) };
                    }
                    Self::load_directory_items(&icloud_path, &mut new_items);
                } else {
                    // Workspace path
                    if let Some(title_lbl) = self.ivars().main_title_label.borrow().as_ref() {
                        unsafe { title_lbl.setStringValue(&NSString::from_str(title)) };
                    }
                    if let Some(sub_lbl) = self.ivars().main_subtitle_label.borrow().as_ref() {
                        unsafe { sub_lbl.setStringValue(&NSString::from_str(id)) };
                    }
                    Self::load_directory_items(id, &mut new_items);
                }
            }
            _ => {}
        }

        *self.ivars().all_main_items.borrow_mut() = new_items.clone();
        *self.ivars().main_items.borrow_mut() = new_items.clone();

        if let Some(table) = self.ivars().main_table.borrow().as_ref() {
            unsafe { table.reloadData() };
        }

        if let Some(empty) = self.ivars().empty_state_view.borrow().as_ref() {
            unsafe { empty.setHidden(!new_items.is_empty()) };
        }
    }

    fn load_directory_items(dir_path: &str, items: &mut Vec<MainAreaItem>) {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let is_dir = path.is_dir();
                let is_pdf = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase() == "pdf")
                    .unwrap_or(false);
                if is_dir || is_pdf {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if !name.starts_with('.') {
                        items.push(MainAreaItem {
                            name,
                            path: path.to_string_lossy().to_string(),
                            is_folder: is_dir,
                        });
                    }
                }
            }
        }
    }

    pub fn refresh_current_view(&self) {
        let current_id = self.ivars().current_sidebar_id.borrow().clone();
        let items = self.ivars().sidebar_items.borrow();
        for item in items.iter() {
            if let SidebarItem::Item(_, _, id) = item {
                if id == &current_id {
                    let item_clone = item.clone();
                    drop(items);
                    self.update_main_area_for_sidebar_selection(&item_clone);
                    return;
                }
            }
        }
    }

    pub fn is_favorite(path: &str) -> bool {
        let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
        let key = NSString::from_str("FavoriteFiles");
        if let Some(existing) = unsafe { defaults.arrayForKey(&key) } {
            for i in 0..existing.count() {
                let s = unsafe {
                    Retained::cast_unchecked::<NSString>(existing.objectAtIndex(i).clone())
                }
                .to_string();
                if s == path {
                    return true;
                }
            }
        }
        false
    }

    pub fn toggle_favorite(path: &str) {
        let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
        let key = NSString::from_str("FavoriteFiles");
        let existing = unsafe { defaults.arrayForKey(&key) };
        let mut arr = Vec::new();
        let mut exists = false;
        if let Some(existing) = existing {
            for i in 0..existing.count() {
                let s = unsafe {
                    Retained::cast_unchecked::<NSString>(existing.objectAtIndex(i).clone())
                }
                .to_string();
                if s == path {
                    exists = true;
                } else {
                    arr.push(s);
                }
            }
        }
        if !exists {
            arr.insert(0, path.to_string());
        }
        let ns_arr: Vec<Retained<NSString>> = arr.iter().map(|s| NSString::from_str(s)).collect();
        let refs: Vec<&NSString> = ns_arr.iter().map(|s| &**s).collect();
        let final_arr = NSArray::from_slice(&refs);
        unsafe { defaults.setObject_forKey(Some(&final_arr), &key) };
    }

    pub fn remove_from_recents(path: &str) {
        let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
        let key1 = NSString::from_str("RecentFiles");
        let key2 = NSString::from_str("RecentItems");

        for key in [&key1, &key2] {
            let existing = unsafe { defaults.arrayForKey(key) };
            let mut arr = Vec::new();
            if let Some(existing) = existing {
                for i in 0..existing.count() {
                    let s = unsafe {
                        Retained::cast_unchecked::<NSString>(existing.objectAtIndex(i).clone())
                    }
                    .to_string();
                    if s != path {
                        arr.push(s);
                    }
                }
            }
            let ns_arr: Vec<Retained<NSString>> =
                arr.iter().map(|s| NSString::from_str(s)).collect();
            let refs: Vec<&NSString> = ns_arr.iter().map(|s| &**s).collect();
            let final_arr = NSArray::from_slice(&refs);
            unsafe { defaults.setObject_forKey(Some(&final_arr), key) };
        }
    }

    pub fn clear_recents() {
        unsafe {
            let defaults = NSUserDefaults::standardUserDefaults();
            defaults.removeObjectForKey(&NSString::from_str("RecentFiles"));
            defaults.removeObjectForKey(&NSString::from_str("RecentItems"));
        }
    }
}

pub struct HomeScreen {
    vc: Retained<NSSplitViewController>,
    delegate: Retained<HomeDelegate>,
}

impl HomeScreen {
    pub fn new(mtm: MainThreadMarker) -> Self {
        let split_vc = NSSplitViewController::new(mtm);
        let delegate = HomeDelegate::new(mtm);

        let mut sidebar_items = vec![
            SidebarItem::Header("".to_string()),
            SidebarItem::Item("Home".to_string(), "house".to_string(), "home".to_string()),
            SidebarItem::Item(
                "Continue Reading".to_string(),
                "book.closed.fill".to_string(),
                "continue_reading".to_string(),
            ),
            SidebarItem::Item(
                "Recent".to_string(),
                "clock".to_string(),
                "recent".to_string(),
            ),
            SidebarItem::Item(
                "Favorites".to_string(),
                "star".to_string(),
                "favorites".to_string(),
            ),
            SidebarItem::Header("LOCATIONS".to_string()),
            SidebarItem::Item(
                "Local Files".to_string(),
                "folder".to_string(),
                "local".to_string(),
            ),
            SidebarItem::Item(
                "iCloud Drive".to_string(),
                "icloud".to_string(),
                "icloud".to_string(),
            ),
            SidebarItem::Header("WORKSPACES".to_string()),
        ];

        let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
        if let Some(workspaces) =
            unsafe { defaults.arrayForKey(&NSString::from_str("RecentWorkspaces")) }
        {
            for i in 0..workspaces.count() {
                let path = unsafe {
                    Retained::cast_unchecked::<NSString>(workspaces.objectAtIndex(i).clone())
                }
                .to_string();
                let name = PathBuf::from(&path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                sidebar_items.push(SidebarItem::Item(name, "folder".to_string(), path));
            }
        }

        *delegate.ivars().sidebar_items.borrow_mut() = sidebar_items.clone();

        // 1. LEFT SIDEBAR
        let left_vc = NSViewController::new(mtm);
        let left_scroll = NSScrollView::new(mtm);
        let left_table = NSTableView::new(mtm);

        unsafe {
            left_table.setIdentifier(Some(&*NSString::from_str("SidebarTable")));
            let col = NSTableColumn::initWithIdentifier(
                NSTableColumn::alloc(mtm),
                &NSString::from_str("SidebarCol"),
            );
            col.setWidth(200.0);
            left_table.addTableColumn(&col);
            left_table.setHeaderView(None);
            left_table.setSelectionHighlightStyle(
                objc2_app_kit::NSTableViewSelectionHighlightStyle::SourceList,
            );
            left_table.setRowHeight(28.0);
            left_table.setIntercellSpacing(NSSize::new(4.0, 2.0));
            left_table.setBackgroundColor(&NSColor::clearColor());

            let del_ptr: *const HomeDelegate = &*delegate;
            let _: () = msg_send![&left_table, setDelegate: del_ptr];
            let _: () = msg_send![&left_table, setDataSource: del_ptr];

            left_scroll.setDocumentView(Some(&left_table));
            left_scroll.setHasVerticalScroller(true);
            left_scroll.setDrawsBackground(false);

            let left_effect = NSVisualEffectView::initWithFrame(
                NSVisualEffectView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(220.0, 700.0)),
            );
            left_effect.setMaterial(NSVisualEffectMaterial::Sidebar);
            left_effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
            left_effect.setState(NSVisualEffectState::FollowsWindowActiveState);
            left_scroll.setTranslatesAutoresizingMaskIntoConstraints(false);
            left_effect.addSubview(&left_scroll);
            left_scroll
                .topAnchor()
                .constraintEqualToAnchor_constant(&left_effect.topAnchor(), 32.0)
                .setActive(true);
            left_scroll
                .bottomAnchor()
                .constraintEqualToAnchor(&left_effect.bottomAnchor())
                .setActive(true);
            left_scroll
                .leadingAnchor()
                .constraintEqualToAnchor(&left_effect.leadingAnchor())
                .setActive(true);
            left_scroll
                .trailingAnchor()
                .constraintEqualToAnchor(&left_effect.trailingAnchor())
                .setActive(true);
            left_vc.setView(&left_effect);
        }

        *delegate.ivars().sidebar_table.borrow_mut() = Some(left_table.clone());

        let left_item = unsafe { NSSplitViewItem::sidebarWithViewController(&left_vc) };
        unsafe {
            left_item.setCanCollapse(false);
            left_item.setMinimumThickness(180.0);
            left_item.setMaximumThickness(260.0);
            left_item.setHoldingPriority(750.0);
        }

        // 2. RIGHT MAIN AREA
        let right_vc = NSViewController::new(mtm);
        let right_container = NSView::new(mtm);
        right_container.setTranslatesAutoresizingMaskIntoConstraints(false);

        // Header View
        let header_view = NSView::new(mtm);
        header_view.setTranslatesAutoresizingMaskIntoConstraints(false);

        let title_label =
            NSTextField::labelWithString(&NSString::from_str("Recent Documents"), mtm);
        unsafe {
            title_label.setFont(Some(&NSFont::boldSystemFontOfSize(22.0)));
            title_label.setTextColor(Some(&NSColor::labelColor()));
            title_label.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingTail);
            title_label.setTranslatesAutoresizingMaskIntoConstraints(false);
            header_view.addSubview(&title_label);
        }

        let subtitle_label =
            NSTextField::labelWithString(&NSString::from_str("Pick up where you left off"), mtm);
        unsafe {
            subtitle_label.setFont(Some(&NSFont::systemFontOfSize(12.0)));
            subtitle_label.setTextColor(Some(&NSColor::secondaryLabelColor()));
            subtitle_label.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingTail);
            subtitle_label.setTranslatesAutoresizingMaskIntoConstraints(false);
            header_view.addSubview(&subtitle_label);
        }

        let search_field = NSSearchField::new(mtm);
        unsafe {
            search_field.setPlaceholderString(Some(&NSString::from_str("Filter files...")));
            search_field.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
            search_field.setTranslatesAutoresizingMaskIntoConstraints(false);
        }

        let target_any: &AnyObject = unsafe { &*(&*delegate as *const _ as *const AnyObject) };

        let open_pdf_btn = unsafe {
            let btn = NSButton::buttonWithTitle_target_action(
                &NSString::from_str("Open PDF..."),
                Some(target_any),
                Some(sel!(openPdf:)),
                mtm,
            );
            btn.setBezelStyle(NSBezelStyle::Rounded);
            btn
        };
        let open_folder_btn = unsafe {
            let btn = NSButton::buttonWithTitle_target_action(
                &NSString::from_str("Open Folder..."),
                Some(target_any),
                Some(sel!(openFolder:)),
                mtm,
            );
            btn.setBezelStyle(NSBezelStyle::Rounded);
            btn
        };

        unsafe {
            open_pdf_btn.setTranslatesAutoresizingMaskIntoConstraints(false);
            open_folder_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

            open_pdf_btn.setContentHuggingPriority_forOrientation(
                objc2_app_kit::NSLayoutPriorityRequired,
                objc2_app_kit::NSLayoutConstraintOrientation::Horizontal,
            );
            open_pdf_btn.setContentCompressionResistancePriority_forOrientation(
                objc2_app_kit::NSLayoutPriorityRequired,
                objc2_app_kit::NSLayoutConstraintOrientation::Horizontal,
            );

            open_folder_btn.setContentHuggingPriority_forOrientation(
                objc2_app_kit::NSLayoutPriorityRequired,
                objc2_app_kit::NSLayoutConstraintOrientation::Horizontal,
            );
            open_folder_btn.setContentCompressionResistancePriority_forOrientation(
                objc2_app_kit::NSLayoutPriorityRequired,
                objc2_app_kit::NSLayoutConstraintOrientation::Horizontal,
            );

            header_view.addSubview(&search_field);
            header_view.addSubview(&open_pdf_btn);
            header_view.addSubview(&open_folder_btn);

            NSLayoutConstraint::activateConstraints(&NSArray::from_slice(&[
                &*title_label
                    .leadingAnchor()
                    .constraintEqualToAnchor_constant(&header_view.leadingAnchor(), 24.0),
                &*title_label
                    .topAnchor()
                    .constraintEqualToAnchor_constant(&header_view.topAnchor(), 8.0),
                &*search_field
                    .trailingAnchor()
                    .constraintEqualToAnchor_constant(&header_view.trailingAnchor(), -24.0),
                &*search_field
                    .centerYAnchor()
                    .constraintEqualToAnchor(&title_label.centerYAnchor()),
                &*search_field.widthAnchor().constraintEqualToConstant(140.0),
                &*subtitle_label
                    .leadingAnchor()
                    .constraintEqualToAnchor_constant(&header_view.leadingAnchor(), 24.0),
                &*subtitle_label
                    .topAnchor()
                    .constraintEqualToAnchor_constant(&title_label.bottomAnchor(), 8.0),
                &*open_pdf_btn
                    .trailingAnchor()
                    .constraintEqualToAnchor_constant(&header_view.trailingAnchor(), -24.0),
                &*open_pdf_btn
                    .centerYAnchor()
                    .constraintEqualToAnchor(&subtitle_label.centerYAnchor()),
                &*open_pdf_btn
                    .widthAnchor()
                    .constraintGreaterThanOrEqualToConstant(88.0),
                &*open_folder_btn
                    .trailingAnchor()
                    .constraintEqualToAnchor_constant(&open_pdf_btn.leadingAnchor(), -8.0),
                &*open_folder_btn
                    .centerYAnchor()
                    .constraintEqualToAnchor(&subtitle_label.centerYAnchor()),
                &*open_folder_btn
                    .widthAnchor()
                    .constraintGreaterThanOrEqualToConstant(104.0),
            ]));
        }

        // Separator
        let separator = NSView::new(mtm);
        separator.setTranslatesAutoresizingMaskIntoConstraints(false);
        separator.setWantsLayer(true);
        if let Some(layer) = unsafe { separator.layer() } {
            let cg = unsafe { NSColor::separatorColor().CGColor() };
            let ptr = Retained::as_ptr(&cg);
            let _: () = unsafe { msg_send![&layer, setBackgroundColor: ptr] };
        }

        // Main Table View
        let right_scroll = NSScrollView::new(mtm);
        right_scroll.setTranslatesAutoresizingMaskIntoConstraints(false);
        let right_table = NSTableView::new(mtm);

        unsafe {
            right_table.setIdentifier(Some(&*NSString::from_str("MainTable")));
            let main_col = NSTableColumn::initWithIdentifier(
                NSTableColumn::alloc(mtm),
                &NSString::from_str("MainCol"),
            );
            main_col.setWidth(600.0);
            right_table.addTableColumn(&main_col);
            right_table.setHeaderView(None);
            right_table.setRowHeight(54.0);
            right_table.setIntercellSpacing(NSSize::new(0.0, 2.0));
            right_table.setBackgroundColor(&NSColor::clearColor());

            let del_ptr: *const HomeDelegate = &*delegate;
            let _: () = msg_send![&right_table, setDelegate: del_ptr];
            let _: () = msg_send![&right_table, setDataSource: del_ptr];

            let menu = NSMenu::new(mtm);

            let open_item = NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str("Open"),
                Some(sel!(contextOpen:)),
                &NSString::from_str(""),
            );
            open_item.setTarget(Some(&*delegate));
            menu.addItem(&open_item);

            let finder_item = NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str("Show in Finder"),
                Some(sel!(contextShowInFinder:)),
                &NSString::from_str(""),
            );
            finder_item.setTarget(Some(&*delegate));
            menu.addItem(&finder_item);

            menu.addItem(&NSMenuItem::separatorItem(mtm));

            let fav_item = NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str("Toggle Favorite"),
                Some(sel!(contextToggleFavorite:)),
                &NSString::from_str(""),
            );
            fav_item.setTarget(Some(&*delegate));
            menu.addItem(&fav_item);

            let rem_item = NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str("Remove from Recents"),
                Some(sel!(contextRemoveRecent:)),
                &NSString::from_str(""),
            );
            rem_item.setTarget(Some(&*delegate));
            menu.addItem(&rem_item);

            let clear_item = NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str("Clear All Recents"),
                Some(sel!(contextClearRecents:)),
                &NSString::from_str(""),
            );
            clear_item.setTarget(Some(&*delegate));
            menu.addItem(&clear_item);

            right_table.setMenu(Some(&menu));
            right_table.setTarget(Some(&*delegate));
            right_table.setDoubleAction(Some(sel!(mainTableDoubleClicked:)));

            right_scroll.setDocumentView(Some(&right_table));
            right_scroll.setHasVerticalScroller(true);
            right_scroll.setDrawsBackground(false);
        }

        // Empty State View
        let empty_view = NSView::new(mtm);
        empty_view.setTranslatesAutoresizingMaskIntoConstraints(false);
        let empty_icon = NSImageView::new(mtm);
        empty_icon.setTranslatesAutoresizingMaskIntoConstraints(false);
        if let Some(img) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str("doc.badge.plus"),
            None,
        ) {
            unsafe {
                empty_icon.setImage(Some(&img));
                empty_icon.setContentTintColor(Some(&NSColor::tertiaryLabelColor()));
            }
        }
        let empty_title =
            NSTextField::labelWithString(&NSString::from_str("No Recent Documents"), mtm);
        unsafe {
            empty_title.setFont(Some(&NSFont::boldSystemFontOfSize(16.0)));
            empty_title.setTextColor(Some(&NSColor::secondaryLabelColor()));
            empty_title.setTranslatesAutoresizingMaskIntoConstraints(false);
        }
        let empty_subtitle = NSTextField::labelWithString(
            &NSString::from_str("Open a PDF or Folder to get started"),
            mtm,
        );
        unsafe {
            empty_subtitle.setFont(Some(&NSFont::systemFontOfSize(13.0)));
            empty_subtitle.setTextColor(Some(&NSColor::tertiaryLabelColor()));
            empty_subtitle.setTranslatesAutoresizingMaskIntoConstraints(false);
        }

        unsafe {
            empty_view.addSubview(&empty_icon);
            empty_view.addSubview(&empty_title);
            empty_view.addSubview(&empty_subtitle);

            NSLayoutConstraint::activateConstraints(&NSArray::from_slice(&[
                &*empty_icon
                    .centerXAnchor()
                    .constraintEqualToAnchor(&empty_view.centerXAnchor()),
                &*empty_icon
                    .centerYAnchor()
                    .constraintEqualToAnchor_constant(&empty_view.centerYAnchor(), -30.0),
                &*empty_icon.widthAnchor().constraintEqualToConstant(48.0),
                &*empty_icon.heightAnchor().constraintEqualToConstant(48.0),
                &*empty_title
                    .centerXAnchor()
                    .constraintEqualToAnchor(&empty_view.centerXAnchor()),
                &*empty_title
                    .topAnchor()
                    .constraintEqualToAnchor_constant(&empty_icon.bottomAnchor(), 12.0),
                &*empty_subtitle
                    .centerXAnchor()
                    .constraintEqualToAnchor(&empty_view.centerXAnchor()),
                &*empty_subtitle
                    .topAnchor()
                    .constraintEqualToAnchor_constant(&empty_title.bottomAnchor(), 4.0),
            ]));
        }

        unsafe {
            right_container.addSubview(&header_view);
            right_container.addSubview(&separator);
            right_container.addSubview(&right_scroll);
            right_container.addSubview(&empty_view);

            NSLayoutConstraint::activateConstraints(&NSArray::from_slice(&[
                &*header_view
                    .topAnchor()
                    .constraintEqualToAnchor_constant(&right_container.topAnchor(), 28.0),
                &*header_view
                    .leadingAnchor()
                    .constraintEqualToAnchor(&right_container.leadingAnchor()),
                &*header_view
                    .trailingAnchor()
                    .constraintEqualToAnchor(&right_container.trailingAnchor()),
                &*header_view.heightAnchor().constraintEqualToConstant(72.0),
                &*separator
                    .topAnchor()
                    .constraintEqualToAnchor(&header_view.bottomAnchor()),
                &*separator
                    .leadingAnchor()
                    .constraintEqualToAnchor(&right_container.leadingAnchor()),
                &*separator
                    .trailingAnchor()
                    .constraintEqualToAnchor(&right_container.trailingAnchor()),
                &*separator.heightAnchor().constraintEqualToConstant(1.0),
                &*right_scroll
                    .topAnchor()
                    .constraintEqualToAnchor(&separator.bottomAnchor()),
                &*right_scroll
                    .leadingAnchor()
                    .constraintEqualToAnchor(&right_container.leadingAnchor()),
                &*right_scroll
                    .trailingAnchor()
                    .constraintEqualToAnchor(&right_container.trailingAnchor()),
                &*right_scroll
                    .bottomAnchor()
                    .constraintEqualToAnchor(&right_container.bottomAnchor()),
                &*empty_view
                    .topAnchor()
                    .constraintEqualToAnchor(&separator.bottomAnchor()),
                &*empty_view
                    .leadingAnchor()
                    .constraintEqualToAnchor(&right_container.leadingAnchor()),
                &*empty_view
                    .trailingAnchor()
                    .constraintEqualToAnchor(&right_container.trailingAnchor()),
                &*empty_view
                    .bottomAnchor()
                    .constraintEqualToAnchor(&right_container.bottomAnchor()),
            ]));

            right_vc.setView(&right_container);
        }

        let right_item = unsafe { NSSplitViewItem::splitViewItemWithViewController(&right_vc) };

        unsafe {
            right_item.setHoldingPriority(250.0);
            split_vc.addSplitViewItem(&left_item);
            split_vc.addSplitViewItem(&right_item);
            let split_view = split_vc.splitView();
            split_view.setDividerStyle(objc2_app_kit::NSSplitViewDividerStyle::Thin);
        }

        *delegate.ivars().main_table.borrow_mut() = Some(right_table);
        *delegate.ivars().search_field.borrow_mut() = Some(search_field);
        *delegate.ivars().main_title_label.borrow_mut() = Some(title_label);
        *delegate.ivars().main_subtitle_label.borrow_mut() = Some(subtitle_label);
        *delegate.ivars().empty_state_view.borrow_mut() = Some(empty_view);

        // Pre-select row 1 ("Home") and populate initial items
        let index_set = unsafe { NSIndexSet::indexSetWithIndex(1) };
        unsafe { left_table.selectRowIndexes_byExtendingSelection(&index_set, false) };
        if let Some(first_item) = sidebar_items.get(1) {
            delegate.update_main_area_for_sidebar_selection(first_item);
        }

        Self {
            vc: split_vc,
            delegate,
        }
    }

    pub fn view_controller(&self) -> Retained<NSSplitViewController> {
        self.vc.clone()
    }

    pub fn refresh(&self) {
        self.delegate.refresh_current_view();
    }
}
