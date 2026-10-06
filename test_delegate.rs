use objc2_app_kit::NSSplitViewController;
use objc2_foundation::MainThreadMarker;

fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let split_vc = NSSplitViewController::new(mtm);
    let split_view = split_vc.splitView();
    let delegate = split_view.delegate();
    if delegate.is_some() {
        println!("NSSplitViewController is the delegate!");
    } else {
        println!("No delegate!");
    }
}
