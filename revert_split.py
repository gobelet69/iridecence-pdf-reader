with open("src/main.rs", "r") as f:
    content = f.read()

bad_code = """            
            let custom_split_alloc: objc2::rc::Allocated<CustomSplitView> = unsafe { objc2::msg_send![CustomSplitView::class(), alloc] };
            let custom_split: Retained<CustomSplitView> = unsafe { objc2::msg_send![custom_split_alloc, initWithFrame: rect] };
            unsafe { split_vc.setView(&custom_split) };"""

content = content.replace(bad_code, "")

with open("src/main.rs", "w") as f:
    f.write(content)
