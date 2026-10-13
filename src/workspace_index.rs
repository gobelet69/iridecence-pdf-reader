use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use objc2::AnyThread;
use objc2_foundation::{NSString, NSURL};
use objc2_pdf_kit::PDFDocument;

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub path: String,
    pub filename: String,
    pub page_num: usize, // 1-indexed
    pub snippet: String,
}

#[derive(Clone, Debug)]
pub struct IndexedPdf {
    pub path: String,
    pub mtime: u64,
    pub file_size: u64,
    pub pages: Vec<(usize, String)>, // (page_num 1-indexed, text)
}

pub struct WorkspaceIndex {
    pub entries: HashMap<String, IndexedPdf>,
    pub current_workspace: Option<String>,
}

pub static INDEX: Lazy<Mutex<WorkspaceIndex>> = Lazy::new(|| {
    let mut idx = WorkspaceIndex {
        entries: HashMap::new(),
        current_workspace: None,
    };
    idx.load_from_disk();
    Mutex::new(idx)
});

pub static INDEXING_STATUS: Lazy<Mutex<Option<(usize, usize)>>> = Lazy::new(|| Mutex::new(None));
pub static CANCEL_INDEXING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

fn cache_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home)
        .join("Library")
        .join("Caches")
        .join("com.iridecence.pdfreader");
    let _ = fs::create_dir_all(&dir);
    dir.join("workspace_index.bin")
}

impl WorkspaceIndex {
    pub fn load_from_disk(&mut self) {
        let path = cache_path();
        if !path.exists() {
            return;
        }
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => return,
        };
        self.load_from_reader(BufReader::new(file));
    }

    pub fn load_from_reader<R: Read>(&mut self, mut reader: R) {
        let mut magic = [0u8; 6];
        if reader.read_exact(&mut magic).is_err() || &magic != b"IRIDX1" {
            return;
        }

        let mut count_buf = [0u8; 4];
        if reader.read_exact(&mut count_buf).is_err() {
            return;
        }
        let count = u32::from_le_bytes(count_buf) as usize;
        if count > 50_000 {
            return;
        }

        for _ in 0..count {
            // Read path
            let mut len_buf = [0u8; 4];
            if reader.read_exact(&mut len_buf).is_err() {
                break;
            }
            let path_len = u32::from_le_bytes(len_buf) as usize;
            if path_len == 0 || path_len > 4096 {
                break;
            }
            let mut path_bytes = vec![0u8; path_len];
            if reader.read_exact(&mut path_bytes).is_err() {
                break;
            }
            let pdf_path = match String::from_utf8(path_bytes) {
                Ok(s) => s,
                Err(_) => break,
            };

            // Read mtime & size
            let mut mtime_buf = [0u8; 8];
            if reader.read_exact(&mut mtime_buf).is_err() {
                break;
            }
            let mtime = u64::from_le_bytes(mtime_buf);

            let mut size_buf = [0u8; 8];
            if reader.read_exact(&mut size_buf).is_err() {
                break;
            }
            let file_size = u64::from_le_bytes(size_buf);

            // Read pages
            let mut page_count_buf = [0u8; 4];
            if reader.read_exact(&mut page_count_buf).is_err() {
                break;
            }
            let page_count = u32::from_le_bytes(page_count_buf) as usize;
            if page_count > 25_000 {
                break;
            }

            let mut pages = Vec::with_capacity(page_count.min(512));
            let mut page_read_failed = false;
            for _ in 0..page_count {
                let mut pnum_buf = [0u8; 4];
                if reader.read_exact(&mut pnum_buf).is_err() {
                    page_read_failed = true;
                    break;
                }
                let pnum = u32::from_le_bytes(pnum_buf) as usize;

                let mut text_len_buf = [0u8; 4];
                if reader.read_exact(&mut text_len_buf).is_err() {
                    page_read_failed = true;
                    break;
                }
                let text_len = u32::from_le_bytes(text_len_buf) as usize;
                if text_len > 5_000_000 {
                    page_read_failed = true;
                    break;
                }
                let mut text_bytes = vec![0u8; text_len];
                if reader.read_exact(&mut text_bytes).is_err() {
                    page_read_failed = true;
                    break;
                }
                if let Ok(text) = String::from_utf8(text_bytes) {
                    pages.push((pnum, text));
                }
            }
            if page_read_failed {
                break;
            }

            self.entries.insert(
                pdf_path.clone(),
                IndexedPdf {
                    path: pdf_path,
                    mtime,
                    file_size,
                    pages,
                },
            );
        }
    }

    pub fn save_to_disk(&self) {
        let path = cache_path();
        let file = match File::create(&path) {
            Ok(f) => f,
            Err(_) => return,
        };
        let _ = self.save_to_writer(BufWriter::new(file));
    }

    pub fn save_to_writer<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(b"IRIDX1")?;
        let count = self.entries.len() as u32;
        writer.write_all(&count.to_le_bytes())?;

        for item in self.entries.values() {
            let path_bytes = item.path.as_bytes();
            writer.write_all(&(path_bytes.len() as u32).to_le_bytes())?;
            writer.write_all(path_bytes)?;

            writer.write_all(&item.mtime.to_le_bytes())?;
            writer.write_all(&item.file_size.to_le_bytes())?;

            let page_count = item.pages.len() as u32;
            writer.write_all(&page_count.to_le_bytes())?;

            for (pnum, text) in &item.pages {
                writer.write_all(&(*pnum as u32).to_le_bytes())?;
                let text_bytes = text.as_bytes();
                writer.write_all(&(text_bytes.len() as u32).to_le_bytes())?;
                writer.write_all(text_bytes)?;
            }
        }
        writer.flush()?;
        Ok(())
    }
}

