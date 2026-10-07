import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# 1. Add fields to AppDelegateIvars
new_ivars = """
    history: std::cell::RefCell<Vec<(String, usize)>>,
    history_index: std::cell::Cell<usize>,
    ignore_history_update: std::cell::Cell<bool>,
}"""
content = re.sub(r'welcome_view: OnceCell<Retained<NSView>>,\n\}', r'welcome_view: OnceCell<Retained<NSView>>,\n' + new_ivars, content)

# 2. Add notification observer for PDFView changes in application_did_finish_launching
observer_code = """
            self.ivars().window.set(window).unwrap();
            self.ivars().pdf_view.set(pdf_view).unwrap();
            
            unsafe {
                let nc = objc2_foundation::NSNotificationCenter::defaultCenter();
                nc.addObserver_selector_name_object(
                    &*(self as *const AppDelegate as *const AnyObject),
                    objc2::sel!(pdfPageChanged:),
                    Some(objc2_foundation::NSString::from_str("PDFViewPageChangedNotification")),
                    None
                );
                nc.addObserver_selector_name_object(
                    &*(self as *const AppDelegate as *const AnyObject),
                    objc2::sel!(pdfScaleChanged:),
                    Some(objc2_foundation::NSString::from_str("PDFViewScaleChangedNotification")),
                    None
                );
                nc.addObserver_selector_name_object(
                    &*(self as *const AppDelegate as *const AnyObject),
                    objc2::sel!(pdfDocumentChanged:),
                    Some(objc2_foundation::NSString::from_str("PDFViewDocumentChangedNotification")),
                    None
                );
            }
"""
content = content.replace("self.ivars().window.set(window).unwrap();\n            self.ivars().pdf_view.set(pdf_view).unwrap();", observer_code)

# 3. Handle restoring position inside pdfDidLoad:
pdf_did_load_orig = """                        if let Some(main_vc) = self.ivars().main_vc.get() {
                            unsafe { main_vc.setView(&**pdf_view) };
                        }
                        crate::add_recent_item(&path);"""

pdf_did_load_new = """                        if let Some(main_vc) = self.ivars().main_vc.get() {
                            unsafe { main_vc.setView(&**pdf_view) };
                        }
                        crate::add_recent_item(&path);
                        
                        // Restore position and zoom
                        unsafe {
                            let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
                            let key = objc2_foundation::NSString::from_str(&format!("PDFPos_{}", path));
                            if let Some(dict) = defaults.dictionaryForKey(&key) {
                                let page_idx_key = objc2_foundation::NSString::from_str("PageIndex");
                                let zoom_key = objc2_foundation::NSString::from_str("Zoom");
                                
                                let mut page_idx = 0;
                                let mut zoom = 0.0;
                                
                                if let Some(idx_obj) = dict.objectForKey(&page_idx_key) {
                                    page_idx = objc2::msg_send![idx_obj, unsignedIntegerValue];
                                }
                                if let Some(zoom_obj) = dict.objectForKey(&zoom_key) {
                                    let z: f64 = objc2::msg_send![zoom_obj, doubleValue];
                                    zoom = z;
                                }
                                
                                if page_idx < page_count {
                                    if let Some(page) = doc.pageAtIndex(page_idx) {
                                        pdf_view.goToPage(&page);
                                    }
                                }
                                if zoom > 0.0 {
                                    pdf_view.setScaleFactor(zoom);
                                }
                            }
                        }
                        
                        if !self.ivars().ignore_history_update.get() {
                            let mut hist = self.ivars().history.borrow_mut();
                            let idx = self.ivars().history_index.get();
                            if idx < hist.len() {
                                hist.truncate(idx);
                            }
                            hist.push((path.clone(), 0));
                            self.ivars().history_index.set(hist.len());
                        }
"""
content = content.replace(pdf_did_load_orig, pdf_did_load_new)

