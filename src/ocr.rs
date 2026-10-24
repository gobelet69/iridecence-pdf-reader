use crate::settings::{PREF_AUTO_COPY_OCR, SettingsManager};
use crate::window_geometry::{WindowGeometry, WindowKind};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSButton, NSColor, NSFont, NSPanel, NSPasteboard, NSScrollView, NSTextField, NSTextView,
    NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState,
    NSVisualEffectView, NSWindowStyleMask,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSDictionary, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize,
    NSString,
};
use objc2_pdf_kit::{PDFDisplayBox, PDFPage};
use std::cell::RefCell;

thread_local! {
    static ACTIVE_OCR_CONTROLLER: RefCell<Option<Retained<OcrPanelController>>> = const { RefCell::new(None) };
    static ACTIVE_OCR_HUD: RefCell<Option<Retained<NSView>>> = const { RefCell::new(None) };
}

pub fn get_active_ocr_panel() -> Option<Retained<NSPanel>> {
    ACTIVE_OCR_CONTROLLER.with(|cell| cell.borrow().as_ref().map(|c| c.ivars().panel.clone()))
}

pub fn is_ocr_running_hud_visible() -> bool {
    ACTIVE_OCR_HUD.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|v| unsafe { !v.isHidden() && v.superview().is_some() })
            .unwrap_or(false)
    })
}

/// Show a non-blocking Liquid Glass HUD status pill inside the reader window while Vision OCR is running.
pub fn show_ocr_running_hud(parent_window: &objc2_app_kit::NSWindow, mtm: MainThreadMarker) {
    hide_ocr_running_hud();
    let Some(content_view) = (unsafe { parent_window.contentView() }) else {
        return;
    };

    unsafe {
        let hud = NSVisualEffectView::initWithFrame(
            NSVisualEffectView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(260.0, 38.0)),
        );
        hud.setMaterial(NSVisualEffectMaterial::HUDWindow);
        hud.setBlendingMode(NSVisualEffectBlendingMode::WithinWindow);
        hud.setState(NSVisualEffectState::Active);
        hud.setWantsLayer(true);
        if let Some(layer) = hud.layer() {
            let _: () = msg_send![&layer, setCornerRadius: 19.0f64];
            let _: () = msg_send![&layer, setMasksToBounds: true];
            let _: () = msg_send![&layer, setBorderWidth: 0.5f64];
            let border_col = NSColor::separatorColor().CGColor();
            let ptr = Retained::as_ptr(&border_col);
            let _: () = msg_send![&layer, setBorderColor: ptr];
        }
        hud.setTranslatesAutoresizingMaskIntoConstraints(false);

        let icon_cfg =
            objc2_app_kit::NSImageSymbolConfiguration::configurationWithPointSize_weight_scale(
                13.0,
                0.2,
                objc2_app_kit::NSImageSymbolScale::Medium,
            );
        let icon_img = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str("text.viewfinder"),
            Some(&NSString::from_str("OCR in progress")),
        )
        .and_then(|i| i.imageWithSymbolConfiguration(&icon_cfg))
        .unwrap_or_else(|| objc2_app_kit::NSImage::new());
        let icon_view = objc2_app_kit::NSImageView::imageViewWithImage(&icon_img, mtm);
        icon_view.setContentTintColor(Some(&NSColor::controlAccentColor()));
        icon_view.setTranslatesAutoresizingMaskIntoConstraints(false);

        let label = NSTextField::labelWithString(&NSString::from_str("Recognizing Text…"), mtm);
        label.setFont(Some(&NSFont::systemFontOfSize_weight(12.0, 0.3)));
        label.setTextColor(Some(&NSColor::labelColor()));
        label.setTranslatesAutoresizingMaskIntoConstraints(false);

        let cancel_btn = NSButton::initWithFrame(
            NSButton::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(56.0, 22.0)),
        );
        cancel_btn.setTitle(&NSString::from_str("Cancel"));
        let _: () = msg_send![&cancel_btn, setBezelStyle: 1usize];
        let _: () = msg_send![&cancel_btn, setControlSize: 1usize];
        cancel_btn.setFont(Some(&NSFont::systemFontOfSize(10.5)));
        cancel_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

        hud.addSubview(&icon_view);
        hud.addSubview(&label);
        hud.addSubview(&cancel_btn);
        content_view.addSubview(&hud);

        hud.centerXAnchor()
            .constraintEqualToAnchor(&content_view.centerXAnchor())
            .setActive(true);
        hud.topAnchor()
            .constraintEqualToAnchor_constant(&content_view.topAnchor(), 62.0)
            .setActive(true);
        hud.widthAnchor()
            .constraintEqualToConstant(256.0)
            .setActive(true);
        hud.heightAnchor()
            .constraintEqualToConstant(36.0)
            .setActive(true);

        icon_view
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&hud.leadingAnchor(), 14.0)
            .setActive(true);
        icon_view
            .centerYAnchor()
            .constraintEqualToAnchor(&hud.centerYAnchor())
            .setActive(true);
        icon_view
            .widthAnchor()
            .constraintEqualToConstant(16.0)
            .setActive(true);
        icon_view
            .heightAnchor()
            .constraintEqualToConstant(16.0)
            .setActive(true);

        label
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&icon_view.trailingAnchor(), 8.0)
            .setActive(true);
        label
            .centerYAnchor()
            .constraintEqualToAnchor(&hud.centerYAnchor())
            .setActive(true);

        cancel_btn
            .trailingAnchor()
            .constraintEqualToAnchor_constant(&hud.trailingAnchor(), -10.0)
            .setActive(true);
        cancel_btn
            .centerYAnchor()
            .constraintEqualToAnchor(&hud.centerYAnchor())
            .setActive(true);
        cancel_btn
            .widthAnchor()
            .constraintEqualToConstant(56.0)
            .setActive(true);

        content_view.layoutSubtreeIfNeeded();

        let hud_view: Retained<NSView> = Retained::cast_unchecked(hud);
        ACTIVE_OCR_HUD.with(|cell| {
            *cell.borrow_mut() = Some(hud_view);
        });
    }
}

