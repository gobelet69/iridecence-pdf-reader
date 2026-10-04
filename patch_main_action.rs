        #[unsafe(method(workspaceClicked:))]
        fn workspace_clicked(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(outline) = sender.downcast_ref::<NSOutlineView>() {
                    let mtm = MainThreadMarker::new().unwrap();
                    let app = NSApplication::sharedApplication(mtm);
                    let mut is_enter = false;
                    if let Some(event) = app.currentEvent() {
                        let ev_type = event.r#type();
                        if ev_type == objc2_app_kit::NSEventType::KeyDown {
                            is_enter = true;
                        }
                    }
                    if !is_enter { return; }
                    
                    let clicked_row = outline.clickedRow();
                    let row = if clicked_row >= 0 { clicked_row } else { outline.selectedRow() };
                    
                    if row >= 0 {
                        if let Some(item) = unsafe { outline.itemAtRow(row) } {
                            let node: &crate::workspace::FileNode = unsafe { &*(&*item as *const AnyObject as *const crate::workspace::FileNode) };
                            if !node.ivars().is_dir.get() {
                                let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&node.ivars().path.borrow())) };
                                let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                                if let Some(doc) = doc {
                                    unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(&doc)) };
                                }
                            } else {
                                // Expand/collapse directory
                                let is_expanded = unsafe { outline.isItemExpanded(&item) };
                                if is_expanded {
                                    unsafe { outline.collapseItem(&item) };
                                } else {
                                    unsafe { outline.expandItem(&item) };
                                }
                            }
                        }
                    }
                }
            }
        }
