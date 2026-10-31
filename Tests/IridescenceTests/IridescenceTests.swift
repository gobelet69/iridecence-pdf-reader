import AppKit
import Foundation
import PDFKit
import Vision
import XCTest
@testable import IridescenceCore

@MainActor
final class IridescenceTests: XCTestCase {

    // MARK: - 1. File Identity & Symlink Canonicalization Tests

    func testCanonicalPathStripsPrivatePrefixAndNormalizes() {
        XCTAssertEqual(UIConstants.canonicalPath("/private/tmp/sample.pdf"), "/tmp/sample.pdf")
        XCTAssertEqual(UIConstants.canonicalPath("/tmp/sample.pdf"), "/tmp/sample.pdf")
        XCTAssertEqual(UIConstants.canonicalPath("/private/var/folders/test.pdf"), "/var/folders/test.pdf")
        XCTAssertEqual(UIConstants.canonicalPath("   "), "")
    }

    // MARK: - 2. Bounded Logger Tests

    func testAppLoggerEnforcesMaxBufferSizeAndPauseState() {
        let logger = AppLogger()
        for i in 0..<(AppLogger.maxBufferSize + 150) {
            logger.log(.pdf, "Entry \(i)")
        }
        XCTAssertEqual(logger.count(), AppLogger.maxBufferSize)
        XCTAssertEqual(logger.entries().first?.message, "Entry 150")
        XCTAssertEqual(logger.entries().last?.message, "Entry \(AppLogger.maxBufferSize + 149)")

        logger.isPaused = true
        logger.log(.error, "Should not be recorded while paused")
        XCTAssertEqual(logger.entries().last?.message, "Entry \(AppLogger.maxBufferSize + 149)")

        logger.isPaused = false
        logger.clear()
        XCTAssertEqual(logger.count(), 0)
    }

    // MARK: - 3. Window Geometry Clamping & NaN Sanitization Tests

    func testWindowGeometrySanitizesNaNAndClampsToVisibleScreen() {
        let visible = NSRect(x: 0, y: 25, width: 1280, height: 775)

        let nanRect = NSRect(x: CGFloat.nan, y: CGFloat.infinity, width: -50, height: CGFloat.nan)
        let sanitizedNaN = WindowGeometry.sanitizeRect(nanRect, for: .home, in: visible)
        XCTAssertEqual(sanitizedNaN.width, WindowKind.home.defaultSize.width)
        XCTAssertEqual(sanitizedNaN.height, WindowKind.home.defaultSize.height)
        XCTAssertTrue(visible.contains(sanitizedNaN))

        let hugeRect = NSRect(x: 5000, y: -2000, width: 3000, height: 2000)
        let sanitizedHuge = WindowGeometry.sanitizeRect(hugeRect, for: .reader, in: visible)
        XCTAssertLessThanOrEqual(sanitizedHuge.width, visible.width - 24.0)
        XCTAssertLessThanOrEqual(sanitizedHuge.height, visible.height - 24.0)
        XCTAssertGreaterThanOrEqual(sanitizedHuge.origin.x, visible.origin.x)
        XCTAssertGreaterThanOrEqual(sanitizedHuge.origin.y, visible.origin.y)
    }

    // MARK: - 4. Settings Persistence & Restore Defaults Tests

    func testSettingsManagerPersistenceAndRestoreDefaults() {
        let suiteName = "com.iridecence.tests.settings.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suiteName)!
        defer { defaults.removePersistentDomain(forName: suiteName) }

        let mgr = SettingsManager(defaults: defaults)
        XCTAssertTrue(mgr.restoreSession)
        XCTAssertEqual(mgr.defaultDisplayMode, 1)
        XCTAssertEqual(mgr.defaultZoomMode, 0)
        XCTAssertFalse(mgr.showHiddenFiles)

        mgr.restoreSession = false
        mgr.defaultDisplayMode = 2
        mgr.defaultHighlightColorIndex = 3
        mgr.showHiddenFiles = true
        mgr.appearanceMode = 2

        XCTAssertFalse(mgr.restoreSession)
        XCTAssertEqual(mgr.defaultDisplayMode, 2)
        XCTAssertEqual(mgr.defaultHighlightColorIndex, 3)
        XCTAssertTrue(mgr.showHiddenFiles)
        XCTAssertEqual(mgr.appearanceMode, 2)

        mgr.restoreAllDefaults()
        XCTAssertTrue(mgr.restoreSession)
        XCTAssertEqual(mgr.defaultDisplayMode, 1)
        XCTAssertEqual(mgr.defaultHighlightColorIndex, 0)
        XCTAssertFalse(mgr.showHiddenFiles)
        XCTAssertEqual(mgr.appearanceMode, 0)
    }

