import Foundation
import PDFKit

public struct BookmarkEntry: Sendable, Equatable {
    public var pdfPath: String
    public var pageIndex: Int
    public var label: String
    public var createdAt: UInt64

    public init(pdfPath: String, pageIndex: Int, label: String, createdAt: UInt64 = UInt64(Date().timeIntervalSince1970)) {
        self.pdfPath = UIConstants.canonicalPath(pdfPath)
        self.pageIndex = max(0, pageIndex)
        self.label = label
        self.createdAt = createdAt
    }

    public func serializedString() -> String {
        let cleanLabel = label.replacingOccurrences(of: "\t", with: " ").replacingOccurrences(of: "\n", with: " ")
        return "\(pdfPath)\t\(pageIndex)\t\(cleanLabel)\t\(createdAt)"
    }

    public static func deserialize(_ raw: String) -> BookmarkEntry? {
        let parts = raw.split(separator: "\t", maxSplits: 3, omittingEmptySubsequences: false).map(String.init)
        guard parts.count >= 4,
              let pageIdx = Int(parts[1]),
              let createdAt = UInt64(parts[3]) else {
            return nil
        }
        return BookmarkEntry(
            pdfPath: parts[0],
            pageIndex: pageIdx,
            label: parts[2],
            createdAt: createdAt
        )
    }
}

/// Manages `UserBookmarks_v2` in `NSUserDefaults`, symlink path normalization,
/// bookmark migration across Save As operations, and Markdown export.
public final class BookmarksManager: @unchecked Sendable {
    public static let shared = BookmarksManager()

    private let defaults: UserDefaults

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    public func loadAllBookmarks() -> [BookmarkEntry] {
        guard let rawArray = defaults.stringArray(forKey: SettingsKeys.userBookmarksV2) else {
            return []
        }
        return rawArray.compactMap(BookmarkEntry.deserialize)
    }

    public func saveAllBookmarks(_ bookmarks: [BookmarkEntry]) {
        let rawArray = bookmarks.map { $0.serializedString() }
        defaults.set(rawArray, forKey: SettingsKeys.userBookmarksV2)
    }

    public func getBookmarks(forDocument pdfPath: String) -> [BookmarkEntry] {
        let normTarget = UIConstants.canonicalPath(pdfPath)
        return loadAllBookmarks()
            .filter { UIConstants.canonicalPath($0.pdfPath) == normTarget }
            .sorted { $0.pageIndex < $1.pageIndex }
    }

    public func isBookmarked(pdfPath: String, pageIndex: Int) -> Bool {
        let normTarget = UIConstants.canonicalPath(pdfPath)
        return loadAllBookmarks().contains {
            UIConstants.canonicalPath($0.pdfPath) == normTarget && $0.pageIndex == pageIndex
        }
    }

    /// Toggles a bookmark for the given document and page. Returns `true` if added, `false` if removed.
    @discardableResult
    public func toggleBookmark(pdfPath: String, pageIndex: Int, defaultLabel: String) -> Bool {
        let normTarget = UIConstants.canonicalPath(pdfPath)
        var all = loadAllBookmarks()
        if let existingIdx = all.firstIndex(where: {
            UIConstants.canonicalPath($0.pdfPath) == normTarget && $0.pageIndex == pageIndex
        }) {
            all.remove(at: existingIdx)
            saveAllBookmarks(all)
            return false
        } else {
            let entry = BookmarkEntry(
                pdfPath: normTarget,
                pageIndex: pageIndex,
                label: defaultLabel
            )
            all.append(entry)
            saveAllBookmarks(all)
            return true
        }
    }

    public func removeBookmark(pdfPath: String, pageIndex: Int) {
        let normTarget = UIConstants.canonicalPath(pdfPath)
        var all = loadAllBookmarks()
        all.removeAll {
            UIConstants.canonicalPath($0.pdfPath) == normTarget && $0.pageIndex == pageIndex
        }
        saveAllBookmarks(all)
    }

    /// Copies all bookmarks associated with `oldPath` to `newPath` (used on Save As).
    public func migrateBookmarks(from oldPath: String, to newPath: String) {
        let normOld = UIConstants.canonicalPath(oldPath)
        let normNew = UIConstants.canonicalPath(newPath)
        guard !normOld.isEmpty, !normNew.isEmpty, normOld != normNew else { return }

        var all = loadAllBookmarks()
        let toCopy = all.filter { UIConstants.canonicalPath($0.pdfPath) == normOld }
        guard !toCopy.isEmpty else { return }

        for bm in toCopy {
            let exists = all.contains {
                UIConstants.canonicalPath($0.pdfPath) == normNew && $0.pageIndex == bm.pageIndex
            }
            if !exists {
                all.append(BookmarkEntry(
                    pdfPath: normNew,
                    pageIndex: bm.pageIndex,
                    label: bm.label,
                    createdAt: bm.createdAt
                ))
            }
        }
        saveAllBookmarks(all)
    }

    /// Generates a formatted Markdown summary of bookmarks and PDF annotations for a document.
    public func exportNotesAndAnnotationsMarkdown(
        pdfPath: String,
        title: String,
        annotations: [AnnotationSummary]
    ) -> String {
        let bookmarks = getBookmarks(forDocument: pdfPath)
        var md = "# Notes & Annotations: \(title)\n\n"
        md += "- **Source File**: `\(pdfPath)`\n"
        md += "- **Bookmarks**: \(bookmarks.count)\n"
        md += "- **Annotations**: \(annotations.count)\n\n"

        md += "## Bookmarks\n\n"
        if bookmarks.isEmpty {
            md += "_No bookmarks in this document._\n\n"
        } else {
            for bm in bookmarks {
                md += "- **Page \(bm.pageIndex + 1)** — \(bm.label)\n"
            }
            md += "\n"
        }

        md += "## Annotations\n\n"
        if annotations.isEmpty {
            md += "_No annotations in this document._\n"
        } else {
            for ann in annotations {
                let text = ann.textPreview.trimmingCharacters(in: .whitespacesAndNewlines)
                if text.isEmpty {
                    md += "- **[\(ann.subtype)] Page \(ann.pageIndex + 1)**\n"
                } else {
                    md += "- **[\(ann.subtype)] Page \(ann.pageIndex + 1)**: \"\(text)\"\n"
                }
            }
        }
        return md
    }
}
