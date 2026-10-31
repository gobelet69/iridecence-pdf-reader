import Foundation
import PDFKit

public struct IndexedDocument: Sendable, Equatable {
    public var path: String
    public var mtime: UInt64
    public var fileSize: UInt64
    public var pageTexts: [String]

    public init(path: String, mtime: UInt64, fileSize: UInt64, pageTexts: [String]) {
        self.path = path
        self.mtime = mtime
        self.fileSize = fileSize
        self.pageTexts = pageTexts
    }
}

public struct WorkspaceSearchMatch: Sendable, Equatable {
    public let pdfPath: String
    public let fileName: String
    public let pageIndex: Int
    public let snippet: String

    public init(pdfPath: String, fileName: String, pageIndex: Int, snippet: String) {
        self.pdfPath = pdfPath
        self.fileName = fileName
        self.pageIndex = pageIndex
        self.snippet = snippet
    }
}

/// Binary `IRIDX1` workspace full-text search index with strict defensive DoS bounds (Rule 3.2).
public final class WorkspaceIndex: @unchecked Sendable {
    public static let shared = WorkspaceIndex()

    public static let magicHeader: [UInt8] = Array("IRIDX1".utf8)
    public static let maxItemCount: Int = 50_000
    public static let maxPathLength: Int = 4_096
    public static let maxPageCount: Int = 25_000
    public static let maxPageTextLength: Int = 5_000_000

    private let lock = NSLock()
    private var entries: [String: IndexedDocument] = [:]

    public init() {}

    public static func defaultCacheURL() -> URL {
        let caches = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first
            ?? URL(fileURLWithPath: NSTemporaryDirectory())
        let appDir = caches.appendingPathComponent("com.iridecence.pdfreader", isDirectory: true)
        try? FileManager.default.createDirectory(at: appDir, withIntermediateDirectories: true)
        return appDir.appendingPathComponent("workspace_index.bin")
    }

    public func upsert(_ doc: IndexedDocument) {
        lock.lock()
        if entries.count < Self.maxItemCount || entries[doc.path] != nil {
            entries[doc.path] = doc
        }
        lock.unlock()
    }

    public func document(forPath path: String) -> IndexedDocument? {
        lock.lock()
        defer { lock.unlock() }
        return entries[path]
    }

    public func allDocuments() -> [IndexedDocument] {
        lock.lock()
        defer { lock.unlock() }
        return Array(entries.values)
    }

    public func clear() {
        lock.lock()
        entries.removeAll()
        lock.unlock()
    }

    // MARK: - Binary IRIDX1 Serialization

    public static func serialize(_ docs: [IndexedDocument]) -> Data {
        var data = Data()
        data.append(contentsOf: magicHeader)

        let clampedDocs = docs.prefix(maxItemCount)
        appendUInt32(&data, UInt32(clampedDocs.count))

        for doc in clampedDocs {
            let pathBytes = Array(doc.path.utf8).prefix(maxPathLength)
            appendUInt32(&data, UInt32(pathBytes.count))
            data.append(contentsOf: pathBytes)
            appendUInt64(&data, doc.mtime)
            appendUInt64(&data, doc.fileSize)

            let clampedPages = doc.pageTexts.prefix(maxPageCount)
            appendUInt32(&data, UInt32(clampedPages.count))

            for pageText in clampedPages {
                let textBytes = Array(pageText.utf8).prefix(maxPageTextLength)
                appendUInt32(&data, UInt32(textBytes.count))
                data.append(contentsOf: textBytes)
            }
        }
        return data
    }

    public static func deserialize(_ data: Data) -> [IndexedDocument]? {
        guard data.count >= 10 else { return nil }
        let header = Array(data.prefix(6))
        guard header == magicHeader else { return nil }

        var offset = 6
        guard let countU32 = readUInt32(data, offset: &offset) else { return nil }
        let count = Int(countU32)
        guard count <= maxItemCount else { return nil }

        var result: [IndexedDocument] = []
        result.reserveCapacity(min(count, 1024))

        for _ in 0..<count {
            guard let pathLenU32 = readUInt32(data, offset: &offset) else { return nil }
            let pathLen = Int(pathLenU32)
            guard pathLen <= maxPathLength, offset + pathLen <= data.count else { return nil }
            let pathSlice = data.subdata(in: offset..<(offset + pathLen))
            offset += pathLen
            guard let path = String(data: pathSlice, encoding: .utf8) else { return nil }

            guard let mtime = readUInt64(data, offset: &offset),
                  let fileSize = readUInt64(data, offset: &offset),
                  let pageCountU32 = readUInt32(data, offset: &offset) else {
                return nil
            }
            let pageCount = Int(pageCountU32)
            guard pageCount <= maxPageCount else { return nil }

            var pages: [String] = []
            pages.reserveCapacity(min(pageCount, 256))

            for _ in 0..<pageCount {
                guard let textLenU32 = readUInt32(data, offset: &offset) else { return nil }
                let textLen = Int(textLenU32)
                guard textLen <= maxPageTextLength, offset + textLen <= data.count else { return nil }
                let textSlice = data.subdata(in: offset..<(offset + textLen))
                offset += textLen
                guard let pageStr = String(data: textSlice, encoding: .utf8) else { return nil }
                pages.append(pageStr)
            }

            result.append(IndexedDocument(path: path, mtime: mtime, fileSize: fileSize, pageTexts: pages))
        }

        return result
    }