    // MARK: - 5. Bookmarks Serialization, Symlink Matching, Migration & Markdown Export Tests

    func testBookmarksLifecycleAndMigrationAndMarkdownExport() {
        let suiteName = "com.iridecence.tests.bookmarks.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suiteName)!
        defer { defaults.removePersistentDomain(forName: suiteName) }

        let mgr = BookmarksManager(defaults: defaults)
        let added = mgr.toggleBookmark(pdfPath: "/private/tmp/docA.pdf", pageIndex: 4, defaultLabel: "Chapter 2")
        XCTAssertTrue(added)
        XCTAssertTrue(mgr.isBookmarked(pdfPath: "/tmp/docA.pdf", pageIndex: 4))

        let list = mgr.getBookmarks(forDocument: "/tmp/docA.pdf")
        XCTAssertEqual(list.count, 1)
        XCTAssertEqual(list.first?.label, "Chapter 2")
        XCTAssertEqual(list.first?.pageIndex, 4)

        mgr.migrateBookmarks(from: "/tmp/docA.pdf", to: "/tmp/docB.pdf")
        let migrated = mgr.getBookmarks(forDocument: "/private/tmp/docB.pdf")
        XCTAssertEqual(migrated.count, 1)
        XCTAssertEqual(migrated.first?.label, "Chapter 2")

        let annots = [
            AnnotationSummary(pageIndex: 1, subtype: "Highlight", textPreview: "Key cryptographic lemma", bounds: NSRect(x: 10, y: 10, width: 100, height: 20))
        ]
        let md = mgr.exportNotesAndAnnotationsMarkdown(pdfPath: "/tmp/docB.pdf", title: "Doc B", annotations: annots)
        XCTAssertTrue(md.contains("# Notes & Annotations: Doc B"))
        XCTAssertTrue(md.contains("Page 5** — Chapter 2"))
        XCTAssertTrue(md.contains("[Highlight] Page 2**: \"Key cryptographic lemma\""))

        let toggledOff = mgr.toggleBookmark(pdfPath: "/tmp/docA.pdf", pageIndex: 4, defaultLabel: "Chapter 2")
        XCTAssertFalse(toggledOff)
        XCTAssertFalse(mgr.isBookmarked(pdfPath: "/tmp/docA.pdf", pageIndex: 4))
    }

    // MARK: - 6. Workspace IRIDX1 Binary Index & Defensive DoS Bounds Tests

    func testWorkspaceIndexBinaryRoundtripAndSearch() {
        let docs = [
            IndexedDocument(
                path: "/tmp/ws/paper1.pdf",
                mtime: 1_700_000_000,
                fileSize: 42_000,
                pageTexts: ["Introduction to differential cryptanalysis", "Linear approximations in block ciphers"]
            ),
            IndexedDocument(
                path: "/tmp/ws/paper2.pdf",
                mtime: 1_700_000_100,
                fileSize: 84_000,
                pageTexts: ["Zero-knowledge proofs and protocol verification"]
            )
        ]

        let data = WorkspaceIndex.serialize(docs)
        let decoded = WorkspaceIndex.deserialize(data)
        XCTAssertNotNil(decoded)
        XCTAssertEqual(decoded?.count, 2)
        XCTAssertEqual(decoded?.first?.pageTexts.count, 2)

        let idx = WorkspaceIndex()
        for d in docs { idx.upsert(d) }
        let matches = idx.search("cryptanalysis")
        XCTAssertEqual(matches.count, 1)
        XCTAssertEqual(matches.first?.fileName, "paper1.pdf")
        XCTAssertEqual(matches.first?.pageIndex, 0)
    }

