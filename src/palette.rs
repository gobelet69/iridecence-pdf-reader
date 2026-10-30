use crate::window_geometry::{WindowGeometry, WindowKind};
use objc2::{AnyThread, DefinedClass, MainThreadOnly, define_class, msg_send, rc::Retained};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSColor, NSControl, NSControlTextEditingDelegate, NSFont,
    NSOutlineView, NSOutlineViewDataSource, NSOutlineViewDelegate, NSPanel, NSScrollView,
    NSStackView, NSTableCellView, NSTableColumn, NSTextField, NSTextFieldDelegate, NSTextView,
    NSUserInterfaceLayoutOrientation, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView, NSWindowCollectionBehavior, NSWindowDelegate,
    NSWindowStyleMask,
};
use objc2_foundation::{
    MainThreadMarker, NSInteger, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect,
    NSSize, NSString, NSUserDefaults, ns_string,
};
use std::cell::RefCell;
use std::path::Path;

#[derive(Default)]
pub struct FloatingPalettePanelIvars;

define_class!(
    #[unsafe(super(NSPanel))]
    #[thread_kind = MainThreadOnly]
    #[name = "FloatingPalettePanel"]
    #[ivars = FloatingPalettePanelIvars]
    pub struct FloatingPalettePanel;

    unsafe impl NSObjectProtocol for FloatingPalettePanel {}

    impl FloatingPalettePanel {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            true
        }

        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main_window(&self) -> bool {
            false
        }

        #[unsafe(method(cancelOperation:))]
        fn cancel_operation(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            unsafe { self.orderOut(None) };
            let mtm = MainThreadMarker::new().unwrap();
            let app = unsafe { NSApplication::sharedApplication(mtm) };
            if let Some(main_win) = unsafe { app.mainWindow() } {
                unsafe { main_win.makeKeyWindow() };
            }
        }

        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &objc2_app_kit::NSEvent) {
            let key_code = unsafe { event.keyCode() };
            if key_code == 53 {
                // Escape key
                unsafe { self.orderOut(None) };
                let mtm = MainThreadMarker::new().unwrap();
                let app = unsafe { NSApplication::sharedApplication(mtm) };
                if let Some(main_win) = unsafe { app.mainWindow() } {
                    unsafe { main_win.makeKeyWindow() };
                }
                return;
            }
            let _: () = unsafe { objc2::msg_send![super(self), keyDown: event] };
        }
    }
);

#[derive(Clone, Debug)]
pub enum PaletteItemKind {
    Command { selector: &'static str },
    File { path: String },
    WorkspaceMatch { path: String, page_num: usize },
    PageJump { page_num: usize },
    OutlineJump { page_num: usize },
    BookmarkJump { path: String, page_num: usize },
    PdfSearch { query: String },
}

#[derive(Clone, Debug)]
pub struct PaletteItemData {
    pub title: String,
    pub subtitle: Option<String>,
    pub badge: Option<String>,
    pub kind: PaletteItemKind,
}

#[derive(Default)]
pub struct PaletteIvars {
    pub panel: std::cell::OnceCell<Retained<NSPanel>>,
    pub search_field: std::cell::OnceCell<Retained<NSTextField>>,
    pub table_view: std::cell::OnceCell<Retained<NSOutlineView>>,
    pub scroll: std::cell::OnceCell<Retained<NSScrollView>>,
    pub separator: std::cell::OnceCell<Retained<objc2_app_kit::NSBox>>,
    pub items: RefCell<Vec<PaletteItemData>>,
    pub hover_coordinator: crate::glass_row::HoverCoordinator,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "CommandPaletteDelegate"]
    #[ivars = PaletteIvars]
    pub struct CommandPaletteDelegate;

    unsafe impl NSObjectProtocol for CommandPaletteDelegate {}
    unsafe impl NSTextFieldDelegate for CommandPaletteDelegate {}

    unsafe impl NSWindowDelegate for CommandPaletteDelegate {
        #[unsafe(method(windowDidResignKey:))]
        fn windowDidResignKey(&self, _notification: &NSNotification) {
            self.dismiss_palette();
        }
    }

    impl CommandPaletteDelegate {
        #[unsafe(method(tableDidScroll:))]
        fn table_did_scroll(&self, _notification: &NSNotification) {
            if let Some(tv) = self.ivars().table_view.get() {
                self.ivars().hover_coordinator.update_hover(tv);
            }
        }

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, _event: &objc2_app_kit::NSEvent) {
            if let Some(tv) = self.ivars().table_view.get() {
                self.ivars().hover_coordinator.update_hover(tv);
            }
        }

