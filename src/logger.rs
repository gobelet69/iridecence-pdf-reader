use std::sync::Mutex;
use once_cell::sync::Lazy;
use objc2_foundation::NSString;

#[derive(Clone, Debug)]
pub struct LogMessage {
    pub timestamp: String,
    pub thread_name: String,
    pub category: String,
    pub event: String,
}

pub static LOG_BUFFER: Lazy<Mutex<Vec<LogMessage>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn log_event(category: &str, event: &str) {
    let is_main = objc2::MainThreadMarker::new().is_some();
    let thread_str = if is_main {
        "MAIN".to_string()
    } else {
        let name = std::thread::current().name().unwrap_or("BG").to_string();
        name
    };
    
    use chrono::Local;
    let ts = Local::now().format("%H:%M:%S.%3f").to_string();
    
    let msg = LogMessage {
        timestamp: ts.clone(),
        thread_name: thread_str.clone(),
        category: category.to_string(),
        event: event.to_string(),
    };
    
    let line = format!("[{}] [{}] [{}] {}", msg.timestamp, msg.thread_name, msg.category, msg.event);
    println!("{}", line);
    
    if let Ok(mut buffer) = LOG_BUFFER.lock() {
        buffer.push(msg);
    }
    
    // Try to update console if open
    unsafe {
        if let Ok(guard) = crate::debug_console::DEBUG_WINDOW_CONTROLLER.lock() {
            if let Some(ptr) = *guard {
                let vc = ptr as *mut objc2::runtime::AnyObject;
                let line_str = NSString::from_str(&format!("{}\n", line));
                let _: () = objc2::msg_send![vc, performSelectorOnMainThread: objc2::sel!(appendLogLine:), withObject: &*line_str, waitUntilDone: false];
            }
        }
    }
}

#[macro_export]
macro_rules! log_ui {
    ($($arg:tt)*) => { $crate::logger::log_event("UI", &format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_workspace {
    ($($arg:tt)*) => { $crate::logger::log_event("WORKSPACE", &format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_icloud {
    ($($arg:tt)*) => { $crate::logger::log_event("ICLOUD", &format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_pdf {
    ($($arg:tt)*) => { $crate::logger::log_event("PDF", &format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::logger::log_event("ERROR", &format!($($arg)*)) };
}
