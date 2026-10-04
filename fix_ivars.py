import re

with open('src/main.rs', 'r') as f:
    content = f.read()

old_ivars = """struct AppDelegateIvars {
    pending_url: std::cell::RefCell<Option<Retained<objc2_foundation::NSURL>>>,
    pending_pdf_path: std::cell::RefCell<String>,
    window: OnceCell<Retained<NSWindow>>,
    pdf_view: OnceCell<Retained<PDFView>>,
    split_vc: OnceCell<Retained<NSSplitViewController>>,
    workspace_delegate: OnceCell<Retained<workspace_delegate::WorkspaceDelegate>>,
    outline_scroll: OnceCell<Retained<NSScrollView>>,
    thumb_view: OnceCell<Retained<PDFThumbnailView>>,
}"""

new_ivars = """struct AppDelegateIvars {
    pending_url: std::cell::RefCell<Option<Retained<objc2_foundation::NSURL>>>,
    pending_pdf_path: std::cell::RefCell<String>,
    window: OnceCell<Retained<NSWindow>>,
    pdf_view: OnceCell<Retained<PDFView>>,
    split_vc: OnceCell<Retained<NSSplitViewController>>,
    workspace_delegate: OnceCell<Retained<workspace_delegate::WorkspaceDelegate>>,
    outline_scroll: OnceCell<Retained<NSScrollView>>,
    thumb_view: OnceCell<Retained<PDFThumbnailView>>,
    explorer_item: OnceCell<Retained<NSSplitViewItem>>,
    navigation_item: OnceCell<Retained<NSSplitViewItem>>,
}"""

content = content.replace(old_ivars, new_ivars)

with open('src/main.rs', 'w') as f:
    f.write(content)
