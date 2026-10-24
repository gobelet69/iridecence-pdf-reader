use crate::settings::{PREF_DEFAULT_HIGHLIGHT_COLOR, SettingsManager};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{AnyThread, msg_send};
use objc2_app_kit::{NSColor, NSModalResponseOK, NSSavePanel, NSWindow};
use objc2_foundation::{MainThreadMarker, NSArray, NSRect, NSString, NSURL};
use objc2_pdf_kit::{PDFAnnotation, PDFDocument, PDFPage, PDFView};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnotationType {
    Highlight,
    Underline,
    Strikeout,
    TextNote,
}

impl AnnotationType {
    pub fn subtype_string(&self) -> &'static str {
        match self {
            AnnotationType::Highlight => "Highlight",
            AnnotationType::Underline => "Underline",
            AnnotationType::Strikeout => "StrikeOut",
            AnnotationType::TextNote => "Text",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AnnotationType::Highlight => "Highlight",
            AnnotationType::Underline => "Underline",
            AnnotationType::Strikeout => "Strikeout",
            AnnotationType::TextNote => "Note",
        }
    }

    pub fn system_symbol_name(&self) -> &'static str {
        match self {
            AnnotationType::Highlight => "highlighter",
            AnnotationType::Underline => "underline",
            AnnotationType::Strikeout => "strikethrough",
            AnnotationType::TextNote => "note.text",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnnotationItem {
    pub page_index: usize,
    pub annot_type: AnnotationType,
    pub contents: String,
    pub bounds: NSRect,
}

/// Adds an annotation to the current text selection of a PDFView.
/// Returns true if an annotation was added, false otherwise.
pub fn add_annotation_to_selection(
    pdf_view: &PDFView,
    annot_type: AnnotationType,
    custom_note: Option<&str>,
) -> bool {
    let sel = match unsafe { pdf_view.currentSelection() } {
        Some(s) => s,
        None => return false,
    };

    let _doc = match unsafe { pdf_view.document() } {
        Some(d) => d,
        None => return false,
    };

    let pages: Retained<NSArray<PDFPage>> = match unsafe { sel.pages() } {
        pages if !pages.is_empty() => pages,
        _ => return false,
    };

    let sel_string = unsafe { sel.string() }
        .map(|s| s.to_string())
        .unwrap_or_default();
    let type_str = NSString::from_str(annot_type.subtype_string());

    let color = match annot_type {
        AnnotationType::Highlight => unsafe {
            let color_idx = SettingsManager::int_pref(PREF_DEFAULT_HIGHLIGHT_COLOR, 0);
            match color_idx {
                1 => NSColor::colorWithSRGBRed_green_blue_alpha(0.35, 0.90, 0.45, 0.75), // Green
                2 => NSColor::colorWithSRGBRed_green_blue_alpha(0.35, 0.70, 1.0, 0.75),  // Blue
                3 => NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 0.45, 0.75, 0.75),  // Pink
                _ => NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 0.88, 0.25, 0.75),  // Yellow
            }
        },
        AnnotationType::Underline => unsafe {
            NSColor::colorWithSRGBRed_green_blue_alpha(0.2, 0.5, 1.0, 0.85)
        },
        AnnotationType::Strikeout => unsafe {
            NSColor::colorWithSRGBRed_green_blue_alpha(0.9, 0.2, 0.2, 0.85)
        },
        AnnotationType::TextNote => unsafe {
            NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 0.85, 0.1, 0.95)
        },
    };

    let mut added = false;
    let line_sels: Retained<NSArray<objc2_pdf_kit::PDFSelection>> =
        unsafe { msg_send![&*sel, selectionsByLine] };

    let pages_count = pages.count();

    for i in 0..pages_count {
        let page = unsafe { pages.objectAtIndex(i) };
        let page_bounds = unsafe { sel.boundsForPage(&page) };

        if page_bounds.size.width <= 0.0 || page_bounds.size.height <= 0.0 {
            continue;
        }

        // Collect all distinct line bounding rects on this page
        let mut line_rects: Vec<NSRect> = Vec::new();
        if !line_sels.is_empty() {
            for l_idx in 0..line_sels.count() {
                let line_sel = unsafe { line_sels.objectAtIndex(l_idx) };
                let lb = unsafe { line_sel.boundsForPage(&page) };
                if lb.size.width > 0.0 && lb.size.height > 0.0 {
                    line_rects.push(lb);
                }
            }
        }
        if line_rects.is_empty() {
            line_rects.push(page_bounds);
        }

        let annot_bounds = if annot_type == AnnotationType::TextNote {
            NSRect::new(
                objc2_foundation::NSPoint::new(
                    page_bounds.origin.x + page_bounds.size.width + 6.0,
                    page_bounds.origin.y,
                ),
                objc2_foundation::NSSize::new(20.0, 20.0),
            )
        } else {
            page_bounds
        };

        let annot: Retained<PDFAnnotation> = unsafe {
            msg_send![
                PDFAnnotation::alloc(),
                initWithBounds: annot_bounds,
                forType: &*type_str,
                withProperties: std::ptr::null::<AnyObject>()
            ]
        };

        unsafe {
            annot.setColor(&color);
            if annot_type != AnnotationType::TextNote {
                let mut quad_points: Vec<Retained<AnyObject>> =
                    Vec::with_capacity(line_rects.len() * 4);
                for rect in &line_rects {
                    let p_tl: Retained<AnyObject> = msg_send![
                        objc2::class!(NSValue),
                        valueWithPoint: objc2_foundation::NSPoint::new(rect.origin.x, rect.origin.y + rect.size.height)
                    ];
                    let p_tr: Retained<AnyObject> = msg_send![
                        objc2::class!(NSValue),
                        valueWithPoint: objc2_foundation::NSPoint::new(rect.origin.x + rect.size.width, rect.origin.y + rect.size.height)
                    ];
                    let p_bl: Retained<AnyObject> = msg_send![
                        objc2::class!(NSValue),
                        valueWithPoint: objc2_foundation::NSPoint::new(rect.origin.x, rect.origin.y)
                    ];
                    let p_br: Retained<AnyObject> = msg_send![
                        objc2::class!(NSValue),
                        valueWithPoint: objc2_foundation::NSPoint::new(rect.origin.x + rect.size.width, rect.origin.y)
                    ];
                    quad_points.push(p_tl);
                    quad_points.push(p_tr);
                    quad_points.push(p_bl);
                    quad_points.push(p_br);
                }
                let quad = NSArray::from_retained_slice(&quad_points);
                let _: () = msg_send![&annot, setQuadrilateralPoints: &*quad];
            }

            let clean_sel: String = sel
                .string()
                .map(|s| s.to_string())
                .unwrap_or_else(|| sel_string.clone())
                .replace(['\n', '\r'], " ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            let contents_text = if let Some(note) = custom_note {
                note.to_string()
            } else if !clean_sel.is_empty() {
                clean_sel
            } else {
                annot_type.display_name().to_string()
            };
            if annot_type == AnnotationType::TextNote {
                annot.setContents(Some(&NSString::from_str(&contents_text)));
            } else {
                let _: () = msg_send![&annot, setContents: &*NSString::from_str(&contents_text)];
                let _: () = msg_send![&annot, setUserName: &*NSString::from_str(&contents_text)];
            }
            let _: () = msg_send![&annot, setShouldDisplay: true];
            let _: () = msg_send![&annot, setShouldPrint: true];
            page.addAnnotation(&annot);
            let _: () = msg_send![pdf_view, annotationsChangedOnPage: &*page];
        }
        added = true;
    }

    if added {
        unsafe {
            pdf_view.layoutDocumentView();
            if let Some(doc_view) = pdf_view.documentView() {
                doc_view.setNeedsDisplay(true);
            }
            pdf_view.setNeedsDisplay(true);
        }
    }

    added
}