        #[unsafe(method(mouseEntered:))]
        fn mouse_entered(&self, _event: &objc2_app_kit::NSEvent) {
            if let Some(tv) = self.ivars().table_view.get() {
                self.ivars().hover_coordinator.update_hover(tv);
            }
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, _event: &objc2_app_kit::NSEvent) {
            if let Some(tv) = self.ivars().table_view.get() {
                self.ivars().hover_coordinator.clear_hover(tv);
            }
        }
    }

    unsafe impl NSOutlineViewDataSource for CommandPaletteDelegate {
        #[unsafe(method(outlineView:numberOfChildrenOfItem:))]
        fn outlineView_numberOfChildrenOfItem(
            &self,
            _outline_view: &NSOutlineView,
            item: Option<&objc2::runtime::AnyObject>,
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
            item: Option<&objc2::runtime::AnyObject>,
        ) -> *mut objc2::runtime::AnyObject {
            if item.is_none() && index >= 0 && (index as usize) < self.ivars().items.borrow().len() {
                let str_idx = format!("{}", index);
                return Retained::autorelease_ptr(NSString::from_str(&str_idx))
                    as *mut objc2::runtime::AnyObject;
            }
            std::ptr::null_mut()
        }

        #[unsafe(method(outlineView:isItemExpandable:))]
        fn outlineView_isItemExpandable(
            &self,
            _outline_view: &NSOutlineView,
            _item: &objc2::runtime::AnyObject,
        ) -> bool {
            false
        }
    }

    unsafe impl NSOutlineViewDelegate for CommandPaletteDelegate {
        #[unsafe(method(outlineView:viewForTableColumn:item:))]
        fn outlineView_viewForTableColumn_item(
            &self,
            outline_view: &NSOutlineView,
            _table_column: Option<&NSTableColumn>,
            item: &objc2::runtime::AnyObject,
        ) -> *mut objc2_app_kit::NSView {
            let mtm = MainThreadMarker::from(self);
            let identifier = ns_string!("PaletteRichCell");

            let Some(retained_item) = (unsafe {
                Retained::retain(
                    item as *const objc2::runtime::AnyObject as *mut objc2::runtime::AnyObject,
                )
            }) else {
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
                        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(600.0, 36.0)),
                    )
                };
                unsafe {
                    let _: () = msg_send![&cell, setIdentifier: identifier];
                };

                let image_view = unsafe {
                    objc2_app_kit::NSImageView::initWithFrame(
                        objc2_app_kit::NSImageView::alloc(mtm),
                        NSRect::new(NSPoint::new(12.0, 8.0), NSSize::new(20.0, 20.0)),
                    )
                };
                let title_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                let subtitle_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                let badge_field = unsafe { NSTextField::labelWithString(ns_string!(""), mtm) };
                let text_stack = unsafe {
                    NSStackView::initWithFrame(
                        NSStackView::alloc(mtm),
                        NSRect::new(NSPoint::new(42.0, 2.0), NSSize::new(450.0, 32.0)),
                    )
                };

                unsafe {
                    image_view.setTag(300);
                    image_view.setTranslatesAutoresizingMaskIntoConstraints(false);

                    title_field.setTag(301);
                    title_field.setFont(Some(&NSFont::systemFontOfSize_weight(13.0, 0.2)));
                    title_field.setTextColor(Some(&NSColor::labelColor()));
                    title_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    title_field.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingTail);

                    subtitle_field.setTag(302);
                    subtitle_field.setFont(Some(&NSFont::systemFontOfSize(11.0)));
                    subtitle_field.setTextColor(Some(&NSColor::secondaryLabelColor()));
                    subtitle_field.setTranslatesAutoresizingMaskIntoConstraints(false);
                    subtitle_field
                        .setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingMiddle);

                    badge_field.setTag(303);
                    badge_field.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
                        11.0, 0.0,
                    )));
                    badge_field.setTextColor(Some(&NSColor::secondaryLabelColor()));
                    badge_field.setTranslatesAutoresizingMaskIntoConstraints(false);

                    text_stack.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
                    text_stack.setSpacing(1.0);
                    let _: () = msg_send![&text_stack, setAlignment: 5usize];
                    text_stack.setTranslatesAutoresizingMaskIntoConstraints(false);
                    text_stack.addArrangedSubview(&title_field);
                    text_stack.addArrangedSubview(&subtitle_field);

                    title_field
                        .leadingAnchor()
                        .constraintEqualToAnchor(&text_stack.leadingAnchor())
                        .setActive(true);
                    subtitle_field
                        .leadingAnchor()
                        .constraintEqualToAnchor(&text_stack.leadingAnchor())
                        .setActive(true);

                    cell.setImageView(Some(&image_view));
                    cell.setTextField(Some(&title_field));
                    cell.addSubview(&image_view);
                    cell.addSubview(&text_stack);
                    cell.addSubview(&badge_field);

                    image_view
                        .leadingAnchor()
                        .constraintEqualToAnchor_constant(&cell.leadingAnchor(), 12.0)
                        .setActive(true);
                    image_view
                        .centerYAnchor()
                        .constraintEqualToAnchor(&cell.centerYAnchor())
                        .setActive(true);
                    image_view
                        .widthAnchor()
                        .constraintEqualToConstant(20.0)
                        .setActive(true);
                    image_view
                        .heightAnchor()
                        .constraintEqualToConstant(20.0)
                        .setActive(true);

                    badge_field
                        .trailingAnchor()
                        .constraintEqualToAnchor_constant(&cell.trailingAnchor(), -12.0)
                        .setActive(true);
                    badge_field
                        .centerYAnchor()
                        .constraintEqualToAnchor(&cell.centerYAnchor())
                        .setActive(true);

                    text_stack
                        .leadingAnchor()
                        .constraintEqualToAnchor_constant(&image_view.trailingAnchor(), 10.0)
                        .setActive(true);
                    text_stack
                        .trailingAnchor()
                        .constraintLessThanOrEqualToAnchor_constant(
                            &badge_field.leadingAnchor(),
                            -8.0,
                        )
                        .setActive(true);
                    text_stack
                        .centerYAnchor()
                        .constraintEqualToAnchor(&cell.centerYAnchor())
                        .setActive(true);
                }
                view = Some(cell);
            }

            if let Some(cell) = &view {
                if let Some(data) = item_data {
                    if let Some(iv) = unsafe { cell.viewWithTag(300) } {
                        if let Some(iv) = iv.downcast_ref::<objc2_app_kit::NSImageView>() {
                            match &data.kind {
                                PaletteItemKind::File { path } => {
                                    let ws = unsafe { objc2_app_kit::NSWorkspace::sharedWorkspace() };
                                    let is_dir = std::path::Path::new(path).is_dir();
                                    let type_ident = if is_dir {
                                        objc2_foundation::ns_string!("public.folder")
                                    } else {
                                        objc2_foundation::ns_string!("pdf")
                                    };
                                    #[allow(deprecated)]
                                    let icon = unsafe { ws.iconForFileType(type_ident) };
                                    unsafe {
                                        iv.setContentTintColor(None);
                                        iv.setImage(Some(&icon));
                                    };
                                }
                                PaletteItemKind::Command { selector } => {
                                    let (sym_name, tint) = match *selector {
                                        "new_tab" | "newTab:" => ("plus.rectangle.on.rectangle", NSColor::controlAccentColor()),
                                        "close_tab" | "closeTab:" => ("xmark.square", NSColor::secondaryLabelColor()),
                                        "next_tab" => ("arrow.right.square", NSColor::controlAccentColor()),
                                        "prev_tab" => ("arrow.left.square", NSColor::controlAccentColor()),
                                        "open_pdf" | "openPdf:" => ("doc.badge.plus", NSColor::controlAccentColor()),
                                        "open_folder" | "openFolder:" => ("folder.badge.plus", NSColor::controlAccentColor()),
                                        "save_document" | "saveDocument:" => ("square.and.arrow.down", NSColor::controlAccentColor()),
                                        "save_document_as" | "saveDocumentAs:" => ("square.and.arrow.down.on.square", NSColor::controlAccentColor()),
                                        "highlight_selection" => ("highlighter", NSColor::systemYellowColor()),
                                        "underline_selection" => ("underline", NSColor::systemBlueColor()),
                                        "strikeout_selection" => ("strikethrough", NSColor::systemRedColor()),
                                        "add_note" => ("note.text.badge.plus", NSColor::systemPurpleColor()),
                                        "showHomeScreen:" => ("house", NSColor::controlAccentColor()),
                                        "toggleSidebar:" => ("sidebar.left", NSColor::controlAccentColor()),
                                        "toggleInspector:" => ("sidebar.right", NSColor::controlAccentColor()),
                                        "fit_width" | "fitWidth:" => ("arrow.left.and.right", NSColor::controlAccentColor()),
                                        "fit_page" | "fitPage:" => ("arrow.up.and.down", NSColor::controlAccentColor()),
                                        "actual_size" | "actualSize:" => ("1.magnifyingglass", NSColor::controlAccentColor()),
                                        "copy_page_ref" | "copy_with_page_ref" => ("doc.on.clipboard", NSColor::controlAccentColor()),
                                        "copy_markdown_quote" => ("text.quote", NSColor::controlAccentColor()),
                                        "search_selection" | "search" | "search:" => ("magnifyingglass", NSColor::controlAccentColor()),
                                        "toggle_presentation" | "togglePresentationMode:" => ("play.rectangle", NSColor::controlAccentColor()),
                                        "toggle_focus" | "toggleFocusMode:" => ("viewfinder", NSColor::controlAccentColor()),
                                        "goBack:" => ("chevron.left", NSColor::controlAccentColor()),
                                        "goForward:" => ("chevron.right", NSColor::controlAccentColor()),
                                        "go_to_page" | "goToPage:" => ("number.square", NSColor::controlAccentColor()),
                                        "toggle_bookmark" | "toggleBookmark:" => ("bookmark.fill", NSColor::systemOrangeColor()),
                                        "get_info" | "showDocumentInfo:" => ("info.circle", NSColor::controlAccentColor()),
                                        "export_notes" => ("square.and.arrow.up", NSColor::controlAccentColor()),
                                        "mode_single" => ("doc", NSColor::controlAccentColor()),
                                        "mode_single_cont" => ("doc.text", NSColor::controlAccentColor()),
                                        "mode_two_up" | "mode_two_up_cont" => ("book", NSColor::controlAccentColor()),
                                        "ocr_page" | "ocrCurrentPage:" | "ocr_document" | "copy_ocr" => ("text.viewfinder", NSColor::controlAccentColor()),
                                        "open_settings" | "openSettings:" => ("gearshape", NSColor::controlAccentColor()),
                                        "openDebugConsole:" => ("terminal", NSColor::controlAccentColor()),
                                        "noop" => ("magnifyingglass", NSColor::secondaryLabelColor()),
                                        _ => ("command", NSColor::controlAccentColor()),
                                    };
                                    let ns_sym = objc2_foundation::NSString::from_str(sym_name);
                                    if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(&ns_sym, None) {
                                        let config = objc2_app_kit::NSImageSymbolConfiguration::configurationWithPointSize_weight_scale(13.5, 0.0, objc2_app_kit::NSImageSymbolScale::Medium);
                                        let cfg_img = img.imageWithSymbolConfiguration(&config);
                                        unsafe {
                                            iv.setContentTintColor(Some(&tint));
                                            iv.setImage(cfg_img.as_deref().or(Some(&img)));
                                        };
                                    }
                                }
                                PaletteItemKind::OutlineJump { .. } => {
                                    let ns_sym = objc2_foundation::ns_string!("list.bullet.indent");
                                    if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(ns_sym, None) {
                                        unsafe {
                                            iv.setContentTintColor(Some(&NSColor::controlAccentColor()));
                                            iv.setImage(Some(&img));
                                        };
                                    }
                                }
                                PaletteItemKind::BookmarkJump { .. } => {
                                    let ns_sym = objc2_foundation::ns_string!("bookmark.fill");
                                    if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(ns_sym, None) {
                                        unsafe {
                                            iv.setContentTintColor(Some(&NSColor::systemOrangeColor()));
                                            iv.setImage(Some(&img));
                                        };
                                    }
                                }
                                PaletteItemKind::WorkspaceMatch { .. } => {
                                    let ns_sym = objc2_foundation::ns_string!("doc.text.magnifyingglass");
                                    if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(ns_sym, None) {
                                        unsafe {
                                            iv.setContentTintColor(Some(&NSColor::controlAccentColor()));
                                            iv.setImage(Some(&img));
                                        };
                                    }
                                }
                                PaletteItemKind::PageJump { .. } => {
                                    let ns_sym = objc2_foundation::ns_string!("doc.text");
                                    if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(ns_sym, None) {
                                        unsafe {
                                            iv.setContentTintColor(Some(&NSColor::controlAccentColor()));
                                            iv.setImage(Some(&img));
                                        };
                                    }
                                }
                                PaletteItemKind::PdfSearch { .. } => {
                                    let ns_sym = objc2_foundation::ns_string!("magnifyingglass");
                                    if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(ns_sym, None) {
                                        unsafe {
                                            iv.setContentTintColor(Some(&NSColor::controlAccentColor()));
                                            iv.setImage(Some(&img));
                                        };
                                    }
                                }
                            }
                        }
                    }
                    if let Some(tf) = unsafe { cell.viewWithTag(301) } {
                        if let Some(tf) = tf.downcast_ref::<NSTextField>() {
                            unsafe {
                                tf.setStringValue(&NSString::from_str(&data.title));
                                tf.setTextColor(Some(&NSColor::labelColor()));
                            };
                        }
                    }
                    if let Some(sub) = unsafe { cell.viewWithTag(302) } {
                        if let Some(sub) = sub.downcast_ref::<NSTextField>() {
                            if let Some(ref text) = data.subtitle {
                                unsafe {
                                    sub.setStringValue(&NSString::from_str(text));
                                    sub.setTextColor(Some(&NSColor::secondaryLabelColor()));
                                    sub.setHidden(false);
                                };
                            } else {
                                unsafe { sub.setHidden(true) };
                            }
                        }
                    }
                    if let Some(badge) = unsafe { cell.viewWithTag(303) } {
                        if let Some(badge) = badge.downcast_ref::<NSTextField>() {
                            if let Some(ref b_text) = data.badge {
                                unsafe {
                                    badge.setStringValue(&NSString::from_str(b_text));
                                    badge.setTextColor(Some(&NSColor::tertiaryLabelColor()));
                                    badge.setHidden(false);
                                };
                            } else {
                                unsafe { badge.setHidden(true) };
                            }
                        }
                    }
                }
            }

            match view {
                Some(v) => Retained::autorelease_ptr(v) as *mut objc2_app_kit::NSView,
                None => std::ptr::null_mut(),
            }
        }

        #[unsafe(method(outlineView:rowViewForItem:))]
        fn outline_view_row_view_for_item(
            &self,
            outline_view: &NSOutlineView,
            item: &objc2::runtime::AnyObject,
        ) -> *mut objc2::runtime::AnyObject {
            let mtm = MainThreadMarker::from(self);
            let row = unsafe { outline_view.rowForItem(Some(item)) };
            let row_view = crate::glass_row::GlassTableRowView::new(mtm);
            if self.ivars().hover_coordinator.hovered_row() == Some(row) {
                row_view.set_hovered(true);
            }
            Retained::autorelease_ptr(unsafe { Retained::cast_unchecked::<objc2::runtime::AnyObject>(row_view) })
        }
    }

    unsafe impl NSControlTextEditingDelegate for CommandPaletteDelegate {
        #[unsafe(method(controlTextDidChange:))]
        fn controlTextDidChange(&self, _notification: &NSNotification) {
            if let Some(tf) = self.ivars().search_field.get() {
                let s = unsafe { tf.stringValue().to_string() };
                self.update_search(&s);
            }
        }

        #[unsafe(method(control:textView:doCommandBySelector:))]
        fn control_textView_doCommandBySelector(
            &self,
            _control: &NSControl,
            _text_view: &NSTextView,
            command_selector: objc2::runtime::Sel,
        ) -> bool {
            let sel_name = command_selector.name();
            if sel_name.to_bytes() == b"moveDown:" || sel_name.to_bytes() == b"insertTab:" {
                if let Some(tv) = self.ivars().table_view.get() {
                    let current_row = unsafe { tv.selectedRow() };
                    let max_row = unsafe { tv.numberOfRows() } - 1;
                    if max_row >= 0 {
                        let next_row = if current_row < 0 {
                            0
                        } else if current_row >= max_row {
                            0
                        } else {
                            current_row + 1
                        };
                        let index_set = unsafe {
                            objc2_foundation::NSIndexSet::indexSetWithIndex(next_row as usize)
                        };
                        unsafe { tv.selectRowIndexes_byExtendingSelection(&index_set, false) };
                        unsafe { tv.scrollRowToVisible(next_row) };
                    }
                }
                true
            } else if sel_name.to_bytes() == b"moveUp:" || sel_name.to_bytes() == b"insertBacktab:" {
                if let Some(tv) = self.ivars().table_view.get() {
                    let current_row = unsafe { tv.selectedRow() };
                    let max_row = unsafe { tv.numberOfRows() } - 1;
                    if max_row >= 0 {
                        let next_row = if current_row <= 0 {
                            max_row
                        } else {
                            current_row - 1
                        };
                        let index_set = unsafe {
                            objc2_foundation::NSIndexSet::indexSetWithIndex(next_row as usize)
                        };
                        unsafe { tv.selectRowIndexes_byExtendingSelection(&index_set, false) };
                        unsafe { tv.scrollRowToVisible(next_row) };
                    }
                }
                true
            } else if sel_name.to_bytes() == b"insertNewline:" {
                self.activate_selected_item();
                self.dismiss_palette();
                true
            } else if sel_name.to_bytes() == b"cancelOperation:" {
                self.dismiss_palette();
                true
            } else {
                false
            }
        }
    }

    impl CommandPaletteDelegate {
        #[unsafe(method(tableClicked:))]
        fn table_clicked(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            if let Some(tv) = self.ivars().table_view.get() {
                let clicked_row = unsafe { tv.clickedRow() };
                if clicked_row >= 0 {
                    let index_set = unsafe {
                        objc2_foundation::NSIndexSet::indexSetWithIndex(clicked_row as usize)
                    };
                    unsafe { tv.selectRowIndexes_byExtendingSelection(&index_set, false) };
                    self.activate_selected_item();
                    self.dismiss_palette();
                }
            }
        }
    }
);

