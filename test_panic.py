with open('src/main.rs', 'r') as f:
    content = f.read()

content = content.replace('crate::log_error!("[DIAGNOSTIC] DOWNLOAD THREAD PANICKED");', '''if let Some(s) = err.downcast_ref::<String>() {
            crate::log_error!("[DIAGNOSTIC] DOWNLOAD THREAD PANICKED: {}", s);
        } else if let Some(s) = err.downcast_ref::<&str>() {
            crate::log_error!("[DIAGNOSTIC] DOWNLOAD THREAD PANICKED: {}", s);
        } else {
            crate::log_error!("[DIAGNOSTIC] DOWNLOAD THREAD PANICKED: {:?}", err);
        }''')
content = content.replace('if result.is_err() {', 'if let Err(err) = result {')

with open('src/main.rs', 'w') as f:
    f.write(content)