/// Retrieves all user-visible annotations from the document.
pub fn get_document_annotations(doc: &PDFDocument) -> Vec<AnnotationItem> {
    let mut results = Vec::new();
    let page_count = unsafe { doc.pageCount() };

    for page_idx in 0..page_count {
        if let Some(page) = unsafe { doc.pageAtIndex(page_idx) } {
            let annots: Retained<NSArray<PDFAnnotation>> = unsafe { page.annotations() };
            let count = annots.count();
            for a_idx in 0..count {
                let annot = unsafe { annots.objectAtIndex(a_idx) };
                if let Some(t) = unsafe { annot.r#type() } {
                    let t_str = t.to_string();
                    let annot_type = if t_str == "Highlight" {
                        Some(AnnotationType::Highlight)
                    } else if t_str == "Underline" {
                        Some(AnnotationType::Underline)
                    } else if t_str == "StrikeOut" {
                        Some(AnnotationType::Strikeout)
                    } else if t_str == "Text" {
                        Some(AnnotationType::TextNote)
                    } else {
                        None
                    };

                    if let Some(annot_type) = annot_type {
                        let contents = unsafe {
                            let c = annot.contents().map(|s| s.to_string()).unwrap_or_default();
                            if !c.trim().is_empty() {
                                c
                            } else {
                                let u: Option<Retained<NSString>> = msg_send![&annot, userName];
                                u.map(|s| s.to_string())
                                    .filter(|s| !s.trim().is_empty())
                                    .unwrap_or_else(|| annot_type.display_name().to_string())
                            }
                        };
                        let bounds = unsafe { annot.bounds() };
                        results.push(AnnotationItem {
                            page_index: page_idx,
                            annot_type,
                            contents,
                            bounds,
                        });
                    }
                }
            }
        }
    }

    results
}

