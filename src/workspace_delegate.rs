use objc2_app_kit::{NSOutlineView, NSTableColumn, NSTableCellView, NSTextField, NSView, NSControl, NSUserInterfaceItemIdentification, NSResponder};
use objc2::runtime::AnyObject;
use crate::workspace::FileNode;
use std::cell::RefCell;
use objc2::rc::Weak;
use objc2::rc::Retained;
use objc2_foundation::{NSObject, NSObjectProtocol, NSInteger, NSRect, NSPoint, NSSize, MainThreadMarker, NSString};
use objc2::{define_class, MainThreadOnly, DefinedClass, msg_send};

#[derive(Default)]
pub struct WorkspaceDelegateIvars {
    pub root_node: RefCell<Option<Retained<FileNode>>>,
    pub outline_view: RefCell<Option<Weak<NSOutlineView>>>,
    pub folder_image: RefCell<Option<Retained<objc2_app_kit::NSImage>>>,
    pub doc_image: RefCell<Option<Retained<objc2_app_kit::NSImage>>>,
}

define_class!(
    #[unsafe(super(NSResponder))]
    #[thread_kind = MainThreadOnly]
    #[name = "WorkspaceDelegate"]
    #[ivars = WorkspaceDelegateIvars]
    pub struct WorkspaceDelegate;

    unsafe impl NSObjectProtocol for WorkspaceDelegate {}

    impl WorkspaceDelegate {



    #[unsafe(method(nodeDidLoadChildren:))]
    fn nodeDidLoadChildren(&self, notification: &objc2_foundation::NSNotification) {
        if let Some(weak_view) = self.ivars().outline_view.borrow().as_ref() {
            if let Some(outline_view) = weak_view.load() {
                let node: Option<objc2::rc::Retained<crate::workspace::FileNode>> = unsafe { objc2::msg_send_id![notification, object] };
                let node = node.unwrap();
                let mut is_root = false;
                if let Some(root) = self.ivars().root_node.borrow().as_ref() {
                    if std::ptr::eq(&**root, &*node) {
                        is_root = true;
                    }
                }
                
                if is_root {
                    unsafe { outline_view.reloadItem_reloadChildren(None, true) };
                } else {
                    let node_any = unsafe { objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(node) };
                    unsafe { outline_view.reloadItem_reloadChildren(Some(&node_any), true) };
                    unsafe { outline_view.expandItem(Some(&node_any)) }; // Keep expanded if it was expanded
                }
            }
        }
    }

    #[unsafe(method(outlineViewSelectionDidChange:))]
    fn outlineViewSelectionDidChange(&self, notification: &objc2_foundation::NSNotification) {
        let outline_view: Option<objc2::rc::Retained<NSOutlineView>> = unsafe { objc2::msg_send_id![notification, object] };
        let outline_view = outline_view.unwrap();
        let selected_row = unsafe { outline_view.selectedRow() };
        if selected_row >= 0 {
            if let Some(item) = unsafe { outline_view.itemAtRow(selected_row) } {
                let node: &crate::workspace::FileNode = unsafe { &*(&*item as *const objc2::runtime::AnyObject as *const crate::workspace::FileNode) };
                if node.ivars().is_loading_placeholder.get() {
                    return;
                }
            }
        }
        
        let target: Option<objc2::rc::Retained<objc2::runtime::AnyObject>> = unsafe { objc2::msg_send_id![&outline_view, target] };
        if let Some(target) = target {
            let _: () = unsafe { objc2::msg_send![&target, workspaceSelectionChanged: &*outline_view] };
        }
    }

        #[unsafe(method(outlineView:numberOfChildrenOfItem:))]
        fn outlineView_numberOfChildrenOfItem(&self, _outline_view: &NSOutlineView, item: Option<&AnyObject>) -> NSInteger {
            let root_borrow = self.ivars().root_node.borrow();
            let Some(root) = root_borrow.as_ref() else { return 0; };
            
            let node: &FileNode = match item {
                Some(it) => {
                    unsafe { &*(it as *const AnyObject as *const FileNode) }
                },
                None => {
                    &**root
                },
            };
            
            node.load_children();
            if let Some(children) = node.ivars().children.borrow().as_ref() {
                children.len() as NSInteger
            } else {
                0
            }
        }

        #[unsafe(method(outlineView:child:ofItem:))]
        fn outlineView_child_ofItem(&self, _outline_view: &NSOutlineView, index: NSInteger, item: Option<&AnyObject>) -> *mut AnyObject {
            let root_borrow = self.ivars().root_node.borrow();
            let Some(root) = root_borrow.as_ref() else { return std::ptr::null_mut(); };
            let node: &FileNode = match item {
                Some(it) => unsafe { &*(it as *const AnyObject as *const FileNode) },
                None => &**root,
            };
            
            let children = node.ivars().children.borrow();
            let child = &children.as_ref().unwrap()[index as usize];
            let retained = unsafe { Retained::cast_unchecked::<AnyObject>(child.clone()) };
            // Auto-release the pointer to return +0 to Objective-C
            Retained::autorelease_ptr(retained)
        }

        #[unsafe(method(outlineView:isItemExpandable:))]
        fn outlineView_isItemExpandable(&self, _outline_view: &NSOutlineView, item: &AnyObject) -> bool {
            let node: &FileNode = unsafe { &*(item as *const AnyObject as *const FileNode) };
            node.ivars().is_dir.get()
        }

        #[unsafe(method(outlineView:viewForTableColumn:item:))]
        fn outlineView_viewForTableColumn_item(&self, outline_view: &NSOutlineView, _table_column: Option<&NSTableColumn>, item: &AnyObject) -> *mut NSView {
            let node: &FileNode = unsafe { &*(item as *const AnyObject as *const FileNode) };
            let mtm = MainThreadMarker::new().unwrap();
            
            let id = objc2_foundation::ns_string!("FileCell");
            let mut view = unsafe { outline_view.makeViewWithIdentifier_owner(id, None) };
            if view.is_none() {
                let cell = unsafe { NSTableCellView::initWithFrame(NSTableCellView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 24.0))) };
                unsafe { cell.setIdentifier(Some(id)) };
                
                let image_view = unsafe { objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(2.0, 4.0), NSSize::new(16.0, 16.0))) };
                let text_field = unsafe { NSTextField::initWithFrame(NSTextField::alloc(mtm), NSRect::new(NSPoint::new(24.0, 3.0), NSSize::new(156.0, 18.0))) };
                unsafe {
                    text_field.setBezeled(false);
                    text_field.setDrawsBackground(false);
                    text_field.setEditable(false);
                    text_field.setSelectable(false);
                    
                    let font = objc2_app_kit::NSFont::systemFontOfSize(13.0);
                    text_field.setFont(Some(&font));
                    text_field.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingMiddle);
                    text_field.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable);
                    
                    cell.addSubview(&image_view);
                    cell.addSubview(&text_field);
                    cell.setImageView(Some(&image_view));
                    cell.setTextField(Some(&text_field));
                    
                    let cloud_iv = unsafe { objc2_app_kit::NSButton::initWithFrame(objc2_app_kit::NSButton::alloc(mtm), NSRect::new(NSPoint::new(180.0, 4.0), NSSize::new(16.0, 16.0))) };
                    unsafe {
                        cloud_iv.setIdentifier(Some(objc2_foundation::ns_string!("CloudIcon")));
                        cloud_iv.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin);
                        cloud_iv.setHidden(true);
                        cloud_iv.setBezelStyle(objc2_app_kit::NSBezelStyle::Inline);
                        cloud_iv.setBordered(false);
                        cloud_iv.setTitle(objc2_foundation::ns_string!(""));
                        cloud_iv.setImagePosition(objc2_app_kit::NSCellImagePosition::ImageOnly);
                        
                        let target: &objc2::runtime::AnyObject = &*(self as *const WorkspaceDelegate as *const objc2::runtime::AnyObject);
                        cloud_iv.setTarget(Some(target));
                        cloud_iv.setAction(Some(objc2::sel!(downloadItem:)));
                    }
                    
                    let progress = objc2_app_kit::NSProgressIndicator::initWithFrame(objc2_app_kit::NSProgressIndicator::alloc(mtm), NSRect::new(NSPoint::new(180.0, 4.0), NSSize::new(16.0, 16.0)));
                    unsafe { progress.setIdentifier(Some(objc2_foundation::ns_string!("DownloadProgress"))) };
                    progress.setHidden(true);
                    progress.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin);
                    progress.setStyle(objc2_app_kit::NSProgressIndicatorStyle::Spinning);
                    progress.setControlSize(objc2_app_kit::NSControlSize::Small);
                    
                    cell.addSubview(&cloud_iv);
                    cell.addSubview(&progress);
                }
                view = Some(unsafe { Retained::cast_unchecked::<NSView>(cell) });
            }
            
            let cell = view.unwrap();
            let table_cell = unsafe { Retained::cast_unchecked::<NSTableCellView>(cell.clone()) };
            
            let is_dir = node.ivars().is_dir.get();
            let symbol_name = if is_dir { objc2_foundation::ns_string!("folder.fill") } else { objc2_foundation::ns_string!("doc.text.fill") };
            
            if let Some(iv) = unsafe { table_cell.imageView() } {
                if node.ivars().is_loading_placeholder.get() {
                    unsafe { iv.setImage(None) }; // No icon for loading placeholder
                } else if is_dir {
                    unsafe { iv.setContentTintColor(Some(&objc2_app_kit::NSColor::systemBlueColor())) };
                    if let Some(img) = self.ivars().folder_image.borrow().as_ref() {
                        unsafe { iv.setImage(Some(img)) };
                    }
                } else {
                    unsafe { iv.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor())) };
                    if let Some(img) = self.ivars().doc_image.borrow().as_ref() {
                        unsafe { iv.setImage(Some(img)) };
                    }
                }
            }
            
            let icloud_status = node.ivars().icloud_status.get();
            let download_progress = node.ivars().download_progress.get();
            
            unsafe {
                let subviews = table_cell.subviews();
                let mut cloud_iv: Option<Retained<objc2_app_kit::NSImageView>> = None;
                let mut progress: Option<Retained<objc2_app_kit::NSProgressIndicator>> = None;
                
                for i in 0..subviews.count() {
                    let v = subviews.objectAtIndex(i);
                    if let Some(ident) = v.identifier() {
                        if ident.isEqualToString(objc2_foundation::ns_string!("CloudIcon")) {
                            cloud_iv = Some(Retained::cast_unchecked::<objc2_app_kit::NSImageView>(v.clone()));
                        } else if ident.isEqualToString(objc2_foundation::ns_string!("DownloadProgress")) {
                            progress = Some(Retained::cast_unchecked::<objc2_app_kit::NSProgressIndicator>(v.clone()));
                        }
                    }
                }
                
                
                let diag_state = if icloud_status == 0 { "Local" } else if icloud_status == 1 { "CloudOnly" } else if icloud_status == 2 { "Downloading" } else if icloud_status == 4 { "Failed" } else { "Unknown" };
                let diag_icon = if icloud_status == 0 { "NONE" } else if icloud_status == 1 { "Cloud" } else if icloud_status == 2 { "Spinner" } else if icloud_status == 4 { "Failed" } else { "NONE" };
                if icloud_status == 0 {
                    println!("[DIAGNOSTIC] RENDER: state = {}, icon = {}", diag_state, diag_icon);
                }

                if let Some(civ) = cloud_iv {
                    if icloud_status == 1 || icloud_status == 4 {
                        civ.setHidden(false);
                        
                        if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(if icloud_status == 1 { objc2_foundation::ns_string!("icloud.and.arrow.down") } else { objc2_foundation::ns_string!("exclamationmark.icloud") }, None) {
                            civ.setImage(Some(&img));
                            if icloud_status == 1 {
                                civ.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
                            } else {
                                civ.setContentTintColor(Some(&objc2_app_kit::NSColor::systemRedColor()));
                            }
                        }
                    } else {
                        civ.setHidden(true);
                    }
                }
                
                if let Some(pi) = progress {
                    if icloud_status == 2 {
                        pi.setHidden(false);
                        pi.startAnimation(None);
                        if download_progress > 0.0 {
                            pi.setIndeterminate(false);
                            pi.setDoubleValue(download_progress);
                        } else {
                            pi.setIndeterminate(true);
                        }
                    } else {
                        pi.setHidden(true);
                        pi.stopAnimation(None);
                    }
                }
            }
            
            if let Some(tf) = unsafe { table_cell.textField() } {
                let name_str = &node.ivars().name.borrow();
                let path_str = &node.ivars().path.borrow();
                unsafe { tf.setStringValue(&NSString::from_str(name_str)) };
                unsafe { table_cell.setToolTip(Some(&NSString::from_str(path_str))) };
            }
            Retained::autorelease_ptr(cell)
        }
            }
);

