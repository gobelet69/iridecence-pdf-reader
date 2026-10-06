import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# Replace pdf_did_load to also reload outline and handle empty state
old_pdf_did_load = """        #[unsafe(method(pdfDidLoad:))]
        fn pdf_did_load(&self, doc: Option<&objc2_pdf_kit::PDFDocument>) {
            crate::log_pdf!("[DIAGNOSTIC] PDF DID LOAD ENTER");
            if let Some(doc) = doc {
                let main_thread = std::thread::current().name() == Some("main");
                crate::log_pdf!("[DIAGNOSTIC] main thread = {}", main_thread);
                
                if let Some(url) = unsafe { doc.documentURL() } {
                    let path = url.path().unwrap().to_string();
                    crate::log_pdf!("[DIAGNOSTIC] path = {}", path);
                    if path == *self.ivars().pending_pdf_path.borrow() {
                        crate::log_pdf!("[DIAGNOSTIC] PDFVIEW BEFORE setDocument");
                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(doc)) };
                        
                        let page_count = unsafe { doc.pageCount() };
                        crate::log_pdf!("[DIAGNOSTIC] PDFVIEW AFTER setDocument: pageCount = {}", page_count);
                    } else {
                        crate::log_pdf!("[DIAGNOSTIC] Discarded obsolete doc: {}", path);
                    }
                }
            } else {
                crate::log_pdf!("[DIAGNOSTIC] doc is None");
            }
        }"""

new_pdf_did_load = """        #[unsafe(method(pdfDidLoad:))]
        fn pdf_did_load(&self, doc: Option<&objc2_pdf_kit::PDFDocument>) {
            crate::log_pdf!("[DIAGNOSTIC] PDF DID LOAD ENTER");
            if let Some(doc) = doc {
                let main_thread = std::thread::current().name() == Some("main");
                crate::log_pdf!("[DIAGNOSTIC] main thread = {}", main_thread);
                
                if let Some(url) = unsafe { doc.documentURL() } {
                    let path = url.path().unwrap().to_string();
                    crate::log_pdf!("[DIAGNOSTIC] path = {}", path);
                    if path == *self.ivars().pending_pdf_path.borrow() {
                        crate::log_pdf!("[DIAGNOSTIC] PDFVIEW BEFORE setDocument");
                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(doc)) };
                        
                        let page_count = unsafe { doc.pageCount() };
                        crate::log_pdf!("[DIAGNOSTIC] PDFVIEW AFTER setDocument: pageCount = {}", page_count);
                        
                        self.update_outline_empty_state();
                    } else {
                        crate::log_pdf!("[DIAGNOSTIC] Discarded obsolete doc: {}", path);
                    }
                }
            } else {
                crate::log_pdf!("[DIAGNOSTIC] doc is None");
            }
        }

        fn update_outline_empty_state(&self) {
            unsafe {
                if let Some(scroll) = self.ivars().right_sidebar_outline_scroll.get() {
                    if let Some(view) = scroll.documentView() {
                        if let Some(outline_view) = view.downcast_ref::<NSOutlineView>() {
                            outline_view.reloadData();
                            
                            // Check for outline root
                            let mut has_outline = false;
                            if let Some(pdf_view) = self.ivars().pdf_view.get() {
                                if let Some(doc) = pdf_view.document() {
                                    if let Some(root) = doc.outlineRoot() {
                                        if root.numberOfChildren() > 0 {
                                            has_outline = true;
                                        }
                                    }
                                }
                            }
                            
                            // Let's find or create the empty state view
                            let empty_tag = 999;
                            let superview = scroll.superview().unwrap();
                            let mut existing_empty = None;
                            let subviews = superview.subviews();
                            for i in 0..subviews.count() {
                                let sub = subviews.objectAtIndex(i);
                                if sub.tag() == empty_tag {
                                    existing_empty = Some(sub);
                                    break;
                                }
                            }
                            
                            if has_outline {
                                if let Some(empty) = existing_empty {
                                    empty.removeFromSuperview();
                                }
                            } else {
                                if existing_empty.is_none() {
                                    let mtm = MainThreadMarker::new().unwrap();
                                    let empty_view = NSView::initWithFrame(NSView::alloc(mtm), scroll.frame());
                                    empty_view.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                                    let _: () = objc2::msg_send![&empty_view, setTag: empty_tag];
                                    
                                    // SF Symbol list.bullet.indent
                                    let img = NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("list.bullet.indent"), None).unwrap();
                                    let img_view = NSImageView::imageViewWithImage(&img);
                                    img_view.setContentTintColor(Some(&NSColor::tertiaryLabelColor()));
                                    img_view.setTranslatesAutoresizingMaskIntoConstraints(false);
                                    
                                    let label = NSTextField::labelWithString(objc2_foundation::ns_string!("No document outline available"), mtm);
                                    label.setTextColor(Some(&NSColor::secondaryLabelColor()));
                                    label.setTranslatesAutoresizingMaskIntoConstraints(false);
                                    
                                    empty_view.addSubview(&img_view);
                                    empty_view.addSubview(&label);
                                    
                                    img_view.centerXAnchor().constraintEqualToAnchor(&empty_view.centerXAnchor()).setActive(true);
                                    img_view.centerYAnchor().constraintEqualToAnchor_constant(&empty_view.centerYAnchor(), -20.0).setActive(true);
                                    let _: () = objc2::msg_send![&img_view, setSymbolConfiguration: &*NSImageSymbolConfiguration::configurationWithPointSize_weight_scale(32.0, NSFontWeightRegular, NSImageSymbolScaleLarge)];
                                    
                                    label.centerXAnchor().constraintEqualToAnchor(&empty_view.centerXAnchor()).setActive(true);
                                    label.topAnchor().constraintEqualToAnchor_constant(&img_view.bottomAnchor(), 16.0).setActive(true);
                                    
                                    // Make sure it doesn't block hits if we need it to, but it's an empty state so it's fine.
                                    
                                    // Hidden initially if outline isn't currently selected, but actually it sits on top of scroll, so we want it to show/hide with scroll.
                                    // A better way: just add it to scroll's superview, and sync its hidden state with scroll's hidden state.
                                    // Or better yet, we just add it to the content_wrapper, but toggle its visibility in switchSidebar.
                                    superview.addSubview(&empty_view);
                                }
                            }
                            
                            // We need to make sure the empty view's hidden state matches the outline scroll view's hidden state.
                            if let Some(empty) = existing_empty {
                                empty.setHidden(scroll.isHidden());
                            } else {
                                // If we just created it:
                                let subviews = superview.subviews();
                                for i in 0..subviews.count() {
                                    let sub = subviews.objectAtIndex(i);
                                    if sub.tag() == empty_tag {
                                        sub.setHidden(scroll.isHidden());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }"""

content = content.replace(old_pdf_did_load, new_pdf_did_load)

with open('src/main.rs', 'w') as f:
    f.write(content)
