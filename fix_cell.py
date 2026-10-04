with open("src/workspace_delegate.rs", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "let text_field = unsafe { NSTextField::initWithFrame" in line:
        # text field is width sizable
        lines.insert(i+1, "                    text_field.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable);\n")
    if "cloud_iv.setHidden(true);" in line:
        lines.insert(i, "                    cloud_iv.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin);\n")
    if "unsafe { progress.setIdentifier" in line:
        lines.insert(i+1, "                    progress.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin);\n")

with open("src/workspace_delegate.rs", "w") as f:
    f.writelines(lines)
