use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::*;
use objc2_foundation::*;
use objc2_pdf_kit::{PDFDocumentDelegate, PDFSelection, PDFView};
use std::cell::RefCell;

#[derive(Default)]
pub struct PdfSearchDelegateIvars {
    pub pdf_view: std::cell::OnceCell<Retained<PDFView>>,
    pub outline_view: std::cell::OnceCell<Retained<NSOutlineView>>,
    pub search_scroll: std::cell::OnceCell<Retained<NSScrollView>>,
    pub results: RefCell<Vec<Retained<PDFSelection>>>,
    pub is_searching: std::cell::Cell<bool>,
    pub search_field: std::cell::OnceCell<Retained<NSTextField>>,
    pub empty_label: std::cell::OnceCell<Retained<NSTextField>>,
    pub status_label: std::cell::OnceCell<Retained<NSTextField>>,
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
                    self.clear_pdf_highlights();
                    if let Some(outline_view) = self.ivars().outline_view.get() {
                        unsafe { outline_view.reloadData() };
                    }
                    self.update_empty_state();
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
                self.clear_pdf_highlights();
                if let Some(outline_view) = self.ivars().outline_view.get() {
                    unsafe { outline_view.reloadData() };
                }
                self.update_empty_state();

                if let Some(pdf_view) = self.ivars().pdf_view.get() {
                    if let Some(window) = unsafe { pdf_view.window() } {
                        unsafe { window.makeFirstResponder(Some(pdf_view.as_ref())) };
                    }
                }
                return true.into();
            } else if sel_name.to_bytes() == b"insertNewline:" || sel_name.to_bytes() == b"moveDown:" {
                self.find_next();
                return true.into();
            } else if sel_name.to_bytes() == b"moveUp:" {
                self.find_previous();
                return true.into();
            }
            false.into()
        }
    }

    unsafe impl PDFDocumentDelegate for PdfSearchDelegate {
        #[unsafe(method(didMatchString:))]
        fn didMatchString(&self, selection: &PDFSelection) {
            if let Some(sel_retained) =
                unsafe { Retained::retain(selection as *const _ as *mut _) }
            {
                self.ivars().results.borrow_mut().push(sel_retained);
                if let Some(outline_view) = self.ivars().outline_view.get() {
                    unsafe { outline_view.reloadData() };
                }
                self.update_empty_state();
            }
        }

        #[unsafe(method(documentDidBeginDocumentFind:))]
        fn documentDidBeginDocumentFind(&self, _notification: &NSNotification) {
            self.ivars().is_searching.set(true);
            self.ivars().results.borrow_mut().clear();
            if let Some(outline_view) = self.ivars().outline_view.get() {
                unsafe { outline_view.reloadData() };
            }
            self.update_empty_state();
        }

        #[unsafe(method(documentDidEndDocumentFind:))]
        fn documentDidEndDocumentFind(&self, _notification: &NSNotification) {
            self.ivars().is_searching.set(false);
            self.update_empty_state();
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
            if item.is_none() && index >= 0 {
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
            let mtm = MainThreadMarker::from(self);
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
                let page_badge = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                unsafe {
                    text_field.setTag(101);
                    text_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    text_field.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
                    text_field.setMaximumNumberOfLines(2);
                    text_field.setFont(Some(&NSFont::systemFontOfSize(12.0)));
                    text_field.setTextColor(Some(&NSColor::labelColor()));

                    page_badge.setTag(102);
                    page_badge.setTranslatesAutoresizingMaskIntoConstraints(false);
                    page_badge.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
                        11.0, 0.0,
                    )));
                    page_badge.setTextColor(Some(&NSColor::secondaryLabelColor()));
                    page_badge.setContentHuggingPriority_forOrientation(
                        NSLayoutPriorityRequired,
                        NSLayoutConstraintOrientation::Horizontal,
                    );
                    page_badge.setContentCompressionResistancePriority_forOrientation(
                        NSLayoutPriorityRequired,
                        NSLayoutConstraintOrientation::Horizontal,
                    );

                    cell.addSubview(&text_field);
                    cell.addSubview(&page_badge);
                    cell.setTextField(Some(&text_field));

                    page_badge
                        .topAnchor()
                        .constraintEqualToAnchor_constant(&cell.topAnchor(), 6.0)
                        .setActive(true);
                    page_badge
                        .trailingAnchor()
                        .constraintEqualToAnchor_constant(&cell.trailingAnchor(), -6.0)
                        .setActive(true);

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
                        .constraintLessThanOrEqualToAnchor_constant(
                            &page_badge.leadingAnchor(),
                            -6.0,
                        )
                        .setActive(true);
                }
                view = Some(cell);
            }

            let selection: &PDFSelection =
                unsafe { &*(&*item as *const AnyObject as *const PDFSelection) };

            let mut page_str = String::new();
            let pages = unsafe { selection.pages() };
            if pages.count() > 0 {
                let page = pages.objectAtIndex(0);
                if let Some(page_label) = unsafe { page.label() } {
                    page_str = format!("p. {}", page_label);
                }
            }

            let snippet = Self::extract_snippet(selection);

            if let Some(cell) = &view {
                if let Some(tf) = unsafe { cell.textField() } {
                    unsafe {
                        tf.setStringValue(&NSString::from_str(&snippet));
                        tf.setToolTip(Some(&NSString::from_str(&snippet)));
                    };
                }
                if let Some(pb) = unsafe { cell.viewWithTag(102) } {
                    if let Some(pb) = pb.downcast_ref::<NSTextField>() {
                        unsafe { pb.setStringValue(&NSString::from_str(&page_str)) };
                    }
                }
            }

            match view {
                Some(v) => Retained::autorelease_ptr(v) as *mut NSView,
                None => std::ptr::null_mut(),
            }
        }

        #[unsafe(method(outlineView:rowViewForItem:))]
        fn outlineView_rowViewForItem(
            &self,
            _outline_view: &NSOutlineView,
            _item: &AnyObject,
        ) -> *mut objc2_app_kit::NSTableRowView {
            let mtm = MainThreadMarker::from(self);
            crate::glass_row::GlassTableRowView::new_row(mtm)
        }

        #[unsafe(method(outlineViewSelectionDidChange:))]
        fn outlineViewSelectionDidChange(&self, notification: &NSNotification) {
            if let Some(obj) = unsafe { notification.object() } {
                if let Some(outline_view) = obj.downcast_ref::<NSOutlineView>() {
                    let row = unsafe { outline_view.selectedRow() };
                    self.navigate_to_row(outline_view, row);
                }
            }
        }
    }

    impl PdfSearchDelegate {
        #[unsafe(method(searchResultClicked:))]
        fn search_result_clicked(&self, sender: Option<&NSOutlineView>) {
            if let Some(outline_view) = sender {
                let clicked = unsafe { outline_view.clickedRow() };
                let row = if clicked >= 0 {
                    clicked
                } else {
                    unsafe { outline_view.selectedRow() }
                };
                self.navigate_to_row(outline_view, row);
            }
        }
    }
);

