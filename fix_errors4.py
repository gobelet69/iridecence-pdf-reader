import re

with open("src/workspace_delegate.rs", "r") as f:
    content = f.read()

download_method = """
    #[unsafe(method(downloadItem:))]
    fn download_item(&self, sender: Option<&objc2_app_kit::NSButton>) {
        if let Some(btn) = sender {
            if let Some(outline_view_weak) = self.ivars().outline_view.borrow().as_ref() {
                if let Some(outline_view) = outline_view_weak.load() {
                    let row = unsafe { outline_view.rowForView(btn) };
                    if row >= 0 {
                        if let Some(item) = unsafe { outline_view.itemAtRow(row) } {
                            let node: &crate::workspace::FileNode = unsafe { &*(item as *const _ as *const crate::workspace::FileNode) };
                            if node.ivars().icloud_status.get() == 1 {
                                let path = node.ivars().path.borrow().clone();
                                let node_ptr = objc2::rc::Retained::into_raw(objc2::rc::Retained::retain(node as *const _ as *mut _).unwrap()) as usize;
                                
                                let weak_ov_thread = outline_view_weak.clone();
                                
                                std::thread::spawn(move || {
                                    use objc2_foundation::{NSURL, NSString};
                                    use objc2::msg_send;
                                    unsafe {
                                        let fm = objc2::msg_send_id![objc2::class!(NSFileManager), defaultManager];
                                        let fm: objc2::rc::Retained<objc2::runtime::AnyObject> = fm;
                                        let path_str = NSString::from_str(&path);
                                        let url = NSURL::fileURLWithPath(&path_str);
                                        let mut error: *mut objc2::runtime::AnyObject = std::ptr::null_mut();
                                        let ok: bool = msg_send![&fm, startDownloadingUbiquitousItemAtURL: &*url, error: &mut error];
                                        if ok {
                                            let node: &crate::workspace::FileNode = &*(node_ptr as *const crate::workspace::FileNode);
                                            node.ivars().icloud_status.set(2);
                                            node.ivars().download_progress.set(0.0);
                                            
                                            if let Some(ov) = weak_ov_thread.load() {
                                                let sel = objc2::sel!(reloadItem:);
                                                let arg = node as *const _ as *mut objc2::runtime::AnyObject;
                                                let _: () = objc2::msg_send![&ov, performSelectorOnMainThread: sel, withObject: arg, waitUntilDone: false];
                                            }

                                            loop {
                                                std::thread::sleep(std::time::Duration::from_millis(200));
                                                let mut status_val: *mut objc2::runtime::AnyObject = std::ptr::null_mut();
                                                let mut err: *mut objc2::runtime::AnyObject = std::ptr::null_mut();
                                                
                                                let is_dl: bool = {
                                                    let key_dl = NSString::from_str("NSURLUbiquitousItemIsDownloadingKey");
                                                    let ok_dl: bool = msg_send![&*url, getResourceValue: &mut status_val, forKey: &*key_dl, error: &mut err];
                                                    if ok_dl && !status_val.is_null() {
                                                        msg_send![status_val, boolValue]
                                                    } else {
                                                        false
                                                    }
                                                };
                                                
                                                let has_dl: bool = {
                                                    let key_has = NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");
                                                    let ok_has: bool = msg_send![&*url, getResourceValue: &mut status_val, forKey: &*key_has, error: &mut err];
                                                    if ok_has && !status_val.is_null() {
                                                        let cur = status_val as *mut NSString;
                                                        let c_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusCurrent");
                                                        msg_send![cur, isEqualToString: &*c_id]
                                                    } else {
                                                        false
                                                    }
                                                };
                                                
                                                if has_dl {
                                                    node.ivars().icloud_status.set(0);
                                                    if let Some(ov) = weak_ov_thread.load() {
                                                        let sel = objc2::sel!(reloadItem:);
                                                        let arg = node as *const _ as *mut objc2::runtime::AnyObject;
                                                        let _: () = objc2::msg_send![&ov, performSelectorOnMainThread: sel, withObject: arg, waitUntilDone: false];
                                                    }
                                                    break;
                                                }
                                                
                                                if is_dl {
                                                    let mut prog_val: *mut objc2::runtime::AnyObject = std::ptr::null_mut();
                                                    let key_prog = NSString::from_str("NSURLUbiquitousItemPercentDownloadedKey");
                                                    let ok_prog: bool = msg_send![&*url, getResourceValue: &mut prog_val, forKey: &*key_prog, error: &mut err];
                                                    if ok_prog && !prog_val.is_null() {
                                                        let p: f64 = msg_send![prog_val, doubleValue];
                                                        node.ivars().download_progress.set(p / 100.0);
                                                        if let Some(ov) = weak_ov_thread.load() {
                                                            let sel = objc2::sel!(reloadItem:);
                                                            let arg = node as *const _ as *mut objc2::runtime::AnyObject;
                                                            let _: () = objc2::msg_send![&ov, performSelectorOnMainThread: sel, withObject: arg, waitUntilDone: false];
                                                        }
                                                    }
                                                } else {
                                                    node.ivars().icloud_status.set(1);
                                                    if let Some(ov) = weak_ov_thread.load() {
                                                        let sel = objc2::sel!(reloadItem:);
                                                        let arg = node as *const _ as *mut objc2::runtime::AnyObject;
                                                        let _: () = objc2::msg_send![&ov, performSelectorOnMainThread: sel, withObject: arg, waitUntilDone: false];
                                                    }
                                                    break;
                                                }
                                            }
                                        }
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

content = re.sub(r'#\[unsafe\(method\(downloadItem:\)\)\].*?\n    #\[unsafe\(method\(outlineViewSelectionDidChange:\)\]', download_method.strip() + "\n    #[unsafe(method(outlineViewSelectionDidChange:))]", content, flags=re.DOTALL)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(content)
