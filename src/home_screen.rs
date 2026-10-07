use objc2::rc::{Retained, Weak};
use objc2::{define_class, msg_send, msg_send_id, MainThreadOnly, DeclaredClass, ClassType, mutability, sel};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::{
    NSSplitViewController, NSSplitViewItem, NSViewController, NSTableView, NSTableViewDataSource,
    NSTableViewDelegate, NSScrollView, NSTableColumn, NSTableCellView, NSTextField, NSImage, NSMenu,
    NSMenuItem, NSSearchField, NSStackView, NSUserInterfaceLayoutOrientation, NSLayoutConstraint, NSControl, NSResponder
};
use objc2_foundation::{
    MainThreadMarker, NSRect, NSPoint, NSSize, NSString, NSInteger, NSObjectProtocol, NSObject,
    NSUserDefaults, NSArray, NSURL, NSFileManager
};
use std::cell::RefCell;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum SidebarItem {
    Header(String),
    Item(String, String, String), // Title, Icon Name, Identifier
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
    pub main_table: RefCell<Option<Weak<NSTableView>>>,
    pub search_field: RefCell<Option<Weak<NSSearchField>>>,
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
            if identifier.to_string() == "SidebarTable" {
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
            if identifier.to_string() == "SidebarTable" {
                let items = self.ivars().sidebar_items.borrow();
                let item = &items[row as usize];
                
                let view_id = objc2_foundation::ns_string!("SidebarCell");
                let mut cell = unsafe { table_view.makeViewWithIdentifier_owner(view_id, Some(self)) };
                let cell_view = if let Some(cell) = cell {
                    unsafe { Retained::cast_unchecked::<NSTableCellView>(cell) }
                } else {
                    let new_cell = NSTableCellView::new(mtm);
                    unsafe { new_cell.setIdentifier(view_id) };
                    let tf = NSTextField::labelWithString(objc2_foundation::ns_string!(""), mtm);
                    unsafe {
                        new_cell.addSubview(&tf);
                        new_cell.setTextField(Some(&tf));
                        tf.setTranslatesAutoresizingMaskIntoConstraints(false);
                        NSLayoutConstraint::activateConstraints(&NSArray::from_vec(vec![
                            tf.leadingAnchor().constraintEqualToAnchor_constant(&new_cell.leadingAnchor(), 4.0),
                            tf.centerYAnchor().constraintEqualToAnchor(&new_cell.centerYAnchor()),
                        ]));
                    }
                    new_cell
                };

                if let Some(tf) = unsafe { cell_view.textField() } {
                    match item {
                        SidebarItem::Header(title) => {
                            let text = format!("{}", title);
                            unsafe { 
                                tf.setStringValue(&NSString::from_str(&text));
                                tf.setFont(Some(&objc2_app_kit::NSFont::boldSystemFontOfSize(12.0)));
                                tf.setTextColor(Some(&objc2_app_kit::NSColor::tertiaryLabelColor()));
                            }
                        }
                        SidebarItem::Item(title, _icon, _id) => {
                            unsafe { 
                                tf.setStringValue(&NSString::from_str(title));
                                tf.setFont(Some(&objc2_app_kit::NSFont::systemFontOfSize(13.0)));
                                tf.setTextColor(Some(&objc2_app_kit::NSColor::labelColor()));
                            }
                        }
                    }
                }
                
                Retained::autorelease_ptr(unsafe { Retained::cast_unchecked::<AnyObject>(cell_view) })
            } else {
                let items = self.ivars().main_items.borrow();
                let item = &items[row as usize];
                
                let view_id = objc2_foundation::ns_string!("MainCell");
                let mut cell = unsafe { table_view.makeViewWithIdentifier_owner(view_id, Some(self)) };
                let cell_view = if let Some(cell) = cell {
                    unsafe { Retained::cast_unchecked::<NSTableCellView>(cell) }
                } else {
                    let new_cell = NSTableCellView::new(mtm);
                    unsafe { new_cell.setIdentifier(view_id) };
                    let tf = NSTextField::labelWithString(objc2_foundation::ns_string!(""), mtm);
                    let subtitle_tf = NSTextField::labelWithString(objc2_foundation::ns_string!(""), mtm);
                    unsafe {
                        subtitle_tf.setFont(Some(&objc2_app_kit::NSFont::systemFontOfSize(11.0)));
                        subtitle_tf.setTextColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
                        
                        new_cell.addSubview(&tf);
                        new_cell.addSubview(&subtitle_tf);
                        new_cell.setTextField(Some(&tf));
                        
                        tf.setTranslatesAutoresizingMaskIntoConstraints(false);
                        subtitle_tf.setTranslatesAutoresizingMaskIntoConstraints(false);
                        
                        NSLayoutConstraint::activateConstraints(&NSArray::from_vec(vec![
                            tf.leadingAnchor().constraintEqualToAnchor_constant(&new_cell.leadingAnchor(), 24.0),
                            tf.topAnchor().constraintEqualToAnchor_constant(&new_cell.topAnchor(), 4.0),
                            
                            subtitle_tf.leadingAnchor().constraintEqualToAnchor_constant(&new_cell.leadingAnchor(), 24.0),
                            subtitle_tf.topAnchor().constraintEqualToAnchor_constant(&tf.bottomAnchor(), 2.0),
                        ]));
                    }
                    new_cell
                };

                let subviews = unsafe { cell_view.subviews() };
                if subviews.len() >= 2 {
                    let tf = unsafe { Retained::cast_unchecked::<NSTextField>(subviews.objectAtIndex(0).clone()) };
                    let subtitle_tf = unsafe { Retained::cast_unchecked::<NSTextField>(subviews.objectAtIndex(1).clone()) };
                    
                    unsafe {
                        tf.setStringValue(&NSString::from_str(&item.name));
                        subtitle_tf.setStringValue(&NSString::from_str(&item.path));
                    }
                }
                
                Retained::autorelease_ptr(unsafe { Retained::cast_unchecked::<AnyObject>(cell_view) })
            }
        }

        #[unsafe(method(tableView:isGroupRow:))]
        fn table_view_is_group_row(&self, table_view: &NSTableView, row: NSInteger) -> bool {
            let identifier = unsafe { table_view.identifier() };
            if identifier.to_string() == "SidebarTable" {
                let items = self.ivars().sidebar_items.borrow();
                if let SidebarItem::Header(_) = items[row as usize] {
                    return true;
                }
            }
            false
        }
        
        #[unsafe(method(tableViewSelectionDidChange:))]
        fn table_view_selection_did_change(&self, notification: &objc2_foundation::NSNotification) {
            let table_view: Retained<NSTableView> = unsafe { msg_send_id![notification, object] };
            let identifier = unsafe { table_view.identifier() };
            
            if identifier.to_string() == "SidebarTable" {
                let row = unsafe { table_view.selectedRow() };
                if row >= 0 {
                    let items = self.ivars().sidebar_items.borrow();
                    let item = &items[row as usize];
                    self.update_main_area_for_sidebar_selection(item);
                }
            }
        }
        
        #[unsafe(method(controlTextDidChange:))]
        fn control_text_did_change(&self, _notification: &objc2_foundation::NSNotification) {
            if let Some(search_field) = self.ivars().search_field.borrow().as_ref().and_then(|w| w.load()) {
                let query = unsafe { search_field.stringValue() }.to_string().to_lowercase();
                
                let all_items = self.ivars().all_main_items.borrow();
                let mut filtered = Vec::new();
                
                for item in all_items.iter() {
                    if query.is_empty() || item.name.to_lowercase().contains(&query) || item.path.to_lowercase().contains(&query) {
                        filtered.push(item.clone());
                    }
                }
                
                *self.ivars().main_items.borrow_mut() = filtered;
                
                if let Some(table) = self.ivars().main_table.borrow().as_ref().and_then(|w| w.load()) {
                    unsafe { table.reloadData() };
                }
            }
        }
        
        #[unsafe(method(mainTableDoubleClicked:))]
        fn main_table_double_clicked(&self, sender: &AnyObject) {
            // let table_view = unsafe { Retained::cast_unchecked::<NSTableView>(Retained::retain(sender).unwrap()) };
            let table_view: *mut NSTableView = sender as *const AnyObject as *mut NSTableView;
            let row = unsafe { (*table_view).clickedRow() };
            if row >= 0 {
                let items = self.ivars().main_items.borrow();
                let item = &items[row as usize];
                
                let ws = unsafe { objc2_app_kit::NSWorkspace::sharedWorkspace() };
                let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&item.path)) };
                unsafe { ws.openURL(&url) };
            }
        }
    }
);

