import sys

# FIX MAIN ARRAY CAST
with open('src/main.rs', 'r') as f:
    main = f.read()

target = "let views: [&objc2_app_kit::NSView; 3] = [&**thumb_view, &**right_outline_scroll, &**search_container]; for v in views {"
replacement = """
                thumb_view.topAnchor().constraintEqualToAnchor(&content_wrapper.topAnchor()).setActive(true);
                thumb_view.bottomAnchor().constraintEqualToAnchor(&content_wrapper.bottomAnchor()).setActive(true);
                thumb_view.leadingAnchor().constraintEqualToAnchor(&content_wrapper.leadingAnchor()).setActive(true);
                thumb_view.trailingAnchor().constraintEqualToAnchor(&content_wrapper.trailingAnchor()).setActive(true);
                
                right_outline_scroll.topAnchor().constraintEqualToAnchor(&content_wrapper.topAnchor()).setActive(true);
                right_outline_scroll.bottomAnchor().constraintEqualToAnchor(&content_wrapper.bottomAnchor()).setActive(true);
                right_outline_scroll.leadingAnchor().constraintEqualToAnchor(&content_wrapper.leadingAnchor()).setActive(true);
                right_outline_scroll.trailingAnchor().constraintEqualToAnchor(&content_wrapper.trailingAnchor()).setActive(true);
                
                search_container.topAnchor().constraintEqualToAnchor(&content_wrapper.topAnchor()).setActive(true);
                search_container.bottomAnchor().constraintEqualToAnchor(&content_wrapper.bottomAnchor()).setActive(true);
                search_container.leadingAnchor().constraintEqualToAnchor(&content_wrapper.leadingAnchor()).setActive(true);
                search_container.trailingAnchor().constraintEqualToAnchor(&content_wrapper.trailingAnchor()).setActive(true);
                if false {
"""
main = main.replace(target, replacement)
main = main.replace("                }", "                }\n                }", 1)

with open('src/main.rs', 'w') as f:
    f.write(main)

# FIX PALETTE setIdentifier and signature
with open('src/palette.rs', 'r') as f:
    pal = f.read()

pal = pal.replace("unsafe { cell.setIdentifier(Some(&*identifier)) };", "// removed setIdentifier")

# Fix signature of control_textView_doCommandBySelector
# The trait expects `fn control_textView_doCommandBySelector(&self, control: &NSControl, text_view: &NSTextView, command: Sel) -> bool`
# but maybe it expects `fn control_textView_doCommandBySelector(&self, _control: &NSControl, _text_view: &NSTextView, command: Sel) -> bool`
# or maybe we should just remove that method and handle escape in another way.
# I'll just remove the NSTextFieldDelegate implementation entirely and use `cancelOperation:` in the view or something, or let it close on click outside.
# Actually, the error might be because `command: objc2::runtime::Sel` should be `&objc2::runtime::Sel` or `objc2::runtime::Sel`.