pub fn hide_ocr_running_hud() {
    ACTIVE_OCR_HUD.with(|cell| {
        if let Some(hud) = cell.borrow_mut().take() {
            unsafe { hud.removeFromSuperview() };
        }
    });
}

pub struct OcrPanelIvars {
    pub panel: Retained<NSPanel>,
    pub text_view: Retained<NSTextView>,
    pub stats_label: Retained<NSTextField>,
    pub copy_btn: Retained<NSButton>,
    pub base_stats: String,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "OcrPanelController"]
    #[ivars = OcrPanelIvars]
    pub struct OcrPanelController;

    unsafe impl NSObjectProtocol for OcrPanelController {}

    impl OcrPanelController {
        #[unsafe(method(copyOcrText:))]
        fn copy_ocr_text(&self, _sender: *mut AnyObject) {
            unsafe {
                let text = self.ivars().text_view.string().to_string();
                let pb = NSPasteboard::generalPasteboard();
                pb.clearContents();
                let ns_str = NSString::from_str(&text);
                let _: bool = msg_send![
                    &*pb,
                    setString: &*ns_str,
                    forType: objc2_app_kit::NSPasteboardTypeString
                ];
                self.ivars()
                    .copy_btn
                    .setTitle(&NSString::from_str("Copied to Clipboard ✓"));
                let updated_stats = format!("{} • Copied to Clipboard ✓", self.ivars().base_stats);
                self.ivars()
                    .stats_label
                    .setStringValue(&NSString::from_str(&updated_stats));
            }
        }
    }
);

/// Extract page image data (TIFF) suitable for Apple Vision recognition.
pub fn extract_page_tiff_data(page: &PDFPage) -> Option<Retained<objc2_foundation::NSData>> {
    let bounds = unsafe { page.boundsForBox(PDFDisplayBox::MediaBox) };
    let width = (bounds.size.width * 2.0).clamp(800.0, 2400.0);
    let height = (bounds.size.height * 2.0).clamp(1000.0, 3200.0);
    let size = NSSize::new(width, height);

    unsafe {
        let image: *mut AnyObject = msg_send![page, thumbnailOfSize: size, forBox: 0usize];
        if image.is_null() {
            return None;
        }
        let tiff_data: *mut objc2_foundation::NSData = msg_send![image, TIFFRepresentation];
        if tiff_data.is_null() {
            None
        } else {
            Some(Retained::retain(tiff_data)?)
        }
    }
}

