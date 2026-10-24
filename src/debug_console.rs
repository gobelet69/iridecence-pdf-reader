use objc2::rc::Retained;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, runtime::AnyObject, sel};
use objc2_app_kit::{
    NSBackingStoreType, NSButton, NSColor, NSFont, NSScrollView, NSTextView, NSView,
    NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
    NSWindow, NSWindowStyleMask,
};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSSize, NSString};
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
            let mut buffer = crate::logger::LOG_BUFFER
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            buffer.clear();
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
                    let buffer = crate::logger::LOG_BUFFER
                        .lock()
                        .unwrap_or_else(|p| p.into_inner());
                    let mut all_text = String::new();
                    for msg in buffer.iter() {
                        all_text.push_str(&format!(
                            "[{}] [{}] [{}] {}\n",
                            msg.timestamp, msg.thread_name, msg.category, msg.event
                        ));
                    }
                    unsafe {
                        self.ivars()
                            .text_view
                            .setString(&NSString::from_str(&all_text));
                        self.ivars()
                            .text_view
                            .setFont(Some(&NSFont::monospacedSystemFontOfSize_weight(11.5, 0.0)));
                        self.ivars()
                            .text_view
                            .setTextColor(Some(&NSColor::labelColor()));
                    };
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
            if self.ivars().is_paused.get() {
                return;
            }

            unsafe {
                let text_storage: *mut AnyObject = msg_send![&self.ivars().text_view, textStorage];
                if !text_storage.is_null() {
                    let mutable_str: *mut AnyObject = msg_send![text_storage, mutableString];
                    if !mutable_str.is_null() {
                        let _: () = msg_send![mutable_str, appendString: line];
                    }
                    self.ivars()
                        .text_view
                        .setFont(Some(&NSFont::monospacedSystemFontOfSize_weight(11.5, 0.0)));
                    self.ivars()
                        .text_view
                        .setTextColor(Some(&NSColor::labelColor()));

                    if self.ivars().auto_scroll.get() {
                        let length: objc2_foundation::NSUInteger = msg_send![text_storage, length];
                        let range = objc2_foundation::NSRange::new(length, 0);
                        let _: () = msg_send![&self.ivars().text_view, scrollRangeToVisible: range];
                    }
                }
            }
        }
    }
);

pub fn show_debug_console() {
    let Some(mtm) = objc2::MainThreadMarker::new() else {
        return;
    };
    let mut guard = DEBUG_WINDOW_CONTROLLER
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    if guard.is_none() {
        let vc = DebugConsoleController::new(mtm);
        *guard = Some(Retained::into_raw(vc) as usize);
    }

    if let Some(ptr) = *guard {
        let vc = ptr as *mut AnyObject;
        let vc_retained: &DebugConsoleController = unsafe { &*(vc as *const _) };
        if let Some(window) = vc_retained.ivars().window.get() {
            crate::window_geometry::WindowGeometry::sanitize_window(
                window,
                crate::window_geometry::WindowKind::DebugConsole,
                mtm,
            );
            let _: () = unsafe {
                msg_send![&**window, makeKeyAndOrderFront: std::ptr::null::<AnyObject>()]
            };
        }
    }
}

pub fn get_debug_window() -> Option<Retained<NSWindow>> {
    let guard = DEBUG_WINDOW_CONTROLLER
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let ptr = (*guard)?;
    let vc_retained: &DebugConsoleController = unsafe { &*(ptr as *const _) };
    vc_retained.ivars().window.get().cloned()
}

