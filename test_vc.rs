use objc2_app_kit::{NSViewController, NSSplitViewController};
use objc2_foundation::MainThreadMarker;
fn test(mtm: MainThreadMarker) {
    let vc = NSViewController::new(mtm);
}
