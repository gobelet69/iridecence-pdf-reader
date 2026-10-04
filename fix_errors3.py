import re

with open("src/workspace_delegate.rs", "r") as f:
    content = f.read()

# Fix btn argument
content = content.replace("outline_view.rowForView(btn as *const _ as *const _)", "outline_view.rowForView(btn)")

# Replace mtm.spawn blocks with performSelectorOnMainThread
old_spawn_1 = """                                            let mtm = objc2_foundation::MainThreadMarker::new().unwrap();
                                            let weak_ov = outline_view_weak.clone();
                                            mtm.spawn(move || {
                                                unsafe {
                                                    if let Some(ov) = weak_ov.load() {
                                                        ov.reloadItem(Some(&*(node as *const _ as *const objc2::runtime::AnyObject)));
                                                    }
                                                }
                                            });"""

old_spawn_1_alt = """                                            let weak_ov = outline_view_weak.clone();
                                            mtm.spawn(move || {
                                                unsafe {
                                                    if let Some(ov) = weak_ov.load() {
                                                        ov.reloadItem(Some(node as *const _ as *const objc2::runtime::AnyObject));
                                                    }
                                                }
                                            });"""

new_spawn = """                                            if let Some(ov) = outline_view_weak.load() {
                                                let sel = objc2::sel!(reloadItem:);
                                                let arg = node as *const _ as *mut objc2::runtime::AnyObject;
                                                let _: () = unsafe { objc2::msg_send![&ov, performSelectorOnMainThread: sel, withObject: arg, waitUntilDone: false] };
                                            }"""

content = re.sub(r'let mtm = objc2_foundation::MainThreadMarker::new\(\)\.unwrap\(\);\s*let weak_ov = outline_view_weak\.clone\(\);\s*mtm\.spawn\(move \|\| \{\s*unsafe \{\s*if let Some\(ov\) = weak_ov\.load\(\) \{\s*ov\.reloadItem\(Some\([^)]+\)\);\s*\}\s*\}\s*\}\);', new_spawn, content)
content = re.sub(r'let weak_ov = outline_view_weak\.clone\(\);\s*mtm\.spawn\(move \|\| \{\s*unsafe \{\s*if let Some\(ov\) = weak_ov\.load\(\) \{\s*ov\.reloadItem\(Some\([^)]+\)\);\s*\}\s*\}\s*\}\);', new_spawn, content)


content = re.sub(r'mtm\.spawn\(move \|\| \{\s*unsafe \{\s*if let Some\(ov\) = weak_ov2\.load\(\) \{\s*ov\.reloadItem\(Some\([^)]+\)\);\s*\}\s*\}\s*\}\);', new_spawn, content)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(content)