impl DebugConsoleController {
    pub fn new(mtm: objc2::MainThreadMarker) -> Retained<Self> {
        let text_view = unsafe {
            NSTextView::initWithFrame(
                NSTextView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(720.0, 480.0)),
            )
        };
        let this = Self::alloc(mtm).set_ivars(DebugConsoleIvars {
            window: std::cell::OnceCell::new(),
            text_view,
            is_paused: std::cell::Cell::new(false),
            auto_scroll: std::cell::Cell::new(true),
        });
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };

        let vis = crate::window_geometry::WindowGeometry::screen_visible_frame(None, mtm);
        let rect = crate::window_geometry::WindowGeometry::default_frame(
            vis,
            crate::window_geometry::WindowKind::DebugConsole,
        );
        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                rect,
                NSWindowStyleMask::Titled
                    | NSWindowStyleMask::Closable
                    | NSWindowStyleMask::Resizable
                    | NSWindowStyleMask::FullSizeContentView,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        unsafe {
            window.setTitle(&NSString::from_str("Live Debug Console"));
            window.setTitlebarAppearsTransparent(true);
            window.setOpaque(false);
            window.setBackgroundColor(Some(&NSColor::clearColor()));
            window.setHasShadow(true);
            window.setReleasedWhenClosed(false);
        };
        crate::window_geometry::WindowGeometry::sanitize_window(
            &window,
            crate::window_geometry::WindowKind::DebugConsole,
            mtm,
        );

        let toolbar = unsafe {
            NSView::initWithFrame(
                NSView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(800.0, 30.0)),
            )
        };

        let btn_clear = unsafe {
            let btn = NSButton::buttonWithTitle_target_action(
                &NSString::from_str("Clear"),
                Some(&*this),
                Some(sel!(clearLogs:)),
                mtm,
            );
            let _: () = msg_send![&btn, setBezelStyle: 1usize];
            let _: () = msg_send![&btn, setControlSize: 1usize];
            btn.setFont(Some(&NSFont::systemFontOfSize(11.0)));
            btn
        };
        let btn_pause = unsafe {
            let btn = NSButton::buttonWithTitle_target_action(
                &NSString::from_str("Pause"),
                Some(&*this),
                Some(sel!(togglePause:)),
                mtm,
            );
            let _: () = msg_send![&btn, setBezelStyle: 1usize];
            let _: () = msg_send![&btn, setControlSize: 1usize];
            btn.setFont(Some(&NSFont::systemFontOfSize(11.0)));
            btn
        };
        let btn_autoscroll = unsafe {
            let btn = NSButton::buttonWithTitle_target_action(
                &NSString::from_str("Auto-scroll"),
                Some(&*this),
                Some(sel!(toggleAutoScroll:)),
                mtm,
            );
            let _: () = msg_send![&btn, setBezelStyle: 1usize];
            let _: () = msg_send![&btn, setControlSize: 1usize];
            btn.setFont(Some(&NSFont::systemFontOfSize(11.0)));
            btn
        };
        let sep = unsafe {
            let v = NSView::initWithFrame(
                NSView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(800.0, 1.0)),
            );
            v.setWantsLayer(true);
            if let Some(layer) = v.layer() {
                let cg = NSColor::separatorColor().CGColor();
                let ptr = Retained::as_ptr(&cg);
                let _: () = msg_send![&layer, setBackgroundColor: ptr];
            }
            v.setTranslatesAutoresizingMaskIntoConstraints(false);
            v
        };
        unsafe {
            btn_autoscroll.setState(1);
            btn_clear.setTranslatesAutoresizingMaskIntoConstraints(false);
            btn_pause.setTranslatesAutoresizingMaskIntoConstraints(false);
            btn_autoscroll.setTranslatesAutoresizingMaskIntoConstraints(false);

            toolbar.addSubview(&btn_clear);
            toolbar.addSubview(&btn_pause);
            toolbar.addSubview(&btn_autoscroll);
            toolbar.addSubview(&sep);

            objc2_app_kit::NSLayoutConstraint::activateConstraints(&NSArray::from_slice(&[
                &*btn_clear
                    .leadingAnchor()
                    .constraintEqualToAnchor_constant(&toolbar.leadingAnchor(), 12.0),
                &*btn_clear
                    .centerYAnchor()
                    .constraintEqualToAnchor(&toolbar.centerYAnchor()),
                &*btn_clear.widthAnchor().constraintEqualToConstant(56.0),
                &*btn_pause
                    .leadingAnchor()
                    .constraintEqualToAnchor_constant(&btn_clear.trailingAnchor(), 6.0),
                &*btn_pause
                    .centerYAnchor()
                    .constraintEqualToAnchor(&toolbar.centerYAnchor()),
                &*btn_pause.widthAnchor().constraintEqualToConstant(56.0),
                &*btn_autoscroll
                    .leadingAnchor()
                    .constraintEqualToAnchor_constant(&btn_pause.trailingAnchor(), 6.0),
                &*btn_autoscroll
                    .centerYAnchor()
                    .constraintEqualToAnchor(&toolbar.centerYAnchor()),
                &*btn_autoscroll.widthAnchor().constraintEqualToConstant(80.0),
                &*sep
                    .leadingAnchor()
                    .constraintEqualToAnchor(&toolbar.leadingAnchor()),
                &*sep
                    .trailingAnchor()
                    .constraintEqualToAnchor(&toolbar.trailingAnchor()),
                &*sep
                    .bottomAnchor()
                    .constraintEqualToAnchor(&toolbar.bottomAnchor()),
                &*sep.heightAnchor().constraintEqualToConstant(1.0),
            ]));
        };

        let scroll = unsafe {
            NSScrollView::initWithFrame(
                NSScrollView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(800.0, 560.0)),
            )
        };
        unsafe { scroll.setHasVerticalScroller(true) };
        unsafe { scroll.setAutohidesScrollers(true) };
        unsafe { scroll.setDrawsBackground(false) };

        unsafe {
            this.ivars().text_view.setEditable(false);
            this.ivars().text_view.setSelectable(true);
            this.ivars().text_view.setRichText(false);
            this.ivars().text_view.setDrawsBackground(false);
            this.ivars()
                .text_view
                .setTextContainerInset(NSSize::new(10.0, 8.0));
            this.ivars()
                .text_view
                .setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable);
            this.ivars().text_view.setMinSize(NSSize::new(0.0, 0.0));
            this.ivars()
                .text_view
                .setMaxSize(NSSize::new(100000.0, 100000.0));
            this.ivars().text_view.setVerticallyResizable(true);
            this.ivars().text_view.setHorizontallyResizable(false);
            this.ivars()
                .text_view
                .setFont(Some(&NSFont::monospacedSystemFontOfSize_weight(11.0, 0.0)));
            this.ivars()
                .text_view
                .setTextColor(Some(&*NSColor::labelColor()));
            this.ivars()
                .text_view
                .setBackgroundColor(&NSColor::clearColor());
        };

        unsafe { scroll.setDocumentView(Some(&*this.ivars().text_view)) };
        let effect =
            unsafe { NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), rect) };
        unsafe {
            effect.setMaterial(NSVisualEffectMaterial::HUDWindow);
            effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
            effect.setState(NSVisualEffectState::Active);

            toolbar.setTranslatesAutoresizingMaskIntoConstraints(false);
            scroll.setTranslatesAutoresizingMaskIntoConstraints(false);

            effect.addSubview(&toolbar);
            effect.addSubview(&scroll);

            toolbar
                .topAnchor()
                .constraintEqualToAnchor_constant(&effect.topAnchor(), 28.0)
                .setActive(true);
            toolbar
                .leadingAnchor()
                .constraintEqualToAnchor(&effect.leadingAnchor())
                .setActive(true);
            toolbar
                .trailingAnchor()
                .constraintEqualToAnchor(&effect.trailingAnchor())
                .setActive(true);
            toolbar
                .heightAnchor()
                .constraintEqualToConstant(30.0)
                .setActive(true);

            scroll
                .topAnchor()
                .constraintEqualToAnchor(&toolbar.bottomAnchor())
                .setActive(true);
            scroll
                .leadingAnchor()
                .constraintEqualToAnchor(&effect.leadingAnchor())
                .setActive(true);
            scroll
                .trailingAnchor()
                .constraintEqualToAnchor(&effect.trailingAnchor())
                .setActive(true);
            scroll
                .bottomAnchor()
                .constraintEqualToAnchor(&effect.bottomAnchor())
                .setActive(true);

            window.setContentView(Some(&effect));
        };

        let _ = this.ivars().window.set(window.clone());

        let mut buffer = crate::logger::LOG_BUFFER
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if buffer.is_empty() {
            buffer.push(crate::logger::LogMessage {
                timestamp: chrono::Local::now().format("%H:%M:%S.%3f").to_string(),
                thread_name: "MAIN".to_string(),
                category: "SYSTEM".to_string(),
                event: "Iridecence PDF Reader debug console initialized".to_string(),
            });
            buffer.push(crate::logger::LogMessage {
                timestamp: chrono::Local::now().format("%H:%M:%S.%3f").to_string(),
                thread_name: "MAIN".to_string(),
                category: "UI".to_string(),
                event: "Window geometry verified against visibleFrame".to_string(),
            });
            buffer.push(crate::logger::LogMessage {
                timestamp: chrono::Local::now().format("%H:%M:%S.%3f").to_string(),
                thread_name: "MAIN".to_string(),
                category: "RENDER".to_string(),
                event: "Liquid Glass HUDWindow material active".to_string(),
            });
        }
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
