use std::cell::{RefCell, Cell};
use std::path::{Path, PathBuf};
use objc2::{define_class, msg_send_id, ClassType, DefinedClass, MainThreadOnly};
use objc2::rc::Retained;
use objc2_foundation::{NSObject, NSObjectProtocol, NSInteger, NSRect, NSPoint, NSSize, MainThreadMarker};
use objc2_app_kit::{NSOutlineView, NSOutlineViewDataSource, NSOutlineViewDelegate, NSTableColumn, NSTableCellView, NSTextField, NSView, NSControl, NSUserInterfaceItemIdentification};
use objc2::runtime::AnyObject;
use crate::workspace::FileNode;

#[derive(Default)]
pub struct WorkspaceDelegateIvars {
    pub root_node: RefCell<Option<Retained<FileNode>>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "WorkspaceDelegate"]
    #[ivars = WorkspaceDelegateIvars]
    pub struct WorkspaceDelegate;

    unsafe impl NSOutlineViewDataSource for WorkspaceDelegate {
        #[unsafe(method(outlineView:numberOfChildrenOfItem:))]
        fn outlineView_numberOfChildrenOfItem(&self, _outline_view: &NSOutlineView, item: Option<&AnyObject>) -> NSInteger {
            let root = self.ivars().root_node.borrow();
            let Some(root) = root.as_ref() else { return 0; };
            
            let node: &FileNode = match item {
                Some(it) => unsafe { &*(it as *const AnyObject as *const FileNode) },
                None => root.as_ref(),
            };
            
            node.load_children();
            if let Some(children) = node.ivars().children.borrow().as_ref() {
                children.len() as NSInteger
            } else {
                0
            }
        }

        #[unsafe(method(outlineView:child:ofItem:))]
        fn outlineView_child_ofItem(&self, _outline_view: &NSOutlineView, index: NSInteger, item: Option<&AnyObject>) -> Retained<AnyObject> {
            let root = self.ivars().root_node.borrow();
            let node: &FileNode = match item {
                Some(it) => unsafe { &*(it as *const AnyObject as *const FileNode) },
                None => root.as_ref().unwrap().as_ref(),
            };
            
            let children = node.ivars().children.borrow();
            let child = &children.as_ref().unwrap()[index as usize];
            unsafe { Retained::cast_unchecked::<AnyObject>(child.clone()) }
        }

        #[unsafe(method(outlineView:isItemExpandable:))]
        fn outlineView_isItemExpandable(&self, _outline_view: &NSOutlineView, item: &AnyObject) -> bool {
            let node: &FileNode = unsafe { &*(item as *const AnyObject as *const FileNode) };
            node.ivars().is_dir.get()
        }
    }

    unsafe impl NSOutlineViewDelegate for WorkspaceDelegate {
        #[unsafe(method(outlineView:viewForTableColumn:item:))]
        fn outlineView_viewForTableColumn_item(&self, outline_view: &NSOutlineView, _table_column: Option<&NSTableColumn>, item: &AnyObject) -> Option<Retained<NSView>> {
            let node: &FileNode = unsafe { &*(item as *const AnyObject as *const FileNode) };
            let mtm = MainThreadMarker::new().unwrap();
            
            let id = objc2_foundation::ns_string!("FileCell");
            let mut view = unsafe { outline_view.makeViewWithIdentifier_owner(id, None) };
            if view.is_none() {
                let cell = unsafe { NSTableCellView::initWithFrame(NSTableCellView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 24.0))) };
                unsafe { cell.setIdentifier(id) };
                
                let text_field = unsafe { NSTextField::initWithFrame(NSTextField::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 24.0))) };
                unsafe {
                    text_field.setBezeled(false);
                    text_field.setDrawsBackground(false);
                    text_field.setEditable(false);
                    text_field.setSelectable(false);
                    cell.addSubview(&text_field);
                    cell.setTextField(Some(&text_field));
                }
                view = Some(unsafe { Retained::cast_unchecked::<NSView>(cell) });
            }
            
            let cell = view.unwrap();
            let table_cell = unsafe { Retained::cast_unchecked::<NSTableCellView>(cell.clone()) };
            if let Some(tf) = unsafe { table_cell.textField() } {
                unsafe { tf.setStringValue(&objc2_foundation::NSString::from_str(&node.ivars().name.borrow())) };
            }
            Some(cell)
        }
    }
);
