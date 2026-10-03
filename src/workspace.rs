use std::cell::{RefCell, Cell};
use std::path::{Path, PathBuf};
use objc2::{define_class, msg_send, ClassType, DefinedClass};
use objc2::rc::Retained;
use objc2_foundation::{NSObject, NSObjectProtocol};
use std::fs;

#[derive(Default)]
pub struct FileNodeIvars {
    pub path: RefCell<String>,
    pub name: RefCell<String>,
    pub is_dir: Cell<bool>,
    pub children: RefCell<Option<Vec<Retained<FileNode>>>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "FileNode"]
    #[ivars = FileNodeIvars]
    pub struct FileNode;

    unsafe impl NSObjectProtocol for FileNode {}
);

impl FileNode {
    pub fn new(path: String, name: String, is_dir: bool) -> Retained<Self> {
        let this: objc2::rc::Allocated<Self> = unsafe { objc2::msg_send_id![Self::class(), alloc] };
        let this = this.set_ivars(FileNodeIvars::default());
        let node: Retained<Self> = unsafe { msg_send![super(this), init] };
        *node.ivars().path.borrow_mut() = path;
        *node.ivars().name.borrow_mut() = name;
        node.ivars().is_dir.set(is_dir);
        node
    }

    pub fn load_children(&self) { println!("load_children called!"); 
        if !self.ivars().is_dir.get() { return; }
        if self.ivars().children.borrow().is_some() { return; }

        let path_str = self.ivars().path.borrow().clone();
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
                if name.starts_with('.') { continue; }
                if !is_dir && !name.to_lowercase().ends_with(".pdf") { continue; }

                children.push(FileNode::new(
                    entry.path().to_string_lossy().to_string(),
                    name,
                    is_dir
                ));
            }
        }
        *self.ivars().children.borrow_mut() = Some(children);
    }
}