impl CommandPaletteDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(PaletteIvars::default());
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };
        this
    }

    pub fn dismiss_palette(&self) {
        if let Some(tv) = self.ivars().table_view.get() {
            self.ivars().hover_coordinator.clear_hover(tv);
        }
        if let Some(panel) = self.ivars().panel.get() {
            if unsafe { panel.isVisible() } {
                unsafe { panel.orderOut(None) };
                let mtm = MainThreadMarker::from(self);
                let app = unsafe { NSApplication::sharedApplication(mtm) };
                if let Some(main_win) = unsafe { app.mainWindow() } {
                    unsafe { main_win.makeKeyWindow() };
                }
            }
        }
    }

    pub fn open_palette(&self, initial_query: &str) {
        if let Some(panel) = self.ivars().panel.get() {
            if unsafe { panel.isVisible() } {
                if let Some(tf) = self.ivars().search_field.get() {
                    let current = unsafe { tf.stringValue().to_string() };
                    // If pressing the same shortcut (e.g. Cmd+P while empty, or Cmd+Shift+P while ">"), toggle close!
                    if (initial_query.is_empty() && !current.starts_with('>'))
                        || (initial_query == ">" && current.starts_with('>'))
                    {
                        self.dismiss_palette();
                        return;
                    }
                } else {
                    self.dismiss_palette();
                    return;
                }
            }

            if let Some(tf) = self.ivars().search_field.get() {
                unsafe {
                    tf.setStringValue(&NSString::from_str(initial_query));
                };
                self.update_search(initial_query);
                unsafe {
                    let mtm = MainThreadMarker::from(self);
                    let vis = WindowGeometry::screen_visible_frame(Some(panel.as_ref()), mtm);
                    let def_frame = WindowGeometry::default_frame(vis, WindowKind::Palette);
                    let mut frame = panel.frame();
                    frame.origin.x = def_frame.origin.x;
                    frame.origin.y = def_frame.origin.y + def_frame.size.height - frame.size.height;
                    panel.setFrame_display(frame, false);

                    panel.makeKeyAndOrderFront(None);
                    panel.makeFirstResponder(Some(&*tf));
                    tf.selectText(None);
                    if let Some(editor) = tf.currentEditor() {
                        let len = tf.stringValue().length();
                        let range = objc2_foundation::NSRange::new(len, 0);
                        let _: () = objc2::msg_send![&editor, setSelectedRange: range];
                    }
                };
            }
        }
    }

    pub fn setup(
        &self,
        panel: Retained<NSPanel>,
        text_field: Retained<NSTextField>,
        table_view: Retained<NSOutlineView>,
        scroll: Retained<NSScrollView>,
        separator: Retained<objc2_app_kit::NSBox>,
    ) {
        let _ = self.ivars().panel.set(panel);
        let _ = self.ivars().search_field.set(text_field);
        let _ = self.ivars().table_view.set(table_view);
        let _ = self.ivars().scroll.set(scroll);
        let _ = self.ivars().separator.set(separator);
    }

    pub fn activate_selected_item(&self) {
        if let Some(tv) = self.ivars().table_view.get() {
            let row = unsafe { tv.selectedRow() };
            let items = self.ivars().items.borrow();
            let selected = if row >= 0 && (row as usize) < items.len() {
                Some(items[row as usize].kind.clone())
            } else {
                None
            };
            drop(items);

            if let Some(kind) = selected {
                self.execute_item(kind);
            }
        }
    }

    pub fn execute_item(&self, kind: PaletteItemKind) {
        let mtm = MainThreadMarker::from(self);
        let app = unsafe { NSApplication::sharedApplication(mtm) };
        if let Some(delegate) = unsafe { app.delegate() } {
            match kind {
                PaletteItemKind::Command { selector } => {
                    if selector == "noop" {
                        return;
                    }
                    let ns_sel = NSString::from_str(selector);
                    let _: () = unsafe { msg_send![&*delegate, executePaletteCommand: &*ns_sel] };
                }
                PaletteItemKind::File { path } => {
                    let ns_path = NSString::from_str(&path);
                    let _: () = unsafe { msg_send![&*delegate, openPathString: &*ns_path] };
                }
                PaletteItemKind::WorkspaceMatch { path, page_num } => {
                    let ns_path = NSString::from_str(&path);
                    let _: () = unsafe {
                        msg_send![&*delegate, openPath: &*ns_path, atPage: page_num as NSInteger]
                    };
                }
                PaletteItemKind::PageJump { page_num } => {
                    let _: () = unsafe { msg_send![&*delegate, jumpToPage: page_num as NSInteger] };
                }
                PaletteItemKind::OutlineJump { page_num } => {
                    let _: () = unsafe { msg_send![&*delegate, jumpToPage: page_num as NSInteger] };
                }
                PaletteItemKind::BookmarkJump { path, page_num } => {
                    let ns_path = NSString::from_str(&path);
                    let _: () = unsafe {
                        msg_send![&*delegate, openPath: &*ns_path, atPage: page_num as NSInteger]
                    };
                }
                PaletteItemKind::PdfSearch { query } => {
                    let ns_query = NSString::from_str(&query);
                    let _: () = unsafe { msg_send![&*delegate, searchPdfWithQuery: &*ns_query] };
                }
            }
        }
    }
}

