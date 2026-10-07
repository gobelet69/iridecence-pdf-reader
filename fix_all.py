import sys

# FIX PALETTE
with open('src/palette.rs', 'r') as f:
    pal = f.read()

# Add DeclaredClass, NSControlTextEditingDelegate, NSWindowLevelFloating
pal = pal.replace("use objc2::{define_class", "use objc2::{DeclaredClass, define_class")
pal = pal.replace("use objc2_app_kit::{", "use objc2_app_kit::{NSControlTextEditingDelegate, ")

# Implement NSControlTextEditingDelegate
nscontroltext = "unsafe impl NSObjectProtocol for CommandPaletteDelegate {}\n\n    unsafe impl NSControlTextEditingDelegate for CommandPaletteDelegate {}\n"
pal = pal.replace("unsafe impl NSObjectProtocol for CommandPaletteDelegate {}", nscontroltext)

# Fix setIdentifier
pal = pal.replace("unsafe { cell.setIdentifier(Some(identifier)) };", "unsafe { cell.setIdentifier(Some(&*identifier)) };")

# Fix sel_name
pal = pal.replace('sel_name == "cancelOperation:"', 'sel_name.to_str().unwrap() == "cancelOperation:"')

# Fix update_search visibility
pal = pal.replace("fn update_search(", "pub fn update_search(")

# Fix setBackgroundColor
pal = pal.replace("panel.setBackgroundColor(&NSColor::windowBackgroundColor());", "panel.setBackgroundColor(Some(&NSColor::windowBackgroundColor()));")

# Fix NSWindowLevel
pal = pal.replace("panel.setLevel(NSWindowLevel::Floating);", "panel.setLevel(objc2_app_kit::NSFloatingWindowLevel as isize);")
# Sometimes NSFloatingWindowLevel is i32, sometimes it's castable to isize. Let's use NSFloatingWindowLevel as isize

# Fix missing True return in doCommandBySelector
pal = pal.replace("""if let Some(p) = self.ivars().panel.get() {
                    unsafe { p.close() };
                }
                return true;""", """if let Some(p) = self.ivars().panel.get() {
                    unsafe { p.close() };
                }
                return true;""")

with open('src/palette.rs', 'w') as f:
    f.write(pal)

# FIX MAIN ARRAY CAST
with open('src/main.rs', 'r') as f:
    main = f.read()

main = main.replace(
    "for v in [(&**thumb_view) as &objc2_app_kit::NSView, (&**right_outline_scroll) as &objc2_app_kit::NSView, (&**search_container) as &objc2_app_kit::NSView] {",
    "let views: [&objc2_app_kit::NSView; 3] = [&**thumb_view, &**right_outline_scroll, &**search_container]; for v in views {"
)

with open('src/main.rs', 'w') as f:
    f.write(main)

