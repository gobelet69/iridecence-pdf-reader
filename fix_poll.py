import re

with open("src/workspace_delegate.rs", "r") as f:
    content = f.read()

old_logic = """                                        if ok {
                                            // Tell node to reload status
                                            let node: &crate::workspace::FileNode = &*(node_ptr as *const crate::workspace::FileNode);
                                            node.ivars().icloud_status.set(2);
                                            node.ivars().download_progress.set(0.0);
                                        }"""

new_logic = """                                        if ok {
                                            let node: &crate::workspace::FileNode = &*(node_ptr as *const crate::workspace::FileNode);
                                            node.ivars().icloud_status.set(2);
                                            node.ivars().download_progress.set(0.0);
                                            
                                            // Reload initially to show spinner
                                            let weak_ov = outline_view_weak.clone();
                                            mtm.spawn(move || {
                                                unsafe {
                                                    if let Some(ov) = weak_ov.upgrade() {
                                                        ov.reloadItem(Some(node as *const _ as *const objc2::runtime::AnyObject));
                                                    }
                                                }
                                            });

                                            let weak_ov2 = outline_view_weak.clone();
                                            std::thread::spawn(move || {
                                                loop {
                                                    std::thread::sleep(std::time::Duration::from_millis(200));
                                                    let mut status_val: *mut objc2::runtime::AnyObject = std::ptr::null_mut();
                                                    let mut error: *mut objc2::runtime::AnyObject = std::ptr::null_mut();
                                                    
                                                    let is_dl: bool = unsafe {
                                                        let key_dl = NSString::from_str("NSURLUbiquitousItemIsDownloadingKey");
                                                        let ok_dl: bool = msg_send![&url, getResourceValue: &mut status_val, forKey: &*key_dl, error: &mut error];
                                                        if ok_dl && !status_val.is_null() {
                                                            msg_send![status_val, boolValue]
                                                        } else {
                                                            false
                                                        }
                                                    };
                                                    
                                                    let has_dl: bool = unsafe {
                                                        let key_has = NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");
                                                        let ok_has: bool = msg_send![&url, getResourceValue: &mut status_val, forKey: &*key_has, error: &mut error];
                                                        if ok_has && !status_val.is_null() {
                                                            let cur = status_val as *mut NSString;
                                                            let c_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusCurrent");
                                                            msg_send![cur, isEqualToString: &*c_id]
                                                        } else {
                                                            false
                                                        }
                                                    };
                                                    
                                                    if has_dl {
                                                        node.ivars().icloud_status.set(0); // Available locally
                                                        mtm.spawn(move || {
                                                            unsafe {
                                                                if let Some(ov) = weak_ov2.upgrade() {
                                                                    ov.reloadItem(Some(node as *const _ as *const objc2::runtime::AnyObject));
                                                                }
                                                            }
                                                        });
                                                        break;
                                                    }
                                                    
                                                    if is_dl {
                                                        unsafe {
                                                            let mut prog_val: *mut objc2::runtime::AnyObject = std::ptr::null_mut();
                                                            let key_prog = NSString::from_str("NSURLUbiquitousItemPercentDownloadedKey");
                                                            let ok_prog: bool = msg_send![&url, getResourceValue: &mut prog_val, forKey: &*key_prog, error: &mut error];
                                                            if ok_prog && !prog_val.is_null() {
                                                                let p: f64 = msg_send![prog_val, doubleValue];
                                                                node.ivars().download_progress.set(p / 100.0);
                                                                mtm.spawn(move || {
                                                                    unsafe {
                                                                        if let Some(ov) = weak_ov2.upgrade() {
                                                                            ov.reloadItem(Some(node as *const _ as *const objc2::runtime::AnyObject));
                                                                        }
                                                                    }
                                                                });
                                                            }
                                                        }
                                                    } else {
                                                        // Failed or stopped
                                                        node.ivars().icloud_status.set(1);
                                                        mtm.spawn(move || {
                                                            unsafe {
                                                                if let Some(ov) = weak_ov2.upgrade() {
                                                                    ov.reloadItem(Some(node as *const _ as *const objc2::runtime::AnyObject));
                                                                }
                                                            }
                                                        });
                                                        break;
                                                    }
                                                }
                                            });
                                        }"""

if old_logic in content:
    content = content.replace(old_logic, new_logic)
    with open("src/workspace_delegate.rs", "w") as f:
        f.write(content)
    print("Replaced!")
else:
    print("Not found!")