/// Perform Vision OCR synchronously on raw image data.
pub fn perform_vision_ocr(tiff_data: &objc2_foundation::NSData) -> Result<String, String> {
    unsafe {
        let request_class = objc2::class!(VNRecognizeTextRequest);
        let request_raw: *mut AnyObject = msg_send![request_class, alloc];
        let request_raw: *mut AnyObject = msg_send![request_raw, init];
        let Some(request) = Retained::from_raw(request_raw) else {
            return Err("Failed to allocate VNRecognizeTextRequest".to_string());
        };

        // recognitionLevel = 1 (VNRequestTextRecognitionLevelAccurate)
        let _: () = msg_send![&*request, setRecognitionLevel: 1usize];
        let _: () = msg_send![&*request, setUsesLanguageCorrection: true];

        let req_ref: &AnyObject = &request;
        let requests_array = NSArray::from_slice(&[req_ref]);

        let handler_class = objc2::class!(VNImageRequestHandler);
        let handler_raw: *mut AnyObject = msg_send![handler_class, alloc];
        let empty_dict = NSDictionary::<NSString, AnyObject>::new();
        let handler_raw: *mut AnyObject =
            msg_send![handler_raw, initWithData: tiff_data, options: &*empty_dict];
        let Some(handler) = Retained::from_raw(handler_raw) else {
            return Err("Failed to allocate VNImageRequestHandler".to_string());
        };

        let mut error: *mut AnyObject = std::ptr::null_mut();
        let success: bool =
            msg_send![&*handler, performRequests: &*requests_array, error: &mut error];
        if !success {
            return Err("Vision performRequests failed".to_string());
        }

        let results: *mut NSArray<AnyObject> = msg_send![&*request, results];
        if results.is_null() {
            return Ok(String::new());
        }

        let count = (*results).count();
        let mut recognized_lines = Vec::with_capacity(count);

        for i in 0..count {
            let obs = (*results).objectAtIndex(i);
            let candidates: *mut NSArray<AnyObject> = msg_send![&*obs, topCandidates: 1usize];
            if !candidates.is_null() && (*candidates).count() > 0 {
                let cand = (*candidates).objectAtIndex(0);
                let cand_str: *mut NSString = msg_send![&*cand, string];
                if !cand_str.is_null() {
                    let s = (*cand_str).to_string();
                    if !s.is_empty() {
                        recognized_lines.push(s);
                    }
                }
            }
        }

        Ok(recognized_lines.join("\n"))
    }
}

