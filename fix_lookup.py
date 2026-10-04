import re

with open("src/workspace_delegate.rs", "r") as f:
    content = f.read()

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

content = content.replace(old_lookup, new_lookup)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(content)