# 4. Add methods for notifications
methods_to_add = """
        #[unsafe(method(pdfPageChanged:))]
        fn pdf_page_changed(&self, _notification: &objc2_foundation::NSNotification) {
            let pdf_view = self.ivars().pdf_view.get().unwrap();
            unsafe {
                if let Some(doc) = pdf_view.document() {
                    if let Some(url) = doc.documentURL() {
                        if let Some(page) = pdf_view.currentPage() {
                            let idx = doc.indexForPage(&page);
                            let path = url.path().unwrap().to_string();
                            
                            let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
                            let key = objc2_foundation::NSString::from_str(&format!("PDFPos_{}", path));
                            
                            let dict = objc2_foundation::NSMutableDictionary::new();
                            let page_idx_key = objc2_foundation::NSString::from_str("PageIndex");
                            let zoom_key = objc2_foundation::NSString::from_str("Zoom");
                            
                            let idx_num: *mut AnyObject = objc2::msg_send![objc2::class!(NSNumber), numberWithUnsignedInteger: idx];
                            dict.setObject_forKey(std::mem::transmute(idx_num), &page_idx_key);
                            
                            let zoom_num: *mut AnyObject = objc2::msg_send![objc2::class!(NSNumber), numberWithDouble: pdf_view.scaleFactor()];
                            dict.setObject_forKey(std::mem::transmute(zoom_num), &zoom_key);
                            
                            defaults.setObject_forKey(Some(&dict), &key);
                            
                            if !self.ivars().ignore_history_update.get() {
                                let mut hist = self.ivars().history.borrow_mut();
                                let h_idx = self.ivars().history_index.get();
                                if h_idx > 0 && h_idx <= hist.len() {
                                    let last = &hist[h_idx - 1];
                                    if last.0 == path && last.1 != idx {
                                        if h_idx < hist.len() {
                                            hist.truncate(h_idx);
                                        }
                                        hist.push((path, idx));
                                        self.ivars().history_index.set(hist.len());
                                    } else if last.0 != path {
                                        if h_idx < hist.len() {
                                            hist.truncate(h_idx);
                                        }
                                        hist.push((path, idx));
                                        self.ivars().history_index.set(hist.len());
                                    }
                                } else {
                                    hist.push((path, idx));
                                    self.ivars().history_index.set(hist.len());
                                }
                            }
                        }
                    }
                }
            }
        }

        #[unsafe(method(pdfScaleChanged:))]
        fn pdf_scale_changed(&self, _notification: &objc2_foundation::NSNotification) {
            let pdf_view = self.ivars().pdf_view.get().unwrap();
            unsafe {
                if let Some(doc) = pdf_view.document() {
                    if let Some(url) = doc.documentURL() {
                        if let Some(page) = pdf_view.currentPage() {
                            let idx = doc.indexForPage(&page);
                            let path = url.path().unwrap().to_string();
                            
                            let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
                            let key = objc2_foundation::NSString::from_str(&format!("PDFPos_{}", path));
                            
                            let dict = objc2_foundation::NSMutableDictionary::new();
                            let page_idx_key = objc2_foundation::NSString::from_str("PageIndex");
                            let zoom_key = objc2_foundation::NSString::from_str("Zoom");
                            
                            let idx_num: *mut AnyObject = objc2::msg_send![objc2::class!(NSNumber), numberWithUnsignedInteger: idx];
                            dict.setObject_forKey(std::mem::transmute(idx_num), &page_idx_key);
                            
                            let zoom_num: *mut AnyObject = objc2::msg_send![objc2::class!(NSNumber), numberWithDouble: pdf_view.scaleFactor()];
                            dict.setObject_forKey(std::mem::transmute(zoom_num), &zoom_key);
                            
                            defaults.setObject_forKey(Some(&dict), &key);
                        }
                    }
                }
            }
        }
        
        #[unsafe(method(pdfDocumentChanged:))]
        fn pdf_document_changed(&self, _notification: &objc2_foundation::NSNotification) {
            // Already handling in pdfDidLoad
        }
"""
content = content.replace("#[unsafe(method(applicationDidFinishLaunching:))]", methods_to_add + "\n        #[unsafe(method(applicationDidFinishLaunching:))]")

# 5. Fix goBack and goForward
go_back_orig = """        #[unsafe(method(goBack:))]
        fn go_back(&self, _sender: Option<&AnyObject>) {}"""
        
