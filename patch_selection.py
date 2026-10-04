with open('src/main.rs', 'r') as f:
    content = f.read()

import re

# We will rewrite workspace_selection_changed thread body
start_marker = "std::thread::spawn(move || {"
end_marker = "                    });\n                                }\n                            }\n                        }\n                    }\n                }\n            }\n        }"
start_idx = content.find(start_marker, content.find("fn workspace_selection_changed"))
end_idx = content.find(end_marker, start_idx) + len(end_marker)

new_logic = """let outline_raw = outline as *const _ as usize;
                                    let item_retained = objc2::rc::Retained::retain(item).unwrap();
                                    let item_raw = objc2::rc::Retained::into_raw(item_retained) as usize;
                                    let node_ptr = node as *const crate::workspace::FileNode as usize;
                                    
                                    std::thread::spawn(move || {
                                        let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };
                                        let node = unsafe { &*(node_ptr as *const crate::workspace::FileNode) };
                                        
                                        // 1. Download if cloud-only
                                        let status = node.ivars().icloud_status.get();
                                        if status == 1 || status == 4 {
                                            unsafe {
                                                node.ivars().icloud_status.set(2); // Downloading
                                                node.ivars().download_progress.set(0.0);
                                                let o_ptr = outline_raw as *mut NSOutlineView;
                                                let i_ptr = item_raw as *mut AnyObject;
                                                let _: () = msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr, waitUntilDone: false];
                                                
                                                let fm = objc2_foundation::NSFileManager::defaultManager();
                                                let mut err: *mut AnyObject = std::ptr::null_mut();
                                                let _: bool = msg_send![&fm, startDownloadingUbiquitousItemAtURL: &*nsurl, error: &mut err];
                                                
                                                loop {
                                                    let mut is_dl_val: *mut AnyObject = std::ptr::null_mut();
                                                    let key = NSString::from_str("NSURLUbiquitousItemIsDownloadingKey");
                                                    let ok: bool = msg_send![&nsurl, getResourceValue: &mut is_dl_val, forKey: &*key, error: std::ptr::null_mut::<AnyObject>()];
                                                    
                                                    let mut still_dl = false;
                                                    if ok && !is_dl_val.is_null() {
                                                        still_dl = msg_send![is_dl_val, boolValue];
                                                    }
                                                    
                                                    if still_dl {
                                                        let mut prog_val: *mut AnyObject = std::ptr::null_mut();
                                                        let key_prog = NSString::from_str("NSURLUbiquitousItemPercentDownloadedKey");
                                                        let ok_prog: bool = msg_send![&nsurl, getResourceValue: &mut prog_val, forKey: &*key_prog, error: std::ptr::null_mut::<AnyObject>()];
                                                        if ok_prog && !prog_val.is_null() {
                                                            let p: f64 = msg_send![prog_val, doubleValue];
                                                            node.ivars().download_progress.set(p);
                                                        }
                                                        let _: () = msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr, waitUntilDone: false];
                                                        std::thread::sleep(std::time::Duration::from_millis(500));
                                                    } else {
                                                        let mut stat_val: *mut AnyObject = std::ptr::null_mut();
                                                        let key_stat = NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");
                                                        let ok_stat: bool = msg_send![&nsurl, getResourceValue: &mut stat_val, forKey: &*key_stat, error: std::ptr::null_mut::<AnyObject>()];
                                                        if ok_stat && !stat_val.is_null() {
                                                            let stat_str = stat_val as *mut NSString;
                                                            let dl_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusDownloaded");
                                                            let current_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusCurrent");
                                                            let is_dl: bool = msg_send![stat_str, isEqualToString: &*dl_id];
                                                            let is_cur: bool = msg_send![stat_str, isEqualToString: &*current_id];
                                                            if is_dl || is_cur {
                                                                node.ivars().icloud_status.set(0); // Local
                                                            } else {
                                                                node.ivars().icloud_status.set(1);
                                                            }
                                                        } else {
                                                            node.ivars().icloud_status.set(1);
                                                        }
                                                        let _: () = msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr, waitUntilDone: false];
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // Once it's downloaded (or if it was already local), load the PDF!
                                        let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &nsurl) };
                                        
                                        unsafe {
                                            // Release the item properly
                                            let _: objc2::rc::Retained<AnyObject> = objc2::rc::Retained::from_raw(item_raw as *mut AnyObject).unwrap();
                                        }
                                        
                                        if let Some(doc) = doc {
                                            let doc_any = unsafe { objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(doc) };
                                            unsafe {
                                                let app_delegate = &*(app_ptr as *const AnyObject);
                                                let _: () = msg_send![app_delegate, performSelectorOnMainThread: objc2::sel!(pdfReady:), withObject: &*doc_any, waitUntilDone: false];
                                            }
                                        } else {
                                            // Failed to load PDF
                                            println!("[PDF ERROR] Failed to load doc: {}", url_str);
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }"""

content = content[:start_idx-100] + content[start_idx-100:start_idx].replace("std::thread::spawn", new_logic.split("std::thread::spawn")[0] + "std::thread::spawn") + new_logic.split("std::thread::spawn")[1] + content[end_idx:]

# Just purely replace the exact region instead of hacky split:
content = content[:start_idx-1] + new_logic + content[end_idx:]

with open('src/main.rs', 'w') as f:
    f.write(content)

