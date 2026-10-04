import re

with open("src/workspace_delegate.rs", "r") as f:
    code = f.read()

# 1. Update Cell Creation
cell_creation_old = """                    cell.addSubview(&image_view);
                    cell.addSubview(&text_field);
                    cell.setImageView(Some(&image_view));
                    cell.setTextField(Some(&text_field));
                }"""

cell_creation_new = """                    cell.addSubview(&image_view);
                    cell.addSubview(&text_field);
                    cell.setImageView(Some(&image_view));
                    cell.setTextField(Some(&text_field));
                    
                    let cloud_iv = objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));
                    let _: () = msg_send![&cloud_iv, setTag: 101isize];
                    cloud_iv.setHidden(true);
                    
                    let progress = objc2_app_kit::NSProgressIndicator::initWithFrame(objc2_app_kit::NSProgressIndicator::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));
                    let _: () = msg_send![&progress, setTag: 102isize];
                    progress.setHidden(true);
                    progress.setStyle(objc2_app_kit::NSProgressIndicatorStyle::Spinning);
                    progress.setControlSize(objc2_app_kit::NSControlSize::Small);
                    
                    cell.addSubview(&cloud_iv);
                    cell.addSubview(&progress);
                }"""

code = code.replace(cell_creation_old, cell_creation_new)

# 2. Update Cell Configuration
cell_config_old = """                } else {
                    unsafe { iv.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor())) };
                    if let Some(img) = self.ivars().doc_image.borrow().as_ref() {
                        unsafe { iv.setImage(Some(img)) };
                    }
                }
            }"""

cell_config_new = """                } else {
                    unsafe { iv.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor())) };
                    if let Some(img) = self.ivars().doc_image.borrow().as_ref() {
                        unsafe { iv.setImage(Some(img)) };
                    }
                }
            }
            
            let icloud_status = node.ivars().icloud_status.get();
            let download_progress = node.ivars().download_progress.get();
            
            unsafe {
                let cloud_iv: Option<Retained<objc2_app_kit::NSImageView>> = msg_send![&table_cell, viewWithTag: 101isize];
                if let Some(civ) = cloud_iv {
                    if icloud_status == 1 {
                        civ.setHidden(false);
                        if let img = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("icloud.and.arrow.down"), None) {
                            civ.setImage(Some(&img));
                            civ.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
                        }
                    } else {
                        civ.setHidden(true);
                    }
                }
                
                let progress: Option<Retained<objc2_app_kit::NSProgressIndicator>> = msg_send![&table_cell, viewWithTag: 102isize];
                if let Some(pi) = progress {
                    if icloud_status == 2 {
                        pi.setHidden(false);
                        pi.startAnimation(None);
                        if download_progress > 0.0 {
                            pi.setIndeterminate(false);
                            pi.setDoubleValue(download_progress);
                        } else {
                            pi.setIndeterminate(true);
                        }
                    } else {
                        pi.setHidden(true);
                        pi.stopAnimation(None);
                    }
                }
            }"""

code = code.replace(cell_config_old, cell_config_new)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(code)
