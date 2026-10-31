import AppKit
import Foundation
import PDFKit

/// Manages the Document Info window (`Cmd+I`) and the Error Panel window.
@MainActor
public final class UtilityWindows: NSObject {
    public static let shared = UtilityWindows()

    public private(set) var documentInfoWindow: NSWindow?
    public private(set) var errorWindow: NSWindow?
    private var currentDocumentPath: String = ""

    public override init() {
        super.init()
    }

    // MARK: - Document Info Window (Cmd+I)

    public func showDocumentInfo(for document: PDFDocument?, filePath: String) {
        documentInfoWindow?.orderOut(nil)
        currentDocumentPath = filePath

        let rect = NSRect(x: 0, y: 0, width: 480, height: 390)
        let win = NSWindow(
            contentRect: rect,
            styleMask: [.titled, .closable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        win.title = "Document Info"
        win.titlebarAppearsTransparent = true
        win.isReleasedWhenClosed = false
        win.isOpaque = false
        win.backgroundColor = .clear
        win.center()
        WindowGeometry.sanitizeWindow(win, kind: .utility)

        let effect = NSVisualEffectView(frame: rect)
        effect.material = .underWindowBackground
        effect.blendingMode = .behindWindow
        effect.state = .followsWindowActiveState
        effect.autoresizingMask = [.width, .height]

        let fileName = filePath.isEmpty ? "No Document Open" : URL(fileURLWithPath: filePath).lastPathComponent
        let headerTitle = NSTextField(labelWithString: fileName)
        headerTitle.font = .systemFont(ofSize: 16.0, weight: .bold)
        headerTitle.textColor = .labelColor
        headerTitle.lineBreakMode = .byTruncatingMiddle
        headerTitle.translatesAutoresizingMaskIntoConstraints = false

        var attrsDict: [AnyHashable: Any] = [:]
        if let dAttrs = document?.documentAttributes {
            attrsDict = dAttrs
        }

        let pdfTitle = (attrsDict[PDFDocumentAttribute.titleAttribute] as? String) ?? "—"
        let pdfAuthor = (attrsDict[PDFDocumentAttribute.authorAttribute] as? String) ?? "—"
        let pdfSubject = (attrsDict[PDFDocumentAttribute.subjectAttribute] as? String) ?? "—"
        let pdfCreator = (attrsDict[PDFDocumentAttribute.creatorAttribute] as? String) ?? "—"
        let pdfProducer = (attrsDict[PDFDocumentAttribute.producerAttribute] as? String) ?? "—"
        let pageCount = document?.pageCount ?? 0

        var pageSizeStr = "—"
        if let firstPage = document?.page(at: 0) {
            let b = firstPage.bounds(for: .mediaBox)
            pageSizeStr = String(format: "%.0f × %.0f pt", b.width, b.height)
        }

        var fileSizeStr = "—"
        if !filePath.isEmpty,
           let fAttrs = try? FileManager.default.attributesOfItem(atPath: filePath),
           let bytes = fAttrs[.size] as? UInt64 {
            fileSizeStr = ByteCountFormatter.string(fromByteCount: Int64(bytes), countStyle: .file)
        }

        let securityStr: String = {
            guard let doc = document else { return "—" }
            return doc.isEncrypted ? "Encrypted (Password Protected)" : "Standard (Unencrypted)"
        }()

        let rows: [(String, String)] = [
            ("Title:", pdfTitle.isEmpty ? "—" : pdfTitle),
            ("Author:", pdfAuthor.isEmpty ? "—" : pdfAuthor),
            ("Subject:", pdfSubject.isEmpty ? "—" : pdfSubject),
            ("Creator:", pdfCreator.isEmpty ? "—" : pdfCreator),
            ("Producer:", pdfProducer.isEmpty ? "—" : pdfProducer),
            ("Pages:", "\(pageCount)"),
            ("Page Size:", pageSizeStr),
            ("File Size:", fileSizeStr),
            ("Security:", securityStr),
            ("Location:", filePath.isEmpty ? "—" : filePath)
        ]

        let grid = NSStackView()
        grid.orientation = .vertical
        grid.alignment = .leading
        grid.spacing = 6.0
        grid.translatesAutoresizingMaskIntoConstraints = false

        for (key, val) in rows {
            let row = NSStackView()
            row.orientation = .horizontal
            row.alignment = .firstBaseline
            row.spacing = 10.0

            let kLbl = NSTextField(labelWithString: key)
            kLbl.font = .systemFont(ofSize: 12.0, weight: .semibold)
            kLbl.textColor = .secondaryLabelColor
            kLbl.alignment = .right
            kLbl.translatesAutoresizingMaskIntoConstraints = false
            kLbl.widthAnchor.constraint(equalToConstant: 80.0).isActive = true

            let vLbl = NSTextField(labelWithString: val)
            vLbl.font = .systemFont(ofSize: 12.0, weight: .regular)
            vLbl.textColor = .labelColor
            vLbl.lineBreakMode = .byTruncatingMiddle
            vLbl.isSelectable = true
            vLbl.translatesAutoresizingMaskIntoConstraints = false
            vLbl.widthAnchor.constraint(lessThanOrEqualToConstant: 340.0).isActive = true

            row.addArrangedSubview(kLbl)
            row.addArrangedSubview(vLbl)
            grid.addArrangedSubview(row)
        }

        let buttonRow = NSStackView()
        buttonRow.orientation = .horizontal
        buttonRow.spacing = 10.0
        buttonRow.translatesAutoresizingMaskIntoConstraints = false

        let copyBtn = NSButton(title: "Copy Path", target: self, action: #selector(copyDocPath(_:)))
        copyBtn.bezelStyle = .rounded
        copyBtn.isEnabled = !filePath.isEmpty

        let revealBtn = NSButton(title: "Reveal in Finder", target: self, action: #selector(revealDocInFinder(_:)))
        revealBtn.bezelStyle = .rounded
        revealBtn.isEnabled = !filePath.isEmpty

        let closeBtn = NSButton(title: "Done", target: self, action: #selector(closeDocInfo(_:)))
        closeBtn.bezelStyle = .rounded
        closeBtn.keyEquivalent = "\r"

        buttonRow.addArrangedSubview(copyBtn)
        buttonRow.addArrangedSubview(revealBtn)
        buttonRow.addArrangedSubview(closeBtn)

        effect.addSubview(headerTitle)
        effect.addSubview(grid)
        effect.addSubview(buttonRow)

        NSLayoutConstraint.activate([
            headerTitle.topAnchor.constraint(equalTo: effect.topAnchor, constant: 38.0),
            headerTitle.leadingAnchor.constraint(equalTo: effect.leadingAnchor, constant: 24.0),
            headerTitle.trailingAnchor.constraint(equalTo: effect.trailingAnchor, constant: -24.0),

            grid.topAnchor.constraint(equalTo: headerTitle.bottomAnchor, constant: 14.0),
            grid.leadingAnchor.constraint(equalTo: effect.leadingAnchor, constant: 24.0),
            grid.trailingAnchor.constraint(equalTo: effect.trailingAnchor, constant: -24.0),

            buttonRow.trailingAnchor.constraint(equalTo: effect.trailingAnchor, constant: -24.0),
            buttonRow.bottomAnchor.constraint(equalTo: effect.bottomAnchor, constant: -16.0)
        ])

        win.contentView = effect
        win.makeKeyAndOrderFront(nil)
        self.documentInfoWindow = win
    }

    public func closeDocumentInfo() {
        documentInfoWindow?.orderOut(nil)
    }

    @objc private func copyDocPath(_ sender: Any?) {
        guard !currentDocumentPath.isEmpty else { return }
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(currentDocumentPath, forType: .string)
    }

    @objc private func revealDocInFinder(_ sender: Any?) {
        guard !currentDocumentPath.isEmpty else { return }
        NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: currentDocumentPath)])
    }

    @objc private func closeDocInfo(_ sender: Any?) {
        closeDocumentInfo()
    }

    // MARK: - Error Panel Window

    public func showError(title: String, message: String) {
        errorWindow?.orderOut(nil)
        AppLogger.shared.log(.error, "\(title): \(message)")

        let rect = NSRect(x: 0, y: 0, width: 420, height: 190)
        let win = NSWindow(
            contentRect: rect,
            styleMask: [.titled, .closable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        win.title = title
        win.titlebarAppearsTransparent = true
        win.isReleasedWhenClosed = false
        win.isOpaque = false
        win.backgroundColor = .clear
        win.center()

        let effect = NSVisualEffectView(frame: rect)
        effect.material = .hudWindow
        effect.blendingMode = .behindWindow
        effect.state = .active
        effect.autoresizingMask = [.width, .height]

        let icon = NSImageView()
        icon.translatesAutoresizingMaskIntoConstraints = false
        if let img = NSImage(systemSymbolName: "exclamationmark.triangle.fill", accessibilityDescription: "Error") {
            let cfg = NSImage.SymbolConfiguration(pointSize: 28.0, weight: .semibold)
            icon.image = img.withSymbolConfiguration(cfg) ?? img
            icon.contentTintColor = .systemOrange
        }

        let titleLbl = NSTextField(labelWithString: title)
        titleLbl.font = .systemFont(ofSize: 14.0, weight: .bold)
        titleLbl.textColor = .labelColor
        titleLbl.translatesAutoresizingMaskIntoConstraints = false

        let msgLbl = NSTextField(wrappingLabelWithString: message)
        msgLbl.font = .systemFont(ofSize: 12.0, weight: .regular)
        msgLbl.textColor = .secondaryLabelColor
        msgLbl.preferredMaxLayoutWidth = 310.0
        msgLbl.translatesAutoresizingMaskIntoConstraints = false

        let okBtn = NSButton(title: "OK", target: self, action: #selector(dismissError(_:)))
        okBtn.bezelStyle = .rounded
        okBtn.keyEquivalent = "\r"
        okBtn.translatesAutoresizingMaskIntoConstraints = false

        effect.addSubview(icon)
        effect.addSubview(titleLbl)
        effect.addSubview(msgLbl)
        effect.addSubview(okBtn)

        NSLayoutConstraint.activate([
            icon.leadingAnchor.constraint(equalTo: effect.leadingAnchor, constant: 24.0),
            icon.topAnchor.constraint(equalTo: effect.topAnchor, constant: 44.0),
            icon.widthAnchor.constraint(equalToConstant: 34.0),
            icon.heightAnchor.constraint(equalToConstant: 34.0),

            titleLbl.leadingAnchor.constraint(equalTo: icon.trailingAnchor, constant: 14.0),
            titleLbl.topAnchor.constraint(equalTo: icon.topAnchor),
            titleLbl.trailingAnchor.constraint(equalTo: effect.trailingAnchor, constant: -20.0),

            msgLbl.leadingAnchor.constraint(equalTo: titleLbl.leadingAnchor),
            msgLbl.topAnchor.constraint(equalTo: titleLbl.bottomAnchor, constant: 6.0),
            msgLbl.trailingAnchor.constraint(equalTo: effect.trailingAnchor, constant: -20.0),

            okBtn.trailingAnchor.constraint(equalTo: effect.trailingAnchor, constant: -20.0),
            okBtn.bottomAnchor.constraint(equalTo: effect.bottomAnchor, constant: -16.0),
            okBtn.widthAnchor.constraint(greaterThanOrEqualToConstant: 72.0)
        ])

        win.contentView = effect
        win.makeKeyAndOrderFront(nil)
        self.errorWindow = win
    }

    @objc private func dismissError(_ sender: Any?) {
        errorWindow?.orderOut(nil)
    }
}