/// Safely and atomically saves the PDF document to the specified file path.
/// Guarantees that protected documents (e.g. in Documents/MASTER) are NEVER overwritten.
pub fn save_document_safely(doc: &PDFDocument, path: &str) -> Result<String, String> {
    if path.is_empty() {
        return Err("No destination path specified".to_string());
    }

    // STRICT INVARIANT: MASTER directory is STRICTLY READ-ONLY!
    if path.contains("/Documents/MASTER") {
        return Err(
            "Cannot overwrite files in MASTER. Please use Save As to choose an external location."
                .to_string(),
        );
    }

    let _parent_dir = match Path::new(path).parent() {
        Some(p) => p,
        None => return Err("Invalid path destination".to_string()),
    };

    let pid = std::process::id();
    let tmp_path = format!("{}.tmp.{}", path, pid);
    let tmp_url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&tmp_path)) };

    // Write to temporary atomic location
    let write_ok: bool = unsafe {
        let res: bool = msg_send![doc, writeToURL: &*tmp_url];
        res
    };

    if !write_ok {
        let _ = std::fs::remove_file(&tmp_path);
        return Err("Failed to serialize PDF data".to_string());
    }

    // Verify written file size > 0
    let meta = match std::fs::metadata(&tmp_path) {
        Ok(m) => m,
        Err(e) => {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("Verification failed: {}", e));
        }
    };

    if meta.len() == 0 {
        let _ = std::fs::remove_file(&tmp_path);
        return Err("Verification failed: temporary file is 0 bytes".to_string());
    }

    // Atomic replace
    if let Err(e) = std::fs::rename(&tmp_path, path) {
        // Fallback: copy and remove in case cross-filesystem
        if let Err(copy_err) = std::fs::copy(&tmp_path, path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!(
                "Atomic rename failed: {} (copy error: {})",
                e, copy_err
            ));
        }
        let _ = std::fs::remove_file(&tmp_path);
    }

    Ok(path.to_string())
}

