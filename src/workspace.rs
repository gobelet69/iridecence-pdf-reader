use std::cell::{RefCell, Cell};
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use objc2::{define_class, msg_send, ClassType, DefinedClass, Message};
use objc2::rc::Retained;
use objc2_foundation::{NSObject, NSObjectProtocol};
use std::fs;

#[derive(Default)]
pub struct FileNodeIvars {
    pub path: RefCell<String>,
    pub name: RefCell<String>,
    pub is_dir: Cell<bool>,
    pub children: RefCell<Option<Vec<Retained<FileNode>>>>,
    pub is_loading: Cell<bool>,
    pub async_data: Arc<Mutex<Option<Vec<(String, String, bool, u8, f64)>>>>,
    pub icloud_status: Cell<u8>,
    pub download_progress: Cell<f64>,
    pub is_loading_placeholder: Cell<bool>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "FileNode"]
    #[ivars = FileNodeIvars]
    pub struct FileNode;

    unsafe impl NSObjectProtocol for FileNode {}

    impl FileNode {
    #[unsafe(method(finishLoading))]
    fn finishLoading(&self) {
        self.ivars().is_loading.set(false);
        let mut data = self.ivars().async_data.lock().unwrap();
        if let Some(entries) = data.take() {
            let mut children = Vec::new();
            for (path, name, is_dir, icloud_status, download_progress) in entries {
                let mut child = FileNode::new(path, name, is_dir);
                child.ivars().icloud_status.set(icloud_status);
                child.ivars().download_progress.set(download_progress);
                children.push(child);
            }
            *self.ivars().children.borrow_mut() = Some(children);
        }
        
        unsafe {
            let nc = objc2_foundation::NSNotificationCenter::defaultCenter();
            let name = objc2_foundation::ns_string!("FileNodeDidLoadChildren");
            nc.postNotificationName_object(name, Some(self));
            
            // Release the +1 retain count from background thread
            let _ = objc2::rc::Retained::from_raw(self as *const _ as *mut Self);
        }
    }
    }
);

impl FileNode {
    pub fn new(path: String, name: String, is_dir: bool) -> Retained<Self> {
        let this: objc2::rc::Allocated<Self> = unsafe { objc2::msg_send_id![Self::class(), alloc] };
        let this = this.set_ivars(FileNodeIvars::default());
        let node: Retained<Self> = unsafe { msg_send![super(this), init] };
        *node.ivars().path.borrow_mut() = path;
        *node.ivars().name.borrow_mut() = name;
        node.ivars().is_dir.set(is_dir);
        node.ivars().is_loading.set(false);
        node.ivars().is_loading_placeholder.set(false);
        node.ivars().icloud_status.set(0);
        node.ivars().download_progress.set(0.0);
        *node.ivars().async_data.lock().unwrap() = None;
        node
    }

    pub fn load_children(&self) {
        if !self.ivars().is_dir.get() { return; }
        if self.ivars().children.borrow().is_some() || self.ivars().is_loading.get() { return; }
        self.ivars().is_loading.set(true);
        
        let placeholder = FileNode::new("".to_string(), "Loading...".to_string(), false);
        placeholder.ivars().is_loading_placeholder.set(true);
        *self.ivars().children.borrow_mut() = Some(vec![placeholder]);

        let path_str = self.ivars().path.borrow().clone();
        let async_data = self.ivars().async_data.clone();
        
        let node_ptr = objc2::rc::Retained::into_raw(self.retain()) as usize;

        std::thread::spawn(move || {
            let mut result = Vec::new();
            if let Ok(entries) = fs::read_dir(&path_str) {
                let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                entries.sort_by_key(|e| {
                    let is_dir = e.metadata().map(|m| m.is_dir()).unwrap_or(false);
                    (!is_dir, e.file_name())
                });

                for entry in entries {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
                    if name.starts_with('.') { continue; }
                    if !is_dir && !name.to_lowercase().ends_with(".pdf") { continue; }
                    
                    let path_str = entry.path().to_string_lossy().to_string();
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
                    
                    result.push((path_str, name, is_dir, icloud_status, progress));
                }
            }
            
            *async_data.lock().unwrap() = Some(result);
            
            let obj_raw = node_ptr as *mut objc2::runtime::AnyObject;
            unsafe {
                let _: () = objc2::msg_send![obj_raw, performSelectorOnMainThread: objc2::sel!(finishLoading), withObject: std::ptr::null_mut::<objc2::runtime::AnyObject>(), waitUntilDone: false];
            }
        });
    }


}
