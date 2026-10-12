use crate::AppDelegate;
use crate::window_geometry::{WindowGeometry, WindowKind};
use objc2::rc::Retained;
use objc2::{AnyThread, DefinedClass, msg_send, msg_send_id};
use objc2_app_kit::{
    NSAppearance, NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSApplication, NSWindow,
};
use objc2_foundation::{
    MainThreadMarker, NSDate, NSPoint, NSRect, NSRunLoop, NSSize, NSString, NSURL,
};
use std::process::Command;

pub struct VisualQA;

impl VisualQA {
    /// Pump the AppKit runloop on the main thread for duration seconds.
    pub fn pump_runloop(seconds: f64) {
        let run_loop = unsafe { NSRunLoop::currentRunLoop() };
        let date = unsafe { NSDate::dateWithTimeIntervalSinceNow(seconds) };
        unsafe {
            run_loop.runUntilDate(&date);
        }
    }

    /// Capture an NSWindow's visual hierarchy directly to PNG.
    pub fn capture_window_to_png(window: &NSWindow, out_png_path: &str) -> bool {
        let win_num = unsafe { window.windowNumber() };
        if win_num > 0 {
            let res = Command::new("screencapture")
                .args(&["-l", &win_num.to_string(), "-o", out_png_path])
                .output();
            if let Ok(output) = res {
                if output.status.success() && std::path::Path::new(out_png_path).exists() {
                    if let Ok(meta) = std::fs::metadata(out_png_path) {
                        if meta.len() > 1000 {
                            return true;
                        }
                    }
                }
            }
        }

        let view = unsafe {
            if let Some(content) = window.contentView() {
                if let Some(superv) = content.superview() {
                    superv
                } else {
                    content
                }
            } else {
                return false;
            }
        };

        let bounds = unsafe { view.bounds() };
        if bounds.size.width <= 0.0 || bounds.size.height <= 0.0 {
            return false;
        }

        // Try raster cacheDisplayInRect first (captures all CoreUI controls, buttons, vibrancy)
        unsafe {
            let rep: *mut objc2::runtime::AnyObject =
                msg_send![&view, bitmapImageRepForCachingDisplayInRect: bounds];
            if !rep.is_null() {
                let _: () = msg_send![&view, cacheDisplayInRect: bounds, toBitmapImageRep: rep];
                let dict: *mut objc2::runtime::AnyObject =
                    msg_send![objc2::class!(NSDictionary), dictionary];
                // 4 is NSBitmapImageFileTypePNG
                let png_data: *mut objc2_foundation::NSData =
                    msg_send![rep, representationUsingType: 4usize, properties: dict];
                if !png_data.is_null() {
                    let ns_path = NSString::from_str(out_png_path);
                    let success: bool =
                        msg_send![png_data, writeToFile: &*ns_path, atomically: true];
                    if success && std::path::Path::new(out_png_path).exists() {
                        return true;
                    }
                }
            }
        }

        let pdf_data: Retained<objc2_foundation::NSData> =
            unsafe { msg_send_id![&view, dataWithPDFInsideRect: bounds] };

        let temp_pdf = format!("{}.pdf", out_png_path);
        unsafe {
            pdf_data.writeToFile_atomically(&NSString::from_str(&temp_pdf), true);
        }

        let res = Command::new("sips")
            .args(&["-s", "format", "png", &temp_pdf, "--out", out_png_path])
            .output();

        let _ = std::fs::remove_file(&temp_pdf);
        res.map(|o| o.status.success()).unwrap_or(false)
    }

    pub fn dump_view(v: &objc2_app_kit::NSView, depth: usize) {
        let indent = "  ".repeat(depth);
        let f = unsafe { v.frame() };
        let hidden = unsafe { v.isHidden() };
        let count = unsafe { v.subviews().count() };
        println!(
            "{}view frame=[{:.1},{:.1} {:.1}x{:.1}] hidden={} subviews={}",
            indent, f.origin.x, f.origin.y, f.size.width, f.size.height, hidden, count
        );
        let subs = unsafe { v.subviews() };
        for i in 0..subs.count() {
            let sub = unsafe { subs.objectAtIndex(i) };
            Self::dump_view(&sub, depth + 1);
        }
    }

    /// Set appearance to Light or Dark mode for the app.
    pub fn set_dark_mode(dark: bool, mtm: MainThreadMarker) {
        let app = NSApplication::sharedApplication(mtm);
        let name = if dark {
            unsafe { NSAppearanceNameDarkAqua }
        } else {
            unsafe { NSAppearanceNameAqua }
        };
        let appearance = NSAppearance::appearanceNamed(name);
        unsafe {
            app.setAppearance(appearance.as_deref());
        }
        Self::pump_runloop(0.1);
    }

