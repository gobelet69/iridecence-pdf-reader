import re

with open("src/workspace.rs", "r") as f:
    ws = f.read()

# Add is_loading_placeholder to FileNodeIvars
ivars = """pub struct FileNodeIvars {
    pub path: RefCell<String>,
    pub name: RefCell<String>,
    pub is_dir: Cell<bool>,
    pub children: RefCell<Option<Vec<Retained<FileNode>>>>,
    pub is_loading: Cell<bool>,
    pub async_data: Arc<Mutex<Option<Vec<(String, String, bool)>>>>,
    pub is_loading_placeholder: Cell<bool>,
}"""
ws = re.sub(r"pub struct FileNodeIvars \{.*?\n\}", ivars, ws, flags=re.DOTALL)

# Add is_loading_placeholder init to FileNode::new
new_init = """    pub fn new(path: String, name: String, is_dir: bool) -> Retained<Self> {
        let this: objc2::rc::Allocated<Self> = unsafe { objc2::msg_send_id![Self::class(), alloc] };
        let this = this.set_ivars(FileNodeIvars::default());
        let node: Retained<Self> = unsafe { msg_send![super(this), init] };
        *node.ivars().path.borrow_mut() = path;
        *node.ivars().name.borrow_mut() = name;
        node.ivars().is_dir.set(is_dir);
        node.ivars().is_loading.set(false);
        node.ivars().is_loading_placeholder.set(false);
        *node.ivars().async_data.lock().unwrap() = None;
        node
    }"""
ws = re.sub(r"    pub fn new\(path: String, name: String, is_dir: bool\) -> Retained<Self> \{.*?    \}", new_init, ws, flags=re.DOTALL)

# Update load_children to insert a placeholder
load_children_new = """    pub fn load_children(&self) {
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
                    
                    result.push((entry.path().to_string_lossy().to_string(), name, is_dir));
                }
            }
            
            *async_data.lock().unwrap() = Some(result);
            
            let obj_raw = node_ptr as *mut objc2::runtime::AnyObject;
            unsafe {
                let _: () = objc2::msg_send![obj_raw, performSelectorOnMainThread: objc2::sel!(finishLoading), withObject: std::ptr::null_mut::<objc2::runtime::AnyObject>(), waitUntilDone: false];
            }
        });
    }"""
ws = re.sub(r"    pub fn load_children\(&self\) \{.*?        \}\);.*?    \}", load_children_new, ws, flags=re.DOTALL)

with open("src/workspace.rs", "w") as f:
    f.write(ws)
