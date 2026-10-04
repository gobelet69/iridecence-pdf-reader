import re

with open("src/main.rs", "r") as f:
    code = f.read()

replacement = """            let thumb_view = unsafe { PDFThumbnailView::initWithFrame(PDFThumbnailView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0))) };
            unsafe {
                thumb_view.setPDFView(Some(&pdf_view));
                // Adaptive thumbnail sizes
                thumb_view.setThumbnailSize(NSSize::new(140.0, 198.0));
                
                // We use msg_send! to bypass missing bindings in objc2_pdf_kit
                let _: () = msg_send![&thumb_view, setMaximumNumberOfColumns: 1usize];
                
                // We can't reduce vertical spacing directly on PDFThumbnailView in standard AppKit easily, 
                // but setting a suitable thumbnail size helps.
                thumb_view.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
            }"""

code = re.sub(
    r"            let thumb_view = unsafe \{ PDFThumbnailView::initWithFrame\([\s\S]*?thumb_view\.setAutoresizingMask\([\s\S]*?\);\n            \}",
    replacement,
    code
)

with open("src/main.rs", "w") as f:
    f.write(code)