impl WorkspaceDelegate {

    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(WorkspaceDelegateIvars::default());
        let this: Retained<Self> = unsafe { objc2::msg_send_id![super(this), init] };
        
        let ws = unsafe { objc2_app_kit::NSWorkspace::sharedWorkspace() };
        let folder_img = unsafe { ws.iconForFileType(objc2_foundation::ns_string!("public.folder")) };
        let doc_img = unsafe { ws.iconForFileType(objc2_foundation::ns_string!("pdf")) };
        
        *this.ivars().folder_image.borrow_mut() = Some(folder_img);
        *this.ivars().doc_image.borrow_mut() = Some(doc_img);
        this
    }


    pub fn find_node_by_path(&self, target_path: &str) -> Option<objc2::rc::Retained<crate::workspace::FileNode>> {
        let root_borrow = self.ivars().root_node.borrow();
        if let Some(root) = root_borrow.as_ref() {
            return Self::search_node(root, target_path);
        }
        None
    }

    fn search_node(node: &crate::workspace::FileNode, target_path: &str) -> Option<objc2::rc::Retained<crate::workspace::FileNode>> {
        if node.ivars().path.borrow().as_str() == target_path {
            return Some(unsafe { objc2::rc::Retained::retain(node as *const _ as *mut _) }.unwrap());
        }
        if let Some(children) = node.ivars().children.borrow().as_ref() {
            for child in children {
                if let Some(found) = Self::search_node(child, target_path) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn set_outline_view(&self, view: &objc2::rc::Retained<NSOutlineView>) {
        *self.ivars().outline_view.borrow_mut() = Some(objc2::rc::Weak::from(view));
        unsafe {
            let nc = objc2_foundation::NSNotificationCenter::defaultCenter();
            let name = objc2_foundation::ns_string!("FileNodeDidLoadChildren");
            nc.addObserver_selector_name_object(self, objc2::sel!(nodeDidLoadChildren:), Some(name), None);
        }
    }

}
