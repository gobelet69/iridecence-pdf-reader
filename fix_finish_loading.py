import re
with open("src/workspace.rs", "r") as f:
    ws = f.read()

finish_loading = """    #[unsafe(method(finishLoading))]
    fn finishLoading(&self) {
        self.ivars().is_loading.set(false);
        let mut data = self.ivars().async_data.lock().unwrap();
        if let Some(entries) = data.take() {
            let mut children = Vec::new();
            for (path, name, is_dir) in entries {
                children.push(FileNode::new(path, name, is_dir));
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
    }"""

# Remove finishLoading from impl FileNode
ws = ws.replace(finish_loading, "")

# Insert into define_class!
define_class = """define_class!(
    #[unsafe(super(NSObject))]
    #[name = "FileNode"]
    #[ivars = FileNodeIvars]
    pub struct FileNode;

    unsafe impl NSObjectProtocol for FileNode {}
);"""

new_define_class = f"""define_class!(
    #[unsafe(super(NSObject))]
    #[name = "FileNode"]
    #[ivars = FileNodeIvars]
    pub struct FileNode;

    unsafe impl NSObjectProtocol for FileNode {{}}

    impl FileNode {{
{finish_loading}
    }}
);"""

ws = ws.replace(define_class, new_define_class)

with open("src/workspace.rs", "w") as f:
    f.write(ws)
