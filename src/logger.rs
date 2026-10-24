use objc2_foundation::NSString;
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct LogMessage {
    pub timestamp: String,
    pub thread_name: String,
    pub category: String,
    pub event: String,
}

pub const MAX_LOG_BUFFER_SIZE: usize = 2000;

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

    let line = format!(
        "[{}] [{}] [{}] {}",
        msg.timestamp, msg.thread_name, msg.category, msg.event
    );
    println!("{}", line);

    let mut buffer = match LOG_BUFFER.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if buffer.len() >= MAX_LOG_BUFFER_SIZE {
        let excess = buffer.len() + 1 - MAX_LOG_BUFFER_SIZE;
        buffer.drain(0..excess);
    }
    buffer.push(msg);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_event_records_message() {
        log_event("TEST_CAT", "Hello world from test");
        let buffer = match LOG_BUFFER.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        assert!(!buffer.is_empty());
        assert!(buffer.iter().any(|e| e.category == "TEST_CAT" && e.event == "Hello world from test"));
    }

    #[test]
    fn test_log_buffer_bounded_capacity() {
        // Log more than MAX_LOG_BUFFER_SIZE to ensure capping works
        for i in 0..100 {
            log_event("BENCH", &format!("Message {}", i));
        }
        let buffer = match LOG_BUFFER.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        assert!(buffer.len() <= MAX_LOG_BUFFER_SIZE);
    }
}
