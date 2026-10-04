import re

with open('src/workspace_delegate.rs', 'r') as f:
    content = f.read()

old_cell = """                let image_view = unsafe { objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(2.0, 4.0), NSSize::new(16.0, 16.0))) };
                let text_field = unsafe { NSTextField::initWithFrame(NSTextField::alloc(mtm), NSRect::new(NSPoint::new(24.0, 3.0), NSSize::new(176.0, 18.0))) };
                unsafe {
                    text_field.setBezeled(false);
                    text_field.setDrawsBackground(false);
                    text_field.setEditable(false);
                    text_field.setSelectable(false);
                    
                    let font = objc2_app_kit::NSFont::systemFontOfSize(13.0);
                    text_field.setFont(Some(&font));
                    text_field.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingMiddle);
                    
                    cell.addSubview(&image_view);
                    cell.addSubview(&text_field);
                    cell.setImageView(Some(&image_view));
                    cell.setTextField(Some(&text_field));
                    
                    let cloud_iv = objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));
                    unsafe { cloud_iv.setIdentifier(Some(objc2_foundation::ns_string!("CloudIcon"))) };
                    cloud_iv.setHidden(true);
                    
                    let progress = objc2_app_kit::NSProgressIndicator::initWithFrame(objc2_app_kit::NSProgressIndicator::alloc(mtm), NSRect::new(NSPoint::new(176.0, 4.0), NSSize::new(16.0, 16.0)));
                    unsafe { progress.setIdentifier(Some(objc2_foundation::ns_string!("CloudProgress"))) };
                    progress.setStyle(objc2_app_kit::NSProgressIndicatorStyle::Spinning);
                    progress.setControlSize(objc2_app_kit::NSControlSize::Small);
                    progress.setHidden(true);
                    
                    cell.addSubview(&cloud_iv);
                    cell.addSubview(&progress);"""

new_cell = """                let image_view = unsafe { objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(2.0, 4.0), NSSize::new(16.0, 16.0))) };
                let text_field = unsafe { NSTextField::initWithFrame(NSTextField::alloc(mtm), NSRect::new(NSPoint::new(24.0, 3.0), NSSize::new(156.0, 18.0))) };
                unsafe {
                    text_field.setBezeled(false);
                    text_field.setDrawsBackground(false);
                    text_field.setEditable(false);
                    text_field.setSelectable(false);
                    
                    let font = objc2_app_kit::NSFont::systemFontOfSize(13.0);
                    text_field.setFont(Some(&font));
                    text_field.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByTruncatingMiddle);
                    text_field.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable);
                    
                    cell.addSubview(&image_view);
                    cell.addSubview(&text_field);
                    cell.setImageView(Some(&image_view));
                    cell.setTextField(Some(&text_field));
                    
                    let cloud_iv = objc2_app_kit::NSImageView::initWithFrame(objc2_app_kit::NSImageView::alloc(mtm), NSRect::new(NSPoint::new(180.0, 4.0), NSSize::new(16.0, 16.0)));
                    unsafe { cloud_iv.setIdentifier(Some(objc2_foundation::ns_string!("CloudIcon"))) };
                    cloud_iv.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin);
                    cloud_iv.setHidden(true);
                    
                    let progress = objc2_app_kit::NSProgressIndicator::initWithFrame(objc2_app_kit::NSProgressIndicator::alloc(mtm), NSRect::new(NSPoint::new(180.0, 4.0), NSSize::new(16.0, 16.0)));
                    unsafe { progress.setIdentifier(Some(objc2_foundation::ns_string!("CloudProgress"))) };
                    progress.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin);
                    progress.setStyle(objc2_app_kit::NSProgressIndicatorStyle::Spinning);
                    progress.setControlSize(objc2_app_kit::NSControlSize::Small);
                    progress.setHidden(true);
                    
                    cell.addSubview(&cloud_iv);
                    cell.addSubview(&progress);"""

if old_cell in content:
    content = content.replace(old_cell, new_cell)
    with open('src/workspace_delegate.rs', 'w') as f:
        f.write(content)
    print("Patched successfully!")
else:
    print("Old cell not found!")
