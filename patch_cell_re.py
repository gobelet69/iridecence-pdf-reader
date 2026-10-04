import re

with open('src/workspace_delegate.rs', 'r') as f:
    content = f.read()

# Replace text_field frame
content = content.replace(
    'let text_field = unsafe { NSTextField::initWithFrame(NSTextField::alloc(mtm), NSRect::new(NSPoint::new(24.0, 3.0), NSSize::new(176.0, 18.0))) };',
    'let text_field = unsafe { NSTextField::initWithFrame(NSTextField::alloc(mtm), NSRect::new(NSPoint::new(24.0, 3.0), NSSize::new(156.0, 18.0))) };'
)

# Add text_field.setAutoresizingMask
if 'text_field.setAutoresizingMask' not in content:
    content = content.replace(
        'text_field.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingMiddle);',
        'text_field.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingMiddle);\n                    text_field.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable);'
    )

# Replace cloud_iv frame and add mask
content = content.replace(
    'let cloud_iv = objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));',
    'let cloud_iv = objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(180.0, 4.0), NSSize::new(16.0, 16.0)));'
)
if 'cloud_iv.setAutoresizingMask' not in content:
    content = content.replace(
        'cloud_iv.setHidden(true);',
        'cloud_iv.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin);\n                    cloud_iv.setHidden(true);'
    )

# Replace progress frame and add mask
content = content.replace(
    'let progress = objc2_app_kit::NSProgressIndicator::initWithFrame(objc2_app_kit::NSProgressIndicator::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));',
    'let progress = objc2_app_kit::NSProgressIndicator::initWithFrame(objc2_app_kit::NSProgressIndicator::alloc(mtm), NSRect::new(NSPoint::new(180.0, 4.0), NSSize::new(16.0, 16.0)));'
)
if 'progress.setAutoresizingMask' not in content:
    content = content.replace(
        'progress.setStyle(objc2_app_kit::NSProgressIndicatorStyle::Spinning);',
        'progress.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin);\n                    progress.setStyle(objc2_app_kit::NSProgressIndicatorStyle::Spinning);'
    )

with open('src/workspace_delegate.rs', 'w') as f:
    f.write(content)
print("Patched cell!")
