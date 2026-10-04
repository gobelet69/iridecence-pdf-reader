import re
with open('src/main.rs', 'r') as f:
    content = f.read()

start_idx = content.find("#[unsafe(method(workspaceSelectionChanged:))]")
end_idx = content.find("#[unsafe(method(applicationDidBecomeActive:))]")

if start_idx != -1 and end_idx != -1:
    new_func = """
        #[unsafe(method(workspaceSelectionChanged:))]
        fn workspace_selection_changed(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(outline) = sender.downcast_ref::<NSOutlineView>() {
                    let mtm = MainThreadMarker::new().unwrap();
                    let app = NSApplication::sharedApplication(mtm);
                    let mut is_click = false;
                    if let Some(event) = app.currentEvent() {
                        let ev_type = event.r#type();
                        if ev_type == objc2_app_kit::NSEventType::LeftMouseDown || ev_type == objc2_app_kit::NSEventType::LeftMouseUp {
                            is_click = true;
                        }
                    }
                    if !is_click { return; }
                    
                    let selected_row = outline.selectedRow();
                    if selected_row >= 0 {
                        if let Some(item) = unsafe { outline.itemAtRow(selected_row) } {
                            let node: &crate::workspace::FileNode = unsafe { &*(&*item as *const AnyObject as *const crate::workspace::FileNode) };
                            if !node.ivars().is_dir.get() {
                                let url_str = node.ivars().path.borrow().clone();
                                let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };
                                
                                let current_doc = unsafe { self.ivars().pdf_view.get().unwrap().document() };
                                let mut already_open = false;
                                if let Some(current_doc) = current_doc {
                                    if let Some(current_url) = unsafe { current_doc.documentURL() } {
                                        if current_url.path().unwrap().to_string() == url_str { already_open = true; }
                                    }
                                }
                                
                                if !already_open {
                                    let icloud_status = node.ivars().icloud_status.get();
                                    
                                    println!("[DIAGNOSTIC] CLICK: path = {}, old_state = {}", url_str, icloud_status);
                                    
                                    if icloud_status == 1 || icloud_status == 4 { // CloudOnly or Failed
                                        // 1. Set to DownloadRequested
                                        node.ivars().icloud_status.set(2);
                                        println!("[DIAGNOSTIC] MODEL UPDATE: old_state = {}, new_state = {}", icloud_status, 2);
                                        
                                        // 2. Reload row immediately
                                        let _: () = unsafe { objc2::msg_send![outline, reloadItem: &*item, reloadChildren: false] };
                                        println!("[DIAGNOSTIC] ROW RELOAD: row = {}", selected_row);
                                        
                                        // 3. Mark as pending open
                                        *self.ivars().pending_pdf_path.borrow_mut() = url_str.clone();
                                        
                                        // 4. Spawn download
                                        let item_retained = objc2::rc::Retained::retain(item).unwrap();
                                        let item_raw = objc2::rc::Retained::into_raw(item_retained) as usize;
                                        let outline_raw = outline as *const _ as usize;
                                        let node_ptr = node as *const crate::workspace::FileNode as usize;
                                        let app_ptr = self as *const AppDelegate as usize;
                                        
                                        std::thread::spawn(move || {
                                            crate::download_file(url_str, item_raw, outline_raw, node_ptr, app_ptr);
                                        });
                                        
                                        return;
                                    }
                                    
                                    // Local file
                                    *self.ivars().pending_pdf_path.borrow_mut() = url_str.clone();
                                    let app_ptr = self as *const AppDelegate as usize;
                                    
                                    std::thread::spawn(move || {
                                        let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };
                                        let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &nsurl) };
                                        if let Some(doc) = doc {
                                            let doc_any = unsafe { objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(doc) };
                                            let app_raw = app_ptr as *mut objc2::runtime::AnyObject;
                                            let _: () = unsafe {
                                                objc2::msg_send![app_raw, performSelectorOnMainThread: objc2::sel!(pdfDidLoad:), withObject: &*doc_any, waitUntilDone: false]
                                            };
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

"""
    content = content[:start_idx] + new_func + content[end_idx:]
    with open('src/main.rs', 'w') as f:
        f.write(content)
