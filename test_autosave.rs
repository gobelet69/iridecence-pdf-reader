use objc2_app_kit::{NSWindow, NSWindowStyleMask, NSBackingStoreType};
use objc2_foundation::{NSRect, NSPoint, NSSize, MainThreadMarker, ns_string};
use objc2::rc::Retained;
pub fn test(mtm: MainThreadMarker) {
    let window = unsafe { NSWindow::initWithContentRect_styleMask_backing_defer(NSWindow::alloc(mtm), NSRect::new(NSPoint::new(0., 0.), NSSize::new(10., 10.)), NSWindowStyleMask::Titled, NSBackingStoreType::Buffered, false) };
    window.setFrameAutosaveName(ns_string!("MainWindow"));
    window.setMinSize(NSSize::new(760.0, 520.0));
}
