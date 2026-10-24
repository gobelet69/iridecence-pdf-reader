use objc2::rc::Retained;
use objc2::{ClassType, DefinedClass, Message, define_class, msg_send};
use objc2_foundation::{NSObject, NSObjectProtocol};
use std::cell::{Cell, RefCell};
use std::fs;
use std::sync::{Arc, Mutex};

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
    pub last_status_check: Cell<f64>,
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
        let mut data = self.ivars().async_data.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(entries) = data.take() {
            let mut children = Vec::new();
            for (path, name, is_dir, icloud_status, download_progress) in entries {
                let child = FileNode::new(path.clone(), name, is_dir);
                child.ivars().icloud_status.set(icloud_status);
                child.ivars().download_progress.set(download_progress);
                if path.is_empty() {
                    child.ivars().is_loading_placeholder.set(true);
                }
                children.push(child);
            }
            if children.is_empty() {
                let empty_node = FileNode::new("".to_string(), "No PDF documents found".to_string(), false);
                empty_node.ivars().is_loading_placeholder.set(true);
                children.push(empty_node);
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
        let this: objc2::rc::Allocated<Self> = unsafe { objc2::msg_send![Self::class(), alloc] };
        let this = this.set_ivars(FileNodeIvars::default());
        let node: Retained<Self> = unsafe { objc2::msg_send![super(this), init] };
        *node.ivars().path.borrow_mut() = path;
        *node.ivars().name.borrow_mut() = name;
        node.ivars().is_dir.set(is_dir);
        node.ivars().is_loading.set(false);
        node.ivars().is_loading_placeholder.set(false);
        node.ivars().icloud_status.set(0);
        node.ivars().last_status_check.set(0.0);
        node.ivars().download_progress.set(0.0);
        *node
            .ivars()
            .async_data
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = None;
        node
    }

    pub fn load_children(&self) {
        if !self.ivars().is_dir.get() {
            return;
        }
        if self.ivars().children.borrow().is_some() || self.ivars().is_loading.get() {
            return;
        }
        self.ivars().is_loading.set(true);

        let placeholder = FileNode::new("".to_string(), "Loading...".to_string(), false);
        placeholder.ivars().is_loading_placeholder.set(true);
        *self.ivars().children.borrow_mut() = Some(vec![placeholder]);

        let path_str = self.ivars().path.borrow().clone();
        let async_data = self.ivars().async_data.clone();
        let show_hidden =
            crate::settings::get_bool_pref(crate::settings::PREF_SHOW_HIDDEN_FILES, false);

        let node_ptr = objc2::rc::Retained::into_raw(self.retain()) as usize;

        std::thread::spawn(move || {
            let mut result = Vec::new();
            match fs::read_dir(&path_str) {
                Ok(entries) => {
                    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                    entries.sort_by_key(|e| {
                        let is_dir = e.metadata().map(|m| m.is_dir()).unwrap_or(false);
                        (!is_dir, e.file_name())
                    });

                    for entry in entries {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
                        if !show_hidden && name.starts_with('.') {
                            continue;
                        }
                        if !is_dir && !name.to_lowercase().ends_with(".pdf") {
                            continue;
                        }

                        let path_str = entry.path().to_string_lossy().to_string();
                        let mut icloud_status = 0u8;
                        let mut progress = 0.0f64;

                        unsafe {
                            use objc2::msg_send;
                            use objc2::runtime::AnyObject;
                            use objc2_foundation::{NSString, NSURL};

                            let url = NSURL::fileURLWithPath(&NSString::from_str(&path_str));
                            let mut status_val: *mut AnyObject = std::ptr::null_mut();
                            let mut error: *mut AnyObject = std::ptr::null_mut();
                            let key = NSString::from_str("NSURLUbiquitousItemDownloadingStatusKey");

                            let ok: bool = msg_send![&url, getResourceValue: &mut status_val, forKey: &*key, error: &mut error];
                            if ok && !status_val.is_null() {
                                let status_str = status_val as *mut NSString;
                                let not_dl_id = NSString::from_str(
                                    "NSURLUbiquitousItemDownloadingStatusNotDownloaded",
                                );
                                let is_not_dl: bool =
                                    msg_send![status_str, isEqualToString: &*not_dl_id];
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
                                    let key_prog = NSString::from_str(
                                        "NSURLUbiquitousItemPercentDownloadedKey",
                                    );
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
                Err(e) => {
                    result.push((
                        "".to_string(),
                        format!("Cannot read folder: {}", e.kind()),
                        false,
                        4,
                        0.0,
                    ));
                }
            }

            *async_data.lock().unwrap_or_else(|p| p.into_inner()) = Some(result);

            let obj_raw = node_ptr as *mut objc2::runtime::AnyObject;
            unsafe {
                let _: () = objc2::msg_send![obj_raw, performSelectorOnMainThread: objc2::sel!(finishLoading), withObject: std::ptr::null_mut::<objc2::runtime::AnyObject>(), waitUntilDone: false];
            }
        });
    }

    pub fn load_children_sync(&self) {
        if !self.ivars().is_dir.get() {
            return;
        }
        let path_str = self.ivars().path.borrow().clone();
        let show_hidden =
            crate::settings::get_bool_pref(crate::settings::PREF_SHOW_HIDDEN_FILES, false);
        let mut children = Vec::new();
        if let Ok(entries) = fs::read_dir(&path_str) {
            let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| {
                let is_dir = e.metadata().map(|m| m.is_dir()).unwrap_or(false);
                (!is_dir, e.file_name())
            });
            for entry in entries {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
                if !show_hidden && name.starts_with('.') {
                    continue;
                }
                if !is_dir && !name.to_lowercase().ends_with(".pdf") {
                    continue;
                }
                let child_path = entry.path().to_string_lossy().to_string();
                let child = FileNode::new(child_path, name, is_dir);
                children.push(child);
            }
        }
        *self.ivars().children.borrow_mut() = Some(children);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_node_creation() {
        let node = FileNode::new("/path/test.pdf".to_string(), "test.pdf".to_string(), false);
        assert_eq!(*node.ivars().name.borrow(), "test.pdf");
        assert_eq!(*node.ivars().path.borrow(), "/path/test.pdf");
        assert!(!node.ivars().is_dir.get());
        assert!(!node.ivars().is_loading_placeholder.get());
    }

    #[test]
    fn test_file_node_directory() {
        let node = FileNode::new("/path/dir".to_string(), "dir".to_string(), true);
        assert!(node.ivars().is_dir.get());
        assert!(node.ivars().children.borrow().is_none());
    }

    #[test]
    fn test_workspace_folder_traversal_and_switching() {
        let test_base = "/tmp/iridecence-real-test";
        let _ = std::fs::create_dir_all(format!("{}/folder_a/sub_a", test_base));
        let _ = std::fs::create_dir_all(format!("{}/folder_b", test_base));
        let _ = std::fs::write(
            format!("{}/folder_a/root_sample.pdf", test_base),
            b"%PDF-1.4 mock",
        );
        let _ = std::fs::write(
            format!("{}/folder_a/sub_a/deep_sample.pdf", test_base),
            b"%PDF-1.4 mock",
        );
        let _ = std::fs::write(
            format!("{}/folder_b/other.pdf", test_base),
            b"%PDF-1.4 mock",
        );

        // 1. Open folder_a
        let folder_a = format!("{}/folder_a", test_base);
        let root_a = FileNode::new(folder_a.clone(), "folder_a".to_string(), true);
        root_a.load_children_sync();

        let children_a = root_a.ivars().children.borrow();
        assert!(children_a.is_some(), "folder_a should have children");
        let list_a = children_a.as_ref().unwrap();
        assert_eq!(
            list_a.len(),
            2,
            "folder_a should contain root_sample.pdf and sub_a"
        );

        let sub_a_node = list_a.iter().find(|n| *n.ivars().name.borrow() == "sub_a");
        assert!(sub_a_node.is_some(), "sub_a node must exist in folder_a");
        let sub_a = sub_a_node.unwrap();
        assert!(sub_a.ivars().is_dir.get(), "sub_a must be a directory");

        // 2. Expand sub_a
        sub_a.load_children_sync();
        let sub_children = sub_a.ivars().children.borrow();
        assert!(sub_children.is_some(), "sub_a should have children");
        let sub_list = sub_children.as_ref().unwrap();
        assert_eq!(sub_list.len(), 1, "sub_a should contain deep_sample.pdf");
        assert_eq!(*sub_list[0].ivars().name.borrow(), "deep_sample.pdf");
        assert!(!sub_list[0].ivars().is_dir.get());

        // 3. Switch to folder_b
        let folder_b = format!("{}/folder_b", test_base);
        let root_b = FileNode::new(folder_b.clone(), "folder_b".to_string(), true);
        root_b.load_children_sync();

        let children_b = root_b.ivars().children.borrow();
        assert!(children_b.is_some(), "folder_b should have children");
        let list_b = children_b.as_ref().unwrap();
        assert_eq!(list_b.len(), 1, "folder_b should contain other.pdf");
        assert_eq!(*list_b[0].ivars().name.borrow(), "other.pdf");

        // 4. Switch back to folder_a
        let root_a_again = FileNode::new(folder_a, "folder_a".to_string(), true);
        root_a_again.load_children_sync();
        let children_a_again = root_a_again.ivars().children.borrow();
        assert!(children_a_again.is_some());
        let list_a_again = children_a_again.as_ref().unwrap();
        assert_eq!(list_a_again.len(), 2);
    }
}
