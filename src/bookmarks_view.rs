use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSApplication, NSColor, NSControlTextEditingDelegate, NSFont, NSImage, NSImageView,
    NSOutlineView, NSOutlineViewDataSource, NSOutlineViewDelegate, NSTableCellView, NSTableColumn,
    NSTextField, NSTextFieldDelegate, NSView,
};
use objc2_foundation::{
    MainThreadMarker, NSInteger, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString,
    ns_string,
};
use objc2_pdf_kit::PDFView;
use std::cell::{OnceCell, RefCell};

#[derive(Clone, Debug)]
pub struct SidebarItemData {
    pub label: String,
    pub page_idx: usize,
    pub icon_symbol: &'static str,
    pub is_bookmark: bool,
}

#[derive(Default)]
pub struct BookmarksDelegateIvars {
    pub pdf_view: OnceCell<Retained<PDFView>>,
    pub outline_view: OnceCell<Retained<NSOutlineView>>,
    pub scroll_view: OnceCell<Retained<objc2_app_kit::NSScrollView>>,
    pub empty_state: RefCell<Option<crate::empty_state::EmptyStateView>>,
    pub items: RefCell<Vec<SidebarItemData>>,
    pub current_path: RefCell<String>,
    /// 0 = Bookmarks only, 1 = Annotations only
    pub filter_mode: std::cell::Cell<u8>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "BookmarksDelegate"]
    #[ivars = BookmarksDelegateIvars]
    pub struct BookmarksDelegate;

    unsafe impl NSObjectProtocol for BookmarksDelegate {}
    unsafe impl NSControlTextEditingDelegate for BookmarksDelegate {}
    unsafe impl NSTextFieldDelegate for BookmarksDelegate {}

    unsafe impl NSOutlineViewDataSource for BookmarksDelegate {
        #[unsafe(method(outlineView:numberOfChildrenOfItem:))]
        fn outlineView_numberOfChildrenOfItem(
            &self,
            _outline_view: &NSOutlineView,
            item: Option<&AnyObject>,
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
            item: Option<&AnyObject>,
        ) -> *mut AnyObject {
            if item.is_none() && index >= 0 && (index as usize) < self.ivars().items.borrow().len() {
                let str_idx = format!("{}", index);
                return Retained::autorelease_ptr(NSString::from_str(&str_idx)) as *mut AnyObject;
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

    unsafe impl NSOutlineViewDelegate for BookmarksDelegate {
        #[unsafe(method(outlineView:viewForTableColumn:item:))]
        fn outlineView_viewForTableColumn_item(
            &self,
            outline_view: &NSOutlineView,
            _table_column: Option<&NSTableColumn>,
            item: &AnyObject,
        ) -> *mut NSView {
            let mtm = MainThreadMarker::from(self);
            let identifier = ns_string!("BookmarkCell");

            let Some(retained_item) =
                (unsafe { Retained::retain(item as *const AnyObject as *mut AnyObject) })
            else {
                return std::ptr::null_mut();
            };
            let str_val = unsafe { Retained::cast_unchecked::<NSString>(retained_item) };
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
                        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 32.0)),
                    )
                };
                unsafe {
                    let _: () = msg_send![&cell, setIdentifier: identifier];
                };
                let icon_view = unsafe { NSImageView::new(mtm) };
                let text_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                let page_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };

                unsafe {
                    icon_view.setTag(401);
                    icon_view.setTranslatesAutoresizingMaskIntoConstraints(false);

                    text_field.setTag(402);
                    text_field.setFont(Some(&NSFont::systemFontOfSize(12.5)));
                    text_field.setTextColor(Some(&NSColor::labelColor()));
                    text_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    text_field.setUsesSingleLineMode(true);
                    text_field.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingTail);

                    page_field.setTag(403);
                    page_field.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
                        11.0, 0.0,
                    )));
                    page_field.setTextColor(Some(&NSColor::secondaryLabelColor()));
                    page_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    page_field.setContentHuggingPriority_forOrientation(
                        objc2_app_kit::NSLayoutPriorityRequired,
                        objc2_app_kit::NSLayoutConstraintOrientation::Horizontal,
                    );
                    page_field.setContentCompressionResistancePriority_forOrientation(
                        objc2_app_kit::NSLayoutPriorityRequired,
                        objc2_app_kit::NSLayoutConstraintOrientation::Horizontal,
                    );

                    cell.addSubview(&icon_view);
                    cell.addSubview(&text_field);
                    cell.addSubview(&page_field);

                    icon_view
                        .leadingAnchor()
                        .constraintEqualToAnchor_constant(&cell.leadingAnchor(), 8.0)
                        .setActive(true);
                    icon_view
                        .centerYAnchor()
                        .constraintEqualToAnchor(&cell.centerYAnchor())
                        .setActive(true);
                    icon_view
                        .widthAnchor()
                        .constraintEqualToConstant(16.0)
                        .setActive(true);
                    icon_view
                        .heightAnchor()
                        .constraintEqualToConstant(16.0)
                        .setActive(true);

                    page_field
                        .trailingAnchor()
                        .constraintEqualToAnchor_constant(&cell.trailingAnchor(), -8.0)
                        .setActive(true);
                    page_field
                        .centerYAnchor()
                        .constraintEqualToAnchor(&cell.centerYAnchor())
                        .setActive(true);

                    text_field
                        .leadingAnchor()
                        .constraintEqualToAnchor_constant(&icon_view.trailingAnchor(), 6.0)
                        .setActive(true);
                    text_field
                        .trailingAnchor()
                        .constraintLessThanOrEqualToAnchor_constant(
                            &page_field.leadingAnchor(),
                            -6.0,
                        )
                        .setActive(true);
                    text_field
                        .centerYAnchor()
                        .constraintEqualToAnchor(&cell.centerYAnchor())
                        .setActive(true);
                }
                view = Some(cell);
            }

            if let Some(cell) = &view {
                if let Some(data) = item_data {
                    if let Some(iv) = unsafe { cell.viewWithTag(401) } {
                        if let Some(iv) = iv.downcast_ref::<NSImageView>() {
                            if let Some(img) =
                                NSImage::imageWithSystemSymbolName_accessibilityDescription(
                                    &NSString::from_str(data.icon_symbol),
                                    None,
                                )
                            {
                                unsafe {
                                    iv.setImage(Some(&img));
                                    let tint = match data.icon_symbol {
                                        "bookmark.fill" | "bookmark" => NSColor::systemOrangeColor(),
                                        "highlighter" => NSColor::systemYellowColor(),
                                        "underline" => NSColor::systemBlueColor(),
                                        "strikethrough" => NSColor::systemRedColor(),
                                        "note.text" => NSColor::systemPurpleColor(),
                                        _ => NSColor::controlAccentColor(),
                                    };
                                    iv.setContentTintColor(Some(&tint));
                                };
                            }
                        }
                    }
                    if let Some(tf) = unsafe { cell.viewWithTag(402) } {
                        if let Some(tf) = tf.downcast_ref::<NSTextField>() {
                            let clean_label: String = data
                                .label
                                .replace(['\n', '\r'], " ")
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ");
                            let ns_lbl = NSString::from_str(&clean_label);
                            unsafe {
                                tf.setStringValue(&ns_lbl);
                                tf.setToolTip(Some(&ns_lbl));
                            };
                        }
                    }
                    if let Some(pf) = unsafe { cell.viewWithTag(403) } {
                        if let Some(pf) = pf.downcast_ref::<NSTextField>() {
                            let p_str = format!("p. {}", data.page_idx + 1);
                            unsafe { pf.setStringValue(&NSString::from_str(&p_str)) };
                        }
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
        fn outlineViewSelectionDidChange(&self, notification: &objc2_foundation::NSNotification) {
            if let Some(obj) = unsafe { notification.object() } {
                if let Some(outline_view) = obj.downcast_ref::<NSOutlineView>() {
                    let row = unsafe { outline_view.selectedRow() };
                    self.navigate_to_row(row);
                }
            }
        }
    }

    impl BookmarksDelegate {
        #[unsafe(method(bookmarkClicked:))]
        fn bookmark_clicked(&self, sender: Option<&NSOutlineView>) {
            if let Some(outline_view) = sender {
                let clicked = unsafe { outline_view.clickedRow() };
                let row = if clicked >= 0 {
                    clicked
                } else {
                    unsafe { outline_view.selectedRow() }
                };
                self.navigate_to_row(row);
            }
        }
    }
);

impl BookmarksDelegate {
    pub fn new(mtm: MainThreadMarker, pdf_view: Retained<PDFView>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(BookmarksDelegateIvars::default());
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };
        let _ = this.ivars().pdf_view.set(pdf_view);
        this
    }

    pub fn set_outline_view(&self, ov: Retained<NSOutlineView>) {
        let _ = self.ivars().outline_view.set(ov);
    }

    pub fn set_scroll_view(&self, sv: Retained<objc2_app_kit::NSScrollView>) {
        let _ = self.ivars().scroll_view.set(sv);
    }

    pub fn set_empty_state(&self, es: crate::empty_state::EmptyStateView) {
        *self.ivars().empty_state.borrow_mut() = Some(es);
    }

    pub fn set_empty_state_hidden(&self, hidden: bool) {
        if let Some(ref es) = *self.ivars().empty_state.borrow() {
            es.set_hidden(hidden);
        }
    }

    pub fn set_filter_mode(&self, mode: u8) {
        self.ivars().filter_mode.set(mode);
        self.reload();
    }

    pub fn set_current_path(&self, path: &str) {
        *self.ivars().current_path.borrow_mut() = path.to_string();
        self.reload();
    }

    pub fn count(&self) -> usize {
        self.ivars().items.borrow().len()
    }

    fn navigate_to_row(&self, row: isize) {
        if row < 0 {
            return;
        }
        let mtm = MainThreadMarker::from(self);
        let items = self.ivars().items.borrow();
        if let Some(it) = items.get(row as usize) {
            let target_page = it.page_idx;
            drop(items);

            if let Some(app) = unsafe { NSApplication::sharedApplication(mtm).delegate() } {
                let _: () = unsafe { msg_send![&app, recordHistory] };
            }

            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    if let Some(page) = unsafe { doc.pageAtIndex(target_page) } {
                        unsafe { pdf_view.goToPage(&page) };
                    }
                }
            }
        }
    }

    pub fn reload(&self) {
        let mut path = self.ivars().current_path.borrow().clone();
        if path.is_empty() {
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    if let Some(url) = unsafe { doc.documentURL() } {
                        if let Some(p) = unsafe { url.path() } {
                            path = p.to_string();
                        }
                    }
                }
            }
        }

        let mode = self.ivars().filter_mode.get();
        let mut items = Vec::new();

        if mode == 0 {
            // 1. Bookmarks Only
            let bms = crate::bookmarks::get_bookmarks_for_path(&path);
            for b in bms {
                items.push(SidebarItemData {
                    label: b.label,
                    page_idx: b.page_idx,
                    icon_symbol: "bookmark.fill",
                    is_bookmark: true,
                });
            }
        } else {
            // 2. Document Annotations Only
            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                if let Some(doc) = unsafe { pdf_view.document() } {
                    let annots = crate::annotations::get_document_annotations(&doc);
                    for a in annots {
                        items.push(SidebarItemData {
                            label: a.contents,
                            page_idx: a.page_index,
                            icon_symbol: a.annot_type.system_symbol_name(),
                            is_bookmark: false,
                        });
                    }
                }
            }
        }

        // Sort by page index ascending
        items.sort_by_key(|it| it.page_idx);
        let is_empty = items.is_empty();
        *self.ivars().items.borrow_mut() = items;

        if let Some(ref es) = *self.ivars().empty_state.borrow() {
            if mode == 0 {
                es.set_content(
                    "bookmark",
                    "No bookmarks in this document",
                    "Press ⌘D to bookmark the current page",
                );
            } else {
                es.set_content(
                    "highlighter",
                    "No annotations in this document",
                    "Select text to highlight, underline, strike out, or add a note",
                );
            }
            es.set_hidden(!is_empty);
        }

        if let Some(ov) = self.ivars().outline_view.get() {
            unsafe { ov.reloadData() };
        }
        if let Some(sv) = self.ivars().scroll_view.get() {
            sv.setHidden(is_empty);
        }
    }
}
