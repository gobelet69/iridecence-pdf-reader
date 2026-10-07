use objc2::rc::Retained;
use objc2::{DeclaredClass, define_class, msg_send, msg_send_id, ClassType, sel, MainThreadOnly};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::{NSControlTextEditingDelegate, 
    NSPanel, NSWindowStyleMask, NSTextField, NSTextFieldDelegate, NSOutlineView, 
    NSOutlineViewDataSource, NSOutlineViewDelegate, NSScrollView, NSTableCellView,
    NSView, NSTableColumn, NSLineBreakMode, NSColor, NSBackingStoreType, NSWindowLevel, NSWindowCollectionBehavior
};
use objc2_foundation::{MainThreadMarker, ns_string, NSString, NSObject, NSObjectProtocol, NSPoint, NSSize, NSRect, NSInteger, NSNotification};
use std::cell::RefCell;

#[derive(Default)]
pub struct PaletteIvars {
    pub panel: std::cell::OnceCell<Retained<NSPanel>>,
    pub text_field: std::cell::OnceCell<Retained<NSTextField>>,
    pub table_view: std::cell::OnceCell<Retained<NSOutlineView>>,
    pub commands: RefCell<Vec<String>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "CommandPaletteDelegate"]
    #[ivars = PaletteIvars]
    pub struct CommandPaletteDelegate;

    unsafe impl NSObjectProtocol for CommandPaletteDelegate {}



    unsafe impl NSOutlineViewDataSource for CommandPaletteDelegate {
        #[unsafe(method(outlineView:numberOfChildrenOfItem:))]
        fn outlineView_numberOfChildrenOfItem(&self, _outline_view: &NSOutlineView, item: Option<&AnyObject>) -> NSInteger {
            if item.is_some() { return 0; }
            self.ivars().commands.borrow().len() as NSInteger
        }

        #[unsafe(method(outlineView:child:ofItem:))]
        fn outlineView_child_ofItem(&self, _outline_view: &NSOutlineView, index: NSInteger, item: Option<&AnyObject>) -> *mut AnyObject {
            if item.is_none() {
                if let Some(cmd) = self.ivars().commands.borrow().get(index as usize) {
                    return Retained::into_raw(NSString::from_str(cmd)) as *mut AnyObject;
                }
            }
            std::ptr::null_mut()
        }

        #[unsafe(method(outlineView:isItemExpandable:))]
        fn outlineView_isItemExpandable(&self, _outline_view: &NSOutlineView, _item: &AnyObject) -> bool {
            false
        }
    }

    unsafe impl NSOutlineViewDelegate for CommandPaletteDelegate {
        #[unsafe(method(outlineView:viewForTableColumn:item:))]
        fn outlineView_viewForTableColumn_item(
            &self,
            outline_view: &NSOutlineView,
            _table_column: Option<&NSTableColumn>,
            item: &AnyObject,
        ) -> *mut NSView {
            let mtm = MainThreadMarker::new().unwrap();
            let identifier = ns_string!("PaletteCell");
            
            let mut view: Option<Retained<NSTableCellView>> = unsafe {
                let v = outline_view.makeViewWithIdentifier_owner(identifier, None);
                v.map(|v| Retained::cast_unchecked::<NSTableCellView>(v))
            };

            if view.is_none() {
                let cell = unsafe { NSTableCellView::initWithFrame(NSTableCellView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 24.0))) };
                // removed
                let text_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                unsafe {
                    text_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    cell.addSubview(&text_field);
                    cell.setTextField(Some(&text_field));
                    text_field.centerYAnchor().constraintEqualToAnchor(&cell.centerYAnchor()).setActive(true);
                    text_field.leadingAnchor().constraintEqualToAnchor_constant(&cell.leadingAnchor(), 8.0).setActive(true);
                    text_field.trailingAnchor().constraintEqualToAnchor_constant(&cell.trailingAnchor(), -8.0).setActive(true);
                }
                view = Some(cell);
            }

            let str_val = unsafe { Retained::cast_unchecked::<NSString>(Retained::retain(item as *const AnyObject as *mut AnyObject).unwrap()) };
            
            if let Some(cell) = &view {
                if let Some(tf) = unsafe { cell.textField() } {
                    unsafe { tf.setStringValue(&str_val) };
                }
            }
            
            match view {
                Some(v) => Retained::into_raw(v) as *mut NSView,
                None => std::ptr::null_mut(),
            }
        }
    }

    unsafe impl NSControlTextEditingDelegate for CommandPaletteDelegate {
        #[unsafe(method(controlTextDidChange:))]
        fn controlTextDidChange(&self, _notification: &NSNotification) {
            if let Some(tf) = self.ivars().text_field.get() {
                let s = unsafe { tf.stringValue().to_string() };
                self.update_search(&s);
            }
        }
    }
);

