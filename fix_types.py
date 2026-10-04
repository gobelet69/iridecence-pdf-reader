import re

with open("src/main.rs", "r") as f:
    content = f.read()

# 1. Add NSInteger to imports
content = content.replace("use objc2_foundation::{NSRect, NSPoint, NSSize, MainThreadMarker, NSString, NSURL};", "use objc2_foundation::{NSRect, NSPoint, NSSize, MainThreadMarker, NSString, NSURL, NSInteger};")

# 2. Add unchecked cast
old = """            self.ivars().split_vc.set(split_vc).unwrap();"""
new = """            let split_vc_base = unsafe { Retained::cast_unchecked::<NSSplitViewController>(split_vc) };
            self.ivars().split_vc.set(split_vc_base).unwrap();"""
content = content.replace(old, new)

with open("src/main.rs", "w") as f:
    f.write(content)