    func testWorkspaceIndexRejectsCorruptedAndOutOfBoundsPayloads() {
        let badHeader = Data("BADHDR0000".utf8)
        XCTAssertNil(WorkspaceIndex.deserialize(badHeader))

        var excessiveItems = Data("IRIDX1".utf8)
        var countLE = UInt32(60_000).littleEndian
        withUnsafeBytes(of: &countLE) { excessiveItems.append(contentsOf: $0) }
        XCTAssertNil(WorkspaceIndex.deserialize(excessiveItems))

        var excessivePath = Data("IRIDX1".utf8)
        var oneLE = UInt32(1).littleEndian
        var pathLenLE = UInt32(9_999).littleEndian
        withUnsafeBytes(of: &oneLE) { excessivePath.append(contentsOf: $0) }
        withUnsafeBytes(of: &pathLenLE) { excessivePath.append(contentsOf: $0) }
        XCTAssertNil(WorkspaceIndex.deserialize(excessivePath))
    }

    // MARK: - 7. TabManager, Reopen Closed Tab & AZERTY/QWERTY Keycode Tests

    func testTabManagerLifecycleAndKeyCodes() {
        let mgr = TabManager()
        let doc1 = PDFDocument()
        let doc2 = PDFDocument()

        let i0 = mgr.addTab(path: "/tmp/tab1.pdf", title: "tab1", document: doc1, mode: .standaloneDocument)
        let i1 = mgr.addTab(path: "/tmp/tab2.pdf", title: "tab2", document: doc2, mode: .workspace, workspaceRoot: "/tmp")
        XCTAssertEqual(i0, 0)
        XCTAssertEqual(i1, 1)
        XCTAssertEqual(mgr.tabCount, 2)
        XCTAssertEqual(mgr.activeIndex, 1)

        let dupIdx = mgr.addTab(path: "/private/tmp/tab1.pdf", title: "tab1", document: doc1, mode: .standaloneDocument)
        XCTAssertEqual(dupIdx, 0)
        XCTAssertEqual(mgr.tabCount, 2)
        XCTAssertEqual(mgr.activeIndex, 0)

        XCTAssertEqual(mgr.nextIndex(), 1)
        XCTAssertEqual(mgr.prevIndex(), 1)

        let removed = mgr.removeTab(at: 1, recordClosed: true)
        XCTAssertEqual(removed?.title, "tab2")
        XCTAssertEqual(mgr.tabCount, 1)

        let reopened = mgr.popLastClosedTab()
        XCTAssertEqual(reopened?.path, "/tmp/tab2.pdf")
        XCTAssertEqual(reopened?.mode, .workspace)

        let expectedKeyCodes: [(UInt16, Int)] = [
            (18, 1), (19, 2), (20, 3), (21, 4), (23, 5), (22, 6), (26, 7), (28, 8), (25, 9)
        ]
        for (code, num) in expectedKeyCodes {
            XCTAssertEqual(TabManager.tabNumberForKeyCode(code), num)
        }
        XCTAssertNil(TabManager.tabNumberForKeyCode(99))
    }

    // MARK: - 8. Annotations, Multi-Line QuadPoints & MASTER Read-Only Protection Tests

    func testMasterDirectoryProtectionAndSafeSaveAndAnnotations() throws {
        XCTAssertTrue(AnnotationsManager.isMasterProtectedPath("/Users/theodeville/Documents/MASTER/test.pdf"))
        XCTAssertTrue(AnnotationsManager.isMasterProtectedPath("/Users/theodeville/Documents/MASTER"))
        XCTAssertFalse(AnnotationsManager.isMasterProtectedPath("/tmp/iridescence-unit-test.pdf"))

        let dummyDoc = PDFDocument()
        XCTAssertThrowsError(try AnnotationsManager.saveDocumentSafely(dummyDoc, to: "/Users/theodeville/Documents/MASTER/forbidden.pdf")) { error in
            XCTAssertEqual(error as? AnnotationSaveError, .masterDirectoryProtected)
        }

        let tmpPath = "/tmp/iridescence-unit-tests-\(UUID().uuidString)/annot_test.pdf"
        defer { try? FileManager.default.removeItem(atPath: (tmpPath as NSString).deletingLastPathComponent) }

        VisualQAHarness.ensureRichTestPDF(at: tmpPath, title: "Annotation Unit Test", forceSynthetic: true)
        let pdfView = PDFView(frame: NSRect(x: 0, y: 0, width: 600, height: 800))
        guard let doc = PDFDocument(url: URL(fileURLWithPath: tmpPath)) else {
            XCTFail("Expected test PDF to load")
            return
        }
        pdfView.document = doc

        if let page0 = doc.page(at: 0),
           let sel = page0.selection(for: NSRect(x: 50, y: 420, width: 490, height: 220)) {
            pdfView.setCurrentSelection(sel, animate: false)
            let added = AnnotationsManager.addAnnotationToSelection(in: pdfView, type: .highlight)
            XCTAssertTrue(added)
        }

        let beforeSave = AnnotationsManager.getDocumentAnnotations(in: doc)
        XCTAssertGreaterThanOrEqual(beforeSave.count, 1)

        try AnnotationsManager.saveDocumentSafely(doc, to: tmpPath)

        guard let reloaded = PDFDocument(url: URL(fileURLWithPath: tmpPath)) else {
            XCTFail("Expected saved PDF to reload")
            return
        }
        let afterSave = AnnotationsManager.getDocumentAnnotations(in: reloaded)
        XCTAssertEqual(afterSave.count, beforeSave.count)
    }