pub fn search_workspace(query: &str) -> Vec<SearchResult> {
    let clean_query = query.trim().to_lowercase();
    if clean_query.is_empty() {
        return Vec::new();
    }

    let index = INDEX.lock().unwrap();
    let mut results = Vec::new();

    for pdf in index.entries.values() {
        let filename = Path::new(&pdf.path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| pdf.path.clone());

        for (page_num, text) in &pdf.pages {
            let lower_text = text.to_lowercase();
            if let Some(pos) = lower_text.find(&clean_query) {
                // Generate clean snippet
                let start = if pos > 35 { pos - 35 } else { 0 };
                let end = (pos + clean_query.len() + 45).min(text.len());

                let mut snippet = text[start..end].replace('\n', " ").replace('\r', " ");
                snippet = snippet.split_whitespace().collect::<Vec<_>>().join(" ");

                let mut formatted_snippet = String::new();
                if start > 0 {
                    formatted_snippet.push_str("…");
                }
                formatted_snippet.push_str(&snippet);
                if end < text.len() {
                    formatted_snippet.push_str("…");
                }

                results.push(SearchResult {
                    path: pdf.path.clone(),
                    filename: filename.clone(),
                    page_num: *page_num,
                    snippet: formatted_snippet,
                });

                if results.len() >= 50 {
                    return results;
                }
            }
        }
    }

    results
}

