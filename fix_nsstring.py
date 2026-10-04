with open('src/workspace_delegate.rs', 'r') as f:
    content = f.read()

content = content.replace(
    'let icon_name = if icloud_status == 1 { "icloud.and.arrow.down" } else { "exclamationmark.icloud" };',
    ''
)
content = content.replace(
    'objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(objc2_foundation::ns_string!(icon_name), None)',
    'objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(if icloud_status == 1 { objc2_foundation::ns_string!("icloud.and.arrow.down") } else { objc2_foundation::ns_string!("exclamationmark.icloud") }, None)'
)

with open('src/workspace_delegate.rs', 'w') as f:
    f.write(content)