impl CommandPaletteDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(PaletteIvars::default());
        let this: Retained<Self> = unsafe { msg_send_id![super(this), init] };
        this
    }
    
    pub fn setup(&self, panel: Retained<NSPanel>, text_field: Retained<NSTextField>, table_view: Retained<NSOutlineView>) {
        self.ivars().panel.set(panel).unwrap();
        self.ivars().text_field.set(text_field).unwrap();
        self.ivars().table_view.set(table_view).unwrap();
    }
    
    pub fn update_search(&self, query: &str) {
        let mut cmds = Vec::new();
        if query.starts_with('>') {
            cmds.push("Fit Width".to_string());
            cmds.push("Fit Page".to_string());
            cmds.push("Actual Size".to_string());
        } else if query.starts_with('/') {
            cmds.push(format!("Search for '{}'", &query[1..]));
        } else if query.starts_with("p:") || query.starts_with(':') {
            cmds.push(format!("Go to page {}", query.replace("p:", "").replace(":", "")));
        } else {
            cmds.push(format!("Search PDFs: {}", query));
        }
        *self.ivars().commands.borrow_mut() = cmds;
        if let Some(tv) = self.ivars().table_view.get() {
            unsafe { tv.reloadData() };
        }
    }
}

pub fn create_palette(mtm: MainThreadMarker) -> (Retained<NSPanel>, Retained<CommandPaletteDelegate>) {
    let panel = unsafe {
        NSPanel::initWithContentRect_styleMask_backing_defer(
            NSPanel::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(600.0, 400.0)),
            NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel | NSWindowStyleMask::Resizable,
            NSBackingStoreType::Buffered,
            false
        )
    };
    
    unsafe {
        panel.setHasShadow(true);
        panel.setLevel(objc2_app_kit::NSFloatingWindowLevel as isize);
        panel.setCollectionBehavior(NSWindowCollectionBehavior::Transient | NSWindowCollectionBehavior::IgnoresCycle);
        panel.setBackgroundColor(Some(&NSColor::windowBackgroundColor()));
        panel.setOpaque(false);
        panel.center();
    }

    let delegate = CommandPaletteDelegate::new(mtm);

    let text_field = unsafe { NSTextField::textFieldWithString(ns_string!(""), mtm) };
    unsafe {
        text_field.setFocusRingType(objc2_app_kit::NSFocusRingType::None);
        text_field.setBezeled(false);
        text_field.setDrawsBackground(false);
        let font = objc2_app_kit::NSFont::systemFontOfSize(24.0);
        let _: () = msg_send![&text_field, setFont: &*font];
        let del_ptr: *const CommandPaletteDelegate = &*delegate;
        let _: () = msg_send![&text_field, setDelegate: del_ptr];
        text_field.setTranslatesAutoresizingMaskIntoConstraints(false);
    }

    let scroll = unsafe { NSScrollView::initWithFrame(NSScrollView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(600.0, 340.0))) };
    let table_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(600.0, 340.0))) };
    let col = unsafe { NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), ns_string!("Col")) };
    unsafe {
        col.setWidth(600.0);
        table_view.addTableColumn(&col);
        table_view.setOutlineTableColumn(Some(&col));
        table_view.setHeaderView(None);
        table_view.setRowHeight(32.0);
        
        let del_ptr: *const CommandPaletteDelegate = &*delegate;
        let _: () = msg_send![&table_view, setDelegate: del_ptr];
        let _: () = msg_send![&table_view, setDataSource: del_ptr];
        
        scroll.setDocumentView(Some(&table_view));
        scroll.setDrawsBackground(false);
        scroll.setTranslatesAutoresizingMaskIntoConstraints(false);
    }
    
    let container = unsafe { NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(600.0, 400.0))) };
    unsafe {
        container.addSubview(&text_field);
        container.addSubview(&scroll);
        
        text_field.topAnchor().constraintEqualToAnchor_constant(&container.topAnchor(), 16.0).setActive(true);
        text_field.leadingAnchor().constraintEqualToAnchor_constant(&container.leadingAnchor(), 16.0).setActive(true);
        text_field.trailingAnchor().constraintEqualToAnchor_constant(&container.trailingAnchor(), -16.0).setActive(true);
        text_field.heightAnchor().constraintEqualToConstant(32.0).setActive(true);
        
        scroll.topAnchor().constraintEqualToAnchor_constant(&text_field.bottomAnchor(), 16.0).setActive(true);
        scroll.leadingAnchor().constraintEqualToAnchor(&container.leadingAnchor()).setActive(true);
        scroll.trailingAnchor().constraintEqualToAnchor(&container.trailingAnchor()).setActive(true);
        scroll.bottomAnchor().constraintEqualToAnchor(&container.bottomAnchor()).setActive(true);
        
        panel.setContentView(Some(&container));
    }
    
    delegate.setup(panel.clone(), text_field.clone(), table_view.clone());
    
    (panel, delegate)
}