pub fn start_indexing_workspace(workspace_dir: String, priority_path: Option<String>) {
    CANCEL_INDEXING.store(true, Ordering::SeqCst);

    // Spawn background thread
    std::thread::spawn(move || {
        // Short pause to allow previous indexing to exit if any
        std::thread::sleep(std::time::Duration::from_millis(50));
        CANCEL_INDEXING.store(false, Ordering::SeqCst);

        {
            let mut idx = INDEX.lock().unwrap();
            idx.current_workspace = Some(workspace_dir.clone());
        }

        // Collect all PDF files in workspace safely
        let mut pdf_files = Vec::new();
        let mut visited = std::collections::HashSet::new();
        collect_pdfs_safe(Path::new(&workspace_dir), &mut pdf_files, &mut visited, 0);

        // Sort so priority_path comes first
        if let Some(ref prio) = priority_path {
            pdf_files.sort_by(|a, b| {
                if a == prio {
                    std::cmp::Ordering::Less
                } else if b == prio {
                    std::cmp::Ordering::Greater
                } else {
                    a.cmp(b)
                }
            });
        }

        let total = pdf_files.len();
        if total == 0 {
            *INDEXING_STATUS.lock().unwrap_or_else(|p| p.into_inner()) = None;
            return;
        }

        let mut _indexed_count = 0;
        let mut changed_anything = false;

        for (i, file_path) in pdf_files.iter().enumerate() {
            if CANCEL_INDEXING.load(Ordering::SeqCst) {
                break;
            }

            *INDEXING_STATUS.lock().unwrap_or_else(|p| p.into_inner()) = Some((i + 1, total));

            let meta = match fs::metadata(file_path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let file_size = meta.len();

            // Check if already up-to-date in index
            let needs_index = {
                let idx = INDEX.lock().unwrap_or_else(|p| p.into_inner());
                match idx.entries.get(file_path) {
                    Some(cached) => cached.mtime != mtime || cached.file_size != file_size,
                    None => true,
                }
            };

            if needs_index {
                if let Some(extracted) = extract_pdf_text(file_path, mtime, file_size) {
                    let mut idx = INDEX.lock().unwrap_or_else(|p| p.into_inner());
                    idx.entries.insert(file_path.clone(), extracted);
                    changed_anything = true;
                    _indexed_count += 1;
                }
            }

            // Yield briefly to ensure zero UI hiccups
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        *INDEXING_STATUS.lock().unwrap_or_else(|p| p.into_inner()) = None;

        if changed_anything {
            let idx = INDEX.lock().unwrap_or_else(|p| p.into_inner());
            idx.save_to_disk();
        }
    });
}

fn collect_pdfs_safe(
    dir: &Path,
    out: &mut Vec<String>,
    visited: &mut std::collections::HashSet<PathBuf>,
    depth: usize,
) {
    if CANCEL_INDEXING.load(Ordering::SeqCst) || depth > 25 || out.len() >= 10_000 {
        return;
    }
    if let Ok(canonical) = dir.canonicalize() {
        if !visited.insert(canonical) {
            return; // Cycle detected
        }
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        if CANCEL_INDEXING.load(Ordering::SeqCst) || out.len() >= 10_000 {
            return;
        }
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        if name.starts_with('.') {
            continue; // Ignore hidden files/dirs
        }
        if path.is_dir() {
            collect_pdfs_safe(&path, out, visited, depth + 1);
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("pdf") {
                    out.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
}

fn extract_pdf_text(path: &str, mtime: u64, file_size: u64) -> Option<IndexedPdf> {
    let url_ns = unsafe { NSURL::fileURLWithPath(&NSString::from_str(path)) };
    let doc = unsafe { PDFDocument::initWithURL(PDFDocument::alloc(), &url_ns) }?;

    let is_locked: bool = unsafe { objc2::msg_send![&doc, isLocked] };
    if is_locked {
        return None;
    }

    let page_count = unsafe { doc.pageCount() };
    if page_count == 0 {
        return None;
    }
    let max_pages = page_count.min(5000);
    let mut pages = Vec::with_capacity(max_pages.min(512));

    for p in 0..max_pages {
        if CANCEL_INDEXING.load(Ordering::SeqCst) {
            return None;
        }
        if let Some(page) = unsafe { doc.pageAtIndex(p) } {
            if let Some(ns_str) = unsafe { page.string() } {
                let text = ns_str.to_string();
                if !text.trim().is_empty() {
                    let truncated = if text.len() > 100_000 {
                        text[..100_000].to_string()
                    } else {
                        text
                    };
                    pages.push((p + 1, truncated)); // 1-indexed
                }
            }
        }
    }

    Some(IndexedPdf {
        path: path.to_string(),
        mtime,
        file_size,
        pages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_roundtrip_index_serialization() {
        let mut original = WorkspaceIndex {
            entries: HashMap::new(),
            current_workspace: Some("/test/workspace".to_string()),
        };

        original.entries.insert(
            "/test/doc1.pdf".to_string(),
            IndexedPdf {
                path: "/test/doc1.pdf".to_string(),
                mtime: 1728000000,
                file_size: 1048576,
                pages: vec![
                    (1, "First page text content".to_string()),
                    (2, "Second page with unicode: 🚀 Iridescence".to_string()),
                ],
            },
        );

        let mut buf = Vec::new();
        original.save_to_writer(&mut buf).expect("Save to writer should succeed");
        assert!(!buf.is_empty());
        assert_eq!(&buf[0..6], b"IRIDX1");

        let mut restored = WorkspaceIndex {
            entries: HashMap::new(),
            current_workspace: None,
        };
        restored.load_from_reader(&buf[..]);

        assert_eq!(restored.entries.len(), 1);
        let entry = restored.entries.get("/test/doc1.pdf").expect("Entry must exist");
        assert_eq!(entry.path, "/test/doc1.pdf");
        assert_eq!(entry.mtime, 1728000000);
        assert_eq!(entry.file_size, 1048576);
        assert_eq!(entry.pages.len(), 2);
        assert_eq!(entry.pages[0], (1, "First page text content".to_string()));
        assert_eq!(entry.pages[1], (2, "Second page with unicode: 🚀 Iridescence".to_string()));
    }

    #[test]
    fn test_corrupt_magic_rejection() {
        let corrupt_data = b"NOTIDX\x01\x00\x00\x00";
        let mut idx = WorkspaceIndex {
            entries: HashMap::new(),
            current_workspace: None,
        };
        idx.load_from_reader(&corrupt_data[..]);
        assert_eq!(idx.entries.len(), 0, "Corrupt magic should be rejected without panicking");
    }

    #[test]
    fn test_truncated_stream_handling() {
        // Valid magic + count = 1, but truncated immediately
        let truncated = b"IRIDX1\x01\x00\x00\x00";
        let mut idx = WorkspaceIndex {
            entries: HashMap::new(),
            current_workspace: None,
        };
        idx.load_from_reader(&truncated[..]);
        assert_eq!(idx.entries.len(), 0, "Truncated stream should abort safely without panic");
    }

    #[test]
    fn test_oversized_count_dos_protection() {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"IRIDX1");
        payload.extend_from_slice(&(100_000u32).to_le_bytes()); // Exceeds 50_000 cap
        let mut idx = WorkspaceIndex {
            entries: HashMap::new(),
            current_workspace: None,
        };
        idx.load_from_reader(&payload[..]);
        assert_eq!(idx.entries.len(), 0, "Oversized count must be rejected to prevent OOM DOS");
    }

    #[test]
    fn test_oversized_path_len_dos_protection() {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"IRIDX1");
        payload.extend_from_slice(&(1u32).to_le_bytes());
        payload.extend_from_slice(&(50_000u32).to_le_bytes()); // Exceeds 4096 path_len cap
        let mut idx = WorkspaceIndex {
            entries: HashMap::new(),
            current_workspace: None,
        };
        idx.load_from_reader(&payload[..]);
        assert_eq!(idx.entries.len(), 0, "Oversized path_len must be rejected");
    }

    #[test]
    fn test_oversized_page_count_dos_protection() {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"IRIDX1");
        payload.extend_from_slice(&(1u32).to_le_bytes());
        // path_len = 4, path = "test"
        payload.extend_from_slice(&(4u32).to_le_bytes());
        payload.extend_from_slice(b"test");
        // mtime = 0, size = 0
        payload.extend_from_slice(&(0u64).to_le_bytes());
        payload.extend_from_slice(&(0u64).to_le_bytes());
        // page_count = 50_000 (exceeds 25_000 cap)
        payload.extend_from_slice(&(50_000u32).to_le_bytes());

        let mut idx = WorkspaceIndex {
            entries: HashMap::new(),
            current_workspace: None,
        };
        idx.load_from_reader(&payload[..]);
        assert_eq!(idx.entries.len(), 0, "Oversized page_count must be rejected");
    }

    #[test]
    fn test_collect_pdfs_safe_cycle_protection() {
        let test_dir = std::env::temp_dir().join(format!("oxireader_test_cycle_{}", std::process::id()));
        let _ = fs::create_dir_all(&test_dir);
        let sub_dir = test_dir.join("sub");
        let _ = fs::create_dir_all(&sub_dir);

        // Create recursive symlink: sub/loop -> test_dir
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink(&test_dir, sub_dir.join("loop"));

        let mut out = Vec::new();
        let mut visited = HashSet::new();
        collect_pdfs_safe(&test_dir, &mut out, &mut visited, 0);

        // Must complete without stack overflow / hanging
        let _ = fs::remove_dir_all(&test_dir);
    }
}