unsafe impl NSTableViewDataSource for HomeDelegate {}
unsafe impl NSTableViewDelegate for HomeDelegate {}
unsafe impl objc2_app_kit::NSSearchFieldDelegate for HomeDelegate {}

impl HomeDelegate {
    fn update_main_area_for_sidebar_selection(&self, item: &SidebarItem) {
        let mut new_items = Vec::new();
        let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
        
        match item {
            SidebarItem::Item(_title, _icon, id) => {
                if id == "recent" {
                    let recents = unsafe { defaults.arrayForKey(&NSString::from_str("RecentFiles")) };
                    if let Some(recents) = recents {
                        for i in 0..recents.count() {
                            let path = unsafe { Retained::cast_unchecked::<NSString>(recents.objectAtIndex(i).clone()) }.to_string();
                            let is_folder = unsafe {
                                let mut is_dir = objc2::ffi::YES;
                                let fm = NSFileManager::defaultManager();
                                fm.fileExistsAtPath_isDirectory(&NSString::from_str(&path), &mut is_dir)
                                && is_dir == objc2::ffi::YES
                            };
                            let name = PathBuf::from(&path).file_name().unwrap_or_default().to_string_lossy().to_string();
                            new_items.push(MainAreaItem { name, path, is_folder });
                        }
                    }
                } else if id == "favorites" {
                    let favs = unsafe { defaults.arrayForKey(&NSString::from_str("FavoriteFiles")) };
                    if let Some(favs) = favs {
                        for i in 0..favs.count() {
                            let path = unsafe { Retained::cast_unchecked::<NSString>(favs.objectAtIndex(i).clone()) }.to_string();
                            let is_folder = unsafe {
                                let mut is_dir = objc2::ffi::YES;
                                let fm = NSFileManager::defaultManager();
                                fm.fileExistsAtPath_isDirectory(&NSString::from_str(&path), &mut is_dir)
                                && is_dir == objc2::ffi::YES
                            };
                            let name = PathBuf::from(&path).file_name().unwrap_or_default().to_string_lossy().to_string();
                            new_items.push(MainAreaItem { name, path, is_folder });
                        }
                    }
                } else if id == "local" {
                    let url = unsafe { NSFileManager::defaultManager().URLsForDirectory_inDomains(objc2_foundation::NSDocumentDirectory, objc2_foundation::NSUserDomainMask).firstObject() };
                    if let Some(url) = url {
                        let url = unsafe { Retained::cast_unchecked::<NSURL>(url) };
                        if let Some(path) = unsafe { url.path() } {
                            new_items.push(MainAreaItem { name: "Local".to_string(), path: path.to_string(), is_folder: true });
                        }
                    }
                }
            }
            _ => {}
        }
        
        *self.ivars().all_main_items.borrow_mut() = new_items.clone();
        *self.ivars().main_items.borrow_mut() = new_items;
        
        if let Some(table) = self.ivars().main_table.borrow().as_ref().and_then(|w| w.load()) {
            unsafe { table.reloadData() };
        }
    }
    
