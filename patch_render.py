import re
with open('src/workspace_delegate.rs', 'r') as f:
    content = f.read()

start_idx = content.find("if let Some(civ) = cloud_iv {")
end_idx = content.find("if let Some(tf) = unsafe { table_cell.textField() } {")

if start_idx != -1 and end_idx != -1:
    new_func = """
                let diag_state = match icloud_status {
                    0 => "Local",
                    1 => "CloudOnly",
                    2 => "Downloading",
                    4 => "Failed",
                    _ => "Unknown",
                };
                let diag_icon = match icloud_status {
                    0 => "NONE",
                    1 => "Cloud",
                    2 => "Spinner",
                    4 => "Failed",
                    _ => "NONE",
                };
                if icloud_status == 0 {
                    println!("[DIAGNOSTIC] RENDER: state = {}, icon = {}", diag_state, diag_icon);
                }

                if let Some(civ) = cloud_iv {
                    if icloud_status == 1 || icloud_status == 4 {
                        civ.setHidden(false);
                        let icon_name = if icloud_status == 1 { "icloud.and.arrow.down" } else { "exclamationmark.icloud" };
                        if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!(icon_name), None) {
                            civ.setImage(Some(&img));
                            if icloud_status == 1 {
                                civ.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
                            } else {
                                civ.setContentTintColor(Some(&objc2_app_kit::NSColor::systemRedColor()));
                            }
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
            }
            
            """
    content = content[:start_idx] + new_func + content[end_idx:]
    with open('src/workspace_delegate.rs', 'w') as f:
        f.write(content)
