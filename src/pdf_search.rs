use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{DeclaredClass, MainThreadOnly, define_class, msg_send, msg_send_id};
use objc2_app_kit::*;
use objc2_foundation::*;
use objc2_pdf_kit::{PDFDocumentDelegate, PDFSelection, PDFView};
use std::cell::RefCell;

#[derive(Default)]
pub struct PdfSearchDelegateIvars {
    pub pdf_view: std::cell::OnceCell<Retained<PDFView>>,
    pub outline_view: std::cell::OnceCell<Retained<NSOutlineView>>,
    pub results: RefCell<Vec<Retained<PDFSelection>>>,
    pub is_searching: std::cell::Cell<bool>,
    pub search_field: std::cell::OnceCell<Retained<NSTextField>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "PdfSearchDelegate"]
    #[ivars = PdfSearchDelegateIvars]
    pub struct PdfSearchDelegate;

    unsafe impl NSObjectProtocol for PdfSearchDelegate {}
    unsafe impl NSControlTextEditingDelegate for PdfSearchDelegate {
        #[unsafe(method(controlTextDidChange:))]
        fn controlTextDidChange(&self, _notification: &NSNotification) {
            if let Some(tf) = self.ivars().search_field.get() {
                let query = unsafe { tf.stringValue() };
                if query.length() > 0 {
                    self.start_search(&query);
                } else {
                    self.cancel_search();
                    self.ivars().results.borrow_mut().clear();
                    if let Some(outline_view) = self.ivars().outline_view.get() {
                        unsafe { outline_view.reloadData() };
                    }
                }
            }
        }

        #[unsafe(method(control:textView:doCommandBySelector:))]
        fn control_textView_doCommandBySelector(
            &self,
            _control: &NSControl,
            _text_view: &NSTextView,
            command_selector: objc2::runtime::Sel,
        ) -> objc2::runtime::Bool {
            let sel_name = command_selector.name();
            if sel_name.to_bytes() == b"cancelOperation:" {
                if let Some(tf) = self.ivars().search_field.get() {
                    unsafe { tf.setStringValue(&NSString::from_str("")) };
                }
                self.cancel_search();
                self.ivars().results.borrow_mut().clear();
                if let Some(outline_view) = self.ivars().outline_view.get() {
                    unsafe { outline_view.reloadData() };
                }

                if let Some(app) = unsafe {
                    NSApplication::sharedApplication(MainThreadMarker::new().unwrap()).delegate()
                } {
                    let _: () = unsafe { msg_send![&app, recordHistory] };
                }
                if let Some(pdf_view) = self.ivars().pdf_view.get() {
                    if let Some(window) = unsafe { pdf_view.window() } {
                        unsafe { window.makeFirstResponder(Some(pdf_view.as_ref())) };
                    }
                }
                return true.into();
            } else if sel_name.to_bytes() == b"insertNewline:" {
                if let Some(outline_view) = self.ivars().outline_view.get() {
                    let results = self.ivars().results.borrow();
                    if !results.is_empty() {
                        let mut current_row = unsafe { outline_view.selectedRow() };
                        if current_row < 0 || current_row >= (results.len() as isize - 1) {
                            current_row = 0;
                        } else {
                            current_row += 1;
                        }
                        let index_set =
                            unsafe { NSIndexSet::indexSetWithIndex(current_row as usize) };
                        unsafe {
                            outline_view.selectRowIndexes_byExtendingSelection(&index_set, false)
                        };
                        unsafe { outline_view.scrollRowToVisible(current_row) };
                    }
                }
                return true.into();
            }
            false.into()
        }
    }

    unsafe impl PDFDocumentDelegate for PdfSearchDelegate {
        #[unsafe(method(didMatchString:))]
        fn didMatchString(&self, selection: &PDFSelection) {
            let sel_retained =
                unsafe { Retained::retain(selection as *const _ as *mut _) }.unwrap();
            self.ivars().results.borrow_mut().push(sel_retained);
            if let Some(outline_view) = self.ivars().outline_view.get() {
                unsafe { outline_view.reloadData() };
            }
        }

        #[unsafe(method(documentDidBeginDocumentFind:))]
        fn documentDidBeginDocumentFind(&self, _notification: &NSNotification) {
            self.ivars().is_searching.set(true);
            self.ivars().results.borrow_mut().clear();
            if let Some(outline_view) = self.ivars().outline_view.get() {
                unsafe { outline_view.reloadData() };
            }
        }

        #[unsafe(method(documentDidEndDocumentFind:))]
        fn documentDidEndDocumentFind(&self, _notification: &NSNotification) {
            self.ivars().is_searching.set(false);
        }
    }

    unsafe impl NSOutlineViewDataSource for PdfSearchDelegate {
        #[unsafe(method(outlineView:numberOfChildrenOfItem:))]
        fn outlineView_numberOfChildrenOfItem(
            &self,
            _outline_view: &NSOutlineView,
            item: Option<&AnyObject>,
        ) -> NSInteger {
            if item.is_some() {
                return 0;
            }
            self.ivars().results.borrow().len() as NSInteger
        }

        #[unsafe(method(outlineView:child:ofItem:))]
        fn outlineView_child_ofItem(
            &self,
            _outline_view: &NSOutlineView,
            index: NSInteger,
            item: Option<&AnyObject>,
        ) -> *mut AnyObject {
            if item.is_none() {
                if let Some(sel) = self.ivars().results.borrow().get(index as usize) {
                    return Retained::as_ptr(sel) as *mut AnyObject;
                }
            }
            std::ptr::null_mut()
        }

        #[unsafe(method(outlineView:isItemExpandable:))]
        fn outlineView_isItemExpandable(
            &self,
            _outline_view: &NSOutlineView,
            _item: &AnyObject,
        ) -> bool {
            false
        }
    }

    unsafe impl NSOutlineViewDelegate for PdfSearchDelegate {
        #[unsafe(method(outlineView:viewForTableColumn:item:))]
        fn outlineView_viewForTableColumn_item(
            &self,
            outline_view: &NSOutlineView,
            _table_column: Option<&NSTableColumn>,
            item: &AnyObject,
        ) -> *mut NSView {
            let mtm = MainThreadMarker::new().unwrap();
            let identifier = ns_string!("SearchCell");

            let mut view: Option<Retained<NSTableCellView>> = unsafe {
                let v = outline_view.makeViewWithIdentifier_owner(identifier, None);
                v.map(|v| Retained::cast_unchecked::<NSTableCellView>(v))
            };

            if view.is_none() {
                let cell = unsafe {
                    NSTableCellView::initWithFrame(
                        NSTableCellView::alloc(mtm),
                        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 44.0)),
                    )
                };
                unsafe { cell.setIdentifier(Some(identifier)) };

                let text_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                unsafe {
                    text_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    text_field.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
                    text_field.setMaximumNumberOfLines(2);
                    cell.addSubview(&text_field);
                    cell.setTextField(Some(&text_field));

                    text_field
                        .centerYAnchor()
                        .constraintEqualToAnchor(&cell.centerYAnchor())
                        .setActive(true);
                    text_field
                        .leadingAnchor()
                        .constraintEqualToAnchor_constant(&cell.leadingAnchor(), 4.0)
                        .setActive(true);
                    text_field
                        .trailingAnchor()
                        .constraintEqualToAnchor_constant(&cell.trailingAnchor(), -4.0)
                        .setActive(true);
                }
                view = Some(cell);
            }

            let selection: &PDFSelection =
                unsafe { &*(&*item as *const AnyObject as *const PDFSelection) };

            let mut label_text = String::new();
            if let Some(pages) = unsafe { Some(selection.pages()) } {
                if pages.count() > 0 {
                    let page = unsafe { pages.objectAtIndex(0) };
                    if let Some(page_label) = unsafe { page.label() } {
                        label_text.push_str(&format!("Page {}: ", page_label));
                    }
                }
            }
            if let Some(text) = unsafe { selection.string() } {
                label_text.push_str(&text.to_string().replace("\n", " ").trim());
            }

            if let Some(cell) = &view {
                if let Some(tf) = unsafe { cell.textField() } {
                    unsafe { tf.setStringValue(&NSString::from_str(&label_text)) };
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
                            let selection: &PDFSelection =
                                unsafe { &*(&*item as *const AnyObject as *const PDFSelection) };

                            if let Some(app) = unsafe {
                                NSApplication::sharedApplication(MainThreadMarker::new().unwrap())
                                    .delegate()
                            } {
                                let _: () = unsafe { msg_send![&app, recordHistory] };
                            }
                            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                                unsafe {
                                    pdf_view.setCurrentSelection(Some(selection));
                                    pdf_view.scrollSelectionToVisible(None);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
);

impl PdfSearchDelegate {
    pub fn new(mtm: MainThreadMarker, pdf_view: Retained<PDFView>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(PdfSearchDelegateIvars::default());
        let this: Retained<Self> = unsafe { msg_send_id![super(this), init] };
        this.ivars().pdf_view.set(pdf_view).unwrap();
        this
    }

    pub fn start_search(&self, query: &NSString) {
        if let Some(app) =
            unsafe { NSApplication::sharedApplication(MainThreadMarker::new().unwrap()).delegate() }
        {
            let _: () = unsafe { msg_send![&app, recordHistory] };
        }
        if let Some(pdf_view) = self.ivars().pdf_view.get() {
            if let Some(doc) = unsafe { pdf_view.document() } {
                unsafe {
                    if doc.isFinding() {
                        doc.cancelFindString();
                    }
                    doc.setDelegate(Some(ProtocolObject::from_ref(self)));
                    doc.beginFindString_withOptions(
                        query,
                        NSStringCompareOptions::CaseInsensitiveSearch,
                    );
                }
            }
        }
    }

    pub fn cancel_search(&self) {
        if let Some(app) =
            unsafe { NSApplication::sharedApplication(MainThreadMarker::new().unwrap()).delegate() }
        {
            let _: () = unsafe { msg_send![&app, recordHistory] };
        }
        if let Some(pdf_view) = self.ivars().pdf_view.get() {
            if let Some(doc) = unsafe { pdf_view.document() } {
                unsafe {
                    if doc.isFinding() {
                        doc.cancelFindString();
                    }
                }
            }
        }
    }

    pub fn clear(&self) {
        self.cancel_search();
        self.ivars().results.borrow_mut().clear();
        if let Some(tf) = self.ivars().search_field.get() {
            unsafe { tf.setStringValue(&NSString::from_str("")) };
        }
        if let Some(outline_view) = self.ivars().outline_view.get() {
            unsafe { outline_view.reloadData() };
        }
    }

    pub fn find_next(&self) {
        if let Some(outline_view) = self.ivars().outline_view.get() {
            let results = self.ivars().results.borrow();
            if !results.is_empty() {
                let mut current_row = unsafe { outline_view.selectedRow() };
                if current_row < 0 || current_row >= (results.len() as isize - 1) {
                    current_row = 0;
                } else {
                    current_row += 1;
                }
                let index_set = unsafe { NSIndexSet::indexSetWithIndex(current_row as usize) };
                unsafe { outline_view.selectRowIndexes_byExtendingSelection(&index_set, false) };
                unsafe { outline_view.scrollRowToVisible(current_row) };
            }
        }
    }

    pub fn find_previous(&self) {
        if let Some(outline_view) = self.ivars().outline_view.get() {
            let results = self.ivars().results.borrow();
            if !results.is_empty() {
                let mut current_row = unsafe { outline_view.selectedRow() };
                if current_row <= 0 {
                    current_row = results.len() as isize - 1;
                } else {
                    current_row -= 1;
                }
                let index_set = unsafe { NSIndexSet::indexSetWithIndex(current_row as usize) };
                unsafe { outline_view.selectRowIndexes_byExtendingSelection(&index_set, false) };
                unsafe { outline_view.scrollRowToVisible(current_row) };
            }
        }
    }
}
