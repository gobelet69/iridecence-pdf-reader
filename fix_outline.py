with open("src/main.rs", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "outline_view.setIntercellSpacing(NSSize::new(4.0, 0.0));" in line:
        lines.insert(i+1, "                outline_view.setIndentationPerLevel(14.0);\n")
        lines.insert(i+2, "                outline_view.setIndentationMarkerFollowsCell(true);\n")
        lines.insert(i+3, "                let _: () = msg_send![&outline_view, setFloatsGroupRows: false];\n")
        break

with open("src/main.rs", "w") as f:
    f.writelines(lines)
