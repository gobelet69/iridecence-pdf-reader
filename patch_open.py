import re

with open('src/main.rs', 'r') as f:
    content = f.read()

old_open = """                                        // 2. Filesystem access latency check
                                        let _meta = std::fs::metadata(&url_str);
                                        let t1 = std::time::Instant::now();
                                        
                                        // 3. Init PDFDocument
                                        let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };
                                        
                                        // Check iCloud downloading status here if needed"""

new_open = """                                        if icloud_node_ptr != 0 {
                                            crate::download_file(url_str.clone(), item_raw, outline_raw, icloud_node_ptr);
                                        }
                                        
                                        // 2. Filesystem access latency check
                                        let _meta = std::fs::metadata(&url_str);
                                        let t1 = std::time::Instant::now();
                                        
                                        // 3. Init PDFDocument
                                        let nsurl = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&url_str)) };"""

# We only want to replace the first occurrence which is in workspaceDoubleClicked
content = content.replace(old_open, new_open, 1)

with open('src/main.rs', 'w') as f:
    f.write(content)
print("Patched openPdf")
