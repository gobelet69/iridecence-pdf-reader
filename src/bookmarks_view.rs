use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{DeclaredClass, MainThreadOnly, define_class, msg_send_id};
use objc2_app_kit::{
    NSColor, NSControlTextEditingDelegate, NSFont, NSImage, NSImageView, NSOutlineView,
    NSOutlineViewDataSource, NSOutlineViewDelegate, NSTableCellView, NSTableColumn, NSTextField,
    NSTextFieldDelegate, NSView,
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
    pub items: RefCell<Vec<SidebarItemData>>,
    pub current_path: RefCell<String>,
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
            if item.is_none() {
                let str_idx = format!("{}", index);
                return Retained::into_raw(NSString::from_str(&str_idx)) as *mut AnyObject;
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
            let mtm = MainThreadMarker::new().unwrap();
            let identifier = ns_string!("BookmarkCell");

            let str_val = unsafe {
                Retained::cast_unchecked::<NSString>(
                    Retained::retain(item as *const AnyObject as *mut AnyObject).unwrap(),
                )
            };
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
                let icon_view = unsafe { NSImageView::new(mtm) };
                let text_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                let page_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };

                unsafe {
                    icon_view.setTag(401);
                    icon_view.setTranslatesAutoresizingMaskIntoConstraints(false);

                    text_field.setTag(402);
                    text_field.setFont(Some(&NSFont::systemFontOfSize(13.0)));
                    text_field.setTextColor(Some(&NSColor::labelColor()));
                    text_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    text_field.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingTail);

                    page_field.setTag(403);
                    page_field.setFont(Some(&NSFont::systemFontOfSize(11.0)));
                    page_field.setTextColor(Some(&NSColor::secondaryLabelColor()));
                    page_field.setTranslatesAutoresizingMaskIntoConstraints(false);

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
                            -4.0,
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
                                    let tint = if data.is_bookmark {
                                        NSColor::systemYellowColor()
                                    } else {
                                        NSColor::controlAccentColor()
                                    };
                                    iv.setContentTintColor(Some(&tint));
                                };
                            }
                        }
                    }
                    if let Some(tf) = unsafe { cell.viewWithTag(402) } {
                        if let Some(tf) = tf.downcast_ref::<NSTextField>() {
                            unsafe { tf.setStringValue(&NSString::from_str(&data.label)) };
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
                Some(v) => Retained::into_raw(v) as *mut NSView,
                None => std::ptr::null_mut(),
            }
        }

        #[unsafe(method(outlineViewSelectionDidChange:))]
        fn outlineViewSelectionDidChange(&self, notification: &objc2_foundation::NSNotification) {
            if let Some(obj) = unsafe { notification.object() } {
                if let Some(outline_view) = obj.downcast_ref::<NSOutlineView>() {
                    let row = unsafe { outline_view.selectedRow() };
                    if row >= 0 {
                        let items = self.ivars().items.borrow();
                        if let Some(it) = items.get(row as usize) {
                            let target_page = it.page_idx;
                            drop(items);

                            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                                if let Some(doc) = unsafe { pdf_view.document() } {
                                    if let Some(page) = unsafe { doc.pageAtIndex(target_page) } {
                                        unsafe { pdf_view.goToPage(&page) };
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
);

impl BookmarksDelegate {
    pub fn new(mtm: MainThreadMarker, pdf_view: Retained<PDFView>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(BookmarksDelegateIvars::default());
        let this: Retained<Self> = unsafe { msg_send_id![super(this), init] };
        this.ivars().pdf_view.set(pdf_view).unwrap();
        this
    }

    pub fn set_outline_view(&self, ov: Retained<NSOutlineView>) {
        let _ = self.ivars().outline_view.set(ov);
    }

    pub fn set_current_path(&self, path: &str) {
        *self.ivars().current_path.borrow_mut() = path.to_string();
        self.reload();
    }

    pub fn reload(&self) {
        let path = self.ivars().current_path.borrow().clone();
        let mut items = Vec::new();

        // 1. Bookmarks
        let bms = crate::bookmarks::get_bookmarks_for_path(&path);
        for b in bms {
            items.push(SidebarItemData {
                label: b.label,
                page_idx: b.page_idx,
                icon_symbol: "bookmark.fill",
                is_bookmark: true,
            });
        }

        // 2. Document Annotations
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

        // Sort by page index ascending
        items.sort_by_key(|it| it.page_idx);
        *self.ivars().items.borrow_mut() = items;

        if let Some(ov) = self.ivars().outline_view.get() {
            unsafe { ov.reloadData() };
        }
    }
}
