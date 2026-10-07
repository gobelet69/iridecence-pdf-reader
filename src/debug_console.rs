use objc2::rc::Retained;
use objc2::{
    ClassType, DeclaredClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    runtime::AnyObject, sel,
};
use objc2_app_kit::{
    NSBackingStoreType, NSBezelStyle, NSButton, NSColor, NSFont, NSScrollView, NSStackView,
    NSTextView, NSUserInterfaceLayoutOrientation, NSVisualEffectBlendingMode,
    NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView, NSWindow, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString, NSThread};
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static DEBUG_WINDOW_CONTROLLER: Lazy<Mutex<Option<usize>>> = Lazy::new(|| Mutex::new(None));

pub struct DebugConsoleIvars {
    pub window: std::cell::OnceCell<objc2::rc::Retained<objc2_app_kit::NSWindow>>,
    pub text_view: objc2::rc::Retained<NSTextView>,
    pub is_paused: std::cell::Cell<bool>,
    pub auto_scroll: std::cell::Cell<bool>,
}

define_class!(
    #[unsafe(super(objc2_foundation::NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "DebugConsoleController"]
    #[ivars = DebugConsoleIvars]
    struct DebugConsoleController;

    impl DebugConsoleController {

        #[unsafe(method(clearLogs:))]
        fn clear_logs(&self, _sender: Option<&AnyObject>) {
            if let Ok(mut buffer) = crate::logger::LOG_BUFFER.lock() {
                buffer.clear();
            }
            unsafe { self.ivars().text_view.setString(&NSString::from_str("")) };
        }

        #[unsafe(method(togglePause:))]
        fn toggle_pause(&self, sender: Option<&NSButton>) {
            let paused = self.ivars().is_paused.get();
            self.ivars().is_paused.set(!paused);
            if let Some(btn) = sender {
                if !paused {
                    unsafe { btn.setTitle(&NSString::from_str("Resume")) };
                } else {
                    unsafe { btn.setTitle(&NSString::from_str("Pause")) };
                    // Flush missed logs? Maybe just keep them in buffer
                    let buffer = crate::logger::LOG_BUFFER.lock().unwrap();
                    let mut all_text = String::new();
                    for msg in buffer.iter() {
                        all_text.push_str(&format!("[{}] [{}] [{}] {}\n", msg.timestamp, msg.thread_name, msg.category, msg.event));
                    }
                    unsafe { self.ivars().text_view.setString(&NSString::from_str(&all_text)) };
                }
            }
        }

        #[unsafe(method(toggleAutoScroll:))]
        fn toggle_auto_scroll(&self, sender: Option<&NSButton>) {
            let auto = self.ivars().auto_scroll.get();
            self.ivars().auto_scroll.set(!auto);
            if let Some(btn) = sender {
                unsafe { btn.setState(if !auto { 1 } else { 0 }) };
            }
        }

        #[unsafe(method(appendLogLine:))]
        fn append_log_line(&self, line: &NSString) {
            if self.ivars().is_paused.get() { return; }

            let text_storage: *mut AnyObject = unsafe { objc2::msg_send![&self.ivars().text_view, textStorage] };
            let attr_str: *mut AnyObject = unsafe { objc2::msg_send![objc2::class!(NSAttributedString), alloc] };
            let attr_str: *mut AnyObject = unsafe { objc2::msg_send![attr_str, initWithString: line] };
            let _: () = unsafe { objc2::msg_send![text_storage, appendAttributedString: attr_str] };

            if self.ivars().auto_scroll.get() {
                let length: objc2_foundation::NSUInteger = unsafe { objc2::msg_send![text_storage, length] };
                let range = objc2_foundation::NSRange::new(length, 0);
                let _: () = unsafe { objc2::msg_send![&self.ivars().text_view, scrollRangeToVisible: range] };
            }
        }
    }
);

pub fn show_debug_console() {
    unsafe {
        let mut guard = DEBUG_WINDOW_CONTROLLER.lock().unwrap();
        if guard.is_none() {
            let vc = DebugConsoleController::new(objc2::MainThreadMarker::new().unwrap());
            *guard = Some(Retained::into_raw(vc) as usize);
        }

        if let Some(ptr) = *guard {
            let vc = ptr as *mut AnyObject;
            let window = {
                let vc_retained: &DebugConsoleController = unsafe { &*(vc as *const _) };
                vc_retained.ivars().window.get().unwrap()
            };
            let _: () =
                objc2::msg_send![&**window, makeKeyAndOrderFront: std::ptr::null::<AnyObject>()];
        }
    }
}