    pub fn add_to_favorites(path: &str) {
        let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
        let key = NSString::from_str("FavoriteFiles");
        let existing = unsafe { defaults.arrayForKey(&key) };
        let mut arr = Vec::new();
        if let Some(existing) = existing {
            for i in 0..existing.count() {
                let s = unsafe { Retained::cast_unchecked::<NSString>(existing.objectAtIndex(i).clone()) }.to_string();
                if s != path { arr.push(s); }
            }
        }
        arr.insert(0, path.to_string());
        
        let ns_arr: Vec<Retained<NSString>> = arr.iter().map(|s| NSString::from_str(s)).collect();
        let final_arr = NSArray::from_vec(ns_arr);
        unsafe { defaults.setObject_forKey(Some(&final_arr), &key) };
    }
    
    pub fn add_to_recents(path: &str) {
        let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
        let key = NSString::from_str("RecentFiles");
        let existing = unsafe { defaults.arrayForKey(&key) };
        let mut arr = Vec::new();
        if let Some(existing) = existing {
            for i in 0..existing.count() {
                let s = unsafe { Retained::cast_unchecked::<NSString>(existing.objectAtIndex(i).clone()) }.to_string();
                if s != path { arr.push(s); }
            }
        }
        arr.insert(0, path.to_string());
        
        let ns_arr: Vec<Retained<NSString>> = arr.iter().map(|s| NSString::from_str(s)).collect();
        let final_arr = NSArray::from_vec(ns_arr);
        unsafe { defaults.setObject_forKey(Some(&final_arr), &key) };
    }
    
    pub fn remove_from_recents(path: &str) {
        let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
        let key = NSString::from_str("RecentFiles");
        let existing = unsafe { defaults.arrayForKey(&key) };
        let mut arr = Vec::new();
        if let Some(existing) = existing {
            for i in 0..existing.count() {
                let s = unsafe { Retained::cast_unchecked::<NSString>(existing.objectAtIndex(i).clone()) }.to_string();
                if s != path { arr.push(s); }
            }
        }
        
        let ns_arr: Vec<Retained<NSString>> = arr.iter().map(|s| NSString::from_str(s)).collect();
        let final_arr = NSArray::from_vec(ns_arr);
        unsafe { defaults.setObject_forKey(Some(&final_arr), &key) };
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
            SidebarItem::Item("Recent".to_string(), "clock".to_string(), "recent".to_string()),
            SidebarItem::Item("Favorites".to_string(), "star".to_string(), "favorites".to_string()),
            SidebarItem::Header("LOCATIONS".to_string()),
            SidebarItem::Item("Local Files".to_string(), "folder".to_string(), "local".to_string()),
            SidebarItem::Item("iCloud Drive".to_string(), "icloud".to_string(), "icloud".to_string()),
            SidebarItem::Header("WORKSPACES".to_string()),
        ];
        
