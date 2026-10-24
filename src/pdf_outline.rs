use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::*;
use objc2_foundation::*;
use objc2_pdf_kit::{PDFOutline, PDFView};

#[derive(Default)]
pub struct PdfOutlineDelegateIvars {
    pub pdf_view: std::cell::OnceCell<Retained<PDFView>>,
    pub outline_view: std::cell::OnceCell<Retained<NSOutlineView>>,
    pub retained_nodes: std::cell::RefCell<Vec<Retained<PDFOutline>>>,
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
            if index < 0 {
                return std::ptr::null_mut();
            }
            if let Some(item) = item {
                let outline: &PDFOutline =
                    unsafe { &*(&*item as *const AnyObject as *const PDFOutline) };
                let count = unsafe { outline.numberOfChildren() };
                if (index as usize) < count {
                    if let Some(child) = unsafe { outline.childAtIndex(index as usize) } {
                        return Retained::autorelease_ptr(child) as *mut AnyObject;
                    }
                }
            } else if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    if let Some(root) = unsafe { doc.outlineRoot() } {
                        let count = unsafe { root.numberOfChildren() };
                        if (index as usize) < count {
                            if let Some(child) = unsafe { root.childAtIndex(index as usize) } {
                                return Retained::autorelease_ptr(child) as *mut AnyObject;
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
            _outline_view: &NSOutlineView,
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
            let mtm = MainThreadMarker::from(self);
            let identifier = ns_string!("OutlineCell");

            let mut view: Option<Retained<NSTableCellView>> = unsafe {
                let v = outline_view.makeViewWithIdentifier_owner(identifier, None);
                v.map(|v| Retained::cast_unchecked::<NSTableCellView>(v))
            };

            if view.is_none() {
                let cell = unsafe {
                    NSTableCellView::initWithFrame(
                        NSTableCellView::alloc(mtm),
                        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 26.0)),
                    )
                };
                unsafe { cell.setIdentifier(Some(identifier)) };

                let text_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                let page_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                unsafe {
                    text_field.setTag(101);
                    text_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    text_field.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
                    text_field.setFont(Some(&NSFont::systemFontOfSize(12.5)));
                    text_field.setTextColor(Some(&NSColor::labelColor()));

                    page_field.setTag(102);
                    page_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    page_field.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
                        11.0, 0.0,
                    )));
                    page_field.setTextColor(Some(&NSColor::secondaryLabelColor()));
                    page_field.setContentHuggingPriority_forOrientation(
                        NSLayoutPriorityRequired,
                        NSLayoutConstraintOrientation::Horizontal,
                    );
                    page_field.setContentCompressionResistancePriority_forOrientation(
                        NSLayoutPriorityRequired,
                        NSLayoutConstraintOrientation::Horizontal,
                    );

                    cell.addSubview(&text_field);
                    cell.addSubview(&page_field);
                    cell.setTextField(Some(&text_field));

                    page_field
                        .centerYAnchor()
                        .constraintEqualToAnchor(&cell.centerYAnchor())
                        .setActive(true);
                    page_field
                        .trailingAnchor()
                        .constraintEqualToAnchor_constant(&cell.trailingAnchor(), -6.0)
                        .setActive(true);

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
                        .constraintLessThanOrEqualToAnchor_constant(&page_field.leadingAnchor(), -6.0)
                        .setActive(true);
                }
                view = Some(cell);
            }

            let outline: &PDFOutline =
                unsafe { &*(&*item as *const AnyObject as *const PDFOutline) };
            if let Some(cell) = &view {
                if let Some(label) = unsafe { outline.label() } {
                    if let Some(tf) = unsafe { cell.textField() } {
                        unsafe {
                            tf.setStringValue(&label);
                            tf.setToolTip(Some(&label));
                        };
                    }
                    unsafe { cell.setToolTip(Some(&label)) };
                }

                let mut page_str = String::new();
                if let Some(pdf_view) = self.ivars().pdf_view.get() {
                    if let Some(doc) = unsafe { pdf_view.document() } {
                        if let Some(dest) = unsafe { outline.destination() } {
                            if let Some(page) = unsafe { dest.page() } {
                                let idx = unsafe { doc.indexForPage(&page) };
                                if idx < unsafe { doc.pageCount() } {
                                    page_str = format!("{}", idx + 1);
                                }
                            }
                        }
                    }
                }
                if let Some(pf) = unsafe { cell.viewWithTag(102) } {
                    if let Some(pf) = pf.downcast_ref::<NSTextField>() {
                        unsafe { pf.setStringValue(&NSString::from_str(&page_str)) };
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

    impl PdfOutlineDelegate {
        #[unsafe(method(outlineClicked:))]
        fn outline_clicked(&self, sender: Option<&NSOutlineView>) {
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

impl PdfOutlineDelegate {
    pub fn new(mtm: MainThreadMarker, pdf_view: Retained<PDFView>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(PdfOutlineDelegateIvars::default());
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };
        let _ = this.ivars().pdf_view.set(pdf_view);
        this
    }

    pub fn set_outline_view(&self, outline_view: Retained<NSOutlineView>) {
        let _ = self.ivars().outline_view.set(outline_view);
    }

    pub fn cache_outline_nodes(&self) -> usize {
        let mut nodes = Vec::new();
        if let Some(pdf_view) = self.ivars().pdf_view.get() {
            if let Some(doc) = unsafe { pdf_view.document() } {
                if let Some(root) = unsafe { doc.outlineRoot() } {
                    Self::collect_nodes(&root, 0, &mut nodes);
                }
            }
        }
        let total = nodes.len();
        *self.ivars().retained_nodes.borrow_mut() = nodes;
        total
    }

    fn collect_nodes(node: &PDFOutline, depth: usize, out: &mut Vec<Retained<PDFOutline>>) {
        if depth > 10 || out.len() >= 5000 {
            return;
        }
        let count = unsafe { node.numberOfChildren() };
        for i in 0..count {
            if out.len() >= 5000 {
                break;
            }
            if let Some(child) = unsafe { node.childAtIndex(i) } {
                Self::collect_nodes(&child, depth + 1, out);
                out.push(child);
            }
        }
    }

    pub fn expand_all_items(&self) {
        if let Some(ov) = self.ivars().outline_view.get() {
            unsafe { ov.expandItem_expandChildren(None, true) };
        }
    }

    pub fn collapse_all_items(&self) {
        if let Some(ov) = self.ivars().outline_view.get() {
            unsafe { ov.collapseItem_collapseChildren(None, true) };
        }
    }

    fn navigate_to_row(&self, outline_view: &NSOutlineView, row: isize) {
        if row < 0 {
            return;
        }
        let mtm = MainThreadMarker::from(self);
        if let Some(item) = unsafe { outline_view.itemAtRow(row) } {
            let outline: &PDFOutline =
                unsafe { &*(&*item as *const AnyObject as *const PDFOutline) };
            if let Some(dest) = unsafe { outline.destination() } {
                if let Some(app) = unsafe { NSApplication::sharedApplication(mtm).delegate() } {
                    let _: () = unsafe { msg_send![&app, recordHistory] };
                }
                if let Some(pdf_view) = self.ivars().pdf_view.get() {
                    unsafe { pdf_view.goToDestination(&dest) };
                }
            } else if let Some(action) = unsafe { outline.action() } {
                if let Some(app) = unsafe { NSApplication::sharedApplication(mtm).delegate() } {
                    let _: () = unsafe { msg_send![&app, recordHistory] };
                }
                if let Some(pdf_view) = self.ivars().pdf_view.get() {
                    unsafe { pdf_view.performAction(&action) };
                }
            }
        }
    }
}
