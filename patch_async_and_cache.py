import re

# ============================
# 1. Patch workspace.rs
# ============================
with open("src/workspace.rs", "r") as f:
    ws_content = f.read()

# Add imports
ws_content = ws_content.replace("use std::cell::{RefCell, Cell};", "use std::cell::{RefCell, Cell};\nuse std::sync::{Arc, Mutex};")

# Update FileNodeIvars
new_ivars = """pub struct FileNodeIvars {
    pub path: RefCell<String>,
    pub name: RefCell<String>,
    pub is_dir: Cell<bool>,
    pub children: RefCell<Option<Vec<Retained<FileNode>>>>,
    pub is_loading: Cell<bool>,
    pub async_data: Arc<Mutex<Option<Vec<(String, String, bool)>>>>,
}"""
ws_content = re.sub(r"pub struct FileNodeIvars \{.*?\n\}", new_ivars, ws_content, flags=re.DOTALL)

# Update FileNode::new
new_init = """let node: Retained<Self> = unsafe { msg_send![super(this), init] };
        *node.ivars().path.borrow_mut() = path;
        *node.ivars().name.borrow_mut() = name;
        node.ivars().is_dir.set(is_dir);
        node.ivars().is_loading.set(false);
        *node.ivars().async_data.lock().unwrap() = None;
        node"""
ws_content = re.sub(r"let node: Retained<Self> = unsafe \{ msg_send\!\[super\(this\), init\] \};.*?node", new_init, ws_content, flags=re.DOTALL)

# Update load_children to be async and add finish_loading
load_children_new = """pub fn load_children(&self) {
        if !self.ivars().is_dir.get() { return; }
        if self.ivars().children.borrow().is_some() || self.ivars().is_loading.get() { return; }
        self.ivars().is_loading.set(true);

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
    }

    #[unsafe(method(finishLoading))]
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
ws_content = re.sub(r"pub fn load_children\(&self\) \{.*?\*self.ivars\(\).children.borrow_mut\(\) = Some\(children\);\n\s*(?:println!\(.*?;\n\s*)?\}", load_children_new, ws_content, flags=re.DOTALL)

with open("src/workspace.rs", "w") as f:
    f.write(ws_content)

# ============================
# 2. Patch workspace_delegate.rs
# ============================
with open("src/workspace_delegate.rs", "r") as f:
    wd_content = f.read()

# Add Weak import
wd_content = wd_content.replace("use std::cell::RefCell;", "use std::cell::RefCell;\nuse objc2::rc::Weak;")

# Update WorkspaceDelegateIvars
new_ivars_delegate = """pub struct WorkspaceDelegateIvars {
    pub root_node: RefCell<Option<Retained<FileNode>>>,
    pub outline_view: RefCell<Option<Weak<NSOutlineView>>>,
    pub folder_image: RefCell<Option<Retained<objc2_app_kit::NSImage>>>,
    pub doc_image: RefCell<Option<Retained<objc2_app_kit::NSImage>>>,
}"""
wd_content = re.sub(r"pub struct WorkspaceDelegateIvars \{.*?\n\}", new_ivars_delegate, wd_content, flags=re.DOTALL)

# Add nodeDidLoadChildren: to impl WorkspaceDelegate
node_loaded = """    #[unsafe(method(nodeDidLoadChildren:))]
    fn nodeDidLoadChildren(&self, notification: &objc2_foundation::NSNotification) {
        if let Some(weak_view) = self.ivars().outline_view.borrow().as_ref() {
            if let Some(outline_view) = weak_view.load() {
                let node: Retained<FileNode> = unsafe { msg_send![notification, object] };
                let node_any = unsafe { objc2::rc::Retained::cast_unchecked::<objc2::runtime::AnyObject>(node) };
                unsafe { outline_view.reloadItem_reloadChildren(Some(&node_any), true) };
            }
        }
    }"""
wd_content = wd_content.replace("impl WorkspaceDelegate {", "impl WorkspaceDelegate {\n" + node_loaded)

# Update WorkspaceDelegate::new
new_init_delegate = """impl WorkspaceDelegate {
    pub fn set_outline_view(&self, view: &NSOutlineView) {
        *self.ivars().outline_view.borrow_mut() = Some(Weak::new(view));
    }
    
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(WorkspaceDelegateIvars::default());
        let delegate: Retained<Self> = unsafe { msg_send![super(this), init] };
        
        let folder_image = unsafe { objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("folder.fill"), None) };
        *delegate.ivars().folder_image.borrow_mut() = folder_image;
        let doc_image = unsafe { objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("doc.text.fill"), None) };
        *delegate.ivars().doc_image.borrow_mut() = doc_image;
        
        unsafe {
            let nc = objc2_foundation::NSNotificationCenter::defaultCenter();
            nc.addObserver_selector_name_object(
                &delegate,
                objc2::sel!(nodeDidLoadChildren:),
                Some(objc2_foundation::ns_string!("FileNodeDidLoadChildren")),
                None
            );
        }
        
        delegate
    }
}"""
wd_content = re.sub(r"impl WorkspaceDelegate \{\n\s*pub fn new.*?\n\s*\}\n\}", new_init_delegate, wd_content, flags=re.DOTALL)

# Optimize viewForTableColumn
view_opt_old = """let image = unsafe { objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(symbol_name, None) };
                if let Some(image) = image {
                    if is_dir {
                        unsafe { iv.setContentTintColor(Some(&objc2_app_kit::NSColor::systemBlueColor())) };
                    } else {
                        unsafe { iv.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor())) };
                    }
                    unsafe { iv.setImage(Some(&image)) };
                }"""
view_opt_new = """if is_dir {
                    unsafe { iv.setContentTintColor(Some(&objc2_app_kit::NSColor::systemBlueColor())) };
                    if let Some(img) = self.ivars().folder_image.borrow().as_ref() {
                        unsafe { iv.setImage(Some(img)) };
                    }
                } else {
                    unsafe { iv.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor())) };
                    if let Some(img) = self.ivars().doc_image.borrow().as_ref() {
                        unsafe { iv.setImage(Some(img)) };
                    }
                }"""
wd_content = wd_content.replace(view_opt_old, view_opt_new)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd_content)

# ============================
# 3. Patch main.rs
# ============================
with open("src/main.rs", "r") as f:
    main_content = f.read()

# Add delegate.set_outline_view(&outline_view)
main_content = main_content.replace("unsafe { outline_view.setDelegate(Some(&delegate)) };", "delegate.set_outline_view(&outline_view);\n            unsafe { outline_view.setDelegate(Some(&delegate)) };")

with open("src/main.rs", "w") as f:
    f.write(main_content)
