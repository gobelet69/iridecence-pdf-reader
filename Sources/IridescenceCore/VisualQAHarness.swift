import AppKit
import Foundation
import PDFKit

/// Automated Visual QA screenshot harness for Iridescence PDF Reader (Native Swift).
/// Supports capturing all 22 required visual states in `/tmp/iridescence-swift-visual-qa/`
/// as well as the 19 canonical screenshots used by `scripts/visual_qa.sh`.
@MainActor
public enum VisualQAHarness {
    public static func pumpRunLoop(_ seconds: TimeInterval) {
        let until = Date(timeIntervalSinceNow: seconds)
        RunLoop.current.run(until: until)
    }

    public static func setDarkMode(_ dark: Bool) {
        let name: NSAppearance.Name = dark ? .darkAqua : .aqua
        NSApp.appearance = NSAppearance(named: name)
        pumpRunLoop(0.12)
    }

    @discardableResult
    public static func captureWindowToPNG(_ window: NSWindow, outPath: String) -> Bool {
        let dir = (outPath as NSString).deletingLastPathComponent
        try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)

        let winNum = window.windowNumber
        if winNum > 0 {
            let proc = Process()
            proc.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
            proc.arguments = ["-l", "\(winNum)", "-o", outPath]
            try? proc.run()
            proc.waitUntilExit()
            if proc.terminationStatus == 0,
               let attrs = try? FileManager.default.attributesOfItem(atPath: outPath),
               let size = attrs[.size] as? UInt64,
               size > 1_000 {
                return true
            }
        }

        guard let contentView = window.contentView else { return false }
        let targetView = contentView.superview ?? contentView
        let bounds = targetView.bounds
        guard bounds.width > 0, bounds.height > 0 else { return false }

