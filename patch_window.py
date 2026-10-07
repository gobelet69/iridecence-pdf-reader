import re

with open("src/main.rs", "r") as f:
    content = f.read()

import_pattern = r"(NSWindow, NSWindowStyleMask, NSOpenPanel, NSModalResponseOK,)"
content = content.replace("NSWindow, NSWindowStyleMask, NSOpenPanel, NSModalResponseOK,", "NSWindow, NSWindowStyleMask, NSOpenPanel, NSModalResponseOK, NSScreen,")

window_creation_old = """
            let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1000.0, 700.0));
            let style = NSWindowStyleMask::Titled 
                      | NSWindowStyleMask::Closable 
                      | NSWindowStyleMask::Resizable 
                      | NSWindowStyleMask::Miniaturizable;
            
            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(mtm),
                    rect,
                    style,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };
            
            window.setTitle(objc2_foundation::ns_string!("OxiReader"));
            window.center();"""

window_creation_new = """
            let mut width = 960.0;
            let mut height = 640.0;
            
            if let Some(screen) = NSScreen::mainScreen(mtm) {
                let frame = screen.visibleFrame();
                let max_width = frame.size.width * 0.85;
                let max_height = frame.size.height * 0.82;
                if width > max_width { width = max_width; }
                if height > max_height { height = max_height; }
            }
            if width < 720.0 { width = 720.0; }
            if height < 500.0 { height = 500.0; }
            
            let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, height));
            let style = NSWindowStyleMask::Titled 
                      | NSWindowStyleMask::Closable 
                      | NSWindowStyleMask::Resizable 
                      | NSWindowStyleMask::Miniaturizable;
            
            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(mtm),
                    rect,
                    style,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };
            
            window.setTitle(objc2_foundation::ns_string!("OxiReader"));
            
            // Handle restoring position but clamped to screen
            window.setFrameAutosaveName(objc2_foundation::ns_string!("MainWindowGeometry"));
            window.center();
            
            // We need to clamp to visible frame explicitly if it's already restored by macOS
            if let Some(screen) = window.screen() {
                let visible = screen.visibleFrame();
                let mut frame = window.frame();
                let mut changed = false;
                if frame.size.width > visible.size.width {
                    frame.size.width = visible.size.width;
                    changed = true;
                }
                if frame.size.height > visible.size.height {
                    frame.size.height = visible.size.height;
                    changed = true;
                }
                if frame.origin.x < visible.origin.x {
                    frame.origin.x = visible.origin.x;
                    changed = true;
                }
                if frame.origin.y < visible.origin.y {
                    frame.origin.y = visible.origin.y;
                    changed = true;
                }
                if frame.origin.x + frame.size.width > visible.origin.x + visible.size.width {
                    frame.origin.x = visible.origin.x + visible.size.width - frame.size.width;
                    changed = true;
                }
                if frame.origin.y + frame.size.height > visible.origin.y + visible.size.height {
                    frame.origin.y = visible.origin.y + visible.size.height - frame.size.height;
                    changed = true;
                }
                if changed {
                    window.setFrame_display(frame, true);
                }
            }"""

if window_creation_old in content:
    content = content.replace(window_creation_old, window_creation_new)
    with open("src/main.rs", "w") as f:
        f.write(content)
    print("Patched main.rs successfully.")
else:
    print("Could not find window creation block.")

