with open("src/workspace_delegate.rs", "r") as f:
    content = f.read()

old_create = """                    let cloud_iv = objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));
                    unsafe { cloud_iv.setIdentifier(Some(objc2_foundation::ns_string!("CloudIcon"))) };
                    cloud_iv.setHidden(true);"""

new_create = """                    let cloud_btn = objc2_app_kit::NSButton::initWithFrame(objc2_app_kit::NSButton::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));
                    unsafe { cloud_btn.setIdentifier(Some(objc2_foundation::ns_string!("CloudIcon"))) };
                    cloud_btn.setHidden(true);
                    unsafe {
                        cloud_btn.setButtonType(objc2_app_kit::NSButtonType::MomentaryChange);
                        cloud_btn.setBezelStyle(objc2_app_kit::NSBezelStyle::Inline);
                        cloud_btn.setBordered(false);
                        cloud_btn.setTitle(objc2_foundation::ns_string!(""));
                        cloud_btn.setImagePosition(objc2_app_kit::NSCellImagePosition::ImageOnly);
                        let target: &objc2::runtime::AnyObject = &*(self as *const WorkspaceDelegate as *const objc2::runtime::AnyObject);
                        cloud_btn.setTarget(Some(target));
                        cloud_btn.setAction(Some(objc2::sel!(downloadItem:)));
                    }"""

content = content.replace(old_create, new_create)
content = content.replace("cell.addSubview(&cloud_iv);", "cell.addSubview(&cloud_btn);")

old_lookup = """                let mut cloud_iv: Option<Retained<objc2_app_kit::NSImageView>> = None;"""
new_lookup = """                let mut cloud_btn: Option<Retained<objc2_app_kit::NSButton>> = None;"""
content = content.replace(old_lookup, new_lookup)

old_cast = """                            cloud_iv = Some(Retained::cast_unchecked::<objc2_app_kit::NSImageView>(v.clone()));"""
new_cast = """                            cloud_btn = Some(Retained::cast_unchecked::<objc2_app_kit::NSButton>(v.clone()));"""
content = content.replace(old_cast, new_cast)

old_use = """                if let Some(civ) = cloud_iv {
                    if icloud_status == 1 {
                        civ.setHidden(false);
                        if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("icloud.and.arrow.down"), None) {
                            civ.setImage(Some(&img));
                            civ.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
                        }
                    } else {
                        civ.setHidden(true);
                    }
                }"""

new_use = """                if let Some(btn) = cloud_btn {
                    if icloud_status == 1 {
                        btn.setHidden(false);
                        if let Some(img) = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!("icloud.and.arrow.down"), None) {
                            btn.setImage(Some(&img));
                            unsafe { btn.setContentTintColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor())) };
                            unsafe { btn.setToolTip(Some(objc2_foundation::ns_string!("Not downloaded. Click to download."))) };
                        }
                    } else {
                        btn.setHidden(true);
                    }
                }"""

content = content.replace(old_use, new_use)

method_str = """
    #[unsafe(method(downloadItem:))]
    fn download_item(&self, sender: Option<&objc2_app_kit::NSButton>) {
        if let Some(btn) = sender {
            if let Some(outline_view_weak) = self.ivars().outline_view.borrow().as_ref() {
                if let Some(outline_view) = outline_view_weak.upgrade() {
                    let row = unsafe { outline_view.rowForView(btn as *const _ as *const _) };
                    if row >= 0 {
                        if let Some(item) = unsafe { outline_view.itemAtRow(row) } {
                            let node: &crate::workspace::FileNode = unsafe { &*(item as *const _ as *const crate::workspace::FileNode) };
                            if node.ivars().icloud_status.get() == 1 {
                                let path = node.ivars().path.borrow().clone();
                                let node_ptr = objc2::rc::Retained::into_raw(node.retain()) as usize;
                                std::thread::spawn(move || {
                                    use objc2_foundation::{NSURL, NSString};
                                    use objc2::msg_send;
                                    unsafe {
                                        let fm = msg_send_id![objc2::class!(NSFileManager), defaultManager];
                                        let fm: objc2::rc::Retained<objc2::runtime::AnyObject> = fm;
                                        let path_str = NSString::from_str(&path);
                                        let url = NSURL::fileURLWithPath(&path_str);
                                        let mut error: *mut objc2::runtime::AnyObject = std::ptr::null_mut();
                                        let ok: bool = msg_send![&fm, startDownloadingUbiquitousItemAtURL: &*url, error: &mut error];
                                        if ok {
                                            // Tell node to reload status
                                            let node: &crate::workspace::FileNode = &*(node_ptr as *const crate::workspace::FileNode);
                                            node.ivars().icloud_status.set(2);
                                            node.ivars().download_progress.set(0.0);
                                        }
                                    }
                                });
                            }
                        }
                    }
                }
            }
        }
    }
"""

if "fn outlineViewSelectionDidChange" in content:
    content = content.replace("    #[unsafe(method(outlineViewSelectionDidChange:))]", method_str + "\n    #[unsafe(method(outlineViewSelectionDidChange:))]")

with open("src/workspace_delegate.rs", "w") as f:
    f.write(content)
