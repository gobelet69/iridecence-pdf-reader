use objc2_app_kit::NSScreen;
use objc2_foundation::MainThreadMarker;
fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let screen = NSScreen::mainScreen(mtm).unwrap();
    let frame = screen.visibleFrame();
    println!("{}x{}", frame.size.width, frame.size.height);
}
