with open("src/main.rs", "r") as f:
    content = f.read()

split_view_def = """
define_class!(
    #[unsafe(super(NSSplitView))]
    #[name = "CustomSplitView"]
    struct CustomSplitView;

    impl CustomSplitView {
        #[unsafe(method(dividerThickness))]
        fn divider_thickness(&self) -> f64 {
            12.0
        }
        
        #[unsafe(method(drawDividerInRect:))]
        fn draw_divider_in_rect(&self, mut rect: NSRect) {
            rect.origin.x += (rect.size.width - 1.0) / 2.0;
            rect.size.width = 1.0;
            unsafe {
                let color = objc2_app_kit::NSColor::separatorColor();
                color.set();
                objc2_app_kit::NSBezierPath::fillRect(rect);
            }
        }
    }
);

"""

content = content.replace("#[derive(Default)]\nstruct AppDelegateIvars", split_view_def + "#[derive(Default)]\nstruct AppDelegateIvars")

setup_code = """
            // Split View Controller
            let split_vc = NSSplitViewController::new(mtm);
            
            let custom_split_alloc: objc2::rc::Allocated<CustomSplitView> = unsafe { objc2::msg_send![CustomSplitView::class(), alloc] };
            let custom_split: Retained<CustomSplitView> = unsafe { objc2::msg_send![custom_split_alloc, initWithFrame: rect] };
            unsafe { split_vc.setView(&custom_split) };
"""

content = content.replace("            // Split View Controller\n            let split_vc = NSSplitViewController::new(mtm);", setup_code)

with open("src/main.rs", "w") as f:
    f.write(content)