    // MARK: - 9. Workspace Traversal, Hidden Files & Folder Switching Tests

    func testWorkspaceDirectoryScanningAndPDFCollection() throws {
        let baseDir = "/tmp/iridescence-ws-test-\(UUID().uuidString)"
        defer { try? FileManager.default.removeItem(atPath: baseDir) }

        try FileManager.default.createDirectory(atPath: "\(baseDir)/SubFolder", withIntermediateDirectories: true)
        try FileManager.default.createDirectory(atPath: "\(baseDir)/.hiddenDir", withIntermediateDirectories: true)
        VisualQAHarness.ensureRichTestPDF(at: "\(baseDir)/root_doc.pdf", forceSynthetic: true)
        VisualQAHarness.ensureRichTestPDF(at: "\(baseDir)/SubFolder/nested_doc.pdf", forceSynthetic: true)
        VisualQAHarness.ensureRichTestPDF(at: "\(baseDir)/.hidden_doc.pdf", forceSynthetic: true)
        try "ignore me".write(toFile: "\(baseDir)/notes.txt", atomically: true, encoding: .utf8)

        let visibleChildren = WorkspaceNode.scanImmediateChildren(at: baseDir, showHiddenFiles: false)
        XCTAssertEqual(visibleChildren.count, 2)
        XCTAssertTrue(visibleChildren[0].isDirectory)
        XCTAssertEqual(visibleChildren[0].name, "SubFolder")
        XCTAssertEqual(visibleChildren[1].name, "root_doc.pdf")

        let allWithHidden = WorkspaceNode.scanImmediateChildren(at: baseDir, showHiddenFiles: true)
        XCTAssertEqual(allWithHidden.count, 4)

        let collectedPDFs = WorkspaceNode.collectAllPDFPaths(in: baseDir, showHiddenFiles: false)
        XCTAssertEqual(collectedPDFs.count, 2)
    }

    // MARK: - 10. Command Palette Scoring, Prefix Routing & Cursor Placement Tests

    func testCommandPaletteScoringPrefixRoutingAndCursorPlacement() {
        let exact = CommandPaletteController.scoreQueryMatch(candidate: "Cryptanalysis", query: "cryptanalysis") ?? 0
        let prefix = CommandPaletteController.scoreQueryMatch(candidate: "Cryptanalysis_Overview", query: "crypt") ?? 0
        let substr = CommandPaletteController.scoreQueryMatch(candidate: "Advanced_Cryptanalysis", query: "crypt") ?? 0
        let fuzzy = CommandPaletteController.scoreQueryMatch(candidate: "Cryptanalysis", query: "cpns") ?? 0

        XCTAssertGreaterThan(exact, prefix)
        XCTAssertGreaterThan(prefix, substr)
        XCTAssertGreaterThan(substr, fuzzy)
        XCTAssertNil(CommandPaletteController.scoreQueryMatch(candidate: "Protocol", query: "xyz"))

        let palette = CommandPaletteController()
        palette.updateSearch(">zoom")
        XCTAssertFalse(palette.items.isEmpty)
        XCTAssertTrue(palette.items.contains { $0.title.contains("Zoom") })

        palette.updateSearch("p:15")
        XCTAssertEqual(palette.items.first?.action, .goToPage(15))

        palette.updateSearch("/cipher")
        XCTAssertEqual(palette.items.first?.action, .searchInPDF("cipher"))
    }

