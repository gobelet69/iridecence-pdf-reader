use objc2_foundation::{NSMutableArray, NSString, NSUserDefaults};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct Bookmark {
    pub path: String,
    pub page_idx: usize,
    pub label: String,
    pub created_at: u64,
}

const BOOKMARKS_KEY: &str = "UserBookmarks_v2";

pub fn load_bookmarks() -> Vec<Bookmark> {
    let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
    let key = NSString::from_str(BOOKMARKS_KEY);
    let arr = match unsafe { defaults.arrayForKey(&key) } {
        Some(a) => a,
        None => return Vec::new(),
    };

    let count = arr.count();
    let mut bookmarks = Vec::with_capacity(count);

    for i in 0..count {
        let obj = arr.objectAtIndex(i);
        if let Some(ns_str) = obj.downcast_ref::<NSString>() {
            let s = ns_str.to_string();
            let parts: Vec<&str> = s.split('\t').collect();
            if parts.len() >= 4 {
                let path = parts[0].to_string();
                let page_idx = parts[1].parse::<usize>().unwrap_or(0);
                let label = parts[2].to_string();
                let created_at = parts[3].parse::<u64>().unwrap_or(0);
                if !path.is_empty() {
                    bookmarks.push(Bookmark {
                        path,
                        page_idx,
                        label,
                        created_at,
                    });
                }
            }
        }
    }

    bookmarks
}

pub fn save_bookmarks(bookmarks: &[Bookmark]) {
    let defaults = unsafe { NSUserDefaults::standardUserDefaults() };
    let key = NSString::from_str(BOOKMARKS_KEY);

    let arr = NSMutableArray::<NSString>::new();
    for bm in bookmarks {
        let line = format!(
            "{}\t{}\t{}\t{}",
            bm.path, bm.page_idx, bm.label, bm.created_at
        );
        arr.addObject(&NSString::from_str(&line));
    }

    unsafe {
        defaults.setObject_forKey(Some(&arr), &key);
    }
}

pub fn get_bookmarks_for_path(path: &str) -> Vec<Bookmark> {
    load_bookmarks()
        .into_iter()
        .filter(|b| b.path == path)
        .collect()
}

pub fn is_bookmarked(path: &str, page_idx: usize) -> bool {
    load_bookmarks()
        .into_iter()
        .any(|b| b.path == path && b.page_idx == page_idx)
}

pub fn toggle_bookmark(path: &str, page_idx: usize, default_label: &str) -> bool {
    let mut bookmarks = load_bookmarks();
    if let Some(pos) = bookmarks
        .iter()
        .position(|b| b.path == path && b.page_idx == page_idx)
    {
        bookmarks.remove(pos);
        save_bookmarks(&bookmarks);
        false // Removed
    } else {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        bookmarks.push(Bookmark {
            path: path.to_string(),
            page_idx,
            label: default_label.to_string(),
            created_at: now,
        });
        save_bookmarks(&bookmarks);
        true // Added
    }
}

pub fn remove_bookmark(path: &str, page_idx: usize) {
    let mut bookmarks = load_bookmarks();
    if let Some(pos) = bookmarks
        .iter()
        .position(|b| b.path == path && b.page_idx == page_idx)
    {
        bookmarks.remove(pos);
        save_bookmarks(&bookmarks);
    }
}

pub fn export_notes_markdown(path: &str, doc_title: &str) -> String {
    let bookmarks = get_bookmarks_for_path(path);
    let mut md = format!("# {}\n\n## Bookmarks\n\n", doc_title);
    if bookmarks.is_empty() {
        md.push_str("No bookmarks recorded for this document.\n");
    } else {
        for b in &bookmarks {
            md.push_str(&format!("- **p. {}** — {}\n", b.page_idx + 1, b.label));
        }
    }
    md
}
