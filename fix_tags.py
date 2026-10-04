import re

with open("src/workspace_delegate.rs", "r") as f:
    content = f.read()

# Replace setTag and viewWithTag logic
old_setup = """                    let cloud_iv = objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));
                    let _: () = msg_send![&cloud_iv, setTag: 101isize];
                    cloud_iv.setHidden(true);
                    
                    let progress = objc2_app_kit::NSProgressIndicator::initWithFrame(objc2_app_kit::NSProgressIndicator::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));
                    let _: () = msg_send![&progress, setTag: 102isize];"""

new_setup = """                    let cloud_iv = objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));
                    unsafe { cloud_iv.setIdentifier(Some(objc2_foundation::ns_string!("CloudIcon"))) };
                    cloud_iv.setHidden(true);
                    
                    let progress = objc2_app_kit::NSProgressIndicator::initWithFrame(objc2_app_kit::NSProgressIndicator::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));
                    unsafe { progress.setIdentifier(Some(objc2_foundation::ns_string!("DownloadProgress"))) };"""

content = content.replace(old_setup, new_setup)

old_lookup = """            unsafe {
                let cloud_iv: Option<Retained<objc2_app_kit::NSImageView>> = objc2::msg_send_id![&table_cell, viewWithTag: 101isize];
                if let Some(civ) = cloud_iv {
                    if icloud_status == 1 {
                        civ.setHidden(false);
                        if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("icloud.and.arrow.down"), None) {
                            civ.setImage(Some(&img));
                            civ.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
                        }
                    } else {
                        civ.setHidden(true);
                    }
                }
                
                let progress: Option<Retained<objc2_app_kit::NSProgressIndicator>> = objc2::msg_send_id![&table_cell, viewWithTag: 102isize];
                if let Some(piv) = progress {
                    if icloud_status == 2 {
                        piv.setHidden(false);
                        piv.startAnimation(None);
                        if download_progress >= 0.0 && download_progress <= 1.0 {
                            piv.setIndeterminate(false);
                            piv.setDoubleValue(download_progress as f64 * 100.0);
                        } else {
                            piv.setIndeterminate(true);
                        }
                    } else {
                        piv.setHidden(true);
                        piv.stopAnimation(None);
                    }
                }
            }"""

new_lookup = """            unsafe {
                let subviews = table_cell.subviews();
                let mut cloud_iv: Option<Retained<objc2_app_kit::NSImageView>> = None;
                let mut progress: Option<Retained<objc2_app_kit::NSProgressIndicator>> = None;
                
                for i in 0..subviews.count() {
                    let v = subviews.objectAtIndex(i);
                    if let Some(ident) = v.identifier() {
                        if ident.isEqualToString(objc2_foundation::ns_string!("CloudIcon")) {
                            cloud_iv = Some(Retained::cast_unchecked::<objc2_app_kit::NSImageView>(v.retain()));
                        } else if ident.isEqualToString(objc2_foundation::ns_string!("DownloadProgress")) {
                            progress = Some(Retained::cast_unchecked::<objc2_app_kit::NSProgressIndicator>(v.retain()));
                        }
                    }
                }
                
                if let Some(civ) = cloud_iv {
                    if icloud_status == 1 {
                        civ.setHidden(false);
                        if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("icloud.and.arrow.down"), None) {
                            civ.setImage(Some(&img));
                            civ.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
                        }
                    } else {
                        civ.setHidden(true);
                    }
                }
                
                if let Some(piv) = progress {
                    if icloud_status == 2 {
                        piv.setHidden(false);
                        piv.startAnimation(None);
                        if download_progress >= 0.0 && download_progress <= 1.0 {
                            piv.setIndeterminate(false);
                            piv.setDoubleValue(download_progress as f64 * 100.0);
                        } else {
                            piv.setIndeterminate(true);
                        }
                    } else {
                        piv.setHidden(true);
                        piv.stopAnimation(None);
                    }
                }
            }"""

content = content.replace(old_lookup, new_lookup)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(content)