    public func saveToDisk(url: URL = WorkspaceIndex.defaultCacheURL()) {
        let docs = allDocuments()
        let data = Self.serialize(docs)
        try? data.write(to: url, options: .atomic)
    }

    public func loadFromDisk(url: URL = WorkspaceIndex.defaultCacheURL()) {
        guard let data = try? Data(contentsOf: url),
              let docs = Self.deserialize(data) else {
            return
        }
        lock.lock()
        for d in docs {
            entries[d.path] = d
        }
        lock.unlock()
    }

    // MARK: - Incremental Indexing & Search

    public func indexWorkspaceDirectory(_ rootPath: String) {
        let pdfPaths = WorkspaceNode.collectAllPDFPaths(in: rootPath)
        var changed = false

        for path in pdfPaths {
            guard let attrs = try? FileManager.default.attributesOfItem(atPath: path) else { continue }
            let mtime = UInt64((attrs[.modificationDate] as? Date)?.timeIntervalSince1970 ?? 0)
            let size = (attrs[.size] as? UInt64) ?? 0

            if let existing = document(forPath: path), existing.mtime == mtime && existing.fileSize == size {
                continue
            }

            autoreleasepool {
                let url = URL(fileURLWithPath: path)
                guard let pdfDoc = PDFDocument(url: url), !pdfDoc.isLocked else { return }
                let pageCount = min(pdfDoc.pageCount, Self.maxPageCount)
                var pageTexts: [String] = []
                pageTexts.reserveCapacity(pageCount)

                for i in 0..<pageCount {
                    let rawText = pdfDoc.page(at: i)?.string ?? ""
                    if rawText.utf8.count > Self.maxPageTextLength {
                        let prefix = String(rawText.prefix(Self.maxPageTextLength / 2))
                        pageTexts.append(prefix)
                    } else {
                        pageTexts.append(rawText)
                    }
                }

                upsert(IndexedDocument(path: path, mtime: mtime, fileSize: size, pageTexts: pageTexts))
                changed = true
            }
        }

        if changed {
            saveToDisk()
        }
    }

    public func search(_ query: String, maxResults: Int = 50) -> [WorkspaceSearchMatch] {
        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return [] }

        let docs = allDocuments().sorted { $0.path < $1.path }
        var matches: [WorkspaceSearchMatch] = []

        for doc in docs {
            let fileName = URL(fileURLWithPath: doc.path).lastPathComponent
            for (pageIdx, pageText) in doc.pageTexts.enumerated() {
                if let range = pageText.range(of: trimmed, options: [.caseInsensitive, .diacriticInsensitive]) {
                    let snippet = Self.buildSnippet(in: pageText, matchRange: range)
                    matches.append(WorkspaceSearchMatch(
                        pdfPath: doc.path,
                        fileName: fileName,
                        pageIndex: pageIdx,
                        snippet: snippet
                    ))
                    if matches.count >= maxResults {
                        return matches
                    }
                }
            }
        }
        return matches
    }

    public static func buildSnippet(in text: String, matchRange: Range<String.Index>, contextChars: Int = 36) -> String {
        let startIdx = text.index(matchRange.lowerBound, offsetBy: -contextChars, limitedBy: text.startIndex) ?? text.startIndex
        let endIdx = text.index(matchRange.upperBound, offsetBy: contextChars, limitedBy: text.endIndex) ?? text.endIndex

        var snippet = String(text[startIdx..<endIdx])
            .replacingOccurrences(of: "\n", with: " ")
            .replacingOccurrences(of: "\r", with: " ")
            .replacingOccurrences(of: "\t", with: " ")

        while snippet.contains("  ") {
            snippet = snippet.replacingOccurrences(of: "  ", with: " ")
        }
        snippet = snippet.trimmingCharacters(in: .whitespaces)
        if startIdx > text.startIndex {
            snippet = "…" + snippet
        }
        if endIdx < text.endIndex {
            snippet = snippet + "…"
        }
        return snippet
    }

    // MARK: - Little-Endian Helpers

    private static func appendUInt32(_ data: inout Data, _ value: UInt32) {
        var le = value.littleEndian
        withUnsafeBytes(of: &le) { data.append(contentsOf: $0) }
    }

    private static func appendUInt64(_ data: inout Data, _ value: UInt64) {
        var le = value.littleEndian
        withUnsafeBytes(of: &le) { data.append(contentsOf: $0) }
    }

    private static func readUInt32(_ data: Data, offset: inout Int) -> UInt32? {
        guard offset + 4 <= data.count else { return nil }
        var value: UInt32 = 0
        _ = withUnsafeMutableBytes(of: &value) { dest in
            data.copyBytes(to: dest, from: offset..<(offset + 4))
        }
        offset += 4
        return UInt32(littleEndian: value)
    }

    private static func readUInt64(_ data: Data, offset: inout Int) -> UInt64? {
        guard offset + 8 <= data.count else { return nil }
        var value: UInt64 = 0
        _ = withUnsafeMutableBytes(of: &value) { dest in
            data.copyBytes(to: dest, from: offset..<(offset + 8))
        }
        offset += 8
        return UInt64(littleEndian: value)
    }
}
