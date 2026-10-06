import re

with open('src/debug_console.rs', 'r') as f:
    content = f.read()

# Replace the method `init` with `new`
content = content.replace("#[unsafe(method(init))]\n        fn init(&self) -> Option<Retained<Self>> {", "fn new(mtm: objc2::MainThreadMarker) -> Retained<Self> {")
content = content.replace("let this: Retained<Self> = unsafe { objc2::msg_send_id![super(self), init] };", "let this = Self::alloc(mtm).set_ivars(DebugConsoleIvars {\n    window: std::cell::OnceCell::new(),\n    text_view: NSTextView::initWithFrame(NSTextView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(800.0, 560.0))),\n    is_paused: std::cell::Cell::new(false),\n    auto_scroll: std::cell::Cell::new(true),\n});\nlet this: Retained<Self> = unsafe { objc2::msg_send_id![super(this), init] };")
content = content.replace("Some(this)", "this")
# Fix the pub window issue inside set_ivars
content = content.replace("pub window: std::cell::OnceCell", "window: std::cell::OnceCell")
# Fix `vc.ivars()` issue: change the guard to store Retained<DebugConsoleController> inside OnceCell instead of usize
# Actually wait, global state with Retained is tricky because it's not Send.