    // MARK: - 11. HoverCoordinator Single-Row Invariant Tests

    func testHoverCoordinatorEnforcesSingleHoveredRowAndClears() {
        let table = NSTableView(frame: NSRect(x: 0, y: 0, width: 200, height: 200))
        let hover = HoverCoordinator()
        XCTAssertEqual(hover.currentRow, -1)

        hover.update(row: 2, in: table)
        XCTAssertEqual(hover.currentRow, 2)

        hover.update(row: 4, in: table)
        XCTAssertEqual(hover.currentRow, 4)

        hover.clear(in: table)
        XCTAssertEqual(hover.currentRow, -1)
    }

    // MARK: - 12. Apple Vision OCR Tests

    func testVisionOCRExtractsTextAndSupportsCancellation() throws {
        let tmpPDF = "/tmp/iridescence-ocr-test-\(UUID().uuidString).pdf"
        defer { try? FileManager.default.removeItem(atPath: tmpPDF) }
        VisualQAHarness.ensureRichTestPDF(at: tmpPDF, title: "Vision OCR Verification", forceSynthetic: true)

        guard let doc = PDFDocument(url: URL(fileURLWithPath: tmpPDF)),
              let page0 = doc.page(at: 0) else {
            XCTFail("Failed to load OCR test PDF")
            return
        }

        let res = try OCRService.recognizeText(on: page0, pageIndex: 0, recognitionLevel: .fast)
        XCTAssertEqual(res.pageIndex, 0)
        XCTAssertGreaterThan(res.wordCount, 3)

        let cancelledToken = OCRCancellationToken()
        cancelledToken.cancel()
        XCTAssertThrowsError(try OCRService.recognizeText(on: page0, pageIndex: 0, token: cancelledToken))
    }

    // MARK: - 13. UI Controllers Integration Tests (Outline, Search, Home, Inspector, AppDelegate)

    func testPDFOutlineAndSearchControllersWithDocument() throws {
        let tmpPDF = "/tmp/iridescence-controllers-test-\(UUID().uuidString).pdf"
        defer { try? FileManager.default.removeItem(atPath: tmpPDF) }
        VisualQAHarness.ensureRichTestPDF(at: tmpPDF, title: "Controller Integration", forceSynthetic: true)

        let pdfView = PDFView(frame: NSRect(x: 0, y: 0, width: 600, height: 800))
        let doc = PDFDocument(url: URL(fileURLWithPath: tmpPDF))!
        if doc.outlineRoot == nil {
            VisualQAHarness.attachSyntheticOutline(to: doc)
        }
        pdfView.document = doc

        let outlineCtrl = PDFOutlineController(pdfView: pdfView)
        outlineCtrl.loadOutline(from: doc)
        XCTAssertFalse(outlineCtrl.isEmpty)
        XCTAssertEqual(outlineCtrl.rootItems.count, 3)
        XCTAssertEqual(outlineCtrl.flattenedEntries().count, 3)
        outlineCtrl.syncActivePage(1)

        let searchCtrl = PDFSearchController(pdfView: pdfView)
        searchCtrl.performSearch("cryptographic")
        XCTAssertFalse(searchCtrl.results.isEmpty)
        searchCtrl.cancelAndReset()
        XCTAssertTrue(searchCtrl.results.isEmpty)
    }

    func testAppDelegateWindowSplitViewAndTabTransitions() throws {
        let tmpDir = "/tmp/iridescence-appdelegate-test-\(UUID().uuidString)"
        defer { try? FileManager.default.removeItem(atPath: tmpDir) }
        let pdf1 = "\(tmpDir)/first.pdf"
        let pdf2 = "\(tmpDir)/second.pdf"
        VisualQAHarness.ensureRichTestPDF(at: pdf1, title: "First Document", forceSynthetic: true)
        VisualQAHarness.ensureRichTestPDF(at: pdf2, title: "Second Document", forceSynthetic: true)

        let app = AppDelegate()
        app.buildMainWindowAndViews()
        XCTAssertEqual(app.contentMode, .home)

        app.openFile(at: URL(fileURLWithPath: pdf1))
        XCTAssertEqual(app.contentMode, .standaloneDocument)
        XCTAssertEqual(app.tabManager.tabCount, 1)

        app.openFile(at: URL(fileURLWithPath: pdf2))
        XCTAssertEqual(app.tabManager.tabCount, 2)
        XCTAssertEqual(app.tabManager.activeIndex, 1)

        app.switchToTab(at: 0)
        XCTAssertEqual(app.tabManager.activeIndex, 0)

        for seg in 0...4 {
            app.switchSidebarToIndex(seg)
            XCTAssertEqual(app.rightSegmentedControl.selectedSegment, seg)
        }

        app.closeTab(at: 0)
        XCTAssertEqual(app.tabManager.tabCount, 1)
        app.closeTab(at: 0)
        XCTAssertEqual(app.tabManager.tabCount, 0)
        XCTAssertEqual(app.contentMode, .home)
    }

