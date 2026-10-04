import re

with open("src/main.rs", "r") as f:
    main = f.read()

# Add pending_pdf_path to ivars
main = main.replace(
    "pending_url: std::cell::RefCell<Option<Retained<objc2_foundation::NSURL>>>,",
    "pending_url: std::cell::RefCell<Option<Retained<objc2_foundation::NSURL>>>,\n    pending_pdf_path: std::cell::RefCell<String>,"
)

# Add pdfDidLoad method to AppDelegate
pdf_load_method = """
        #[unsafe(method(pdfDidLoad:))]
        fn pdf_did_load(&self, doc: Option<&PDFDocument>) {
            if let Some(doc) = doc {
                if let Some(url) = unsafe { doc.documentURL() } {
                    let path = url.path().unwrap().to_string();
                    if path == *self.ivars().pending_pdf_path.borrow() {
                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(doc)) };
                        println!("[PDF TIMING] Async load complete and applied for: {}", path);
                    } else {
                        println!("[PDF TIMING] Discarded obsolete doc: {}", path);
                    }
                }
            }
        }

        #[unsafe(method(workspaceSelectionChanged:))]"""

main = main.replace(
    "#[unsafe(method(workspaceSelectionChanged:))]",
    pdf_load_method
)

# Modify workspace_selection_changed
async_load = """                                if !already_open {
                                    let url_str = url.path().unwrap().to_string();
                                    *self.ivars().pending_pdf_path.borrow_mut() = url_str.clone();
                                    let app_ptr = self as *const AppDelegate as usize;
                                    
                                    std::thread::spawn(move || {
                                        let t0 = std::time::Instant::now();
                                        
                                        // 2. Filesystem access latency check
                                        let _meta = std::fs::metadata(&url_str);
                                        let t1 = std::time::Instant::now();
                                        
                                        // 3. Init PDFDocument
                                        let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };
                                        
                                        // Check iCloud downloading status here if needed
                                        
                                        let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &nsurl) };
                                        let t2 = std::time::Instant::now();
                                        
                                        println!("[PDF TIMING] Click to Metadata: {:?}", t1 - t0);
                                        println!("[PDF TIMING] PDFDocument::initWithURL: {:?}", t2 - t1);
                                        
                                        if let Some(doc) = doc {
                                            let doc_any = unsafe { objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(doc) };
                                            let app_raw = app_ptr as *mut objc2::runtime::AnyObject;
                                            let _: () = unsafe {
                                                objc2::msg_send![app_raw, performSelectorOnMainThread: objc2::sel!(pdfDidLoad:), withObject: &*doc_any, waitUntilDone: false]
                                            };
                                        }
                                    });
                                }"""

# We need to replace the original block
main = main.replace(
    """                                if !already_open {
                                    let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                                    if let Some(doc) = doc {
                                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(&doc)) };
                                    }
                                }""",
    async_load
)

with open("src/main.rs", "w") as f:
    f.write(main)
