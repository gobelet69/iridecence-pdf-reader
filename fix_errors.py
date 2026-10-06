import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# Fix imports
if "NSImage, NSImageView, NSColor, NSTextField, NSImageSymbolConfiguration, NSFontWeightRegular, NSImageSymbolScaleLarge" not in content:
    content = content.replace("NSTableColumn, NSOutlineViewDelegate, NSOutlineViewDataSource", 
                              "NSTableColumn, NSOutlineViewDelegate, NSOutlineViewDataSource, NSImage, NSImageView, NSColor, NSTextField, NSImageSymbolConfiguration, NSFontWeightRegular, NSImageSymbolScaleLarge")

# Fix method definition
old_method = """        fn update_outline_empty_state(&self) {"""
new_method = """        #[unsafe(method(updateOutlineEmptyState))]
        fn update_outline_empty_state(&self) {"""

content = content.replace(old_method, new_method)

# Fix method call
old_call = """self.update_outline_empty_state();"""
new_call = """let _: () = unsafe { objc2::msg_send![self, updateOutlineEmptyState] };"""

content = content.replace(old_call, new_call)

with open('src/main.rs', 'w') as f:
    f.write(content)