pub fn format_path_subtitle(path_str: &str, ws_dir: Option<&str>) -> String {
    let clean_path = path_str.strip_prefix("/private").unwrap_or(path_str);
    if let Some(ws) = ws_dir {
        let clean_ws = ws.strip_prefix("/private").unwrap_or(ws);
        let path = Path::new(clean_path);
        let ws_path = Path::new(clean_ws);
        if let Ok(rel) = path.strip_prefix(ws_path) {
            if let Some(parent) = rel.parent() {
                let p_str = parent.to_string_lossy();
                if !p_str.is_empty() {
                    return p_str.to_string();
                }
            }
            return "Workspace root".to_string();
        }
    }
    let path = Path::new(clean_path);
    if let Some(parent) = path.parent() {
        let parent_str = parent.to_string_lossy().to_string();
        if let Ok(home) = std::env::var("HOME") {
            let clean_home = home.strip_prefix("/private").unwrap_or(&home);
            if let Some(stripped) = parent_str.strip_prefix(clean_home) {
                return format!("~{}", stripped);
            }
        }
        parent_str
    } else {
        path_str.to_string()
    }
}

impl CommandPaletteDelegate {
    pub fn update_search(&self, query: &str) {
        if let Some(sf) = self.ivars().search_field.get() {
            let cur = unsafe { sf.stringValue() }.to_string();
            if cur != query {
                unsafe { sf.setStringValue(&NSString::from_str(query)) };
            }
        }
        let mut items = Vec::new();

        if query.starts_with("//") {
            // Workspace-wide PDF content search
            let term = query[2..].trim();
            if term.is_empty() {
                items.push(PaletteItemData {
                    title: "Type to search workspace PDF text...".to_string(),
                    subtitle: Some(
                        "Fast incremental full-text search across current workspace".to_string(),
                    ),
                    badge: Some("Workspace Search".to_string()),
                    kind: PaletteItemKind::Command { selector: "noop" },
                });
            } else {
                let results = crate::workspace_index::search_workspace(term);
                if results.is_empty() {
                    items.push(PaletteItemData {
                        title: format!("No matches for '{}'", term),
                        subtitle: Some(
                            "Check spelling or verify workspace PDFs are indexed".to_string(),
                        ),
                        badge: Some("0 results".to_string()),
                        kind: PaletteItemKind::Command { selector: "noop" },
                    });
                } else {
                    for r in results {
                        items.push(PaletteItemData {
                            title: format!("{} — Page {}", r.filename, r.page_num),
                            subtitle: Some(r.snippet),
                            badge: Some("Match".to_string()),
                            kind: PaletteItemKind::WorkspaceMatch {
                                path: r.path,
                                page_num: r.page_num.saturating_sub(1),
                            },
                        });
                    }
                }
            }
        } else if query.starts_with('@') {
            // Current PDF Outline section search
            let term = query[1..].trim().to_lowercase();
            let mtm = MainThreadMarker::from(self);
            let app = unsafe { NSApplication::sharedApplication(mtm) };
            if let Some(delegate) = unsafe { app.delegate() } {
                let sections: Vec<(String, usize)> = unsafe {
                    let ptr = Retained::as_ptr(&delegate) as *const crate::AppDelegate;
                    let del_ref = &*ptr;
                    del_ref.get_current_outline_sections()
                };

                for (sec_title, page_idx) in sections {
                    if term.is_empty() || sec_title.to_lowercase().contains(&term) {
                        items.push(PaletteItemData {
                            title: sec_title,
                            subtitle: Some(format!("Outline section at page {}", page_idx + 1)),
                            badge: Some("Outline".to_string()),
                            kind: PaletteItemKind::OutlineJump { page_num: page_idx },
                        });
                    }
                }
                if items.is_empty() {
                    items.push(PaletteItemData {
                        title: "No matching outline sections".to_string(),
                        subtitle: None,
                        badge: Some("Outline".to_string()),
                        kind: PaletteItemKind::Command { selector: "noop" },
                    });
                }
            }
        } else if query.starts_with('#') {
            // Bookmarks search
            let term = query[1..].trim().to_lowercase();
            let bookmarks = crate::bookmarks::load_bookmarks();
            for bm in bookmarks {
                let fname = Path::new(&bm.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if term.is_empty()
                    || bm.label.to_lowercase().contains(&term)
                    || fname.to_lowercase().contains(&term)
                {
                    items.push(PaletteItemData {
                        title: bm.label,
                        subtitle: Some(format!("{} — Page {}", fname, bm.page_idx + 1)),
                        badge: Some("Bookmark".to_string()),
                        kind: PaletteItemKind::BookmarkJump {
                            path: bm.path,
                            page_num: bm.page_idx,
                        },
                    });
                }
            }
            if items.is_empty() {
                items.push(PaletteItemData {
                    title: "No matching bookmarks".to_string(),
                    subtitle: Some("Press Cmd+D to bookmark current page".to_string()),
                    badge: Some("Bookmarks".to_string()),
                    kind: PaletteItemKind::Command { selector: "noop" },
                });
            }
        } else if query.starts_with('>') {
            // Commands
            let term = query[1..].trim().to_lowercase();
            let all_commands: Vec<(
                &'static str,
                &'static str,
                Option<&'static str>,
                &'static str,
            )> = vec![
                (
                    "New Tab",
                    "Open a PDF document in a new tab",
                    Some("⌘T"),
                    "new_tab",
                ),
                (
                    "Close Tab",
                    "Close the active document tab",
                    Some("⌘W"),
                    "close_tab",
                ),
                (
                    "Home Screen",
                    "Return to the Home screen and recent documents",
                    Some("⌥⌘H"),
                    "home_screen",
                ),
                (
                    "Show Next Tab",
                    "Switch to the next document tab",
                    Some("⇧⌘]"),
                    "next_tab",
                ),
                (
                    "Show Previous Tab",
                    "Switch to the previous document tab",
                    Some("⇧⌘["),
                    "prev_tab",
                ),
                (
                    "Save Document",
                    "Atomically save changes to current PDF",
                    Some("⌘S"),
                    "save_document",
                ),
                (
                    "Save Document As...",
                    "Save document to a new location",
                    Some("⇧⌘S"),
                    "save_document_as",
                ),
                (
                    "Highlight Selection",
                    "Add highlight annotation to selected text",
                    Some("⇧⌘H"),
                    "highlight_selection",
                ),
                (
                    "Underline Selection",
                    "Add blue underline annotation to selected text",
                    Some("⇧⌘U"),
                    "underline_selection",
                ),
                (
                    "Strikeout Selection",
                    "Add red strikeout annotation to selected text",
                    Some("⇧⌘X"),
                    "strikeout_selection",
                ),
                (
                    "Add Note to Selection",
                    "Attach a text note / comment annotation",
                    Some("⇧⌘N"),
                    "add_note",
                ),
                (
                    "Fit to Width",
                    "Zoom document to fill view width",
                    None,
                    "fit_width",
                ),
                (
                    "Fit to Page",
                    "Zoom document to fit entire page",
                    None,
                    "fit_page",
                ),
                (
                    "Actual Size (100%)",
                    "View document at original scale",
                    Some("⌘0"),
                    "actual_size",
                ),
                (
                    "Copy Page Reference",
                    "Copy citation e.g. document.pdf, p. 54",
                    None,
                    "copy_page_ref",
                ),
                (
                    "Copy with Page Reference",
                    "Copy selected text with citation",
                    None,
                    "copy_with_page_ref",
                ),
                (
                    "Copy as Markdown Quote",
                    "Copy selection as blockquote citation",
                    Some("⇧⌘M"),
                    "copy_markdown_quote",
                ),
                (
                    "Search Selection in Document",
                    "Find selected text in current PDF",
                    Some("⌘E"),
                    "search_selection",
                ),
                (
                    "Enter Presentation Mode",
                    "Fullscreen lecture presentation with page controls",
                    Some("⌘↩"),
                    "toggle_presentation",
                ),
                (
                    "Focus Reading Mode",
                    "Windowed reading without sidebars or toolbars",
                    Some("⇧⌘F"),
                    "toggle_focus",
                ),
                (
                    "Toggle Bookmark",
                    "Bookmark or remove bookmark on current page",
                    Some("⌘D"),
                    "toggle_bookmark",
                ),
                (
                    "Get Document Info",
                    "View metadata, pages, author, security",
                    Some("⌘I"),
                    "get_info",
                ),
                (
                    "Export Bookmarks & Notes as Markdown",
                    "Save notes, annotations, and bookmarks to Markdown file",
                    None,
                    "export_notes",
                ),
                (
                    "Display: Single Page",
                    "View one page at a time",
                    None,
                    "mode_single",
                ),
                (
                    "Display: Single Page Continuous",
                    "Continuous vertical page scroll",
                    None,
                    "mode_single_cont",
                ),
                (
                    "Display: Two-Up",
                    "View two pages side by side",
                    None,
                    "mode_two_up",
                ),
                (
                    "Display: Two-Up Continuous",
                    "Continuous two-page spread scroll",
                    None,
                    "mode_two_up_cont",
                ),
                (
                    "OCR Current Page",
                    "Recognize text on current page using Apple Vision OCR",
                    Some("⇧⌘R"),
                    "ocr_page",
                ),
                (
                    "OCR Entire Document",
                    "Run Apple Vision OCR across all pages in document",
                    None,
                    "ocr_document",
                ),
                (
                    "Copy Recognized Text (OCR)",
                    "Run Vision OCR on page and copy directly to clipboard",
                    None,
                    "copy_ocr",
                ),
                (
                    "Settings...",
                    "Open application settings and preferences",
                    Some("⌘,"),
                    "open_settings",
                ),
            ];

            for (title, desc, shortcut, sel) in all_commands {
                if term.is_empty()
                    || title.to_lowercase().contains(&term)
                    || desc.to_lowercase().contains(&term)
                {
                    items.push(PaletteItemData {
                        title: title.to_string(),
                        subtitle: Some(desc.to_string()),
                        badge: shortcut.map(|s| s.to_string()),
                        kind: PaletteItemKind::Command { selector: sel },
                    });
                }
            }
        } else if query.starts_with('/') {
            // Current PDF text search
            let term = query[1..].trim();
            items.push(PaletteItemData {
                title: if term.is_empty() {
                    "Search in current PDF...".to_string()
                } else {
                    format!("Search current PDF for '{}'", term)
                },
                subtitle: Some("Opens Document Search sidebar and finds matches".to_string()),
                badge: Some("Search".to_string()),
                kind: PaletteItemKind::PdfSearch {
                    query: term.to_string(),
                },
            });
        } else if query.starts_with("p:") || query.starts_with(':') {
            let num_str = query.trim_start_matches(|c| c == 'p' || c == ':').trim();
            if let Ok(p) = num_str.parse::<usize>() {
                if p > 0 {
                    items.push(PaletteItemData {
                        title: format!("Go to Page {}", p),
                        subtitle: Some("Press Return to jump to page".to_string()),
                        badge: Some(format!("p. {}", p)),
                        kind: PaletteItemKind::PageJump { page_num: p - 1 },
                    });
                }
            }
        } else {
            // Default: search workspace files, recents, favorites, and fallback commands
            let term = query.trim().to_lowercase();
            let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
            let mut seen_paths = std::collections::HashSet::new();

            // If user typed a pure number, offer page jump first!
            if let Ok(p) = term.parse::<usize>() {
                if p > 0 {
                    items.push(PaletteItemData {
                        title: format!("Go to Page {}", p),
                        subtitle: Some("Press Return to jump to page".to_string()),
                        badge: Some(format!("p. {}", p)),
                        kind: PaletteItemKind::PageJump { page_num: p - 1 },
                    });
                }
            }

            // 1. Collect Workspace PDFs instantly from memory (0 ms, non-blocking)
            let (ws_dir, workspace_pdfs) = crate::workspace_index::get_workspace_pdfs();

            for p in &workspace_pdfs {
                let norm_key = p.strip_prefix("/private").unwrap_or(p).to_string();
                let fname = Path::new(p)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if (term.is_empty()
                    || fname.to_lowercase().contains(&term)
                    || p.to_lowercase().contains(&term))
                    && !seen_paths.contains(&norm_key)
                {
                    seen_paths.insert(norm_key);
                    items.push(PaletteItemData {
                        title: fname,
                        subtitle: Some(format_path_subtitle(p, ws_dir.as_deref())),
                        badge: Some("Workspace".to_string()),
                        kind: PaletteItemKind::File { path: p.clone() },
                    });
                    if items.len() >= 12 {
                        break;
                    }
                }
            }

            // 2. Favorites
            if items.len() < 12 {
                let favs = unsafe {
                    defaults
                        .arrayForKey(&NSString::from_str("FavoriteFiles"))
                        .or_else(|| defaults.arrayForKey(&NSString::from_str("Favorites")))
                };
                if let Some(favs) = favs {
                    for i in 0..favs.count() {
                        let p = unsafe {
                            Retained::cast_unchecked::<NSString>(favs.objectAtIndex(i).clone())
                        }
                        .to_string();
                        let norm_key = p.strip_prefix("/private").unwrap_or(&p).to_string();
                        let fname = Path::new(&p)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if (term.is_empty() || fname.to_lowercase().contains(&term))
                            && !seen_paths.contains(&norm_key)
                        {
                            seen_paths.insert(norm_key);
                            items.push(PaletteItemData {
                                title: fname,
                                subtitle: Some(format_path_subtitle(&p, ws_dir.as_deref())),
                                badge: Some("Favorite".to_string()),
                                kind: PaletteItemKind::File { path: p },
                            });
                            if items.len() >= 12 {
                                break;
                            }
                        }
                    }
                }
            }

            // 3. Recents
            if items.len() < 12 {
                let recents = unsafe {
                    defaults
                        .arrayForKey(&NSString::from_str("RecentFiles"))
                        .or_else(|| defaults.arrayForKey(&NSString::from_str("RecentItems")))
                };
                if let Some(recents) = recents {
                    for i in 0..recents.count() {
                        let p = unsafe {
                            Retained::cast_unchecked::<NSString>(recents.objectAtIndex(i).clone())
                        }
                        .to_string();
                        let norm_key = p.strip_prefix("/private").unwrap_or(&p).to_string();
                        let fname = Path::new(&p)
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if (term.is_empty() || fname.to_lowercase().contains(&term))
                            && !seen_paths.contains(&norm_key)
                        {
                            seen_paths.insert(norm_key);
                            items.push(PaletteItemData {
                                title: fname,
                                subtitle: Some(format_path_subtitle(&p, ws_dir.as_deref())),
                                badge: Some("Recent".to_string()),
                                kind: PaletteItemKind::File { path: p },
                            });
                            if items.len() >= 12 {
                                break;
                            }
                        }
                    }
                }
            }

            // 4. If query is non-empty and few files matched, search commands too
            if !term.is_empty() && items.len() < 8 {
                let quick_cmds: Vec<(
                    &'static str,
                    &'static str,
                    Option<&'static str>,
                    &'static str,
                )> = vec![
                    (
                        "Open PDF...",
                        "Open a PDF document from disk",
                        Some("⌘O"),
                        "open_pdf",
                    ),
                    (
                        "Open Folder...",
                        "Open a workspace folder",
                        Some("⇧⌘O"),
                        "open_folder",
                    ),
                    ("New Tab", "Open a new document tab", Some("⌘T"), "new_tab"),
                    ("Close Tab", "Close current tab", Some("⌘W"), "close_tab"),
                    (
                        "Fit to Width",
                        "Zoom to fill window width",
                        Some("⌘1"),
                        "fit_width",
                    ),
                    ("Fit to Page", "Zoom to fit page", Some("⌘2"), "fit_page"),
                    (
                        "Actual Size (100%)",
                        "View at 100% scale",
                        Some("⌘0"),
                        "actual_size",
                    ),
                    (
                        "Search in Document",
                        "Search text in current PDF",
                        Some("⌘F"),
                        "search",
                    ),
                    (
                        "Go to Page...",
                        "Jump directly to page number",
                        Some("⌘L"),
                        "go_to_page",
                    ),
                    (
                        "Bookmark Page",
                        "Toggle bookmark on current page",
                        Some("⌘D"),
                        "toggle_bookmark",
                    ),
                    (
                        "Settings...",
                        "Open preferences and settings",
                        Some("⌘,"),
                        "open_settings",
                    ),
                ];
                for (title, desc, shortcut, sel) in quick_cmds {
                    if title.to_lowercase().contains(&term) || desc.to_lowercase().contains(&term) {
                        items.push(PaletteItemData {
                            title: title.to_string(),
                            subtitle: Some(desc.to_string()),
                            badge: shortcut.map(|s| s.to_string()),
                            kind: PaletteItemKind::Command { selector: sel },
                        });
                        if items.len() >= 12 {
                            break;
                        }
                    }
                }
            }

            // 5. If query is empty and we still have fewer than 3 items (fresh launch fallback)
            if term.is_empty() && items.len() < 3 {
                let starter_cmds: Vec<(
                    &'static str,
                    &'static str,
                    Option<&'static str>,
                    &'static str,
                )> = vec![
                    (
                        "Open PDF...",
                        "Open a PDF document from disk",
                        Some("⌘O"),
                        "open_pdf",
                    ),
                    (
                        "Open Folder / Workspace...",
                        "Open a directory containing PDFs",
                        Some("⇧⌘O"),
                        "open_folder",
                    ),
                    (
                        "Settings...",
                        "Preferences and display options",
                        Some("⌘,"),
                        "open_settings",
                    ),
                ];
                for (title, desc, shortcut, sel) in starter_cmds {
                    items.push(PaletteItemData {
                        title: title.to_string(),
                        subtitle: Some(desc.to_string()),
                        badge: shortcut.map(|s| s.to_string()),
                        kind: PaletteItemKind::Command { selector: sel },
                    });
                }
            }

            // 6. If non-empty query produced 0 items, show clean placeholder
            if !term.is_empty() && items.is_empty() {
                items.push(PaletteItemData {
                    title: format!("No matches for \"{}\"", term),
                    subtitle: Some(
                        "Type > for commands, // to search inside PDFs, or @ for outline"
                            .to_string(),
                    ),
                    badge: Some("0 results".to_string()),
                    kind: PaletteItemKind::Command { selector: "noop" },
                });
            }
        }

        *self.ivars().items.borrow_mut() = items.clone();

        let display_rows = (items.len() as f64).min(8.0);
        let has_results = display_rows > 0.0;

        if let Some(sep) = self.ivars().separator.get() {
            unsafe { sep.setHidden(!has_results) };
        }
        if let Some(sc) = self.ivars().scroll.get() {
            unsafe { sc.setHidden(!has_results) };
        }

        if let Some(tv) = self.ivars().table_view.get() {
            unsafe {
                tv.reloadData();
                let should_select = has_results
                    && (!query.trim().is_empty()
                        || items
                            .iter()
                            .any(|it| matches!(it.kind, PaletteItemKind::File { .. })));
                if should_select {
                    let index_set = objc2_foundation::NSIndexSet::indexSetWithIndex(0);
                    tv.selectRowIndexes_byExtendingSelection(&index_set, false);
                } else {
                    let empty_set = objc2_foundation::NSIndexSet::indexSet();
                    tv.selectRowIndexes_byExtendingSelection(&empty_set, false);
                }
            };
        }

        if let Some(tf) = self.ivars().search_field.get() {
            let cur = unsafe { tf.stringValue().to_string() };
            if cur != query {
                unsafe { tf.setStringValue(&NSString::from_str(query)) };
            }
        }

        if let Some(panel) = self.ivars().panel.get() {
            let mtm = MainThreadMarker::from(self);
            let vis = WindowGeometry::screen_visible_frame(Some(panel.as_ref()), mtm);
            let max_palette_h = WindowGeometry::max_height(vis, WindowKind::Palette);

            let new_height = if !has_results {
                48.0
            } else {
                (48.5 + (display_rows * 38.0) + 16.0).min(max_palette_h)
            };

            let mut frame = unsafe { panel.frame() };
            frame.origin.y += frame.size.height - new_height;
            frame.size.height = new_height;
            let sanitized = WindowGeometry::sanitize_frame(frame, vis, WindowKind::Palette);
            unsafe { panel.setFrame_display(sanitized, true) };
        }
    }

    pub fn scroll_and_select_row(&self, row: isize) {
        if let Some(tv) = self.ivars().table_view.get() {
            let total = unsafe { tv.numberOfRows() };
            if row >= 0 && row < total {
                let idx_set =
                    unsafe { objc2_foundation::NSIndexSet::indexSetWithIndex(row as usize) };
                unsafe {
                    tv.selectRowIndexes_byExtendingSelection(&idx_set, false);
                    tv.scrollRowToVisible(row);
                }
            }
        }
    }

    pub fn result_count(&self) -> usize {
        self.ivars().items.borrow().len()
    }

    pub fn selected_row(&self) -> isize {
        self.ivars()
            .table_view
            .get()
            .map(|tv| unsafe { tv.selectedRow() })
            .unwrap_or(-1)
    }
}

pub fn create_palette(
    mtm: MainThreadMarker,
) -> (Retained<NSPanel>, Retained<CommandPaletteDelegate>) {
    let vis = WindowGeometry::screen_visible_frame(None, mtm);
    let init_frame = WindowGeometry::default_frame(vis, WindowKind::Palette);

    let panel: Retained<NSPanel> = unsafe {
        let p = FloatingPalettePanel::alloc(mtm).set_ivars(FloatingPalettePanelIvars);
        let panel_obj: Option<Retained<FloatingPalettePanel>> = objc2::msg_send![
            super(p),
            initWithContentRect: init_frame,
            styleMask: NSWindowStyleMask::Borderless,
            backing: NSBackingStoreType::Buffered,
            defer: false
        ];
        Retained::cast_unchecked::<NSPanel>(
            panel_obj.expect("Failed to initialize FloatingPalettePanel"),
        )
    };

    unsafe {
        panel.setReleasedWhenClosed(false);
        panel.setHasShadow(true);
        panel.setLevel(objc2_app_kit::NSFloatingWindowLevel as NSInteger);
        panel.setFloatingPanel(true);
        panel.setBecomesKeyOnlyIfNeeded(false);
        panel.setHidesOnDeactivate(false);
        panel.setMovableByWindowBackground(true);
        panel.setCollectionBehavior(
            NSWindowCollectionBehavior::Transient | NSWindowCollectionBehavior::IgnoresCycle,
        );
        panel.setBackgroundColor(Some(&NSColor::clearColor()));
        panel.setOpaque(false);
    }
    WindowGeometry::sanitize_window(&panel, WindowKind::Palette, mtm);

    let delegate = CommandPaletteDelegate::new(mtm);
    let del_ptr: *const CommandPaletteDelegate = &*delegate;
    unsafe {
        let _: () = msg_send![&panel, setDelegate: del_ptr];
    }

    let visual_effect = unsafe {
        NSVisualEffectView::initWithFrame(
            NSVisualEffectView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), init_frame.size),
        )
    };
    unsafe {
        visual_effect.setMaterial(NSVisualEffectMaterial::Popover);
        visual_effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        visual_effect.setState(NSVisualEffectState::Active);
        visual_effect.setWantsLayer(true);
        let layer: Retained<objc2::runtime::AnyObject> = msg_send![&visual_effect, layer];
        let _: () = msg_send![&layer, setCornerRadius: 16.0_f64];
        let _: () = msg_send![&layer, setMasksToBounds: true];
        let cg_border = NSColor::separatorColor().CGColor();
        let ptr_border = Retained::as_ptr(&cg_border);
        let _: () = msg_send![&layer, setBorderWidth: 0.5_f64];
        let _: () = msg_send![&layer, setBorderColor: ptr_border];

        // Mask image with 16pt corner radius and 16pt cap insets to mask WindowServer backdrop
        let mask_size = NSSize::new(34.0, 34.0);
        let mask_img = unsafe {
            objc2_app_kit::NSImage::initWithSize(objc2_app_kit::NSImage::alloc(), mask_size)
        };
        let _: () = unsafe { msg_send![&*mask_img, lockFocus] };
        let rect = NSRect::new(NSPoint::new(0.0, 0.0), mask_size);
        let path: *mut objc2::runtime::AnyObject = unsafe {
            msg_send![
                objc2::class!(NSBezierPath),
                bezierPathWithRoundedRect: rect,
                xRadius: 16.0_f64,
                yRadius: 16.0_f64
            ]
        };
        let _: () = unsafe { msg_send![&NSColor::blackColor(), setFill] };
        let _: () = unsafe { msg_send![path, fill] };
        let _: () = unsafe { msg_send![&*mask_img, unlockFocus] };
        let insets = objc2_foundation::NSEdgeInsets {
            top: 16.0,
            left: 16.0,
            bottom: 16.0,
            right: 16.0,
        };
        let _: () = unsafe { msg_send![&*mask_img, setCapInsets: insets] };
        let _: () = unsafe { msg_send![&*mask_img, setResizingMode: 1usize] }; // stretch
        let _: () = unsafe { msg_send![&visual_effect, setMaskImage: &*mask_img] };

        visual_effect.setAutoresizingMask(
            objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable
                | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable,
        );
    }

    let search_icon = unsafe {
        objc2_app_kit::NSImageView::imageViewWithImage(
            &objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(
                ns_string!("magnifyingglass"),
                None,
            )
            .unwrap(),
            mtm,
        )
    };
    unsafe {
        let config =
            objc2_app_kit::NSImageSymbolConfiguration::configurationWithPointSize_weight_scale(
                16.0,
                0.0,
                objc2_app_kit::NSImageSymbolScale::Medium,
            );
        let _: () = msg_send![&search_icon, setSymbolConfiguration: &*config];
        search_icon.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
        search_icon.setTranslatesAutoresizingMaskIntoConstraints(false);
    }

    let search_field = unsafe { NSTextField::textFieldWithString(ns_string!(""), mtm) };
    unsafe {
        search_field.setFocusRingType(objc2_app_kit::NSFocusRingType::None);
        search_field.setBezeled(false);
        search_field.setDrawsBackground(false);
        search_field.setEditable(true);
        search_field.setSelectable(true);
        search_field.setEnabled(true);
        search_field.setPlaceholderString(Some(ns_string!(
            "Search files, commands (>), workspace (//), outline (@)..."
        )));

        let font = objc2_app_kit::NSFont::systemFontOfSize(15.0);
        let _: () = msg_send![&search_field, setFont: &*font];
        let _: () = msg_send![&search_field, setDelegate: del_ptr];
        search_field.setTranslatesAutoresizingMaskIntoConstraints(false);
    }

    let separator = unsafe {
        let b = objc2_app_kit::NSBox::initWithFrame(
            objc2_app_kit::NSBox::alloc(mtm),
            NSRect::new(
                NSPoint::new(0.0, 48.0),
                NSSize::new(init_frame.size.width, 1.0),
            ),
        );
        b.setBoxType(objc2_app_kit::NSBoxType::Separator);
        b.setTranslatesAutoresizingMaskIntoConstraints(false);
        b.setHidden(true);
        b
    };

    let scroll = unsafe {
        NSScrollView::initWithFrame(
            NSScrollView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(620.0, 0.0)),
        )
    };
    let table_view = unsafe {
        NSOutlineView::initWithFrame(
            NSOutlineView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(620.0, 0.0)),
        )
    };
    let col =
        unsafe { NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), ns_string!("Col")) };
    unsafe {
        col.setWidth(608.0);
        table_view.addTableColumn(&col);
        table_view.setOutlineTableColumn(Some(&col));
        table_view.setHeaderView(None);
        table_view.setRowHeight(36.0);
        table_view.setIntercellSpacing(NSSize::new(0.0, 2.0));
        table_view.setBackgroundColor(&NSColor::clearColor());
        table_view
            .setSelectionHighlightStyle(objc2_app_kit::NSTableViewSelectionHighlightStyle::Regular);
        let _: () = msg_send![&table_view, setStyle: 2usize];

        let _: () = msg_send![&table_view, setDelegate: del_ptr];
        let _: () = msg_send![&table_view, setDataSource: del_ptr];
        let _: () = msg_send![&table_view, setTarget: del_ptr];
        let _: () = msg_send![&table_view, setAction: objc2::sel!(tableClicked:)];
        let _: () = msg_send![&table_view, setDoubleAction: objc2::sel!(tableClicked:)];

        scroll.setDocumentView(Some(&table_view));
        scroll.setDrawsBackground(false);
        scroll.setTranslatesAutoresizingMaskIntoConstraints(false);
        scroll.setHidden(true);

        let ta_opts: usize = 0x01 | 0x02 | 0x80 | 0x200;
        let ta_alloc: *mut objc2::runtime::AnyObject =
            msg_send![objc2::class!(NSTrackingArea), alloc];
        let ta: *mut objc2::runtime::AnyObject = msg_send![
            ta_alloc,
            initWithRect: table_view.bounds(),
            options: ta_opts,
            owner: del_ptr,
            userInfo: std::ptr::null::<objc2::runtime::AnyObject>()
        ];
        if !ta.is_null() {
            let _: () = msg_send![&table_view, addTrackingArea: ta];
        }

        let clip = scroll.contentView();
        clip.setDrawsBackground(false);
        clip.setPostsBoundsChangedNotifications(true);
        let nc = objc2_foundation::NSNotificationCenter::defaultCenter();
        unsafe {
            nc.addObserver_selector_name_object(
                &*delegate,
                objc2::sel!(tableDidScroll:),
                Some(objc2_foundation::ns_string!(
                    "NSViewBoundsDidChangeNotification"
                )),
                Some(&clip),
            );
        }
    }

    unsafe {
        visual_effect.addSubview(&search_icon);
        visual_effect.addSubview(&search_field);
        visual_effect.addSubview(&separator);
        visual_effect.addSubview(&scroll);

        search_icon
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.leadingAnchor(), 16.0)
            .setActive(true);
        search_icon
            .topAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.topAnchor(), 15.0)
            .setActive(true);
        search_icon
            .widthAnchor()
            .constraintEqualToConstant(18.0)
            .setActive(true);
        search_icon
            .heightAnchor()
            .constraintEqualToConstant(18.0)
            .setActive(true);

        search_field
            .centerYAnchor()
            .constraintEqualToAnchor(&search_icon.centerYAnchor())
            .setActive(true);
        search_field
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&search_icon.trailingAnchor(), 10.0)
            .setActive(true);
        search_field
            .trailingAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.trailingAnchor(), -16.0)
            .setActive(true);
        search_field
            .heightAnchor()
            .constraintEqualToConstant(26.0)
            .setActive(true);

        separator
            .topAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.topAnchor(), 48.0)
            .setActive(true);
        separator
            .leadingAnchor()
            .constraintEqualToAnchor(&visual_effect.leadingAnchor())
            .setActive(true);
        separator
            .trailingAnchor()
            .constraintEqualToAnchor(&visual_effect.trailingAnchor())
            .setActive(true);
        separator
            .heightAnchor()
            .constraintEqualToConstant(1.0)
            .setActive(true);

        scroll
            .topAnchor()
            .constraintEqualToAnchor_constant(&separator.bottomAnchor(), 4.0)
            .setActive(true);
        scroll
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.leadingAnchor(), 6.0)
            .setActive(true);
        scroll
            .trailingAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.trailingAnchor(), -6.0)
            .setActive(true);
        scroll
            .bottomAnchor()
            .constraintEqualToAnchor_constant(&visual_effect.bottomAnchor(), -6.0)
            .setActive(true);

        panel.setContentView(Some(&visual_effect));
        panel.invalidateShadow();
    }

    delegate.setup(
        panel.clone(),
        search_field.clone(),
        table_view.clone(),
        scroll.clone(),
        separator.clone(),
    );

    (panel, delegate)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_palette_item_creation() {
        let item = PaletteItemData {
            title: "Test PDF".to_string(),
            subtitle: Some("/path/to/test.pdf".to_string()),
            badge: Some("Workspace".to_string()),
            kind: PaletteItemKind::File {
                path: "/path/to/test.pdf".to_string(),
            },
        };
        assert_eq!(item.title, "Test PDF");
        assert_eq!(item.badge.as_deref(), Some("Workspace"));
        match item.kind {
            PaletteItemKind::File { ref path } => assert_eq!(path, "/path/to/test.pdf"),
            _ => panic!("Expected File item"),
        }
    }

    #[test]
    fn test_palette_page_jump_parsing() {
        let queries = vec!["p:12", ":12", "12", "p: 99", ":1"];
        for q in queries {
            let num_str = q.trim_start_matches(|c| c == 'p' || c == ':').trim();
            let parsed = num_str.parse::<usize>();
            assert!(parsed.is_ok(), "Failed to parse page query: {}", q);
            assert!(parsed.unwrap() > 0);
        }
    }

    #[test]
    fn test_palette_command_item_filtering() {
        let term = "tab";
        let cmds = vec![
            ("New Tab", "new_tab"),
            ("Close Tab", "close_tab"),
            ("Open PDF", "open_pdf"),
        ];
        let filtered: Vec<_> = cmds
            .into_iter()
            .filter(|(t, _)| t.to_lowercase().contains(term))
            .collect();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].1, "new_tab");
        assert_eq!(filtered[1].1, "close_tab");
    }

    #[test]
    fn test_format_path_subtitle() {
        let ws = "/Users/testuser/Documents/OxiReader";
        let in_ws = "/Users/testuser/Documents/OxiReader/docs/manual.pdf";
        let sub = format_path_subtitle(in_ws, Some(ws));
        assert_eq!(sub, "docs");

        let at_root = "/Users/testuser/Documents/OxiReader/readme.pdf";
        let sub_root = format_path_subtitle(at_root, Some(ws));
        assert_eq!(sub_root, "Workspace root");

        if let Ok(home) = std::env::var("HOME") {
            let outside = format!("{}/Desktop/other.pdf", home);
            let sub_out = format_path_subtitle(&outside, Some(ws));
            assert_eq!(sub_out, "~/Desktop");
        }
    }

    #[test]
    fn test_tracking_area_owner_implements_all_enter_exit_callbacks() {
        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        let delegate = CommandPaletteDelegate::new(mtm);

        let entered: bool =
            unsafe { msg_send![&delegate, respondsToSelector: objc2::sel!(mouseEntered:)] };
        let moved: bool =
            unsafe { msg_send![&delegate, respondsToSelector: objc2::sel!(mouseMoved:)] };
        let exited: bool =
            unsafe { msg_send![&delegate, respondsToSelector: objc2::sel!(mouseExited:)] };

        assert!(entered && moved && exited);
    }
}
