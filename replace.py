with open("src/workspace_delegate.rs", "r") as f:
    lines = f.readlines()

new_method = """        #[unsafe(method(outlineView:viewForTableColumn:item:))]
        fn outlineView_viewForTableColumn_item(&self, outline_view: &NSOutlineView, _table_column: Option<&NSTableColumn>, item: &AnyObject) -> *mut NSView {
            let node: &FileNode = unsafe { &*(item as *const AnyObject as *const FileNode) };
            let mtm = MainThreadMarker::new().unwrap();
            
            let id = objc2_foundation::ns_string!("FileCell");
            let mut view = unsafe { outline_view.makeViewWithIdentifier_owner(id, None) };
            if view.is_none() {
                let cell = unsafe { NSTableCellView::initWithFrame(NSTableCellView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 24.0))) };
                unsafe { cell.setIdentifier(Some(id)) };
                
                let image_view = unsafe { objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(2.0, 4.0), NSSize::new(16.0, 16.0))) };
                let text_field = unsafe { NSTextField::initWithFrame(NSTextField::alloc(mtm), NSRect::new(NSPoint::new(24.0, 3.0), NSSize::new(176.0, 18.0))) };
                unsafe {
                    text_field.setBezeled(false);
                    text_field.setDrawsBackground(false);
                    text_field.setEditable(false);
                    text_field.setSelectable(false);
                    
                    let font = objc2_app_kit::NSFont::systemFontOfSize(13.0);
                    text_field.setFont(Some(&font));
                    text_field.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingMiddle);
                    
                    cell.addSubview(&image_view);
                    cell.addSubview(&text_field);
                    cell.setImageView(Some(&image_view));
                    cell.setTextField(Some(&text_field));
                }
                view = Some(unsafe { Retained::cast_unchecked::<NSView>(cell) });
            }
            
            let cell = view.unwrap();
            let table_cell = unsafe { Retained::cast_unchecked::<NSTableCellView>(cell.clone()) };
            
            let is_dir = node.ivars().is_dir.get();
            let symbol_name = if is_dir { objc2_foundation::ns_string!("folder.fill") } else { objc2_foundation::ns_string!("doc.text.fill") };
            
            if let Some(iv) = unsafe { table_cell.imageView() } {
                let image = unsafe { objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(symbol_name, None) };
                if let Some(image) = image {
                    if is_dir {
                        unsafe { iv.setContentTintColor(Some(&objc2_app_kit::NSColor::systemBlueColor())) };
                    } else {
                        unsafe { iv.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor())) };
                    }
                    unsafe { iv.setImage(Some(&image)) };
                }
            }
            
            if let Some(tf) = unsafe { table_cell.textField() } {
                let name_str = &node.ivars().name.borrow();
                let path_str = &node.ivars().path.borrow();
                unsafe { tf.setStringValue(&NSString::from_str(name_str)) };
                unsafe { table_cell.setToolTip(Some(&NSString::from_str(path_str))) };
            }
            Retained::autorelease_ptr(cell)
        }
"""

with open("src/workspace_delegate.rs", "w") as f:
    f.writelines(lines[:77])
    f.write(new_method)
    f.writelines(lines[107:])
