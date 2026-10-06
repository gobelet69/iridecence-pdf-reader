import re

with open("src/main.rs", "r") as f:
    content = f.read()

# 1. Add CustomSplitViewController definition
controller_def = """
define_class!(
    #[unsafe(super(NSSplitViewController))]
    #[name = "CustomSplitViewController"]
    struct CustomSplitViewController;

    impl CustomSplitViewController {
        #[unsafe(method(splitView:effectiveRect:forDrawnRect:ofDividerAtIndex:))]
        #[allow(non_snake_case)]
        fn splitView_effectiveRect_forDrawnRect_ofDividerAtIndex(
            &self,
            _split_view: &NSSplitView,
            proposed_effective_rect: NSRect,
            _drawn_rect: NSRect,
            _divider_index: NSInteger,
        ) -> NSRect {
            let mut rect = proposed_effective_rect;
            // Expand the invisible hit area by 5 points on each side
            rect.origin.x -= 5.0;
            rect.size.width += 10.0;
            rect
        }
    }
);

"""
content = content.replace("#[derive(Default)]\nstruct AppDelegateIvars", controller_def + "#[derive(Default)]\nstruct AppDelegateIvars")

# 2. Update initialization
init_old = r"// Split View Controller\s*let split_vc = NSSplitViewController::new\(mtm\);"
init_new = """// Split View Controller
            let split_vc_alloc: objc2::rc::Allocated<CustomSplitViewController> = unsafe { objc2::msg_send![CustomSplitViewController::class(), alloc] };
            let split_vc: Retained<CustomSplitViewController> = unsafe { objc2::msg_send![split_vc_alloc, init] };"""
content = re.sub(init_old, init_new, content, flags=re.DOTALL)

with open("src/main.rs", "w") as f:
    f.write(content)
