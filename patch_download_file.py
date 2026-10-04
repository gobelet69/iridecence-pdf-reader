import re
with open('src/main.rs', 'r') as f:
    content = f.read()

start_idx = content.find("pub fn download_file(url_str: String, item_raw: usize, outline_raw: usize, node_ptr: usize)")
end_idx = content.find("pub fn refresh_file_status")

if start_idx != -1 and end_idx != -1:
    new_func = """pub fn download_file(url_str: String, item_raw: usize, outline_raw: usize, node_ptr: usize, app_ptr: usize) {
    unsafe {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use objc2_foundation::{NSURL, NSString, NSFileManager, NSThread};
        use objc2_app_kit::NSOutlineView;
        use objc2_pdf_kit::PDFDocument;
        
        println!("[DIAGNOSTIC] DOWNLOAD START: state = Downloading");
        
        let nsurl = NSURL::fileURLWithPath(&NSString::from_str(&url_str));
        let fm = NSFileManager::defaultManager();
        let mut err: *mut AnyObject = std::ptr::null_mut();
        let _: bool = msg_send![&fm, startDownloadingUbiquitousItemAtURL: &*nsurl, error: &mut err];
        
        let node = &*(node_ptr as *const crate::workspace::FileNode);
        
        let mut iterations = 0;
        loop {
            iterations += 1;
            if iterations > 120 {
                node.ivars().icloud_status.set(4); // Failed if timeout
                let o_ptr = outline_raw as *mut NSOutlineView;
                let i_ptr = item_raw as *mut AnyObject;
                let _: () = objc2::msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr, waitUntilDone: false];
                let _: objc2::rc::Retained<AnyObject> = objc2::rc::Retained::from_raw(i_ptr as *mut AnyObject).unwrap();
                break;
            }

            let mut is_dl_val: *mut AnyObject = std::ptr::null_mut();
            let mut error: *mut AnyObject = std::ptr::null_mut();
            let key = NSString::from_str("NSURLUbiquitousItemIsDownloadingKey");
            let ok: bool = msg_send![&nsurl, getResourceValue: &mut is_dl_val, forKey: &*key, error: &mut error];
            
            let mut still_dl = false;
            if ok && !is_dl_val.is_null() {
                let is_dl: bool = msg_send![is_dl_val, boolValue];
                still_dl = is_dl;
            }
            
            let mut stat_val: *mut AnyObject = std::ptr::null_mut();
            let key_stat = NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");
            let ok_stat: bool = msg_send![&nsurl, getResourceValue: &mut stat_val, forKey: &*key_stat, error: std::ptr::null_mut::<AnyObject>()];
            
            let mut is_current = false;
            let mut is_downloaded = false;
            if ok_stat && !stat_val.is_null() {
                let stat_str = stat_val as *mut NSString;
                let dl_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusDownloaded");
                let current_id = NSString::from_str("NSURLUbiquitousItemDownloadingStatusCurrent");
                is_downloaded = msg_send![stat_str, isEqualToString: &*dl_id];
                is_current = msg_send![stat_str, isEqualToString: &*current_id];
            }
            
            println!("[DIAGNOSTIC] METADATA: status_current = {}, isDownloading = {}, isDownloaded = {}", is_current, still_dl, is_downloaded);
            
            if still_dl && !is_current && !is_downloaded {
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
                if is_downloaded || is_current {
                    println!("[DIAGNOSTIC] MODEL UPDATE: old_state = {}, new_state = 0", node.ivars().icloud_status.get());
                    node.ivars().icloud_status.set(0); // Local
                } else {
                    node.ivars().icloud_status.set(4); // Failed
                }
                
                let o_ptr = outline_raw as *mut NSOutlineView;
                let i_ptr = item_raw as *mut AnyObject;
                let _: () = msg_send![o_ptr, performSelectorOnMainThread: objc2::sel!(reloadItem:), withObject: i_ptr, waitUntilDone: false];
                
                if node.ivars().icloud_status.get() == 0 && app_ptr != 0 {
                    let doc = PDFDocument::initWithURL(PDFDocument::alloc(), &nsurl);
                    if let Some(doc) = doc {
                        let doc_any = objc2::rc::Retained::cast_unchecked::<AnyObject>(doc);
                        let app_raw = app_ptr as *mut AnyObject;
                        let _: () = msg_send![app_raw, performSelectorOnMainThread: objc2::sel!(pdfDidLoad:), withObject: &*doc_any, waitUntilDone: false];
                    }
                }
                
                let _: objc2::rc::Retained<AnyObject> = objc2::rc::Retained::from_raw(i_ptr).unwrap();
                break;
            }
        }
    }
}

"""
    content = content[:start_idx] + new_func + content[end_idx:]
    with open('src/main.rs', 'w') as f:
        f.write(content)
