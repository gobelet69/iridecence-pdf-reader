use objc2_app_kit::NSOutlineView;
use objc2::rc::{Retained, Weak};
pub fn test(view: &Retained<NSOutlineView>) {
    let w1 = Weak::new(view);
    let w2 = Weak::from(view);
    let w3 = Weak::from_retained(view);
    let w4 = view.downgrade();
}
