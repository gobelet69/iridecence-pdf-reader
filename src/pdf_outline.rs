use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{
    AnyThread, ClassType, DeclaredClass, MainThreadOnly, define_class, msg_send, msg_send_id, sel,
};
use objc2_app_kit::*;
use objc2_foundation::*;
use objc2_pdf_kit::{PDFOutline, PDFView};
use std::cell::RefCell;

#[derive(Default)]
pub struct PdfOutlineDelegateIvars {
    pdf_view: std::cell::OnceCell<Retained<PDFView>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "PdfOutlineDelegate"]
    #[ivars = PdfOutlineDelegateIvars]
    pub struct PdfOutlineDelegate;

    unsafe impl NSObjectProtocol for PdfOutlineDelegate {}
    unsafe impl NSControlTextEditingDelegate for PdfOutlineDelegate {}
    unsafe impl NSTextFieldDelegate for PdfOutlineDelegate {}

    unsafe impl NSOutlineViewDataSource for PdfOutlineDelegate {
        #[unsafe(method(outlineView:numberOfChildrenOfItem:))]
        fn outlineView_numberOfChildrenOfItem(
            &self,
            _outline_view: &NSOutlineView,
            item: Option<&AnyObject>,
        ) -> NSInteger {
            if let Some(item) = item {
                let outline: &PDFOutline =
                    unsafe { &*(&*item as *const AnyObject as *const PDFOutline) };
                unsafe { outline.numberOfChildren() as NSInteger }
            } else {
                if let Some(pdf_view) = self.ivars().pdf_view.get() {
                    if let Some(doc) = unsafe { pdf_view.document() } {
                        if let Some(root) = unsafe { doc.outlineRoot() } {
                            return unsafe { root.numberOfChildren() as NSInteger };
                        }
                    }
                }
                0
            }
        }

        #[unsafe(method(outlineView:child:ofItem:))]
        fn outlineView_child_ofItem(
            &self,
            _outline_view: &NSOutlineView,
            index: NSInteger,
            item: Option<&AnyObject>,
        ) -> *mut AnyObject {
            if let Some(item) = item {
                let outline: &PDFOutline =
                    unsafe { &*(&*item as *const AnyObject as *const PDFOutline) };
                if let Some(child) = unsafe { outline.childAtIndex(index as usize) } {
                    return Retained::into_raw(child) as *mut AnyObject;
                }
            } else {
                if let Some(pdf_view) = self.ivars().pdf_view.get() {
                    if let Some(doc) = unsafe { pdf_view.document() } {
                        if let Some(root) = unsafe { doc.outlineRoot() } {
                            if let Some(child) = unsafe { root.childAtIndex(index as usize) } {
                                return Retained::into_raw(child) as *mut AnyObject;
                            }
                        }
                    }
                }
            }
            std::ptr::null_mut()
        }

        #[unsafe(method(outlineView:isItemExpandable:))]
        fn outlineView_isItemExpandable(
            &self,
            outline_view: &NSOutlineView,
            item: &AnyObject,
        ) -> bool {
            let outline: &PDFOutline =
                unsafe { &*(&*item as *const AnyObject as *const PDFOutline) };
            unsafe { outline.numberOfChildren() > 0 }
        }
    }

    unsafe impl NSOutlineViewDelegate for PdfOutlineDelegate {
        #[unsafe(method(outlineView:viewForTableColumn:item:))]
        fn outlineView_viewForTableColumn_item(
            &self,
            outline_view: &NSOutlineView,
            _table_column: Option<&NSTableColumn>,
            item: &AnyObject,
        ) -> *mut NSView {
            let mtm = MainThreadMarker::new().unwrap();
            let identifier = ns_string!("OutlineCell");

            let mut view: Option<Retained<NSTableCellView>> = unsafe {
                let v = outline_view.makeViewWithIdentifier_owner(identifier, None);
                v.map(|v| Retained::cast_unchecked::<NSTableCellView>(v))
            };

            if view.is_none() {
                let cell = unsafe {
                    NSTableCellView::initWithFrame(
                        NSTableCellView::alloc(mtm),
                        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 24.0)),
                    )
                };
                unsafe { cell.setIdentifier(Some(identifier)) };

                let text_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                unsafe {
                    text_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    text_field.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
                    cell.addSubview(&text_field);
                    cell.setTextField(Some(&text_field));

                    text_field
                        .centerYAnchor()
                        .constraintEqualToAnchor(&cell.centerYAnchor())
                        .setActive(true);
                    text_field
                        .leadingAnchor()
                        .constraintEqualToAnchor_constant(&cell.leadingAnchor(), 2.0)
                        .setActive(true);
                    text_field
                        .trailingAnchor()
                        .constraintEqualToAnchor_constant(&cell.trailingAnchor(), -2.0)
                        .setActive(true);
                }
                view = Some(cell);
            }

            let outline: &PDFOutline =
                unsafe { &*(&*item as *const AnyObject as *const PDFOutline) };
            if let Some(label) = unsafe { outline.label() } {
                if let Some(cell) = &view {
                    if let Some(tf) = unsafe { cell.textField() } {
                        unsafe { tf.setStringValue(&label) };
                    }
                }
            }

            match view {
                Some(v) => Retained::into_raw(v) as *mut NSView,
                None => std::ptr::null_mut(),
            }
        }

        #[unsafe(method(outlineViewSelectionDidChange:))]
        fn outlineViewSelectionDidChange(&self, notification: &NSNotification) {
            if let Some(obj) = unsafe { notification.object() } {
                if let Some(outline_view) = obj.downcast_ref::<NSOutlineView>() {
                    let row = unsafe { outline_view.selectedRow() };
                    if row >= 0 {
                        if let Some(item) = unsafe { outline_view.itemAtRow(row) } {
                            let outline: &PDFOutline =
                                unsafe { &*(&*item as *const AnyObject as *const PDFOutline) };
                            if let Some(dest) = unsafe { outline.destination() } {
                                if let Some(pdf_view) = self.ivars().pdf_view.get() {
                                    unsafe { pdf_view.goToDestination(&dest) };
                                }
                            } else if let Some(action) = unsafe { outline.action() } {
                                if let Some(pdf_view) = self.ivars().pdf_view.get() {
                                    unsafe { pdf_view.performAction(&action) };
                                }
                            }
                        }
                    }
                }
            }
        }
    }
);

impl PdfOutlineDelegate {
    pub fn new(mtm: MainThreadMarker, pdf_view: Retained<PDFView>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(PdfOutlineDelegateIvars::default());
        let this: Retained<Self> = unsafe { msg_send_id![super(this), init] };
        this.ivars().pdf_view.set(pdf_view).unwrap();
        this
    }
}