impl DebugConsoleController {
    pub fn new(mtm: objc2::MainThreadMarker) -> Retained<Self> {
        let text_view = unsafe {
            NSTextView::initWithFrame(
                NSTextView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(800.0, 560.0)),
            )
        };
        let this = Self::alloc(mtm).set_ivars(DebugConsoleIvars {
            window: std::cell::OnceCell::new(),
            text_view,
            is_paused: std::cell::Cell::new(false),
            auto_scroll: std::cell::Cell::new(true),
        });
        let this: Retained<Self> = unsafe { objc2::msg_send_id![super(this), init] };

        let rect = NSRect::new(NSPoint::new(100.0, 100.0), NSSize::new(800.0, 600.0));
        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                rect,
                NSWindowStyleMask::Titled
                    | NSWindowStyleMask::Closable
                    | NSWindowStyleMask::Resizable,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        unsafe { window.setTitle(&NSString::from_str("Live Debug Console")) };
        unsafe { window.setReleasedWhenClosed(false) };

        let stack = unsafe { NSStackView::initWithFrame(NSStackView::alloc(mtm), rect) };
        unsafe { stack.setOrientation(NSUserInterfaceLayoutOrientation::Vertical) };
        unsafe { stack.setSpacing(0.0) };

        let toolbar = unsafe {
            NSStackView::initWithFrame(
                NSStackView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(800.0, 40.0)),
            )
        };
        unsafe { toolbar.setOrientation(NSUserInterfaceLayoutOrientation::Horizontal) };
        unsafe { toolbar.setSpacing(10.0) };
        unsafe {
            toolbar.setEdgeInsets(objc2_foundation::NSEdgeInsets {
                top: 10.0,
                left: 10.0,
                bottom: 10.0,
                right: 10.0,
            })
        };

        let btn_clear = unsafe {
            NSButton::buttonWithTitle_target_action(
                &NSString::from_str("Clear"),
                Some(&*this),
                Some(sel!(clearLogs:)),
                mtm,
            )
        };
        let btn_pause = unsafe {
            NSButton::buttonWithTitle_target_action(
                &NSString::from_str("Pause"),
                Some(&*this),
                Some(sel!(togglePause:)),
                mtm,
            )
        };
        let btn_autoscroll = unsafe {
            NSButton::buttonWithTitle_target_action(
                &NSString::from_str("Auto-scroll"),
                Some(&*this),
                Some(sel!(toggleAutoScroll:)),
                mtm,
            )
        };
        unsafe { btn_autoscroll.setState(1) };

        unsafe {
            toolbar.addView_inGravity(&btn_clear, objc2_app_kit::NSStackViewGravity::Leading)
        };
        unsafe {
            toolbar.addView_inGravity(&btn_pause, objc2_app_kit::NSStackViewGravity::Leading)
        };
        unsafe {
            toolbar.addView_inGravity(&btn_autoscroll, objc2_app_kit::NSStackViewGravity::Leading)
        };

        let scroll = unsafe {
            NSScrollView::initWithFrame(
                NSScrollView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(800.0, 560.0)),
            )
        };
        unsafe { scroll.setHasVerticalScroller(true) };
        unsafe { scroll.setAutohidesScrollers(true) };

        unsafe { this.ivars().text_view.setEditable(false) };
        unsafe { this.ivars().text_view.setSelectable(true) };
        unsafe { this.ivars().text_view.setRichText(false) };
        unsafe {
            this.ivars()
                .text_view
                .setFont(Some(&NSFont::monospacedSystemFontOfSize_weight(12.0, 0.0)))
        };
        unsafe {
            this.ivars()
                .text_view
                .setTextColor(Some(&*NSColor::textColor()))
        };
        unsafe {
            this.ivars()
                .text_view
                .setBackgroundColor(&NSColor::textBackgroundColor())
        };

        unsafe { scroll.setDocumentView(Some(&*this.ivars().text_view)) };
        unsafe { stack.addView_inGravity(&toolbar, objc2_app_kit::NSStackViewGravity::Top) };
        unsafe { stack.addView_inGravity(&scroll, objc2_app_kit::NSStackViewGravity::Top) };
        unsafe { window.setContentView(Some(&stack)) };

        this.ivars().window.set(window.clone());

        let buffer = crate::logger::LOG_BUFFER.lock().unwrap();
        let mut all_text = String::new();
        for msg in buffer.iter() {
            all_text.push_str(&format!(
                "[{}] [{}] [{}] {}\n",
                msg.timestamp, msg.thread_name, msg.category, msg.event
            ));
        }
        unsafe {
            this.ivars()
                .text_view
                .setString(&NSString::from_str(&all_text))
        };

        this
    }
}

unsafe impl objc2_foundation::NSObjectProtocol for DebugConsoleController {}
