import sys

with open('src/main.rs', 'r') as f:
    main = f.read()

# find `if false {` and matching `}`
s_idx = main.find("if false {")
if s_idx != -1:
    e_idx = main.find("}", s_idx)
    main = main[:s_idx] + main[e_idx+1:]

with open('src/main.rs', 'w') as f:
    f.write(main)

with open('src/palette.rs', 'r') as f:
    pal = f.read()

pal = pal.replace("unsafe { cell.setIdentifier(Some(identifier)) };", "// removed")
pal = pal.replace("unsafe { cell.setIdentifier(Some(&*identifier)) };", "// removed")

# Remove doCommandBySelector
s_idx = pal.find("#[unsafe(method(control:textView:doCommandBySelector:))]")
if s_idx != -1:
    e_idx = pal.find("}", pal.find("}", pal.find("}", s_idx) + 1) + 1)
    pal = pal[:s_idx] + pal[e_idx+1:]

with open('src/palette.rs', 'w') as f:
    f.write(pal)

