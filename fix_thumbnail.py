import re

with open('src/main.rs', 'r') as f:
    content = f.read()

old_thumb = """                #[unsafe(method(thumbnailFrameChanged:))]
        fn thumbnail_frame_changed(&self, notification: &objc2_foundation::NSNotification) {
            if let Some(obj) = unsafe { notification.object() } {
                if let Some(view) = obj.downcast_ref::<NSView>() {
                    let frame = view.frame();
                    let width = frame.size.width;
                    let thumb_width = (width - 40.0).max(50.0);
                    let thumb_height = thumb_width * 1.414;
                    if let Some(thumb_view) = self.ivars().thumb_view.get() {
                        unsafe { thumb_view.setThumbnailSize(NSSize::new(thumb_width, thumb_height)) };
                    }
                }
            }
        }"""
        
new_thumb = """                #[unsafe(method(thumbnailFrameChanged:))]
        fn thumbnail_frame_changed(&self, notification: &objc2_foundation::NSNotification) {
            if let Some(obj) = unsafe { notification.object() } {
                if let Some(view) = obj.downcast_ref::<NSView>() {
                    let frame = view.frame();
                    let width = frame.size.width;
                    
                    if let Some(thumb_view) = self.ivars().thumb_view.get() {
                        let old_frame = thumb_view.frame();
                        let new_thumb_width = (width * 0.75).max(50.0);
                        let new_thumb_height = new_thumb_width * 1.414;
                        
                        // Throttle updates: only resize if width changed by > 5pt
                        if (old_frame.size.width - new_thumb_width).abs() > 5.0 {
                            unsafe { 
                                thumb_view.setFrame(NSRect::new(NSPoint::new((width - new_thumb_width) / 2.0, 0.0), NSSize::new(new_thumb_width, frame.size.height)));
                                thumb_view.setThumbnailSize(NSSize::new(new_thumb_width, new_thumb_height));
                            }
                        }
                    }
                }
            }
        }"""
        
content = content.replace(old_thumb, new_thumb)

# Fix remaining 200.0 width issues in content_wrapper.setFrame
content = content.replace("content_wrapper.setFrame(NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 700.0 - 41.0)));", "content_wrapper.setFrame(NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0 - 41.0)));")

with open('src/main.rs', 'w') as f:
    f.write(content)
