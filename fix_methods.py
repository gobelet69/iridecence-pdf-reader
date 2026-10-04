import re
with open("src/main.rs", "r") as f:
    content = f.read()

# I will just write a python script to replace the messed up methods
# Find where workspaceDoubleClicked starts
start = content.find("#[unsafe(method(workspaceDoubleClicked:))]")
# Find where applicationDidFinishLaunching: starts
end = content.find("#[unsafe(method(applicationDidFinishLaunching:))]")

new_methods = """#[unsafe(method(workspaceDoubleClicked:))]
        fn workspace_double_clicked(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(outline) = sender.downcast_ref::<NSOutlineView>() {
                    let clicked_row = unsafe { outline.clickedRow() };
                    if clicked_row >= 0 {
                        if let Some(item) = unsafe { outline.itemAtRow(clicked_row) } {
                            let node: &crate::workspace::FileNode = unsafe { &*(&*item as *const AnyObject as *const crate::workspace::FileNode) };
                            if node.ivars().is_dir.get() {
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
                                
                                let current_doc = unsafe { self.ivars().pdf_view.get().unwrap().document() };
                                let mut already_open = false;
                                if let Some(current_doc) = current_doc {
                                    if let Some(current_url) = unsafe { current_doc.documentURL() } {
                                        if unsafe { current_url.isEqual(&url) } { already_open = true; }
                                    }
                                }
                                
                                if !already_open {
                                    let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                                    if let Some(doc) = doc {
                                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(&doc)) };
                                    }
                                }
                            } else {
                                let is_expanded = unsafe { outline.isItemExpanded(&item) };
                                if is_expanded { unsafe { outline.collapseItem(&item) }; }
                                else { unsafe { outline.expandItem(&item) }; }
                            }
                        }
                    }
                }
            }
        }

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
                                let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&node.ivars().path.borrow())) };
                                
                                let current_doc = unsafe { self.ivars().pdf_view.get().unwrap().document() };
                                let mut already_open = false;
                                if let Some(current_doc) = current_doc {
                                    if let Some(current_url) = unsafe { current_doc.documentURL() } {
                                        if unsafe { current_url.isEqual(&url) } { already_open = true; }
                                    }
                                }
                                
                                if !already_open {
                                    let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url) };
                                    if let Some(doc) = doc {
                                        unsafe { self.ivars().pdf_view.get().unwrap().setDocument(Some(&doc)) };
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        """

with open("src/main.rs", "w") as f:
    f.write(content[:start] + new_methods + content[end:])