        if let rep = targetView.bitmapImageRepForCachingDisplay(in: bounds) {
            targetView.cacheDisplay(in: bounds, to: rep)
            if let pngData = rep.representation(using: .png, properties: [:]) {
                do {
                    try pngData.write(to: URL(fileURLWithPath: outPath), options: .atomic)
                    return true
                } catch {}
            }
        }
        return false
    }

    /// Ensures a realistic multi-page PDF with selectable text and an outline hierarchy exists at `targetPath`.
    public static func ensureRichTestPDF(
        at targetPath: String,
        title: String = "Cryptanalysis & Protocol Verification",
        forceSynthetic: Bool = false
    ) {
        let dir = (targetPath as NSString).deletingLastPathComponent
        try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)

        if !forceSynthetic {
            let home = FileManager.default.homeDirectoryForCurrentUser.path
            let sampleDownload = "\(home)/Downloads/synthese_cryptanalyse_protocoles.pdf"
            if FileManager.default.fileExists(atPath: sampleDownload) {
                try? FileManager.default.removeItem(atPath: targetPath)
                try? FileManager.default.copyItem(atPath: sampleDownload, toPath: targetPath)
                if let doc = PDFDocument(url: URL(fileURLWithPath: targetPath)), doc.outlineRoot == nil {
                    attachSyntheticOutline(to: doc)
                    doc.write(toFile: targetPath)
                }
                return
            }
        }

        // Synthesize a clean 3-page PDF with rich text and outline if sampleDownload is absent
        let pdfData = NSMutableData()
        var mediaBox = CGRect(x: 0, y: 0, width: 612, height: 792)
        guard let consumer = CGDataConsumer(data: pdfData as CFMutableData),
              let ctx = CGContext(consumer: consumer, mediaBox: &mediaBox, nil) else {
            return
        }

        let chapters = [
            ("1. Introduction to Formal Protocol Analysis", "Formal verification of cryptographic protocols ensures security properties hold against active Dolev-Yao adversaries. Key exchange primitives and symmetric ciphers require rigorous state-space exploration."),
            ("2. Differential & Linear Cryptanalysis", "Differential cryptanalysis analyzes how differences in plaintext pairs propagate through substitution-permutation networks. Sample protocol bounds and key schedule resistance are evaluated across 16 rounds."),
            ("3. Post-Quantum Lattice Constructions", "Module-LWE key encapsulation mechanisms provide strong security reductions. Hybrid TLS 1.3 handshakes combine classical elliptic curves with lattice-based encapsulation.")
        ]

        for (idx, chap) in chapters.enumerated() {
            ctx.beginPDFPage(nil)
            NSGraphicsContext.saveGraphicsState()
            NSGraphicsContext.current = NSGraphicsContext(cgContext: ctx, flipped: false)

            let headerAttrs: [NSAttributedString.Key: Any] = [
                .font: NSFont.systemFont(ofSize: 20, weight: .bold),
                .foregroundColor: NSColor.black
            ]
            let bodyAttrs: [NSAttributedString.Key: Any] = [
                .font: NSFont.systemFont(ofSize: 13, weight: .regular),
                .foregroundColor: NSColor.darkGray
            ]

            NSString(string: "\(title) — Page \(idx + 1)").draw(at: NSPoint(x: 60, y: 710), withAttributes: headerAttrs)
            NSString(string: chap.0).draw(at: NSPoint(x: 60, y: 650), withAttributes: headerAttrs)
            NSString(string: chap.1).draw(in: NSRect(x: 60, y: 420, width: 490, height: 210), withAttributes: bodyAttrs)

            NSGraphicsContext.restoreGraphicsState()
            ctx.endPDFPage()
        }
        ctx.closePDF()
        pdfData.write(toFile: targetPath, atomically: true)

        if let doc = PDFDocument(url: URL(fileURLWithPath: targetPath)) {
            attachSyntheticOutline(to: doc)
            doc.write(toFile: targetPath)
        }
    }

    public static func attachSyntheticOutline(to doc: PDFDocument) {
        guard doc.pageCount > 0 else { return }
        let root = PDFOutline()
        let labels = [
            "1. Introduction & Threat Model",
            "2. Cryptographic Primitives & Bounds",
            "3. Formal Verification Results"
        ]
        for (i, label) in labels.enumerated() {
            let child = PDFOutline()
            child.label = label
            let pageIdx = min(i, doc.pageCount - 1)
            if let page = doc.page(at: pageIdx) {
                let dest = PDFDestination(page: page, at: NSPoint(x: 0, y: 700))
                child.destination = dest
                child.action = PDFActionGoTo(destination: dest)
            }
            root.insertChild(child, at: i)
        }
        doc.outlineRoot = root
    }

    // MARK: - Complete 22-State Swift Visual QA Suite

    public static func runComplete22StateSwiftVisualQA(app: AppDelegate) {
        let outDir = ProcessInfo.processInfo.environment["IRIDESCENCE_SWIFT_QA_DIR"]
            ?? "/tmp/iridescence-swift-visual-qa"
        try? FileManager.default.createDirectory(atPath: outDir, withIntermediateDirectories: true)

        let fixtureDir = "/tmp/iridescence-swift-qa-fixtures"
        try? FileManager.default.removeItem(atPath: fixtureDir)
        let wsDir = "\(fixtureDir)/Workspace"
        try? FileManager.default.createDirectory(atPath: "\(wsDir)/Cryptography", withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(atPath: "\(wsDir)/Protocols", withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(atPath: "\(wsDir)/Reference Papers", withIntermediateDirectories: true)

        let pdf1 = "\(wsDir)/Cryptography/Cryptanalysis_Overview.pdf"
        let pdf2 = "\(wsDir)/Protocols/TLS_1_3_Formal_Analysis.pdf"
        let pdf3 = "\(wsDir)/Reference Papers/Symmetric_Ciphers_Handbook.pdf"

        ensureRichTestPDF(at: pdf1, title: "Cryptanalysis Overview")
        ensureRichTestPDF(at: pdf2, title: "TLS 1.3 Formal Analysis")
        ensureRichTestPDF(at: pdf3, title: "Symmetric Ciphers Handbook")

        guard let window = app.window else { return }

        // 1. Home Light (Empty / Clean)
        UserDefaults.standard.removeObject(forKey: SettingsKeys.recentFiles)
        UserDefaults.standard.removeObject(forKey: SettingsKeys.recentWorkspaces)
        setDarkMode(false)
        app.showHomeScreen(nil)
        pumpRunLoop(0.18)
        captureWindowToPNG(window, outPath: "\(outDir)/01-home-light.png")

        // 2. Home Dark
        setDarkMode(true)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(outDir)/02-home-dark.png")

        // 3. Home Populated (Light)
        setDarkMode(false)
        UserDefaults.standard.set([pdf1, pdf2, pdf3], forKey: SettingsKeys.recentFiles)
        UserDefaults.standard.set([wsDir], forKey: SettingsKeys.recentWorkspaces)
        UserDefaults.standard.set([pdf1], forKey: SettingsKeys.favoriteFiles)
        app.homeScreenController.refreshContent()
        pumpRunLoop(0.18)
        captureWindowToPNG(window, outPath: "\(outDir)/03-home-populated.png")

        // 4. Reader Standalone Light
        app.openFile(at: URL(fileURLWithPath: pdf1))
        app.switchSidebarToIndex(0)
        pumpRunLoop(0.25)
        captureWindowToPNG(window, outPath: "\(outDir)/04-reader-standalone-light.png")

        // 5. Reader Standalone Dark
        setDarkMode(true)
        pumpRunLoop(0.18)
        captureWindowToPNG(window, outPath: "\(outDir)/05-reader-standalone-dark.png")

        // 6. Reader Workspace Light
        setDarkMode(false)
        app.openFolder(at: URL(fileURLWithPath: wsDir))
        WorkspaceIndex.shared.indexWorkspaceDirectory(wsDir)
        pumpRunLoop(0.25)
        captureWindowToPNG(window, outPath: "\(outDir)/06-reader-workspace-light.png")

        // 7. Reader Workspace Dark
        setDarkMode(true)
        pumpRunLoop(0.18)
        captureWindowToPNG(window, outPath: "\(outDir)/07-reader-workspace-dark.png")

        // Return to Light mode for inspector & feature screenshots
        setDarkMode(false)
        pumpRunLoop(0.15)

        // 8. Right Sidebar — Pages
        app.switchSidebarToIndex(0)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(outDir)/08-sidebar-pages.png")

        // 9. Right Sidebar — Outline
        if let doc = app.pdfView.document, (doc.outlineRoot == nil || doc.outlineRoot?.numberOfChildren == 0) {
            attachSyntheticOutline(to: doc)
            app.pdfOutlineController.loadOutline(from: doc)
        }
        app.switchSidebarToIndex(1)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(outDir)/09-sidebar-outline.png")

        // 10. Right Sidebar — Search
        app.switchSidebarToIndex(2)
        app.pdfSearchController.performSearch("a")
        pumpRunLoop(0.20)
        captureWindowToPNG(window, outPath: "\(outDir)/10-sidebar-search.png")

        // 11. Right Sidebar — Bookmarks
        _ = BookmarksManager.shared.toggleBookmark(pdfPath: pdf1, pageIndex: 0, defaultLabel: "Introduction & Key Primitives")
        app.refreshBookmarksView()
        app.switchSidebarToIndex(3)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(outDir)/11-sidebar-bookmarks.png")

        // 12. Right Sidebar — Annotations (with Highlight, Underline, Strikeout, Note)
        if let doc = app.pdfView.document, let page0 = doc.page(at: 0) {
            if let sel1 = page0.selection(for: NSRect(x: 50, y: 620, width: 420, height: 28)) {
                app.pdfView.setCurrentSelection(sel1, animate: false)
                app.highlightSelection(nil)
            }
            if let sel2 = page0.selection(for: NSRect(x: 50, y: 540, width: 380, height: 24)) {
                app.pdfView.setCurrentSelection(sel2, animate: false)
                app.underlineSelection(nil)
            }
            if let sel3 = page0.selection(for: NSRect(x: 50, y: 460, width: 340, height: 24)) {
                app.pdfView.setCurrentSelection(sel3, animate: false)
                app.strikeoutSelection(nil)
            }
        }
        app.switchSidebarToIndex(4)
        pumpRunLoop(0.20)
        captureWindowToPNG(window, outPath: "\(outDir)/12-sidebar-annotations.png")

        // Save annotations safely
        app.saveDocument(nil)

        // 13. Multi-Document Tabs (Open 3 tabs + dirty state on tab 2)
        app.openFile(at: URL(fileURLWithPath: pdf2), fromWorkspaceSidebar: true)
        app.openFile(at: URL(fileURLWithPath: pdf3), fromWorkspaceSidebar: true)
        app.markDocumentEdited(true)
        pumpRunLoop(0.20)
        captureWindowToPNG(window, outPath: "\(outDir)/13-tabs-multi-document.png")
        app.markDocumentEdited(false)

        // 14. Command Palette — Default / Recent
        app.quickOpen(nil)
        pumpRunLoop(0.18)
        captureWindowToPNG(app.commandPaletteController.panel, outPath: "\(outDir)/14-palette-empty.png")

        // 15. Command Palette — File Search Results
        app.commandPaletteController.updateSearch("Crypt")
        pumpRunLoop(0.15)
        captureWindowToPNG(app.commandPaletteController.panel, outPath: "\(outDir)/15-palette-search-results.png")

        // 16. Command Palette — Command Mode (`>`)
        app.commandPalette(nil)
        pumpRunLoop(0.15)
        captureWindowToPNG(app.commandPaletteController.panel, outPath: "\(outDir)/16-palette-command-mode.png")

        // 17. Command Palette — Workspace Full-Text Search (`//`)
        app.commandPaletteController.updateSearch("//crypt")
        pumpRunLoop(0.15)
        captureWindowToPNG(app.commandPaletteController.panel, outPath: "\(outDir)/17-palette-workspace-search.png")
        app.commandPaletteController.dismiss()

        // 18. Document Info Window
        app.showDocumentInfoWindow()
        pumpRunLoop(0.18)
        if let infoWin = UtilityWindows.shared.documentInfoWindow {
            captureWindowToPNG(infoWin, outPath: "\(outDir)/18-document-info.png")
        }
        app.closeDocumentInfoWindow(nil)

        // 19. OCR Running HUD & OCR Result Panel
        OCRService.shared.showRunningHUD(forPage: 0)
        pumpRunLoop(0.15)
        if let hud = OCRService.shared.runningHUDPanel {
            captureWindowToPNG(hud, outPath: "\(outDir)/19-ocr-running-hud.png")
        }
        OCRService.shared.dismissRunningHUD()

        if let doc = app.pdfView.document, let p0 = doc.page(at: 0),
           let ocrRes = try? OCRService.recognizeText(on: p0, pageIndex: 0, recognitionLevel: .fast) {
            OCRService.shared.showResultPanel(result: ocrRes)
            pumpRunLoop(0.18)
            if let resWin = OCRService.shared.resultWindow {
                captureWindowToPNG(resWin, outPath: "\(outDir)/19b-ocr-result-panel.png")
                resWin.orderOut(nil)
            }
        }

        // 20. Settings Window (General & Reading)
        SettingsWindowController.shared.showWindow(tabIndex: 0)
        pumpRunLoop(0.18)
        if let setWin = SettingsWindowController.shared.window {
            captureWindowToPNG(setWin, outPath: "\(outDir)/20-settings-general.png")
            SettingsWindowController.shared.selectTab(index: 1)
            pumpRunLoop(0.15)
            captureWindowToPNG(setWin, outPath: "\(outDir)/20b-settings-reading.png")
            setWin.orderOut(nil)
        }

        // 21. Error Panel & Debug Console
        UtilityWindows.shared.showError(
            title: "Cannot Open Corrupted PDF",
            message: "The file “corrupted_draft.pdf” is damaged or encrypted and could not be rendered."
        )
        pumpRunLoop(0.15)
        if let errWin = UtilityWindows.shared.errorWindow {
            captureWindowToPNG(errWin, outPath: "\(outDir)/21-error-panel.png")
            errWin.orderOut(nil)
        }

        app.openDebugConsole(nil)
        pumpRunLoop(0.18)
        if let dbgWin = DebugConsoleController.shared.window {
            captureWindowToPNG(dbgWin, outPath: "\(outDir)/21b-debug-console.png")
            dbgWin.orderOut(nil)
        }

        // 22. Fullscreen Presentation Mode Return
        app.togglePresentationMode(nil)
        pumpRunLoop(0.15)
        app.togglePresentationMode(nil)
        pumpRunLoop(0.18)
        captureWindowToPNG(window, outPath: "\(outDir)/22-fullscreen-presentation-return.png")

        print("[VISUAL_QA_DONE] Completed all 22+ Swift Visual QA screenshots in \(outDir)")
        if ProcessInfo.processInfo.environment["IRIDESCENCE_QA_EXIT"] != nil {
            exit(0)
        }
    }

    // MARK: - Canonical 19-Screenshot Suite (`scripts/visual_qa.sh` compatibility)

    public static func runCanonical19VisualQA(app: AppDelegate) {
        let passName = ProcessInfo.processInfo.environment["IRIDESCENCE_VISUAL_QA_PASS"] ?? "pass-01"
        let qaDir = ProcessInfo.processInfo.environment["IRIDESCENCE_QA_DIR"]
            ?? "/tmp/iridecence-visual-qa/\(passName)"
        try? FileManager.default.createDirectory(atPath: qaDir, withIntermediateDirectories: true)

        let wsDir = "/tmp/iridescence-qa-workspace"
        try? FileManager.default.createDirectory(atPath: "\(wsDir)/Cryptography", withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(atPath: "\(wsDir)/Protocols", withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(atPath: "\(wsDir)/Reference Papers", withIntermediateDirectories: true)

        let pdf1 = "\(wsDir)/Cryptography/Cryptanalysis_Overview.pdf"
        let pdf2 = "\(wsDir)/Protocols/TLS_1_3_Formal_Analysis.pdf"
        let pdf3 = "\(wsDir)/Reference Papers/Symmetric_Ciphers_Handbook.pdf"
        ensureRichTestPDF(at: pdf1)
        ensureRichTestPDF(at: pdf2)
        ensureRichTestPDF(at: pdf3)

        guard let window = app.window else { return }

        setDarkMode(false)
        app.showHomeScreen(nil)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(qaDir)/home-light.png")

        setDarkMode(true)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(qaDir)/home-dark.png")

        setDarkMode(false)
        app.openFile(at: URL(fileURLWithPath: pdf1))
        app.switchSidebarToIndex(0)
        pumpRunLoop(0.25)
        captureWindowToPNG(window, outPath: "\(qaDir)/reader-standalone-light.png")

        setDarkMode(true)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(qaDir)/reader-standalone-dark.png")

        setDarkMode(false)
        app.openFolder(at: URL(fileURLWithPath: wsDir))
        pumpRunLoop(0.25)
        captureWindowToPNG(window, outPath: "\(qaDir)/reader-workspace-light.png")

        setDarkMode(true)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(qaDir)/reader-workspace-dark.png")

        setDarkMode(false)
        app.openFile(at: URL(fileURLWithPath: pdf1))
        app.switchSidebarToIndex(0)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(qaDir)/sidebar-pages.png")

        if let doc = app.pdfView.document, (doc.outlineRoot == nil || doc.outlineRoot?.numberOfChildren == 0) {
            attachSyntheticOutline(to: doc)
            app.pdfOutlineController.loadOutline(from: doc)
        }
        app.switchSidebarToIndex(1)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(qaDir)/sidebar-outline.png")

        app.switchSidebarToIndex(2)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(qaDir)/sidebar-search.png")

        app.switchSidebarToIndex(3)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(qaDir)/sidebar-bookmarks.png")

        app.quickOpen(nil)
        pumpRunLoop(0.15)
        let panel = app.commandPaletteController.panel
        captureWindowToPNG(panel, outPath: "\(qaDir)/palette-empty.png")

        app.commandPaletteController.updateSearch("Crypt")
        pumpRunLoop(0.15)
        captureWindowToPNG(panel, outPath: "\(qaDir)/palette-results.png")

        app.commandPalette(nil)
        pumpRunLoop(0.15)
        captureWindowToPNG(panel, outPath: "\(qaDir)/palette-command.png")

        app.commandPaletteController.updateSearch("//")
        pumpRunLoop(0.15)
        captureWindowToPNG(panel, outPath: "\(qaDir)/palette-workspace-search.png")

        app.commandPaletteController.updateSearch("@")
        pumpRunLoop(0.15)
        captureWindowToPNG(panel, outPath: "\(qaDir)/palette-outline.png")

        app.commandPaletteController.updateSearch("#")
        pumpRunLoop(0.15)
        captureWindowToPNG(panel, outPath: "\(qaDir)/palette-bookmarks.png")

        app.commandPaletteController.updateSearch("/protocol")
        pumpRunLoop(0.15)
        captureWindowToPNG(panel, outPath: "\(qaDir)/palette-pdf-search.png")
        app.commandPaletteController.dismiss()

        app.showDocumentInfoWindow()
        pumpRunLoop(0.15)
        if let infoWin = UtilityWindows.shared.documentInfoWindow {
            captureWindowToPNG(infoWin, outPath: "\(qaDir)/document-info.png")
        }
        app.closeDocumentInfoWindow(nil)

        app.openDebugConsole(nil)
        pumpRunLoop(0.15)
        if let dbgWin = DebugConsoleController.shared.window {
            captureWindowToPNG(dbgWin, outPath: "\(qaDir)/debug-console.png")
            dbgWin.orderOut(nil)
        }

        app.togglePresentationMode(nil)
        pumpRunLoop(0.15)
        app.togglePresentationMode(nil)
        pumpRunLoop(0.15)
        captureWindowToPNG(window, outPath: "\(qaDir)/fullscreen-return.png")

        print("[VISUAL_QA_DONE] Completed all 19 canonical screenshots in \(qaDir)")
        exit(0)
    }

    public static func runAnnotationsVisualQA(app: AppDelegate) {
        runComplete22StateSwiftVisualQA(app: app)
    }

    public static func runTabsVisualQA(app: AppDelegate) {
        runComplete22StateSwiftVisualQA(app: app)
    }

    public static func runOCRVisualQA(app: AppDelegate) {
        runComplete22StateSwiftVisualQA(app: app)
    }
}