    // MARK: - 14. iCloud Synchronization & 7-State Download Coordinator Tests

    func testICloudStatusMappingAndPlaceholderConversion() {
        let logicalURL = URL(fileURLWithPath: "/tmp/sample_paper.pdf")
        let placeholderURL = ICloudCoordinator.placeholderURL(for: logicalURL)
        XCTAssertEqual(placeholderURL.lastPathComponent, ".sample_paper.pdf.icloud")
        XCTAssertEqual(ICloudCoordinator.logicalURL(for: placeholderURL).path, logicalURL.path)
        XCTAssertEqual(ICloudCoordinator.canonicalLogicalKey(for: placeholderURL), ICloudCoordinator.canonicalLogicalKey(for: logicalURL))

        XCTAssertTrue(ICloudStatus.cloudOnly.isBadgeVisible)
        XCTAssertTrue(ICloudStatus.downloadRequested.isInFlight)
        XCTAssertTrue(ICloudStatus.downloading.isInFlight)
        XCTAssertFalse(ICloudStatus.downloadedVerified.isInFlight)
        XCTAssertFalse(ICloudStatus.local.isInFlight)
    }

    final class MockICloudBackend: ICloudProviderBackend, @unchecked Sendable {
        var statusToReturn: ICloudStatus = .cloudOnly
        var downloadInvokedURLs: [URL] = []
        var shouldThrowOnDownload = false

        func inspectStatus(for url: URL) -> (status: ICloudStatus, progress: Double?, errorMessage: String?) {
            (statusToReturn, statusToReturn == .downloading ? 0.45 : nil, nil)
        }

        func startUbiquitousDownload(for url: URL) throws {
            if shouldThrowOnDownload {
                throw NSError(domain: "iCloudMock", code: -1, userInfo: [NSLocalizedDescriptionKey: "Network timeout"])
            }
            downloadInvokedURLs.append(url)
        }
    }

    func testICloudDownloadManagerDeduplicationAndAutoOpenFlow() {
        let tmpDir = "/tmp/iridescence-icloud-test-\(UUID().uuidString)"
        defer { try? FileManager.default.removeItem(atPath: tmpDir) }
        try? FileManager.default.createDirectory(atPath: tmpDir, withIntermediateDirectories: true)
        let placeholderURL = URL(fileURLWithPath: "\(tmpDir)/.paper.pdf.icloud")
        let logicalURL = URL(fileURLWithPath: "\(tmpDir)/paper.pdf")
        FileManager.default.createFile(atPath: placeholderURL.path, contents: Data([0x00, 0x01]))

        let mock = MockICloudBackend()
        let mgr = ICloudDownloadManager.shared
        let originalBackend = mgr.backend
        mgr.backend = mock
        defer { mgr.backend = originalBackend }

        var stateChanges: [ICloudStatus] = []
        mgr.onStateChanged = { session in
            stateChanges.append(session.status)
        }

        var openedURL: URL?
        mgr.onAutoOpenReady = { url in
            openedURL = url
        }

        // 1. Initial request -> downloadRequested -> downloading
        let s1 = mgr.requestDownload(for: placeholderURL, autoOpenWhenReady: true)
        XCTAssertTrue(s1.status.isInFlight)
        XCTAssertEqual(mock.downloadInvokedURLs.count, 1)

        // 2. Concurrent request deduplication
        let s2 = mgr.requestDownload(for: logicalURL, autoOpenWhenReady: true)
        XCTAssertEqual(s2.canonicalKey, s1.canonicalKey)
        XCTAssertEqual(mock.downloadInvokedURLs.count, 1, "Concurrent request must be deduplicated")

        // 3. Simulate materialization of valid PDF
        VisualQAHarness.ensureRichTestPDF(at: logicalURL.path, title: "Materialized PDF", forceSynthetic: true)
        mgr.notifyFileSystemChanged(for: logicalURL)

        XCTAssertNotNil(openedURL)
        XCTAssertEqual(openedURL?.path, logicalURL.path)
        XCTAssertEqual(mgr.effectiveStatus(for: logicalURL), .downloadedVerified)
    }

