use objc2::{msg_send, rc::Retained};
use objc2_pdf_kit::{PDFOutline, PDFDocument};

pub fn check_outline(doc: &PDFDocument) {
    let outline: Option<Retained<PDFOutline>> = unsafe { doc.outlineRoot() };
    if let Some(out) = outline {
        let count = unsafe { out.numberOfChildren() };
        if count > 0 {
            let child: Option<Retained<PDFOutline>> = unsafe { out.childAtIndex(0) };
            if let Some(c) = child {
                let lbl = unsafe { c.label() };
                let _dst = unsafe { c.destination() };
                let _act = unsafe { c.action() };
            }
        }
    }
}
