use objc2_app_kit::{NSSplitViewItem, NSViewController};
use objc2::rc::Retained;

fn test(vc: &NSViewController) -> Retained<NSSplitViewItem> {
    unsafe { NSSplitViewItem::inspectorWithViewController(vc) }
}
