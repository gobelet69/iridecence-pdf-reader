with open("src/main.rs", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "window.center();" in line:
        lines.insert(i+1, "            unsafe { window.setTitlebarAppearsTransparent(true) };\n")
        # Keep title visibility default (or Hidden)
        # lines.insert(i+2, "            unsafe { window.setTitleVisibility(objc2_app_kit::NSWindowTitleVisibility::Hidden) };\n")
        break

with open("src/main.rs", "w") as f:
    f.writelines(lines)
