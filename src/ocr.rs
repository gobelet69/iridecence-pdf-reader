use crate::window_geometry::{WindowGeometry, WindowKind};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadOnly, msg_send};
use objc2_app_kit::{
    NSButton, NSColor, NSFont, NSPanel, NSScrollView, NSTextField, NSTextView, NSView,
    NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
    NSWindowStyleMask,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSDictionary, NSPoint, NSRect, NSSize, NSString,
};
use objc2_pdf_kit::{PDFDisplayBox, PDFPage};

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
        let request: *mut AnyObject = msg_send![request_class, alloc];
        let request: *mut AnyObject = msg_send![request, init];
        if request.is_null() {
            return Err("Failed to allocate VNRecognizeTextRequest".to_string());
        }

        // recognitionLevel = 1 (VNRequestTextRecognitionLevelAccurate)
        let _: () = msg_send![request, setRecognitionLevel: 1usize];
        let _: () = msg_send![request, setUsesLanguageCorrection: true];

        let req_ref: &AnyObject = &*request;
        let requests_array = NSArray::from_slice(&[req_ref]);

        let handler_class = objc2::class!(VNImageRequestHandler);
        let handler: *mut AnyObject = msg_send![handler_class, alloc];
        let empty_dict = NSDictionary::<NSString, AnyObject>::new();
        let handler: *mut AnyObject =
            msg_send![handler, initWithData: tiff_data, options: &*empty_dict];
        if handler.is_null() {
            return Err("Failed to allocate VNImageRequestHandler".to_string());
        }

        let mut error: *mut AnyObject = std::ptr::null_mut();
        let success: bool =
            msg_send![handler, performRequests: &*requests_array, error: &mut error];
        if !success {
            return Err("Vision performRequests failed".to_string());
        }

        let results: *mut NSArray<AnyObject> = msg_send![request, results];
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
    let screen_vis = WindowGeometry::screen_visible_frame(None, mtm);
    let panel_w = 640.0f64.min(screen_vis.size.width * 0.85);
    let panel_h = 520.0f64.min(screen_vis.size.height * 0.85);

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
        panel.setTitle(&NSString::from_str("Recognized Text (OCR)"));
        panel.setTitlebarAppearsTransparent(true);
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

    let stats_label = unsafe {
        let s = format!(
            "Page {} of {} • {} lines • {} words • Copied to Clipboard ✓",
            page_num + 1,
            total_pages,
            line_count,
            word_count
        );
        let lbl = NSTextField::labelWithString(&NSString::from_str(&s), mtm);
        lbl.setFont(Some(&NSFont::systemFontOfSize(11.0)));
        lbl.setTextColor(Some(&NSColor::secondaryLabelColor()));
        lbl
    };
    stats_label.setTranslatesAutoresizingMaskIntoConstraints(false);

    // Scrollable TextView
    let scroll_view = unsafe {
        let sv = NSScrollView::initWithFrame(
            NSScrollView::alloc(mtm),
            NSRect::new(
                NSPoint::new(16.0, 52.0),
                NSSize::new(panel_w - 32.0, panel_h - 116.0),
            ),
        );
        sv.setHasVerticalScroller(true);
        sv.setDrawsBackground(false);
        sv.setAutohidesScrollers(true);
        sv
    };
    scroll_view.setTranslatesAutoresizingMaskIntoConstraints(false);

    let text_view = unsafe {
        let tv = NSTextView::initWithFrame(
            NSTextView::alloc(mtm),
            NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(panel_w - 32.0, panel_h - 116.0),
            ),
        );
        tv.setString(&NSString::from_str(text));
        tv.setEditable(false);
        tv.setSelectable(true);
        tv.setFont(Some(&NSFont::monospacedSystemFontOfSize_weight(12.0, 0.0)));
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
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(140.0, 28.0)),
        );
        btn.setTitle(&NSString::from_str("Copy to Clipboard"));
        btn.setBezelStyle(objc2_app_kit::NSBezelStyle::Rounded);
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
        btn.setBezelStyle(objc2_app_kit::NSBezelStyle::Rounded);
        btn.setKeyEquivalent(&NSString::from_str("\u{1b}")); // Escape
        btn.setTarget(Some(&*panel));
        btn.setAction(Some(objc2::sel!(performClose:)));
        btn
    };
    close_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

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
            .constraintEqualToAnchor_constant(&effect_view.topAnchor(), 20.0)
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
            .constraintEqualToConstant(44.0)
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
            .constraintEqualToAnchor_constant(&title_label.bottomAnchor(), 4.0)
            .setActive(true);
        stats_label
            .leadingAnchor()
            .constraintEqualToAnchor(&header_box.leadingAnchor())
            .setActive(true);

        scroll_view
            .topAnchor()
            .constraintEqualToAnchor_constant(&header_box.bottomAnchor(), 10.0)
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