/// Create and present a native Liquid Glass OCR Result Panel.
pub fn show_ocr_result_panel(
    text: &str,
    page_num: usize,
    total_pages: usize,
    mtm: MainThreadMarker,
) {
    hide_ocr_running_hud();

    // Close any previously open OCR panel cleanly before presenting the new one
    ACTIVE_OCR_CONTROLLER.with(|cell| {
        if let Some(prev) = cell.borrow_mut().take() {
            prev.ivars().panel.orderOut(None);
        }
    });

    let screen_vis = WindowGeometry::screen_visible_frame(None, mtm);
    let panel_w = 620.0f64.min(screen_vis.size.width * 0.85);
    let panel_h = 460.0f64.min(screen_vis.size.height * 0.85);

    let frame = NSRect::new(
        NSPoint::new(
            screen_vis.origin.x + (screen_vis.size.width - panel_w) / 2.0,
            screen_vis.origin.y + (screen_vis.size.height - panel_h) / 2.0,
        ),
        NSSize::new(panel_w, panel_h),
    );

    let panel = unsafe {
        NSPanel::initWithContentRect_styleMask_backing_defer(
            NSPanel::alloc(mtm),
            frame,
            NSWindowStyleMask::Titled
                | NSWindowStyleMask::Closable
                | NSWindowStyleMask::Resizable
                | NSWindowStyleMask::FullSizeContentView,
            objc2_app_kit::NSBackingStoreType::Buffered,
            false,
        )
    };

    unsafe {
        panel.setReleasedWhenClosed(false);
        panel.setTitle(&NSString::from_str("Recognized Text (OCR)"));
        panel.setTitleVisibility(objc2_app_kit::NSWindowTitleVisibility::Hidden);
        panel.setTitlebarAppearsTransparent(true);
        panel.setOpaque(false);
        panel.setBackgroundColor(Some(&NSColor::clearColor()));
        panel.setHasShadow(true);
        panel.setMovableByWindowBackground(true);
        panel.setFloatingPanel(true);
        panel.setHidesOnDeactivate(false);
    }

    // Liquid Glass background
    let effect_view = unsafe {
        NSVisualEffectView::initWithFrame(
            NSVisualEffectView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(panel_w, panel_h)),
        )
    };
    unsafe {
        effect_view.setMaterial(NSVisualEffectMaterial::HUDWindow);
        effect_view.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        effect_view.setState(NSVisualEffectState::FollowsWindowActiveState);
        effect_view.setAutoresizingMask(
            objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable
                | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable,
        );
    }

    // Header container
    let header_box = unsafe {
        NSView::initWithFrame(
            NSView::alloc(mtm),
            NSRect::new(
                NSPoint::new(0.0, panel_h - 56.0),
                NSSize::new(panel_w, 56.0),
            ),
        )
    };
    header_box.setTranslatesAutoresizingMaskIntoConstraints(false);

    let line_count = text.lines().count();
    let word_count = text.split_whitespace().count();

    let title_label = unsafe {
        let lbl = NSTextField::labelWithString(&NSString::from_str("Apple Vision OCR"), mtm);
        lbl.setFont(Some(&NSFont::boldSystemFontOfSize(14.0)));
        lbl.setTextColor(Some(&NSColor::labelColor()));
        lbl
    };
    title_label.setTranslatesAutoresizingMaskIntoConstraints(false);

    let base_stats = format!(
        "Page {} of {} • {} lines • {} words",
        page_num + 1,
        total_pages,
        line_count,
        word_count
    );
    let auto_copied = SettingsManager::bool_pref(PREF_AUTO_COPY_OCR, true);
    let initial_stats = if auto_copied {
        format!("{} • Auto-copied to Clipboard", base_stats)
    } else {
        base_stats.clone()
    };

    let stats_label = unsafe {
        let lbl = NSTextField::labelWithString(&NSString::from_str(&initial_stats), mtm);
        lbl.setFont(Some(&NSFont::systemFontOfSize(11.0)));
        lbl.setTextColor(Some(&NSColor::secondaryLabelColor()));
        lbl
    };
    stats_label.setTranslatesAutoresizingMaskIntoConstraints(false);

    // Scrollable TextView with subtle rounded border
    let scroll_view = unsafe {
        let sv = NSScrollView::initWithFrame(
            NSScrollView::alloc(mtm),
            NSRect::new(
                NSPoint::new(20.0, 52.0),
                NSSize::new(panel_w - 40.0, panel_h - 120.0),
            ),
        );
        sv.setHasVerticalScroller(true);
        sv.setDrawsBackground(false);
        sv.setAutohidesScrollers(true);
        sv.setWantsLayer(true);
        if let Some(layer) = sv.layer() {
            let _: () = msg_send![&layer, setCornerRadius: 8.0f64];
            let _: () = msg_send![&layer, setBorderWidth: 0.5f64];
            let border_cg = NSColor::separatorColor().CGColor();
            let ptr = Retained::as_ptr(&border_cg);
            let _: () = msg_send![&layer, setBorderColor: ptr];
            let bg_cg = NSColor::labelColor()
                .colorWithAlphaComponent(0.03)
                .CGColor();
            let bg_ptr = Retained::as_ptr(&bg_cg);
            let _: () = msg_send![&layer, setBackgroundColor: bg_ptr];
        }
        sv
    };
    scroll_view.setTranslatesAutoresizingMaskIntoConstraints(false);

    let text_view = unsafe {
        let tv = NSTextView::initWithFrame(
            NSTextView::alloc(mtm),
            NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(panel_w - 40.0, panel_h - 120.0),
            ),
        );
        tv.setString(&NSString::from_str(text));
        tv.setEditable(false);
        tv.setSelectable(true);
        tv.setTextContainerInset(NSSize::new(12.0, 10.0));
        tv.setFont(Some(&NSFont::monospacedSystemFontOfSize_weight(11.5, 0.0)));
        tv.setTextColor(Some(&NSColor::labelColor()));
        tv.setDrawsBackground(false);
        tv
    };
    unsafe { scroll_view.setDocumentView(Some(&text_view)) };

    // Bottom Bar (Buttons)
    let bottom_box = unsafe {
        NSView::initWithFrame(
            NSView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(panel_w, 48.0)),
        )
    };
    bottom_box.setTranslatesAutoresizingMaskIntoConstraints(false);

    let copy_btn = unsafe {
        let btn = NSButton::initWithFrame(
            NSButton::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(150.0, 28.0)),
        );
        btn.setTitle(&NSString::from_str("Copy to Clipboard"));
        let _: () = msg_send![&btn, setBezelStyle: 1usize];
        btn.setKeyEquivalent(&NSString::from_str("c"));
        btn.setKeyEquivalentModifierMask(objc2_app_kit::NSEventModifierFlags::Command);
        btn
    };
    copy_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

    let close_btn = unsafe {
        let btn = NSButton::initWithFrame(
            NSButton::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(80.0, 28.0)),
        );
        btn.setTitle(&NSString::from_str("Done"));
        let _: () = msg_send![&btn, setBezelStyle: 1usize];
        btn.setKeyEquivalent(&NSString::from_str("\u{1b}")); // Escape
        btn.setTarget(Some(&*panel));
        btn.setAction(Some(sel!(performClose:)));
        btn
    };
    close_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

    let controller_alloc = mtm.alloc::<OcrPanelController>().set_ivars(OcrPanelIvars {
        panel: panel.clone(),
        text_view: text_view.clone(),
        stats_label: stats_label.clone(),
        copy_btn: copy_btn.clone(),
        base_stats,
    });
    let controller: Retained<OcrPanelController> =
        unsafe { msg_send![super(controller_alloc), init] };

    unsafe {
        copy_btn.setTarget(Some(&*controller));
        copy_btn.setAction(Some(sel!(copyOcrText:)));
    }

    ACTIVE_OCR_CONTROLLER.with(|cell| {
        *cell.borrow_mut() = Some(controller);
    });

    unsafe {
        header_box.addSubview(&title_label);
        header_box.addSubview(&stats_label);
        bottom_box.addSubview(&copy_btn);
        bottom_box.addSubview(&close_btn);

        effect_view.addSubview(&header_box);
        effect_view.addSubview(&scroll_view);
        effect_view.addSubview(&bottom_box);

        panel.setContentView(Some(&effect_view));

        // Auto Layout Constraints
        header_box
            .topAnchor()
            .constraintEqualToAnchor_constant(&effect_view.topAnchor(), 34.0)
            .setActive(true);
        header_box
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&effect_view.leadingAnchor(), 20.0)
            .setActive(true);
        header_box
            .trailingAnchor()
            .constraintEqualToAnchor_constant(&effect_view.trailingAnchor(), -20.0)
            .setActive(true);
        header_box
            .heightAnchor()
            .constraintEqualToConstant(42.0)
            .setActive(true);

        title_label
            .topAnchor()
            .constraintEqualToAnchor(&header_box.topAnchor())
            .setActive(true);
        title_label
            .leadingAnchor()
            .constraintEqualToAnchor(&header_box.leadingAnchor())
            .setActive(true);

        stats_label
            .topAnchor()
            .constraintEqualToAnchor_constant(&title_label.bottomAnchor(), 3.0)
            .setActive(true);
        stats_label
            .leadingAnchor()
            .constraintEqualToAnchor(&header_box.leadingAnchor())
            .setActive(true);

        scroll_view
            .topAnchor()
            .constraintEqualToAnchor_constant(&header_box.bottomAnchor(), 8.0)
            .setActive(true);
        scroll_view
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&effect_view.leadingAnchor(), 20.0)
            .setActive(true);
        scroll_view
            .trailingAnchor()
            .constraintEqualToAnchor_constant(&effect_view.trailingAnchor(), -20.0)
            .setActive(true);
        scroll_view
            .bottomAnchor()
            .constraintEqualToAnchor_constant(&bottom_box.topAnchor(), -10.0)
            .setActive(true);

        bottom_box
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&effect_view.leadingAnchor(), 20.0)
            .setActive(true);
        bottom_box
            .trailingAnchor()
            .constraintEqualToAnchor_constant(&effect_view.trailingAnchor(), -20.0)
            .setActive(true);
        bottom_box
            .bottomAnchor()
            .constraintEqualToAnchor_constant(&effect_view.bottomAnchor(), -12.0)
            .setActive(true);
        bottom_box
            .heightAnchor()
            .constraintEqualToConstant(32.0)
            .setActive(true);

        close_btn
            .trailingAnchor()
            .constraintEqualToAnchor(&bottom_box.trailingAnchor())
            .setActive(true);
        close_btn
            .centerYAnchor()
            .constraintEqualToAnchor(&bottom_box.centerYAnchor())
            .setActive(true);

        copy_btn
            .trailingAnchor()
            .constraintEqualToAnchor_constant(&close_btn.leadingAnchor(), -10.0)
            .setActive(true);
        copy_btn
            .centerYAnchor()
            .constraintEqualToAnchor(&bottom_box.centerYAnchor())
            .setActive(true);
    }

    WindowGeometry::sanitize_window(&panel, WindowKind::Utility, mtm);
    unsafe { panel.makeKeyAndOrderFront(None) };
}
