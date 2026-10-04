import re

with open("src/workspace_delegate.rs", "r") as f:
    code = f.read()

replacement = """    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(WorkspaceDelegateIvars::default());
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };
        
        let ws = unsafe { objc2_app_kit::NSWorkspace::sharedWorkspace() };
        let folder_img = unsafe { ws.iconForFileType(objc2_foundation::ns_string!("public.folder")) };
        let doc_img = unsafe { ws.iconForFileType(objc2_foundation::ns_string!("pdf")) };
        
        *this.ivars().folder_image.borrow_mut() = Some(folder_img);
        *this.ivars().doc_image.borrow_mut() = Some(doc_img);
        this
    }"""

code = re.sub(
    r"    pub fn new\(mtm: MainThreadMarker\) -> Retained<Self> \{[\s\S]*?\}",
    replacement,
    code,
    count=1
)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(code)
