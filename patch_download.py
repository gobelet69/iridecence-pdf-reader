import re

with open('src/main.rs', 'r') as f:
    content = f.read()

download_fn = """pub fn download_file(url_str: String, item_raw: usize, outline_raw: usize, node_ptr: usize) {
    unsafe {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use objc2_foundation::{NSURL, NSString, NSFileManager, NSThread};
        use objc2_app_kit::NSOutlineView;
        
        let nsurl = NSURL::fileURLWithPath(&NSString::from_str(&url_str));
        let fm = NSFileManager::defaultManager();
        let mut err: *mut AnyObject = std::ptr::null_mut();
        let _: bool = msg_send![&fm, startDownloadingUbiquitousItemAtURL: &nsurl, error: &mut err];
        
        let node = &*(node_ptr as *const crate::workspace::FileNode);
        
        loop {
            let mut is_dl_val: *mut AnyObject = std::ptr::null_mut();
            let mut error: *mut AnyObject = std::ptr::null_mut();
            let key = NSString::from_str("NSURLUbiquitousItemIsDownloadingKey");
            let ok: bool = msg_send![&nsurl, getResourceValue: &mut is_dl_val, forKey: &*key, error: &mut error];
            
            let mut still_dl = false;
            if ok && !is_dl_val.is_null() {
                let is_dl: bool = msg_send![is_dl_val, boolValue];
                still_dl = is_dl;
            }
            
            if still_dl {
                let mut prog_val: *mut AnyObject = std::ptr::null_mut();
                let key_prog = NSString::from_str("NSURLUbiquitousItemPercentDownloadedKey");
                let ok_prog: bool = msg_send![&nsurl, getResourceValue: &mut prog_val, forKey: &*key_prog, error: std::ptr::null_mut::<AnyObject>()];
                if ok_prog && !prog_val.is_null() {
                    let p: f64 = msg_send![prog_val, doubleValue];
                    node.ivars().download_progress.set(p);
                }
                
                let o_ptr = outline_raw as *mut NSOutlineView;
                let i_ptr = item_raw as *mut AnyObject;
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
                        // Failed or other?
                        node.ivars().icloud_status.set(1); // Revert to cloud-only if failed?
                    }
                } else {
                    node.ivars().icloud_status.set(1);
                }
                
                let o_ptr = outline_raw as *mut NSOutlineView;
                let i_ptr = item_raw as *mut AnyObject;
                let _: () = msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr, waitUntilDone: false];
                
                // Release the retained item properly
                let _: objc2::rc::Retained<AnyObject> = objc2::rc::Retained::from_raw(i_ptr as *mut AnyObject).unwrap();
                
                break;
            }
        }
    }
}
"""

if "pub fn download_file" not in content:
    content = content.replace("fn main() {", download_fn + "\nfn main() {")
    with open('src/main.rs', 'w') as f:
        f.write(content)
    print("Patched download_file!")
