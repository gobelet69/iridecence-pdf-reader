import re

with open("src/main.rs", "r") as f:
    content = f.read()

# 1. Remove define_class! for CustomSplitView
pattern_class = r"define_class!\(\s*#\[unsafe\(super\(NSSplitView\)\)\]\s*#\[name = \"CustomSplitView\"\]\s*struct CustomSplitView;.*?\}\s*\);\n"
content = re.sub(pattern_class, "", content, flags=re.DOTALL)

# 2. Replace the split_vc initialization block
pattern_init = r"// Split View Controller\s*let split_vc = NSSplitViewController::new\(mtm\);\s*let custom_split_alloc.*?split_vc\.setView\(&custom_split\)\s*\}?;"
replacement = "// Split View Controller\n            let split_vc = NSSplitViewController::new(mtm);"
content = re.sub(pattern_init, replacement, content, flags=re.DOTALL)

with open("src/main.rs", "w") as f:
    f.write(content)