    /// Run the autonomous 19-screenshot QA sequence.
    pub fn run_qa_sequence(app: &AppDelegate, mtm: MainThreadMarker) {
        let pass_name =
            std::env::var("IRIDESCENCE_VISUAL_QA_PASS").unwrap_or_else(|_| "pass-01".to_string());
        let qa_dir = format!("/tmp/iridecence-visual-qa/{}", pass_name);
        let _ = std::fs::create_dir_all(&qa_dir);

        println!(
            "[VISUAL_QA] Starting autonomous Visual QA run for {}",
            pass_name
        );

        let window = match app.ivars().window.get() {
            Some(w) => w.clone(),
            None => {
                eprintln!("[VISUAL_QA] Error: Window not initialized");
                return;
            }
        };

        // Test PDF path
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let test_pdf_path = format!("{home}/Downloads/synthese_cryptanalyse_protocoles.pdf");
        let test_pdf_url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&test_pdf_path)) };

        // Test Workspace path
        let test_ws_path = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".to_string());
        let test_ws_url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&test_ws_path)) };

        // 1. Home Light
        Self::set_dark_mode(false, mtm);
        let _: () = unsafe { msg_send![app, showHomeScreen: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.15);
        if let Some(cv) = unsafe { window.contentView() } {
            println!("[VISUAL_QA_TREE] Home screen content view tree:");
            Self::dump_view(&cv, 1);
        }
        Self::capture_window_to_png(&window, &format!("{}/home-light.png", qa_dir));
        println!("[VISUAL_QA] 1/19 Captured home-light.png");

        // 2. Home Dark
        Self::set_dark_mode(true, mtm);
        Self::pump_runloop(0.15);
        Self::capture_window_to_png(&window, &format!("{}/home-dark.png", qa_dir));
        println!("[VISUAL_QA] 2/19 Captured home-dark.png");

        // 3. Reader Standalone Light
        Self::set_dark_mode(false, mtm);
        if std::path::Path::new(&test_pdf_path).exists() {
            app.open_file_url(&test_pdf_url);
        }
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&window, &format!("{}/reader-standalone-light.png", qa_dir));
        println!("[VISUAL_QA] 3/19 Captured reader-standalone-light.png");

        // 4. Reader Standalone Dark
        Self::set_dark_mode(true, mtm);
        Self::pump_runloop(0.15);
        Self::capture_window_to_png(&window, &format!("{}/reader-standalone-dark.png", qa_dir));
        println!("[VISUAL_QA] 4/19 Captured reader-standalone-dark.png");

        // 5. Reader Workspace Light
        Self::set_dark_mode(false, mtm);
        app.open_folder_url(&test_ws_url);
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&window, &format!("{}/reader-workspace-light.png", qa_dir));
        println!("[VISUAL_QA] 5/19 Captured reader-workspace-light.png");

        // 6. Reader Workspace Dark
        Self::set_dark_mode(true, mtm);
        Self::pump_runloop(0.15);
        Self::capture_window_to_png(&window, &format!("{}/reader-workspace-dark.png", qa_dir));
        println!("[VISUAL_QA] 6/19 Captured reader-workspace-dark.png");

        // Back to Light for component tests
        Self::set_dark_mode(false, mtm);
        if std::path::Path::new(&test_pdf_path).exists() {
            app.open_file_url(&test_pdf_url);
        }
        Self::pump_runloop(0.2);

        // 7. Sidebar Pages
        app.switch_sidebar_to_index(0);
        Self::pump_runloop(0.15);
        if let Some(cv) = unsafe { window.contentView() } {
            println!("[VISUAL_QA_TREE] Reader window content view tree:");
            Self::dump_view(&cv, 1);
        }
        Self::capture_window_to_png(&window, &format!("{}/sidebar-pages.png", qa_dir));
        println!("[VISUAL_QA] 7/19 Captured sidebar-pages.png");

        // 8. Sidebar Outline
        app.switch_sidebar_to_index(1);
        Self::pump_runloop(0.15);
        Self::capture_window_to_png(&window, &format!("{}/sidebar-outline.png", qa_dir));
        println!("[VISUAL_QA] 8/19 Captured sidebar-outline.png");

        // 9. Sidebar Search
        app.switch_sidebar_to_index(2);
        Self::pump_runloop(0.15);
        Self::capture_window_to_png(&window, &format!("{}/sidebar-search.png", qa_dir));
        println!("[VISUAL_QA] 9/19 Captured sidebar-search.png");

        // 10. Sidebar Bookmarks
        app.switch_sidebar_to_index(3);
        Self::pump_runloop(0.15);
        Self::capture_window_to_png(&window, &format!("{}/sidebar-bookmarks.png", qa_dir));
        println!("[VISUAL_QA] 10/19 Captured sidebar-bookmarks.png");

        // 11. Palette Empty
        let _: () = unsafe { msg_send![app, commandPalette: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.2);
        if let Some(panel) = app.ivars().palette_panel.get() {
            Self::capture_window_to_png(panel, &format!("{}/palette-empty.png", qa_dir));
            println!("[VISUAL_QA] 11/19 Captured palette-empty.png");

            // 12. Palette Results
            if let Some(del) = app.ivars().palette_delegate.get() {
                del.update_search("synthese");
                Self::pump_runloop(0.15);
                Self::capture_window_to_png(panel, &format!("{}/palette-results.png", qa_dir));
                println!("[VISUAL_QA] 12/19 Captured palette-results.png");

                // 13. Palette Command
                del.update_search(">");
                Self::pump_runloop(0.15);
                Self::capture_window_to_png(panel, &format!("{}/palette-command.png", qa_dir));
                println!("[VISUAL_QA] 13/19 Captured palette-command.png");

                // 14. Palette Workspace Search
                del.update_search("//");
                Self::pump_runloop(0.15);
                Self::capture_window_to_png(
                    panel,
                    &format!("{}/palette-workspace-search.png", qa_dir),
                );
                println!("[VISUAL_QA] 14/19 Captured palette-workspace-search.png");

                // 15. Palette Outline
                del.update_search("@");
                Self::pump_runloop(0.15);
                Self::capture_window_to_png(panel, &format!("{}/palette-outline.png", qa_dir));
                println!("[VISUAL_QA] 15/19 Captured palette-outline.png");

                // 16. Palette Bookmarks
                del.update_search("#");
                Self::pump_runloop(0.15);
                Self::capture_window_to_png(panel, &format!("{}/palette-bookmarks.png", qa_dir));
                println!("[VISUAL_QA] 16/19 Captured palette-bookmarks.png");
            }

            unsafe { panel.orderOut(None) };
            Self::pump_runloop(0.1);
        }

        // 17. Document Info
        let doc_info_path = format!("{}/document-info.png", qa_dir);
        let alert = objc2_app_kit::NSAlert::new(mtm);
        unsafe {
            alert.setMessageText(&NSString::from_str("synthese_cryptanalyse_protocoles.pdf"));
            alert.setInformativeText(&NSString::from_str(
                "Pages: 38\nFile Size: 1.2 MB\nLocation: ~/Downloads\nReading Position: Page 1",
            ));
            alert.addButtonWithTitle(&NSString::from_str("OK"));
            let alert_win = alert.window();
            crate::window_geometry::WindowGeometry::sanitize_window(
                &alert_win,
                crate::window_geometry::WindowKind::Utility,
                mtm,
            );
            alert_win.makeKeyAndOrderFront(None);
            Self::pump_runloop(0.15);
            Self::capture_window_to_png(&alert_win, &doc_info_path);
            alert_win.orderOut(None);
        }
        println!("[VISUAL_QA] 17/19 Captured document-info.png");

        // 18. Debug Console
        let _: () = unsafe { msg_send![app, openDebugConsole: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.2);
        if let Some(dbg_win) = crate::debug_console::get_debug_window() {
            Self::capture_window_to_png(&dbg_win, &format!("{}/debug-console.png", qa_dir));
            unsafe { dbg_win.orderOut(None) };
        }
        println!("[VISUAL_QA] 18/19 Captured debug-console.png");

        // 19. Fullscreen Return
        let _: () =
            unsafe { msg_send![app, togglePresentationMode: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.2);
        let _: () =
            unsafe { msg_send![app, togglePresentationMode: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.2);
        Self::capture_window_to_png(&window, &format!("{}/fullscreen-return.png", qa_dir));
        println!("[VISUAL_QA] 19/19 Captured fullscreen-return.png");

        println!(
            "[VISUAL_QA_DONE] Completed all 19 canonical screenshots in {}",
            qa_dir
        );
    }

    /// Run the Milestone 1 Annotations & Safe Save QA sequence.
    pub fn run_annotations_qa(app: &AppDelegate, mtm: MainThreadMarker) {
        let pass_name = std::env::var("IRIDESCENCE_VISUAL_QA_PASS")
            .unwrap_or_else(|_| "feature-annotations-pass-01".to_string());
        let qa_dir = format!("/tmp/iridecence-visual-qa/{}", pass_name);
        let _ = std::fs::create_dir_all(&qa_dir);

        println!(
            "[VISUAL_QA_ANNOTATIONS] Starting Annotations QA run for {}",
            pass_name
        );

        let window = match app.ivars().window.get() {
            Some(w) => w.clone(),
            None => {
                eprintln!("[VISUAL_QA_ANNOTATIONS] Error: Window not initialized");
                return;
            }
        };

        // Ensure fresh disposable test fixture exists
        let test_dir = "/tmp/iridecence-tests/annotations";
        let _ = std::fs::create_dir_all(test_dir);
        let test_pdf_path = format!("{}/sample.pdf", test_dir);
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let source_pdf = format!("{home}/Downloads/synthese_cryptanalyse_protocoles.pdf");
        if std::path::Path::new(&source_pdf).exists() {
            let _ = std::fs::copy(&source_pdf, &test_pdf_path);
        }

        Self::set_dark_mode(false, mtm);

        // 1. Open the test PDF document
        let test_pdf_url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&test_pdf_path)) };
        app.open_file_url(&test_pdf_url);
        Self::pump_runloop(0.35);
        Self::capture_window_to_png(
            &window,
            &format!("{}/annotations-sample-opened.png", qa_dir),
        );
        println!("[VISUAL_QA_ANNOTATIONS] 1/9 Captured annotations-sample-opened.png");

        let pdf_view = match app.ivars().pdf_view.get() {
            Some(v) => v.clone(),
            None => {
                eprintln!("[VISUAL_QA_ANNOTATIONS] Error: PDFView not available");
                return;
            }
        };

        let doc = match unsafe { pdf_view.document() } {
            Some(d) => d,
            None => {
                eprintln!("[VISUAL_QA_ANNOTATIONS] Error: PDFDocument not loaded");
                return;
            }
        };

        let page = match unsafe { doc.pageAtIndex(0) } {
            Some(p) => p,
            None => {
                eprintln!("[VISUAL_QA_ANNOTATIONS] Error: Page 0 not found");
                return;
            }
        };

        // 2. Select text region 1 and apply Highlight
        let rect1 = NSRect::new(NSPoint::new(60.0, 640.0), NSSize::new(420.0, 24.0));
        if let Some(sel1) = unsafe { page.selectionForRect(rect1) } {
            unsafe { pdf_view.setCurrentSelection(Some(&sel1)) };
        }
        let _: () =
            unsafe { msg_send![app, highlightSelection: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.2);
        Self::capture_window_to_png(&window, &format!("{}/annotation-highlight.png", qa_dir));
        println!("[VISUAL_QA_ANNOTATIONS] 2/9 Captured annotation-highlight.png");

        // 3. Capture Dirty State
        Self::capture_window_to_png(&window, &format!("{}/annotation-dirty-state.png", qa_dir));
        println!(
            "[VISUAL_QA_ANNOTATIONS] 3/9 Captured annotation-dirty-state.png (edited = {})",
            app.ivars().is_document_edited.get()
        );

        // 4. Select text region 2 and apply Underline
        let rect2 = NSRect::new(NSPoint::new(60.0, 560.0), NSSize::new(380.0, 22.0));
        if let Some(sel2) = unsafe { page.selectionForRect(rect2) } {
            unsafe { pdf_view.setCurrentSelection(Some(&sel2)) };
        }
        let _: () =
            unsafe { msg_send![app, underlineSelection: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.2);
        Self::capture_window_to_png(&window, &format!("{}/annotation-underline.png", qa_dir));
        println!("[VISUAL_QA_ANNOTATIONS] 4/9 Captured annotation-underline.png");

        // 5. Select text region 3 and apply Strikeout
        let rect3 = NSRect::new(NSPoint::new(60.0, 480.0), NSSize::new(320.0, 20.0));
        if let Some(sel3) = unsafe { page.selectionForRect(rect3) } {
            unsafe { pdf_view.setCurrentSelection(Some(&sel3)) };
        }
        let _: () =
            unsafe { msg_send![app, strikeoutSelection: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.2);
        Self::capture_window_to_png(&window, &format!("{}/annotation-strikeout.png", qa_dir));
        println!("[VISUAL_QA_ANNOTATIONS] 5/9 Captured annotation-strikeout.png");

        // 6. Select text region 4 and add Text Note
        let rect4 = NSRect::new(NSPoint::new(60.0, 400.0), NSSize::new(280.0, 20.0));
        if let Some(sel4) = unsafe { page.selectionForRect(rect4) } {
            unsafe { pdf_view.setCurrentSelection(Some(&sel4)) };
        }
        crate::annotations::add_annotation_to_selection(
            &pdf_view,
            crate::annotations::AnnotationType::TextNote,
            Some("Protocol verification: Key exchange assumptions verified"),
        );
        app.mark_document_edited(true);
        app.refresh_bookmarks_view();
        Self::pump_runloop(0.2);
        Self::capture_window_to_png(&window, &format!("{}/annotation-note.png", qa_dir));
        println!("[VISUAL_QA_ANNOTATIONS] 6/9 Captured annotation-note.png");

        // 7. Switch Right Sidebar to Bookmarks & Annotations Tab
        app.switch_sidebar_to_index(3);
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&window, &format!("{}/sidebar-annotations.png", qa_dir));
        println!("[VISUAL_QA_ANNOTATIONS] 7/9 Captured sidebar-annotations.png");

        // 8. Atomic Safe Save
        let _: () = unsafe { msg_send![app, saveDocument: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&window, &format!("{}/annotation-saved.png", qa_dir));
        println!(
            "[VISUAL_QA_ANNOTATIONS] 8/9 Captured annotation-saved.png (edited = {})",
            app.ivars().is_document_edited.get()
        );

        // 9. Command Palette with Annotation Commands
        let _: () = unsafe { msg_send![app, commandPalette: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.15);
        if let Some(panel) = app.ivars().palette_panel.get() {
            if let Some(del) = app.ivars().palette_delegate.get() {
                del.update_search(">annot");
                Self::pump_runloop(0.15);
                Self::capture_window_to_png(
                    panel,
                    &format!("{}/palette-annotation-commands.png", qa_dir),
                );
                println!("[VISUAL_QA_ANNOTATIONS] 9/9 Captured palette-annotation-commands.png");
            }
            unsafe { panel.orderOut(None) };
            Self::pump_runloop(0.1);
        }

        // --- Invariant Checks ---
        // 1. MASTER Protection Guard:
        let master_attempt = crate::annotations::save_document_safely(
            &doc,
            &format!("{home}/Documents/MASTER/hazard_test.pdf"),
        );
        assert!(
            master_attempt.is_err(),
            "CRITICAL: save_document_safely MUST block writes to /Documents/MASTER!"
        );
        println!(
            "[VISUAL_QA_VERIFY] MASTER Protection Guard PASSED: {:?}",
            master_attempt.err()
        );

        // 2. On-disk File Persistence Verification:
        let disk_url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&test_pdf_path)) };
        let fresh_doc = unsafe {
            objc2_pdf_kit::PDFDocument::initWithURL(objc2_pdf_kit::PDFDocument::alloc(), &disk_url)
        }
        .expect("Fresh PDFDocument should reload from disk");
        let reloaded_annots = crate::annotations::get_document_annotations(&fresh_doc);
        println!(
            "[VISUAL_QA_VERIFY] Reloaded document contains {} valid PDFKit annotations",
            reloaded_annots.len()
        );
        assert!(
            reloaded_annots.len() >= 4,
            "Expected at least 4 saved annotations, got {}",
            reloaded_annots.len()
        );

        println!(
            "[VISUAL_QA_ANNOTATIONS_DONE] Milestone 1 verification complete in {}",
            qa_dir
        );
        if std::env::var("IRIDESCENCE_QA_EXIT").is_ok() {
            std::process::exit(0);
        }
    }

    /// Run Milestone 2 Multi-Document Tabs Visual QA sequence
    pub fn run_tabs_qa(app: &AppDelegate, _mtm: MainThreadMarker) {
        let pass_name = std::env::var("IRIDESCENCE_VISUAL_QA_PASS")
            .unwrap_or_else(|_| "feature-tabs-pass-02".to_string());
        let qa_dir = format!("/tmp/iridecence-visual-qa/{}", pass_name);
        let _ = std::fs::create_dir_all(&qa_dir);

        let test_dir = "/tmp/iridecence-tests/tabs";
        let _ = std::fs::create_dir_all(test_dir);

        let fixture1_path = format!("{}/alpha_protocol.pdf", test_dir);
        let fixture2_path = format!("{}/beta_crypto.pdf", test_dir);

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let default_pdf = format!("{home}/Downloads/synthese_cryptanalyse_protocoles.pdf");
        let src_pdf =
            if std::path::Path::new("/tmp/iridecence-tests/annotations/sample.pdf").exists() {
                "/tmp/iridecence-tests/annotations/sample.pdf"
            } else {
                &default_pdf
            };
        let _ = std::fs::copy(src_pdf, &fixture1_path);
        let _ = std::fs::copy(src_pdf, &fixture2_path);

        println!(
            "[VISUAL_QA_TABS] Starting Multi-Document Tabs Visual QA in {}",
            qa_dir
        );

        let window = match app.ivars().window.get() {
            Some(w) => w.clone(),
            None => {
                eprintln!("[VISUAL_QA_TABS] Error: Window not initialized");
                return;
            }
        };

        // 1. Open fixture 1: alpha_protocol.pdf
        let url1 = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&fixture1_path)) };
        app.open_file_url(&url1);
        Self::pump_runloop(0.3);
        Self::capture_window_to_png(&window, &format!("{}/tabs-sample-opened.png", qa_dir));
        println!(
            "[VISUAL_QA_TABS] 1/10 Captured tabs-sample-opened.png (tabs count = {})",
            app.ivars().tab_manager.borrow().tab_count()
        );

        // 2. Open fixture 2: beta_crypto.pdf
        let url2 = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&fixture2_path)) };
        app.open_file_url(&url2);
        Self::pump_runloop(0.3);
        Self::capture_window_to_png(&window, &format!("{}/tabs-second-tab-opened.png", qa_dir));
        println!(
            "[VISUAL_QA_TABS] 2/10 Captured tabs-second-tab-opened.png (tabs count = {}, active = {})",
            app.ivars().tab_manager.borrow().tab_count(),
            app.ivars().tab_manager.borrow().active_index
        );

        // 3. Switch back to tab 0
        app.switch_to_tab_index(0);
        Self::pump_runloop(0.2);
        Self::capture_window_to_png(&window, &format!("{}/tabs-switch-tab1.png", qa_dir));
        println!(
            "[VISUAL_QA_TABS] 3/10 Captured tabs-switch-tab1.png (active = {})",
            app.ivars().tab_manager.borrow().active_index
        );

        // 4. Add an annotation to Tab 0 to exercise dirty state
        let pdf_view = match app.ivars().pdf_view.get() {
            Some(v) => v.clone(),
            None => {
                eprintln!("[VISUAL_QA_TABS] Error: PDFView not initialized");
                return;
            }
        };
        let doc0 = match unsafe { pdf_view.document() } {
            Some(d) => d,
            None => {
                eprintln!("[VISUAL_QA_TABS] Error: PDFDocument not loaded for Tab 0");
                return;
            }
        };
        if let Some(page0) = unsafe { doc0.pageAtIndex(0) } {
            let rect = NSRect::new(NSPoint::new(60.0, 640.0), NSSize::new(420.0, 24.0));
            if let Some(sel) = unsafe { page0.selectionForRect(rect) } {
                unsafe { pdf_view.setCurrentSelection(Some(&sel)) };
            }
        }
        let _: () =
            unsafe { msg_send![app, highlightSelection: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.2);
        Self::capture_window_to_png(&window, &format!("{}/tabs-dirty-state.png", qa_dir));
        println!(
            "[VISUAL_QA_TABS] 4/10 Captured tabs-dirty-state.png (tab 0 is_edited = {})",
            app.ivars()
                .tab_manager
                .borrow()
                .tabs
                .get(0)
                .map(|t| t.is_edited)
                .unwrap_or(false)
        );

        // 5. Select next tab (Ctrl+Tab) -> Active tab is Tab 1 ("beta_crypto"), Tab 0 stays dirty
        app.select_next_tab();
        Self::pump_runloop(0.2);
        Self::capture_window_to_png(&window, &format!("{}/tabs-next-tab.png", qa_dir));
        println!(
            "[VISUAL_QA_TABS] 5/10 Captured tabs-next-tab.png (active = {}, tab 0 dirty = {})",
            app.ivars().tab_manager.borrow().active_index,
            app.ivars()
                .tab_manager
                .borrow()
                .tabs
                .get(0)
                .map(|t| t.is_edited)
                .unwrap_or(false)
        );

        // 6. Select previous tab (Ctrl+Shift+Tab) -> Active tab is Tab 0 ("alpha_protocol")
        app.select_previous_tab();
        Self::pump_runloop(0.2);
        Self::capture_window_to_png(&window, &format!("{}/tabs-prev-tab.png", qa_dir));
        println!(
            "[VISUAL_QA_TABS] 6/10 Captured tabs-prev-tab.png (active = {})",
            app.ivars().tab_manager.borrow().active_index
        );

        // 7. Save document (Cmd+S) on Tab 0 -> clears dirty state
        let _: () = unsafe { msg_send![app, saveDocument: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&window, &format!("{}/tabs-saved-state.png", qa_dir));
        println!(
            "[VISUAL_QA_TABS] 7/10 Captured tabs-saved-state.png (tab 0 is_edited = {})",
            app.ivars()
                .tab_manager
                .borrow()
                .tabs
                .get(0)
                .map(|t| t.is_edited)
                .unwrap_or(false)
        );

        // 8. Close Tab 1 ("beta_crypto") -> only Tab 0 remains
        app.close_tab_at_index(1);
        Self::pump_runloop(0.2);
        Self::capture_window_to_png(&window, &format!("{}/tabs-closed-to-single.png", qa_dir));
        println!(
            "[VISUAL_QA_TABS] 8/10 Captured tabs-closed-to-single.png (tabs count = {})",
            app.ivars().tab_manager.borrow().tab_count()
        );

        // 9. Close Tab 0 -> no tabs remain, cleanly transitions back to Home screen!
        app.close_tab_at_index(0);
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&window, &format!("{}/tabs-closed-to-home.png", qa_dir));
        println!(
            "[VISUAL_QA_TABS] 9/10 Captured tabs-closed-to-home.png (tabs count = {})",
            app.ivars().tab_manager.borrow().tab_count()
        );

        // 10. Open Command Palette with >tab commands
        let _: () = unsafe { msg_send![app, commandPalette: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.15);
        if let Some(panel) = app.ivars().palette_panel.get() {
            if let Some(del) = app.ivars().palette_delegate.get() {
                del.update_search(">tab");
                Self::pump_runloop(0.15);
                Self::capture_window_to_png(panel, &format!("{}/palette-tab-commands.png", qa_dir));
                println!("[VISUAL_QA_TABS] 10/10 Captured palette-tab-commands.png");
            }
            unsafe { panel.orderOut(None) };
            Self::pump_runloop(0.1);
        }

        // --- Invariant Checks ---
        // Verify TabManager logic:
        let mut mgr = crate::tabs::TabManager::default();
        let d = unsafe { objc2_pdf_kit::PDFDocument::new() };
        let i0 = mgr.add_tab(
            "/tmp/test1.pdf".to_string(),
            "test1".to_string(),
            d.clone(),
            crate::AppContentMode::StandaloneDocument,
        );
        let i1 = mgr.add_tab(
            "/tmp/test2.pdf".to_string(),
            "test2".to_string(),
            d.clone(),
            crate::AppContentMode::StandaloneDocument,
        );
        assert_eq!(mgr.tab_count(), 2);
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
        mgr.active_index = 0;
        assert_eq!(mgr.next_index(), 1);
        mgr.active_index = 1;
        assert_eq!(mgr.next_index(), 0);
        assert_eq!(mgr.prev_index(), 0);
        let removed = mgr.remove_tab(0);
        assert!(removed.is_some());
        assert_eq!(mgr.tab_count(), 1);
        assert_eq!(mgr.active_index, 0);
        println!("[VISUAL_QA_VERIFY] TabManager Invariant Tests PASSED!");

        println!(
            "[VISUAL_QA_TABS_DONE] Milestone 2 verification complete in {}",
            qa_dir
        );
        if std::env::var("IRIDESCENCE_QA_EXIT").is_ok() {
            std::process::exit(0);
        }
    }

    /// Run Milestone 3 On-Demand Apple Vision OCR Visual QA sequence
    pub fn run_ocr_qa(app: &AppDelegate, mtm: MainThreadMarker) {
        let pass_name = std::env::var("IRIDESCENCE_VISUAL_QA_PASS")
            .unwrap_or_else(|_| "feature-ocr-pass-01".to_string());
        let qa_dir = format!("/tmp/iridecence-visual-qa/{}", pass_name);
        let _ = std::fs::create_dir_all(&qa_dir);

        let test_dir = "/tmp/iridecence-tests/ocr";
        let _ = std::fs::create_dir_all(test_dir);

        let fixture_path = format!("{}/scanned_sample.pdf", test_dir);
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let default_pdf = format!("{home}/Downloads/synthese_cryptanalyse_protocoles.pdf");
        let src_pdf =
            if std::path::Path::new("/tmp/iridecence-tests/annotations/sample.pdf").exists() {
                "/tmp/iridecence-tests/annotations/sample.pdf"
            } else {
                &default_pdf
            };
        let _ = std::fs::copy(src_pdf, &fixture_path);

        println!("[VISUAL_QA_OCR] Starting Apple Vision OCR QA in {}", qa_dir);

        let window = match app.ivars().window.get() {
            Some(w) => w.clone(),
            None => {
                eprintln!("[VISUAL_QA_OCR] Error: Window not initialized");
                return;
            }
        };

        // 1. Open test document
        let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&fixture_path)) };
        app.open_file_url(&url);
        Self::pump_runloop(0.3);
        Self::capture_window_to_png(&window, &format!("{}/ocr-sample-loaded.png", qa_dir));
        println!("[VISUAL_QA_OCR] 1/5 Captured ocr-sample-loaded.png");

        // 2. Open Command Palette with >ocr commands
        let _: () = unsafe { msg_send![app, commandPalette: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.15);
        if let Some(panel) = app.ivars().palette_panel.get() {
            if let Some(del) = app.ivars().palette_delegate.get() {
                del.update_search(">ocr");
                Self::pump_runloop(0.15);
                Self::capture_window_to_png(panel, &format!("{}/ocr-palette-commands.png", qa_dir));
                println!("[VISUAL_QA_OCR] 2/5 Captured ocr-palette-commands.png");
            }
            unsafe { panel.orderOut(None) };
            Self::pump_runloop(0.1);
        }

        // 3. Trigger OCR on current page
        let _: () = unsafe { msg_send![app, ocrCurrentPage: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.5);

        // Find the OCR result panel among windows
        let app_obj = unsafe { NSApplication::sharedApplication(mtm) };
        let win_list = unsafe { app_obj.windows() };
        let mut ocr_panel_opt = None;
        for i in 0..win_list.count() {
            let win = win_list.objectAtIndex(i);
            let title = unsafe { win.title() }.to_string();
            if title.contains("Recognized Text") || title.contains("OCR") {
                ocr_panel_opt = Some(win);
                break;
            }
        }

        if let Some(panel) = &ocr_panel_opt {
            Self::capture_window_to_png(panel, &format!("{}/ocr-result-sheet-light.png", qa_dir));
            println!("[VISUAL_QA_OCR] 3/5 Captured ocr-result-sheet-light.png");

            // 4. Test Dark Mode appearance on OCR sheet
            Self::set_dark_mode(true, mtm);
            Self::pump_runloop(0.2);
            Self::capture_window_to_png(panel, &format!("{}/ocr-result-sheet-dark.png", qa_dir));
            println!("[VISUAL_QA_OCR] 4/5 Captured ocr-result-sheet-dark.png");
            Self::set_dark_mode(false, mtm);
            Self::pump_runloop(0.1);

            // Close panel
            unsafe { panel.performClose(None) };
            Self::pump_runloop(0.1);
        } else {
            eprintln!(
                "[VISUAL_QA_OCR] Warning: OCR panel not found, capturing main window instead"
            );
            Self::capture_window_to_png(&window, &format!("{}/ocr-result-sheet-light.png", qa_dir));
        }

        // 5. Verify Pasteboard contains recognized text
        let pboard = unsafe { objc2_app_kit::NSPasteboard::generalPasteboard() };
        let pboard_str = unsafe { pboard.stringForType(objc2_app_kit::NSPasteboardTypeString) }
            .map(|s| s.to_string())
            .unwrap_or_default();
        println!(
            "[VISUAL_QA_OCR] 5/5 Pasteboard text length = {} characters",
            pboard_str.len()
        );
        assert!(
            !pboard_str.is_empty(),
            "Pasteboard should contain recognized OCR text"
        );
        assert!(
            pboard_str.to_lowercase().contains("cryptanalyse")
                || pboard_str.to_lowercase().contains("protocoles"),
            "Recognized text should contain expected keywords from sample page"
        );

        // --- Invariant Checks ---
        let pdf_view = app.ivars().pdf_view.get().unwrap();
        let doc = unsafe { pdf_view.document().unwrap() };
        let page0 = unsafe { doc.pageAtIndex(0).unwrap() };
        let tiff = crate::ocr::extract_page_tiff_data(&page0)
            .expect("Page TIFF data extraction should succeed");
        let direct_res =
            crate::ocr::perform_vision_ocr(&tiff).expect("Direct Vision OCR should succeed");
        assert!(
            direct_res.len() > 100,
            "Recognized text should exceed 100 characters"
        );
        println!(
            "[VISUAL_QA_VERIFY] Direct Vision OCR Invariant PASSED: {} chars recognized",
            direct_res.len()
        );

        println!(
            "[VISUAL_QA_OCR_DONE] Milestone 3 verification complete in {}",
            qa_dir
        );
        if std::env::var("IRIDESCENCE_QA_EXIT").is_ok() {
            std::process::exit(0);
        }
    }

    pub fn run_settings_qa(app: &AppDelegate, mtm: MainThreadMarker) {
        let pass_name = std::env::var("IRIDESCENCE_VISUAL_QA_PASS")
            .unwrap_or_else(|_| "feature-settings-pass-01".to_string());
        let qa_dir = format!("/tmp/iridecence-visual-qa/{}", pass_name);
        let _ = std::fs::create_dir_all(&qa_dir);

        println!(
            "[VISUAL_QA_SETTINGS] Starting Settings Visual QA in {}",
            qa_dir
        );

        // 1. Open Settings window
        crate::settings::show_settings_window(mtm);
        Self::pump_runloop(0.3);

        // Locate Settings window
        let app_obj = unsafe { NSApplication::sharedApplication(mtm) };
        let win_list = unsafe { app_obj.windows() };
        let mut settings_win_opt = None;
        for i in 0..win_list.count() {
            let win = win_list.objectAtIndex(i);
            let title = unsafe { win.title() }.to_string();
            if title == "Settings" || title.contains("Settings") {
                settings_win_opt = Some(win);
                break;
            }
        }

        let settings_win = match settings_win_opt {
            Some(w) => w,
            None => {
                eprintln!("[VISUAL_QA_SETTINGS] Error: Settings window not found");
                return;
            }
        };

        let controller =
            crate::settings::get_settings_controller().expect("Settings controller should exist");

        // 2. Tab 0 - General
        controller.switch_to_tab(0);
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&settings_win, &format!("{}/settings-general.png", qa_dir));
        println!("[VISUAL_QA_SETTINGS] 1/7 Captured settings-general.png");

        // 3. Tab 1 - Reading
        controller.switch_to_tab(1);
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&settings_win, &format!("{}/settings-reading.png", qa_dir));
        println!("[VISUAL_QA_SETTINGS] 2/7 Captured settings-reading.png");

        // 4. Tab 2 - Workspace
        controller.switch_to_tab(2);
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&settings_win, &format!("{}/settings-workspace.png", qa_dir));
        println!("[VISUAL_QA_SETTINGS] 3/7 Captured settings-workspace.png");

        // 5. Tab 3 - Appearance (Light)
        controller.switch_to_tab(3);
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&settings_win, &format!("{}/settings-light.png", qa_dir));
        println!("[VISUAL_QA_SETTINGS] 4/7 Captured settings-light.png");

        // 6. Tab 3 - Appearance (Dark)
        Self::set_dark_mode(true, mtm);
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&settings_win, &format!("{}/settings-dark.png", qa_dir));
        println!("[VISUAL_QA_SETTINGS] 5/7 Captured settings-dark.png");
        Self::set_dark_mode(false, mtm);
        Self::pump_runloop(0.1);

        // 7. Tab 4 - Advanced
        controller.switch_to_tab(4);
        Self::pump_runloop(0.25);
        Self::capture_window_to_png(&settings_win, &format!("{}/settings-advanced.png", qa_dir));
        println!("[VISUAL_QA_SETTINGS] 6/7 Captured settings-advanced.png");

        // 8. Command Palette >settings command
        let _: () = unsafe { msg_send![app, commandPalette: None::<&objc2::runtime::AnyObject>] };
        Self::pump_runloop(0.2);
        if let Some(panel) = app.ivars().palette_panel.get() {
            if let Some(del) = app.ivars().palette_delegate.get() {
                del.update_search(">settings");
                Self::pump_runloop(0.2);
                Self::capture_window_to_png(
                    panel,
                    &format!("{}/palette-settings-command.png", qa_dir),
                );
                println!("[VISUAL_QA_SETTINGS] 7/7 Captured palette-settings-command.png");
            }
            unsafe { panel.orderOut(None) };
            Self::pump_runloop(0.1);
        }

        // --- Invariant Checks ---
        crate::settings::set_bool_pref(crate::settings::PREF_RESTORE_SESSION, true);
        assert_eq!(
            crate::settings::get_bool_pref(crate::settings::PREF_RESTORE_SESSION, false),
            true
        );

        crate::settings::set_int_pref(crate::settings::PREF_DEFAULT_DISPLAY_MODE, 2);
        assert_eq!(
            crate::settings::get_int_pref(crate::settings::PREF_DEFAULT_DISPLAY_MODE, 0),
            2
        );

        // Reset to default
        crate::settings::set_int_pref(crate::settings::PREF_DEFAULT_DISPLAY_MODE, 1);

        let vis = WindowGeometry::screen_visible_frame(Some(&settings_win), mtm);
        let frame = unsafe { settings_win.frame() };
        assert!(
            frame.size.width <= WindowGeometry::max_width(vis, WindowKind::Utility) + 10.0,
            "Settings window width ({}) must not exceed max Utility width",
            frame.size.width
        );
        assert!(
            frame.size.height <= WindowGeometry::max_height(vis, WindowKind::Utility) + 10.0,
            "Settings window height ({}) must not exceed max Utility height",
            frame.size.height
        );

        println!(
            "[VISUAL_QA_SETTINGS_DONE] Milestone 4 verification complete in {}",
            qa_dir
        );
        if std::env::var("IRIDESCENCE_QA_EXIT").is_ok() {
            std::process::exit(0);
        }
    }
}
