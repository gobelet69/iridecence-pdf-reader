import re

with open("src/workspace.rs", "r") as f:
    code = f.read()

# 1. Update FileNodeIvars
code = code.replace(
    "pub async_data: Arc<Mutex<Option<Vec<(String, String, bool)>>>>,",
    "pub async_data: Arc<Mutex<Option<Vec<(String, String, bool, u8, f64)>>>>,\n    pub icloud_status: Cell<u8>,\n    pub download_progress: Cell<f64>,"
)

# 2. Update FileNode::finishLoading
code = code.replace(
    "for (path, name, is_dir) in entries {\n                children.push(FileNode::new(path, name, is_dir));\n            }",
    """for (path, name, is_dir, icloud_status, download_progress) in entries {
                let mut child = FileNode::new(path, name, is_dir);
                child.ivars().icloud_status.set(icloud_status);
                child.ivars().download_progress.set(download_progress);
                children.push(child);
            }"""
)

# 3. Update FileNode::new
code = code.replace(
    "node.ivars().is_loading_placeholder.set(false);",
    "node.ivars().is_loading_placeholder.set(false);\n        node.ivars().icloud_status.set(0);\n        node.ivars().download_progress.set(0.0);"
)

# 4. Update the background thread
bg_logic_old = """                    result.push((entry.path().to_string_lossy().to_string(), name, is_dir));"""
bg_logic_new = """                    let path_str = entry.path().to_string_lossy().to_string();
                    let mut icloud_status = 0u8;
                    let mut progress = 0.0f64;
                    
                    unsafe {
                        use objc2_foundation::{NSURL, NSString, NSNumber};
                        use objc2::runtime::AnyObject;
                        use objc2::msg_send;
                        
                        let url = NSURL::fileURLWithPath(&NSString::from_str(&path_str));
                        let mut status_val: *mut AnyObject = std::ptr::null_mut();
                        let mut error: *mut AnyObject = std::ptr::null_mut();
                        let key = NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");
                        
                        let ok: bool = msg_send![&url, getResourceValue: &mut status_val, forKey: &*key, error: &mut error];
                        if ok && !status_val.is_null() {
                            let status_str = status_val as *mut NSString;
                            let not_dl_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusNotDownloaded");
                            let is_not_dl: bool = msg_send![status_str, isEqualToString: &*not_dl_id];
                            if is_not_dl {
                                icloud_status = 1;
                            }
                        }
                        
                        let mut is_dl_val: *mut AnyObject = std::ptr::null_mut();
                        let mut error2: *mut AnyObject = std::ptr::null_mut();
                        let key_dl = NSString::from_str("NSURLUbiquitousItemIsDownloadingKey");
                        let ok_dl: bool = msg_send![&url, getResourceValue: &mut is_dl_val, forKey: &*key_dl, error: &mut error2];
                        if ok_dl && !is_dl_val.is_null() {
                            let is_dl: bool = msg_send![is_dl_val, boolValue];
                            if is_dl {
                                icloud_status = 2; // Downloading
                                
                                let mut prog_val: *mut AnyObject = std::ptr::null_mut();
                                let mut error3: *mut AnyObject = std::ptr::null_mut();
                                let key_prog = NSString::from_str("NSURLUbiquitousItemPercentDownloadedKey");
                                let ok_prog: bool = msg_send![&url, getResourceValue: &mut prog_val, forKey: &*key_prog, error: &mut error3];
                                if ok_prog && !prog_val.is_null() {
                                    let p: f64 = msg_send![prog_val, doubleValue];
                                    progress = p;
                                }
                            }
                        }
                    }
                    
                    result.push((path_str, name, is_dir, icloud_status, progress));"""

code = code.replace(bg_logic_old, bg_logic_new)

with open("src/workspace.rs", "w") as f:
    f.write(code)
