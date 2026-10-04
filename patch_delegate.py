import re

with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()

# 1. Update nodeDidLoadChildren (to handle None correctly)
# I already did this in patch_root.py, but I will make sure it's correct.

# 2. Update outlineViewSelectionDidChange
sel_old = """    #[unsafe(method(outlineViewSelectionDidChange:))]
    fn outlineViewSelectionDidChange(&self, notification: &objc2_foundation::NSNotification) {
        let outline_view: objc2::rc::Retained<NSOutlineView> = unsafe { objc2::msg_send![notification, object] };
        
        let target: Option<objc2::rc::Retained<objc2::runtime::AnyObject>> = unsafe { objc2::msg_send![&outline_view, target] };
        if let Some(target) = target {
            let _: () = unsafe { objc2::msg_send![&target, workspaceSelectionChanged: &*outline_view] };
        }
    }"""
sel_new = """    #[unsafe(method(outlineViewSelectionDidChange:))]
    fn outlineViewSelectionDidChange(&self, notification: &objc2_foundation::NSNotification) {
        let outline_view: objc2::rc::Retained<NSOutlineView> = unsafe { objc2::msg_send![notification, object] };
        let selected_row = unsafe { outline_view.selectedRow() };
        if selected_row >= 0 {
            if let Some(item) = unsafe { outline_view.itemAtRow(selected_row) } {
                let node: &crate::workspace::FileNode = unsafe { &*(item as *const objc2::runtime::AnyObject as *const crate::workspace::FileNode) };
                if node.ivars().is_loading_placeholder.get() {
                    return;
                }
            }
        }
        
        let target: Option<objc2::rc::Retained<objc2::runtime::AnyObject>> = unsafe { objc2::msg_send![&outline_view, target] };
        if let Some(target) = target {
            let _: () = unsafe { objc2::msg_send![&target, workspaceSelectionChanged: &*outline_view] };
        }
    }"""
wd = wd.replace(sel_old, sel_new)

# 3. Update outlineView_viewForTableColumn_item
view_old = """                if is_dir {
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
view_new = """                if node.ivars().is_loading_placeholder.get() {
                    unsafe { iv.setImage(None) }; // No icon for loading placeholder
                } else if is_dir {
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
wd = wd.replace(view_old, view_new)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd)
