import re

with open('src/workspace_delegate.rs', 'r') as f:
    content = f.read()

old_iv = """                    let cloud_iv = objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(180.0, 4.0), NSSize::new(16.0, 16.0)));
                    unsafe { cloud_iv.setIdentifier(Some(objc2_foundation::ns_string!("CloudIcon"))) };
                    cloud_iv.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin);
                    cloud_iv.setHidden(true);"""

new_iv = """                    let cloud_iv = unsafe { objc2_app_kit::NSButton::initWithFrame(objc2_app_kit::NSButton::alloc(mtm), NSRect::new(NSPoint::new(180.0, 4.0), NSSize::new(16.0, 16.0))) };
                    unsafe {
                        cloud_iv.setIdentifier(Some(objc2_foundation::ns_string!("CloudIcon")));
                        cloud_iv.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin);
                        cloud_iv.setHidden(true);
                        cloud_iv.setBezelStyle(objc2_app_kit::NSBezelStyle::Inline);
                        cloud_iv.setBordered(false);
                        cloud_iv.setTitle(objc2_foundation::ns_string!(""));
                        cloud_iv.setImagePosition(objc2_app_kit::NSCellImagePosition::ImageOnly);
                        
                        let target: &objc2::runtime::AnyObject = &*(self as *const WorkspaceDelegate as *const objc2::runtime::AnyObject);
                        cloud_iv.setTarget(Some(target));
                        cloud_iv.setAction(Some(objc2::sel!(downloadItem:)));
                    }"""

if old_iv in content:
    content = content.replace(old_iv, new_iv)
    with open('src/workspace_delegate.rs', 'w') as f:
        f.write(content)
    print("Patched button!")
else:
    print("Old iv not found!")
