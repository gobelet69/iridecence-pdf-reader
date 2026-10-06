import re
with open('src/main.rs', 'r') as f:
    content = f.read()

pdf_did_load = """
        #[unsafe(method(pdfDidLoad:))]
        fn pdf_did_load(&self, doc: Option<&objc2_pdf_kit::PDFDocument>) {
            println!("[DIAGNOSTIC] PDF DID LOAD ENTER");
            if let Some(doc) = doc {
                let main_thread = std::thread::current().name() == Some("main");
                println!("[DIAGNOSTIC] main thread = {}", main_thread);
                
                if let Some(url) = unsafe { doc.documentURL() } {
                    let path = url.path().unwrap().to_string();
                    println!("[DIAGNOSTIC] path = {}", path);
                    if path == *self.ivars().pending_pdf_path.borrow() {
                        println!("[DIAGNOSTIC] PDFVIEW BEFORE setDocument");
                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(doc)) };
                        
                        let page_count = unsafe { doc.pageCount() };
                        println!("[DIAGNOSTIC] PDFVIEW AFTER setDocument: pageCount = {}", page_count);
                    } else {
                        println!("[DIAGNOSTIC] Discarded obsolete doc: {}", path);
                    }
                }
            } else {
                println!("[DIAGNOSTIC] doc is None");
            }
        }
"""

content = content.replace("#[unsafe(method(workspaceSelectionChanged:))]", pdf_did_load + "\n        #[unsafe(method(workspaceSelectionChanged:))]")

with open('src/main.rs', 'w') as f:
    f.write(content)
