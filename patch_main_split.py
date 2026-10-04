import re
with open('src/main.rs', 'r') as f:
    content = f.read()

split_vc_code = """
define_class!(
    #[unsafe(super(NSSplitViewController))]
    #[name = "MainSplitViewController"]
    struct MainSplitViewController;

    impl MainSplitViewController {
        #[unsafe(method(splitView:effectiveRect:forDrawnRect:ofDividerAtIndex:))]
        fn effective_rect(&self, split_view: &NSSplitView, proposed_effective_rect: NSRect, drawn_rect: NSRect, divider_index: isize) -> NSRect {
            let mut rect = proposed_effective_rect;
            // The right divider is index 1 (between main and right)
            if divider_index == 1 {
                // Expand the hit area to the left and right
                rect.origin.x -= 8.0;
                rect.size.width += 16.0;
            }
            rect
        }
    }
);
"""

# Insert before AppDelegate
idx = content.find("define_class!(")
content = content[:idx] + split_vc_code + "\n" + content[idx:]

# Replace NSSplitViewController::new(mtm) with MainSplitViewController::new(mtm)
# Wait, MainSplitViewController doesn't have a new() method directly exposed if we don't define it.
# We can define `fn new(mtm: MainThreadMarker) -> Retained<Self>`
split_vc_code_full = """
define_class!(
    #[unsafe(super(NSSplitViewController))]
    #[name = "MainSplitViewController"]
    struct MainSplitViewController;

    impl MainSplitViewController {
        #[unsafe(method(splitView:effectiveRect:forDrawnRect:ofDividerAtIndex:))]
        fn effective_rect(&self, split_view: &NSSplitView, proposed_effective_rect: NSRect, drawn_rect: NSRect, divider_index: isize) -> NSRect {
            let mut rect = proposed_effective_rect;
            // The right divider is index 1 (between main and right)
            if divider_index == 1 {
                // Expand the hit area to the left and right
                rect.origin.x -= 8.0;
                rect.size.width += 16.0;
            }
            rect
        }
    }
);

impl MainSplitViewController {
    fn new(mtm: objc2::rc::MainThreadMarker) -> objc2::rc::Retained<Self> {
        let this = Self::alloc(mtm);
        unsafe { objc2::msg_send_id![super(this), init] }
    }
}
"""

with open('src/main.rs', 'r') as f:
    content = f.read()

idx = content.find("define_class!(")
content = content[:idx] + split_vc_code_full + "\n" + content[idx:]

content = content.replace("let split_vc = NSSplitViewController::new(mtm);", "let split_vc = MainSplitViewController::new(mtm);")
content = content.replace("Retained::cast_unchecked::<NSSplitViewController>(split_vc)", "Retained::cast_unchecked::<NSSplitViewController>(split_vc as _)")

with open('src/main.rs', 'w') as f:
    f.write(content)