impl PdfSearchDelegate {
    pub fn new(mtm: MainThreadMarker, pdf_view: Retained<PDFView>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(PdfSearchDelegateIvars::default());
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };
        let _ = this.ivars().pdf_view.set(pdf_view);
        this
    }

    fn extract_snippet(selection: &PDFSelection) -> String {
        unsafe {
            let copied: *mut PDFSelection = msg_send![selection, copy];
            if !copied.is_null() {
                if let Some(copied_sel) = Retained::from_raw(copied) {
                    let _: () = msg_send![&copied_sel, extendSelectionAtStart: 24isize];
                    let _: () = msg_send![&copied_sel, extendSelectionAtEnd: 44isize];
                    if let Some(ext_str) = copied_sel.string() {
                        let cleaned = ext_str
                            .to_string()
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ");
                        if !cleaned.is_empty() {
                            return format!("…{}…", cleaned);
                        }
                    }
                }
            }
            if let Some(text) = selection.string() {
                return text.to_string().replace('\n', " ").trim().to_string();
            }
        }
        String::new()
    }

    fn clear_pdf_highlights(&self) {
        if let Some(pdf_view) = self.ivars().pdf_view.get() {
            unsafe {
                let _: () = msg_send![
                    pdf_view,
                    setHighlightedSelections: std::ptr::null::<AnyObject>()
                ];
            }
        }
    }

    fn navigate_to_row(&self, outline_view: &NSOutlineView, row: isize) {
        if row < 0 {
            return;
        }
        let mtm = MainThreadMarker::from(self);
        if let Some(item) = unsafe { outline_view.itemAtRow(row) } {
            let selection: &PDFSelection =
                unsafe { &*(&*item as *const AnyObject as *const PDFSelection) };

            if let Some(app) = unsafe { NSApplication::sharedApplication(mtm).delegate() } {
                let _: () = unsafe { msg_send![&app, recordHistory] };
            }
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                unsafe {
                    let pages = selection.pages();
                    if pages.count() > 0 {
                        let page = pages.objectAtIndex(0);
                        pdf_view.goToPage(&page);
                    }
                    pdf_view.setCurrentSelection(Some(selection));
                    pdf_view.scrollSelectionToVisible(None);
                }
            }
        }
    }

    pub fn update_empty_state(&self) {
        let count = self.ivars().results.borrow().len();
        let has_results = count > 0;
        let is_searching = self.ivars().is_searching.get();
        let query_len = self
            .ivars()
            .search_field
            .get()
            .map(|tf| unsafe { tf.stringValue().length() })
            .unwrap_or(0);

        if let Some(scroll) = self.ivars().search_scroll.get() {
            unsafe { scroll.setHidden(!has_results) };
        }

        if let Some(status) = self.ivars().status_label.get() {
            unsafe {
                if query_len == 0 {
                    status.setStringValue(ns_string!(""));
                    status.setHidden(true);
                } else if is_searching && count == 0 {
                    status.setStringValue(ns_string!("Searching…"));
                    status.setHidden(false);
                } else {
                    let text = if count == 1 {
                        "1 match".to_string()
                    } else {
                        format!("{} matches", count)
                    };
                    status.setStringValue(&NSString::from_str(&text));
                    status.setHidden(false);
                }
            }
        }

        if let Some(lbl) = self.ivars().empty_label.get() {
            unsafe {
                if has_results {
                    lbl.setHidden(true);
                } else if is_searching {
                    lbl.setStringValue(ns_string!("Searching…"));
                    lbl.setHidden(false);
                } else if query_len > 0 {
                    lbl.setStringValue(ns_string!("No matches found in document"));
                    lbl.setHidden(false);
                } else {
                    lbl.setStringValue(ns_string!("Type above to search in document"));
                    lbl.setHidden(false);
                }
            }
        }
    }

    pub fn clear_results(&self) {
        self.cancel_search();
        self.ivars().results.borrow_mut().clear();
        self.clear_pdf_highlights();
        if let Some(outline_view) = self.ivars().outline_view.get() {
            unsafe { outline_view.reloadData() };
        }
        self.update_empty_state();
    }

    pub fn start_search(&self, query: &NSString) {
        if let Some(pdf_view) = self.ivars().pdf_view.get() {
            if let Some(doc) = unsafe { pdf_view.document() } {
                unsafe {
                    if doc.isFinding() {
                        doc.cancelFindString();
                    }
                    let page_count = doc.pageCount();
                    if page_count <= 500 {
                        self.ivars().is_searching.set(false);
                        let matches: Option<Retained<NSArray<PDFSelection>>> = msg_send![
                            &*doc,
                            findString: query,
                            withOptions: NSStringCompareOptions::CaseInsensitiveSearch.0
                        ];
                        let mut vec = Vec::new();
                        if let Some(ref arr) = matches {
                            for i in 0..arr.count() {
                                vec.push(arr.objectAtIndex(i));
                            }
                            let _: () = msg_send![pdf_view, setHighlightedSelections: &**arr];
                        } else {
                            self.clear_pdf_highlights();
                        }
                        let has_any = !vec.is_empty();
                        *self.ivars().results.borrow_mut() = vec;
                        if let Some(outline_view) = self.ivars().outline_view.get() {
                            outline_view.reloadData();
                            if has_any {
                                let idx = NSIndexSet::indexSetWithIndex(0);
                                outline_view.selectRowIndexes_byExtendingSelection(&idx, false);
                                self.navigate_to_row(outline_view, 0);
                            }
                        }
                        self.update_empty_state();
                    } else {
                        doc.setDelegate(Some(ProtocolObject::from_ref(self)));
                        doc.beginFindString_withOptions(
                            query,
                            NSStringCompareOptions::CaseInsensitiveSearch,
                        );
                    }
                }
            }
        }
    }

    pub fn cancel_search(&self) {
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
        self.clear_pdf_highlights();
        if let Some(tf) = self.ivars().search_field.get() {
            unsafe { tf.setStringValue(&NSString::from_str("")) };
        }
        if let Some(outline_view) = self.ivars().outline_view.get() {
            unsafe { outline_view.reloadData() };
        }
        self.update_empty_state();
    }

    pub fn find_next(&self) {
        if let Some(outline_view) = self.ivars().outline_view.get() {
            let count = self.ivars().results.borrow().len();
            if count > 0 {
                let mut current_row = unsafe { outline_view.selectedRow() };
                if current_row < 0 || current_row >= (count as isize - 1) {
                    current_row = 0;
                } else {
                    current_row += 1;
                }
                let index_set = unsafe { NSIndexSet::indexSetWithIndex(current_row as usize) };
                unsafe { outline_view.selectRowIndexes_byExtendingSelection(&index_set, false) };
                unsafe { outline_view.scrollRowToVisible(current_row) };
                self.navigate_to_row(outline_view, current_row);
            }
        }
    }

    pub fn find_previous(&self) {
        if let Some(outline_view) = self.ivars().outline_view.get() {
            let count = self.ivars().results.borrow().len();
            if count > 0 {
                let mut current_row = unsafe { outline_view.selectedRow() };
                if current_row <= 0 {
                    current_row = count as isize - 1;
                } else {
                    current_row -= 1;
                }
                let index_set = unsafe { NSIndexSet::indexSetWithIndex(current_row as usize) };
                unsafe { outline_view.selectRowIndexes_byExtendingSelection(&index_set, false) };
                unsafe { outline_view.scrollRowToVisible(current_row) };
                self.navigate_to_row(outline_view, current_row);
            }
        }
    }
}
