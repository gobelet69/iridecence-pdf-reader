import sys

with open('src/palette.rs', 'r') as f:
    pal = f.read()

pal = pal.replace("item as *const _ as *mut _", "item as *const AnyObject as *mut AnyObject")

with open('src/palette.rs', 'w') as f:
    f.write(pal)