        let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
        if let Some(workspaces) = unsafe { defaults.arrayForKey(&NSString::from_str("RecentWorkspaces")) } {
            for i in 0..workspaces.count() {
                let path = unsafe { Retained::cast_unchecked::<NSString>(workspaces.objectAtIndex(i).clone()) }.to_string();
                let name = PathBuf::from(&path).file_name().unwrap_or_default().to_string_lossy().to_string();
                sidebar_items.push(SidebarItem::Item(name, "folder".to_string(), path));
            }
        }
        
        *delegate.ivars().sidebar_items.borrow_mut() = sidebar_items;
        
        let left_vc = NSViewController::new(mtm);
        let left_scroll = NSScrollView::new(mtm);
        let left_table = NSTableView::new(mtm);
        
        unsafe {
            left_table.setIdentifier(&NSString::from_str("SidebarTable"));
            left_table.addTableColumn(&NSTableColumn::new(objc2_foundation::ns_string!("Col1")));
            left_table.setHeaderView(None);
            
            let ds = ProtocolObject::from_ref(&*delegate);
            left_table.setDataSource(Some(ds));
            left_table.setDelegate(Some(ds));
            
            left_scroll.setDocumentView(Some(&left_table));
            left_vc.setView(&left_scroll);
        }
        
        let left_item = unsafe { NSSplitViewItem::splitViewItemWithViewController(&left_vc) };
        
        let right_vc = NSViewController::new(mtm);
        let right_stack = NSStackView::new(mtm);
        unsafe {
            right_stack.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
            right_stack.setSpacing(0.0);
        }
        
        let search_field = NSSearchField::new(mtm);
        unsafe {
            let ds = ProtocolObject::from_ref(&*delegate);
            search_field.setDelegate(Some(ds));
            right_stack.addView_inGravity(&search_field, objc2_app_kit::NSStackViewGravity::Top);
        }
        
        let right_scroll = NSScrollView::new(mtm);
        let right_table = NSTableView::new(mtm);
        
        unsafe {
            right_table.setIdentifier(&NSString::from_str("MainTable"));
            right_table.addTableColumn(&NSTableColumn::new(objc2_foundation::ns_string!("Col1")));
            right_table.setHeaderView(None);
            right_table.setRowHeight(40.0);
            
            let ds = ProtocolObject::from_ref(&*delegate);
            right_table.setDataSource(Some(ds));
            right_table.setDelegate(Some(ds));
            
            
            let menu = NSMenu::new(mtm);
            let open_item = NSMenuItem::new(mtm);
            unsafe {
                open_item.setTitle(&NSString::from_str("Open"));
                menu.addItem(&open_item);
                
                let finder_item = NSMenuItem::new(mtm);
                finder_item.setTitle(&NSString::from_str("Show in Finder"));
                menu.addItem(&finder_item);
                
                let fav_item = NSMenuItem::new(mtm);
                fav_item.setTitle(&NSString::from_str("Add to Favorites"));
                menu.addItem(&fav_item);
                
                let rem_item = NSMenuItem::new(mtm);
                rem_item.setTitle(&NSString::from_str("Remove from Recents"));
                menu.addItem(&rem_item);
                
                right_table.setMenu(Some(&menu));
            }

            right_table.setTarget(Some(&*delegate));
            right_table.setDoubleAction(Some(sel!(mainTableDoubleClicked:)));
            
            right_scroll.setDocumentView(Some(&right_table));
            right_stack.addView_inGravity(&right_scroll, objc2_app_kit::NSStackViewGravity::Top);
            
            right_vc.setView(&right_stack);
        }
        
        *delegate.ivars().main_table.borrow_mut() = Some(Weak::retain(&right_table));
        *delegate.ivars().search_field.borrow_mut() = Some(Weak::retain(&search_field));
        
        let right_item = unsafe { NSSplitViewItem::splitViewItemWithViewController(&right_vc) };
        
        unsafe { split_vc.addSplitViewItem(&left_item) };
        unsafe { split_vc.addSplitViewItem(&right_item) };
        
        Self { vc: split_vc, delegate }
    }
    
    pub fn view_controller(&self) -> &NSSplitViewController {
        &self.vc
    }
}