/// Presents a native NSSavePanel to save the document to a user-chosen location.
pub fn show_save_panel(
    _window: &NSWindow,
    doc: &PDFDocument,
    default_name: &str,
    mtm: MainThreadMarker,
) -> Option<String> {
    let panel = unsafe { NSSavePanel::savePanel(mtm) };
    crate::window_geometry::WindowGeometry::sanitize_window(
        &panel,
        crate::window_geometry::WindowKind::Utility,
        mtm,
    );

    let filename = if default_name.ends_with(".pdf") {
        default_name.to_string()
    } else {
        format!("{}.pdf", default_name)
    };

    unsafe {
        panel.setNameFieldStringValue(&NSString::from_str(&filename));
        panel.setCanCreateDirectories(true);
        panel.setTitle(Some(&NSString::from_str("Save PDF Document")));
        panel.setMessage(Some(&NSString::from_str(
            "Choose a location to save your annotated document:",
        )));
    }

    let response = unsafe { panel.runModal() };
    if response == NSModalResponseOK {
        if let Some(url) = unsafe { panel.URL() } {
            if let Some(path) = unsafe { url.path() } {
                let path_str = path.to_string();
                // Ensure it does not target MASTER
                if path_str.contains("/Documents/MASTER") {
                    let alert = objc2_app_kit::NSAlert::new(mtm);
                    unsafe {
                        alert.setMessageText(&NSString::from_str("Invalid Destination"));
                        alert.setInformativeText(&NSString::from_str(
                            "The MASTER directory is strictly read-only. Please choose another folder.",
                        ));
                        alert.addButtonWithTitle(&NSString::from_str("OK"));
                        alert.runModal();
                    }
                    return None;
                }

                if save_document_safely(doc, &path_str).is_ok() {
                    return Some(path_str);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_foundation::{NSPoint, NSSize};

    #[test]
    fn test_annotation_type_subtypes() {
        assert_eq!(AnnotationType::Highlight.subtype_string(), "Highlight");
        assert_eq!(AnnotationType::Underline.subtype_string(), "Underline");
        assert_eq!(AnnotationType::Strikeout.subtype_string(), "StrikeOut");
        assert_eq!(AnnotationType::TextNote.subtype_string(), "Text");
    }

    #[test]
    fn test_annotation_type_symbols() {
        assert_eq!(
            AnnotationType::Highlight.system_symbol_name(),
            "highlighter"
        );
        assert_eq!(AnnotationType::Underline.system_symbol_name(), "underline");
        assert_eq!(
            AnnotationType::Strikeout.system_symbol_name(),
            "strikethrough"
        );
        assert_eq!(AnnotationType::TextNote.system_symbol_name(), "note.text");
    }

    #[test]
    fn test_annotation_type_display_names() {
        assert_eq!(AnnotationType::Highlight.display_name(), "Highlight");
        assert_eq!(AnnotationType::Underline.display_name(), "Underline");
        assert_eq!(AnnotationType::Strikeout.display_name(), "Strikeout");
        assert_eq!(AnnotationType::TextNote.display_name(), "Note");
    }

    #[test]
    fn test_multiline_annotation_quadrilateral_points() {
        let line1 = NSRect::new(NSPoint::new(50.0, 100.0), NSSize::new(200.0, 14.0));
        let line2 = NSRect::new(NSPoint::new(50.0, 80.0), NSSize::new(250.0, 14.0));
        let union_bounds = NSRect::new(NSPoint::new(50.0, 80.0), NSSize::new(250.0, 34.0));

        let type_str = NSString::from_str("Highlight");
        let annot: Retained<PDFAnnotation> = unsafe {
            msg_send![
                PDFAnnotation::alloc(),
                initWithBounds: union_bounds,
                forType: &*type_str,
                withProperties: std::ptr::null::<AnyObject>()
            ]
        };

        let mut quad_points: Vec<Retained<AnyObject>> = Vec::with_capacity(8);
        for rect in &[line1, line2] {
            unsafe {
                let p_tl: Retained<AnyObject> = msg_send![
                    objc2::class!(NSValue),
                    valueWithPoint: NSPoint::new(rect.origin.x, rect.origin.y + rect.size.height)
                ];
                let p_tr: Retained<AnyObject> = msg_send![
                    objc2::class!(NSValue),
                    valueWithPoint: NSPoint::new(rect.origin.x + rect.size.width, rect.origin.y + rect.size.height)
                ];
                let p_bl: Retained<AnyObject> = msg_send![
                    objc2::class!(NSValue),
                    valueWithPoint: NSPoint::new(rect.origin.x, rect.origin.y)
                ];
                let p_br: Retained<AnyObject> = msg_send![
                    objc2::class!(NSValue),
                    valueWithPoint: NSPoint::new(rect.origin.x + rect.size.width, rect.origin.y)
                ];
                quad_points.push(p_tl);
                quad_points.push(p_tr);
                quad_points.push(p_bl);
                quad_points.push(p_br);
            }
        }
        let quad = NSArray::from_retained_slice(&quad_points);
        unsafe {
            let _: () = msg_send![&annot, setQuadrilateralPoints: &*quad];
        }

        let q_out: *mut AnyObject = unsafe { msg_send![&annot, quadrilateralPoints] };
        assert!(!q_out.is_null());
        let count: usize = unsafe { msg_send![q_out, count] };
        assert_eq!(
            count, 8,
            "Expected 8 quad points for 2 selected lines (4 points per line)"
        );
    }
}