    func testICloudStaleTargetProtection() {
        let tmpDir = "/tmp/iridescence-icloud-stale-\(UUID().uuidString)"
        defer { try? FileManager.default.removeItem(atPath: tmpDir) }
        try? FileManager.default.createDirectory(atPath: tmpDir, withIntermediateDirectories: true)
        let placeholderURL = URL(fileURLWithPath: "\(tmpDir)/.stale.pdf.icloud")
        let logicalURL = URL(fileURLWithPath: "\(tmpDir)/stale.pdf")
        FileManager.default.createFile(atPath: placeholderURL.path, contents: Data([0x00]))

        let mock = MockICloudBackend()
        let mgr = ICloudDownloadManager.shared
        let originalBackend = mgr.backend
        mgr.backend = mock
        defer { mgr.backend = originalBackend }

        _ = mgr.requestDownload(for: placeholderURL, autoOpenWhenReady: true)

        // User navigates away to a different document
        mgr.userDidNavigateToDocument(at: URL(fileURLWithPath: "/tmp/other_document.pdf"))
        XCTAssertFalse(mgr.isTargetStillActive(for: placeholderURL))

        var autoOpened = false
        mgr.onAutoOpenReady = { _ in
            autoOpened = true
        }

        VisualQAHarness.ensureRichTestPDF(at: logicalURL.path, title: "Stale File", forceSynthetic: true)
        mgr.notifyFileSystemChanged(for: logicalURL)

        XCTAssertFalse(autoOpened, "Stale target should NOT auto-open when user navigated away")
    }

    // MARK: - 15. PDFPageThumbnailController Tests

    func testPDFPageThumbnailControllerActivePageHighlightAndCache() {
        let tmpPDF = "/tmp/iridescence-thumb-test-\(UUID().uuidString).pdf"
        defer { try? FileManager.default.removeItem(atPath: tmpPDF) }
        VisualQAHarness.ensureRichTestPDF(at: tmpPDF, title: "Thumbnails Verification", forceSynthetic: true)

        let pdfView = PDFView(frame: NSRect(x: 0, y: 0, width: 600, height: 800))
        let doc = PDFDocument(url: URL(fileURLWithPath: tmpPDF))!
        pdfView.document = doc

        let controller = PDFPageThumbnailController(pdfView: pdfView)
        controller.reload(for: doc, documentPath: tmpPDF)

        XCTAssertEqual(controller.numberOfRows(in: controller.tableView), doc.pageCount)

        controller.updateActivePageIndex(1)
        controller.updateActivePageIndex(2)

        let cell = controller.tableView(controller.tableView, viewFor: nil, row: 2)
        XCTAssertNotNil(cell)
    }

    // MARK: - 16. Iridescence Design System Tokens & Surface Washes

    func testIridescentDesignTokensAndSurfaces() {
        XCTAssertNotNil(IridescenceDesign.primaryViolet)
        XCTAssertNotNil(IridescenceDesign.secondaryIndigo)
        XCTAssertNotNil(IridescenceDesign.accentCyan)
        XCTAssertNotNil(IridescenceDesign.accentRose)
        XCTAssertNotNil(IridescenceDesign.accentAmber)

        let washSidebar = IridescentSurfaceWashView(role: .sidebar)
        XCTAssertEqual(washSidebar.role, .sidebar)

        let washHero = IridescentSurfaceWashView(role: .welcomeHero)
        XCTAssertEqual(washHero.role, .welcomeHero)

        let emblem = IridescentEmblemView(size: 64.0)
        XCTAssertNotNil(emblem)

        let pill = IridescentPillBadge(text: "RECENT", style: .violet)
        XCTAssertEqual(pill.stringValue, "RECENT")
        XCTAssertEqual(pill.style, .violet)
    }
}
