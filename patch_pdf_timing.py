import re

with open("src/main.rs", "r") as f:
    main = f.read()

replacement = """                                if !already_open {
                                    let t0 = std::time::Instant::now();
                                    
                                    // 2. Filesystem access latency check
                                    let meta = std::fs::metadata(url.path().unwrap().to_string());
                                    let t1 = std::time::Instant::now();
                                    
                                    // 3. Init PDFDocument
                                    let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                                    let t2 = std::time::Instant::now();
                                    
                                    if let Some(doc) = doc {
                                        // 4. Assign doc to PDFView
                                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(&doc)) };
                                        let t3 = std::time::Instant::now();
                                        
                                        println!("[PDF TIMING] Click to Metadata: {:?}", t1 - t0);
                                        println!("[PDF TIMING] PDFDocument::initWithURL: {:?}", t2 - t1);
                                        println!("[PDF TIMING] PDFView::setDocument: {:?}", t3 - t2);
                                        println!("[PDF TIMING] Total sync time: {:?}", t3 - t0);
                                    }
                                }"""

main = main.replace(
    """                                if !already_open {
                                    let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                                    if let Some(doc) = doc {
                                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(&doc)) };
                                    }
                                }""",
    replacement
)

with open("src/main.rs", "w") as f:
    f.write(main)
