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

pub fn normalize_bookmark_path(path: &str) -> String {
    if let Ok(canon) = std::fs::canonicalize(path) {
        if let Some(s) = canon.to_str() {
            return s.strip_prefix("/private").unwrap_or(s).to_string();
        }
    }
    path.strip_prefix("/private").unwrap_or(path).to_string()
}

pub fn get_bookmarks_for_path(path: &str) -> Vec<Bookmark> {
    let target = normalize_bookmark_path(path);
    load_bookmarks()
        .into_iter()
        .filter(|b| normalize_bookmark_path(&b.path) == target)
        .collect()
}

pub fn is_bookmarked(path: &str, page_idx: usize) -> bool {
    let target = normalize_bookmark_path(path);
    load_bookmarks()
        .into_iter()
        .any(|b| normalize_bookmark_path(&b.path) == target && b.page_idx == page_idx)
}

pub fn toggle_bookmark(path: &str, page_idx: usize, default_label: &str) -> bool {
    let target = normalize_bookmark_path(path);
    let mut bookmarks = load_bookmarks();
    if let Some(pos) = bookmarks
        .iter()
        .position(|b| normalize_bookmark_path(&b.path) == target && b.page_idx == page_idx)
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
            path: target,
            page_idx,
            label: default_label.to_string(),
            created_at: now,
        });
        save_bookmarks(&bookmarks);
        true // Added
    }
}

pub fn remove_bookmark(path: &str, page_idx: usize) {
    let target = normalize_bookmark_path(path);
    let mut bookmarks = load_bookmarks();
    if let Some(pos) = bookmarks
        .iter()
        .position(|b| normalize_bookmark_path(&b.path) == target && b.page_idx == page_idx)
    {
        bookmarks.remove(pos);
        save_bookmarks(&bookmarks);
    }
}

pub fn migrate_bookmarks(old_path: &str, new_path: &str) {
    let old_target = normalize_bookmark_path(old_path);
    let new_target = normalize_bookmark_path(new_path);
    if old_target.is_empty() || new_target.is_empty() || old_target == new_target {
        return;
    }
    let mut bookmarks = load_bookmarks();
    let mut changed = false;
    for bm in &mut bookmarks {
        if normalize_bookmark_path(&bm.path) == old_target {
            bm.path = new_target.clone();
            changed = true;
        }
    }
    if changed {
        save_bookmarks(&bookmarks);
    }
}

pub fn export_notes_markdown(path: &str, doc_title: &str) -> String {
    export_notes_and_annotations_markdown(path, doc_title, None)
}

pub fn export_notes_and_annotations_markdown(
    path: &str,
    doc_title: &str,
    doc: Option<&objc2_pdf_kit::PDFDocument>,
) -> String {
    let bookmarks = get_bookmarks_for_path(path);
    let mut md = format!("# {}\n\n## Bookmarks\n\n", doc_title);
    if bookmarks.is_empty() {
        md.push_str("No bookmarks recorded for this document.\n");
    } else {
        for b in &bookmarks {
            md.push_str(&format!("- **p. {}** — {}\n", b.page_idx + 1, b.label));
        }
    }

    if let Some(pdf_doc) = doc {
        let annotations = crate::annotations::get_document_annotations(pdf_doc);
        md.push_str("\n## Annotations & Notes\n\n");
        if annotations.is_empty() {
            md.push_str("No annotations recorded for this document.\n");
        } else {
            for a in &annotations {
                md.push_str(&format!(
                    "- **p. {}** (*{}*) — {}\n",
                    a.page_index + 1,
                    a.annot_type.display_name(),
                    a.contents
                ));
            }
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_bookmark_path_strips_private_prefix() {
        assert_eq!(
            normalize_bookmark_path("/private/tmp/nonexistent_test.pdf"),
            "/tmp/nonexistent_test.pdf"
        );
        assert_eq!(
            normalize_bookmark_path("/tmp/nonexistent_test.pdf"),
            "/tmp/nonexistent_test.pdf"
        );
    }

    #[test]
    fn test_export_notes_and_annotations_markdown_includes_sections() {
        let doc = unsafe { objc2_pdf_kit::PDFDocument::new() };
        let md = export_notes_and_annotations_markdown(
            "/tmp/nonexistent_doc_for_export.pdf",
            "Sample Protocol Paper",
            Some(&doc),
        );
        assert!(md.contains("# Sample Protocol Paper"));
        assert!(md.contains("## Bookmarks"));
        assert!(md.contains("## Annotations & Notes"));
    }

    #[test]
    fn test_toggle_bookmark_and_migrate() {
        let p1 = "/tmp/iridx_test_doc_a.pdf";
        let p2 = "/tmp/iridx_test_doc_b.pdf";
        // Clean up first
        remove_bookmark(p1, 2);
        remove_bookmark(p2, 2);

        // Toggle on
        let added = toggle_bookmark(p1, 2, "Test Bookmark Page 3");
        assert!(added);
        assert!(is_bookmarked(p1, 2));

        // Migrate from p1 to p2
        migrate_bookmarks(p1, p2);
        assert!(!is_bookmarked(p1, 2));
        assert!(is_bookmarked(p2, 2));

        // Toggle off on p2
        let added_again = toggle_bookmark(p2, 2, "Test Bookmark Page 3");
        assert!(!added_again);
        assert!(!is_bookmarked(p2, 2));
    }
}