go_back_new = """        #[unsafe(method(goBack:))]
        fn go_back(&self, _sender: Option<&AnyObject>) {
            let idx = self.ivars().history_index.get();
            if idx > 1 {
                self.ivars().history_index.set(idx - 1);
                let hist = self.ivars().history.borrow();
                let entry = &hist[idx - 2];
                self.ivars().ignore_history_update.set(true);
                
                let path = &entry.0;
                let page_idx = entry.1;
                
                // Load PDF if not already loaded
                let current_path = *self.ivars().pending_pdf_path.borrow();
                if current_path != *path {
                    *self.ivars().pending_pdf_path.borrow_mut() = path.clone();
                    let nsurl = unsafe { objc2_foundation::NSURL::fileURLWithPath(&objc2_foundation::NSString::from_str(path)) };
                    let doc = unsafe { objc2_pdf_kit::PDFDocument::initWithURL(objc2_pdf_kit::PDFDocument::alloc(), &nsurl) };
                    if let Some(doc) = doc {
                        let pdf_view = self.ivars().pdf_view.get().unwrap();
                        unsafe { pdf_view.setDocument(Some(&doc)) };
                        if let Some(page) = unsafe { doc.pageAtIndex(page_idx) } {
                            unsafe { pdf_view.goToPage(&page) };
                        }
                        if let Some(title) = unsafe { nsurl.lastPathComponent() } {
                            self.ivars().window.get().unwrap().setTitle(&title);
                        }
                    }
                } else {
                    if let Some(pdf_view) = self.ivars().pdf_view.get() {
                        if let Some(doc) = unsafe { pdf_view.document() } {
                            if let Some(page) = unsafe { doc.pageAtIndex(page_idx) } {
                                unsafe { pdf_view.goToPage(&page) };
                            }
                        }
                    }
                }
                
                self.ivars().ignore_history_update.set(false);
            }
        }"""

go_forward_orig = """        #[unsafe(method(goForward:))]
        fn go_forward(&self, _sender: Option<&AnyObject>) {}"""
        
go_forward_new = """        #[unsafe(method(goForward:))]
        fn go_forward(&self, _sender: Option<&AnyObject>) {
            let idx = self.ivars().history_index.get();
            let hist = self.ivars().history.borrow();
            if idx < hist.len() {
                self.ivars().history_index.set(idx + 1);
                let entry = &hist[idx];
                self.ivars().ignore_history_update.set(true);
                
                let path = &entry.0;
                let page_idx = entry.1;
                
                let current_path = *self.ivars().pending_pdf_path.borrow();
                if current_path != *path {
                    *self.ivars().pending_pdf_path.borrow_mut() = path.clone();
                    let nsurl = unsafe { objc2_foundation::NSURL::fileURLWithPath(&objc2_foundation::NSString::from_str(path)) };
                    let doc = unsafe { objc2_pdf_kit::PDFDocument::initWithURL(objc2_pdf_kit::PDFDocument::alloc(), &nsurl) };
                    if let Some(doc) = doc {
                        let pdf_view = self.ivars().pdf_view.get().unwrap();
                        unsafe { pdf_view.setDocument(Some(&doc)) };
                        if let Some(page) = unsafe { doc.pageAtIndex(page_idx) } {
                            unsafe { pdf_view.goToPage(&page) };
                        }
                        if let Some(title) = unsafe { nsurl.lastPathComponent() } {
                            self.ivars().window.get().unwrap().setTitle(&title);
                        }
                    }
                } else {
                    if let Some(pdf_view) = self.ivars().pdf_view.get() {
                        if let Some(doc) = unsafe { pdf_view.document() } {
                            if let Some(page) = unsafe { doc.pageAtIndex(page_idx) } {
                                unsafe { pdf_view.goToPage(&page) };
                            }
                        }
                    }
                }
                
                self.ivars().ignore_history_update.set(false);
            }
        }"""

content = content.replace(go_back_orig, go_back_new)
content = content.replace(go_forward_orig, go_forward_new)


with open('src/main.rs', 'w') as f:
    f.write(content)

