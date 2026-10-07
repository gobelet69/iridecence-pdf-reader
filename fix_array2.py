import sys

with open('src/main.rs', 'r') as f:
    content = f.read()

target = "for v in [thumb_view.as_ref(), right_outline_scroll.as_ref(), search_container.as_ref()] {"
replacement = "for v in [(&**thumb_view) as &objc2_app_kit::NSView, (&**right_outline_scroll) as &objc2_app_kit::NSView, (&**search_container) as &objc2_app_kit::NSView] {"
content = content.replace(target, replacement)

with open('src/main.rs', 'w') as f:
    f.write(content)
