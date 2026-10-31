import AppKit
import Foundation
import PDFKit
import Vision

public struct OCRPageResult: Sendable, Equatable {
    public let pageIndex: Int
    public let text: String
    public let lineCount: Int
    public let wordCount: Int

    public init(pageIndex: Int, text: String) {
        self.pageIndex = pageIndex
        self.text = text
        let lines = text.split(whereSeparator: \.isNewline)
        self.lineCount = lines.count
        self.wordCount = text.split(whereSeparator: \.isWhitespace).count
    }
}

public enum OCRError: LocalizedError {
    case cancelled
    case renderFailed
    case visionFailed(String)

    public var errorDescription: String? {
        switch self {
        case .cancelled:
            return "OCR operation was cancelled."
        case .renderFailed:
            return "Failed to rasterize PDF page for Vision OCR."
        case .visionFailed(let msg):
            return "Apple Vision OCR failed: \(msg)"
        }
    }
}

public final class OCRCancellationToken: @unchecked Sendable {
    private let lock = NSLock()
    private var cancelled = false

    public init() {}

    public var isCancelled: Bool {
        lock.lock()
        defer { lock.unlock() }
        return cancelled
    }

    public func cancel() {
        lock.lock()
        cancelled = true
        lock.unlock()
    }
}

/// Coordinates on-demand Apple Vision (`VNRecognizeTextRequest`) OCR on the current PDF page,
/// including a floating Liquid Glass progress HUD with a live Cancel button and a Result Panel.
@MainActor
public final class OCRService: NSObject {
    public static let shared = OCRService()

    public private(set) var runningHUDPanel: NSPanel?
    public private(set) var resultWindow: NSWindow?
    private var currentToken: OCRCancellationToken?
    private var lastExtractedText: String = ""

    public override init() {
        super.init()
    }

    /// Rasterizes a `PDFPage` at 2x resolution and runs `VNRecognizeTextRequest`.
    nonisolated public static func recognizeText(
        on page: PDFPage,
        pageIndex: Int,
        token: OCRCancellationToken = OCRCancellationToken(),
        recognitionLevel: VNRequestTextRecognitionLevel = .accurate
    ) throws -> OCRPageResult {
        if token.isCancelled {
            throw OCRError.cancelled
        }

        let bounds = page.bounds(for: .mediaBox)
        let scale: CGFloat = recognitionLevel == .fast ? 1.25 : 2.0
        let width = max(1, Int((bounds.width * scale).rounded()))
        let height = max(1, Int((bounds.height * scale).rounded()))

        let colorSpace = CGColorSpaceCreateDeviceRGB()
        guard let ctx = CGContext(
            data: nil,
            width: width,
            height: height,
            bitsPerComponent: 8,
            bytesPerRow: 0,
            space: colorSpace,
            bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
        ) else {
            throw OCRError.renderFailed
        }

        ctx.setFillColor(CGColor(red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0))
        ctx.fill(CGRect(x: 0, y: 0, width: width, height: height))
        ctx.saveGState()
        ctx.scaleBy(x: scale, y: scale)
        page.draw(with: .mediaBox, to: ctx)
        ctx.restoreGState()

        guard let cgImage = ctx.makeImage() else {
            throw OCRError.renderFailed
        }

        if token.isCancelled {
            throw OCRError.cancelled
        }

        let request = VNRecognizeTextRequest()
        request.recognitionLevel = recognitionLevel
        request.usesLanguageCorrection = (recognitionLevel == .accurate)

        let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
        do {
            try handler.perform([request])
        } catch {
            throw OCRError.visionFailed(error.localizedDescription)
        }

        if token.isCancelled {
            throw OCRError.cancelled
        }

        let observations = request.results ?? []
        var lines: [String] = []
        lines.reserveCapacity(observations.count)
        for obs in observations {
            if let candidate = obs.topCandidates(1).first {
                lines.append(candidate.string)
            }
        }

        let fullText = lines.joined(separator: "\n")
        return OCRPageResult(pageIndex: pageIndex, text: fullText)
    }

    // MARK: - Running HUD & Result Panel UI

    public func showRunningHUD(forPage pageIndex: Int, token: OCRCancellationToken? = nil) {
        dismissRunningHUD()
        self.currentToken = token

        let rect = NSRect(x: 0, y: 0, width: 320, height: 64)
        let panel = NSPanel(
            contentRect: rect,
            styleMask: [.titled, .fullSizeContentView, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        panel.titleVisibility = .hidden
        panel.titlebarAppearsTransparent = true
        panel.isFloatingPanel = true
        panel.level = .floating
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hasShadow = true

        if let mainWin = NSApp.mainWindow {
            let mf = mainWin.frame
            let x = mf.origin.x + (mf.width - rect.width) * 0.5
            let y = mf.origin.y + (mf.height - rect.height) * 0.5
            panel.setFrameOrigin(NSPoint(x: x.rounded(), y: y.rounded()))
        } else {
            panel.center()
        }

        let effect = NSVisualEffectView(frame: rect)
        effect.material = .hudWindow
        effect.blendingMode = .behindWindow
        effect.state = .active
        effect.wantsLayer = true
        effect.layer?.cornerRadius = UIConstants.radiusCard

        let spinner = NSProgressIndicator(frame: NSRect(x: 18, y: 20, width: 24, height: 24))
        spinner.style = .spinning
        spinner.controlSize = .small
        spinner.startAnimation(nil)

        let label = NSTextField(labelWithString: "Running Vision OCR on Page \(pageIndex + 1)…")
        label.font = .systemFont(ofSize: 12.5, weight: .medium)
        label.textColor = .labelColor
        label.frame = NSRect(x: 52, y: 23, width: 186, height: 18)

        let cancelBtn = NSButton(title: "Cancel", target: self, action: #selector(cancelOCRClicked(_:)))
        cancelBtn.bezelStyle = .rounded
        cancelBtn.controlSize = .small
        cancelBtn.frame = NSRect(x: 242, y: 18, width: 64, height: 26)

        effect.addSubview(spinner)
        effect.addSubview(label)
        effect.addSubview(cancelBtn)
        panel.contentView = effect
        panel.orderFront(nil)
        self.runningHUDPanel = panel
    }

    public func dismissRunningHUD() {
        runningHUDPanel?.orderOut(nil)
        runningHUDPanel = nil
    }

    @objc public func cancelOCRClicked(_ sender: Any?) {
        currentToken?.cancel()
        dismissRunningHUD()
        AppLogger.shared.log(.pdf, "User cancelled running Vision OCR")
    }

    public func showResultPanel(result: OCRPageResult, autoCopy: Bool = SettingsManager.shared.autoCopyOcr) {
        dismissRunningHUD()
        lastExtractedText = result.text

        if autoCopy && !result.text.isEmpty {
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(result.text, forType: .string)
        }

        resultWindow?.orderOut(nil)

        let rect = WindowGeometry.initialFrame(for: .utility)
        let win = NSWindow(
            contentRect: NSRect(x: rect.origin.x, y: rect.origin.y, width: 540, height: 420),
            styleMask: [.titled, .closable, .resizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        win.title = "Vision OCR — Page \(result.pageIndex + 1)"
        win.titlebarAppearsTransparent = true
        win.isReleasedWhenClosed = false
        win.isOpaque = false
        win.backgroundColor = .clear
        WindowGeometry.sanitizeWindow(win, kind: .utility)

        let effect = NSVisualEffectView(frame: NSRect(x: 0, y: 0, width: 540, height: 420))
        effect.material = .underWindowBackground
        effect.blendingMode = .behindWindow
        effect.state = .followsWindowActiveState
        effect.autoresizingMask = [.width, .height]

        let statsText = "\(result.wordCount) words · \(result.lineCount) lines" + (autoCopy ? " · Copied to Clipboard" : "")
        let statsLabel = NSTextField(labelWithString: statsText)
        statsLabel.font = .systemFont(ofSize: 11.5, weight: .medium)
        statsLabel.textColor = .secondaryLabelColor
        statsLabel.translatesAutoresizingMaskIntoConstraints = false

        let copyBtn = NSButton(title: "Copy Text", target: self, action: #selector(copyResultText(_:)))
        copyBtn.bezelStyle = .rounded
        copyBtn.translatesAutoresizingMaskIntoConstraints = false

        let scroll = NSScrollView(frame: NSRect(x: 16, y: 16, width: 508, height: 320))
        scroll.hasVerticalScroller = true
        scroll.drawsBackground = true
        scroll.backgroundColor = NSColor.textBackgroundColor.withAlphaComponent(0.5)
        scroll.wantsLayer = true
        scroll.layer?.cornerRadius = UIConstants.radiusRow
        scroll.translatesAutoresizingMaskIntoConstraints = false

        let textView = NSTextView(frame: NSRect(x: 0, y: 0, width: 490, height: 320))
        textView.isEditable = false
        textView.isSelectable = true
        textView.drawsBackground = false
        textView.font = .monospacedSystemFont(ofSize: 12.0, weight: .regular)
        textView.textColor = .labelColor
        textView.string = result.text.isEmpty ? "(No recognizable text found on this page)" : result.text
        textView.textContainerInset = NSSize(width: 10, height: 10)
        textView.autoresizingMask = [.width]
        scroll.documentView = textView

        effect.addSubview(statsLabel)
        effect.addSubview(copyBtn)
        effect.addSubview(scroll)

        NSLayoutConstraint.activate([
            statsLabel.topAnchor.constraint(equalTo: effect.topAnchor, constant: 42.0),
            statsLabel.leadingAnchor.constraint(equalTo: effect.leadingAnchor, constant: 18.0),

            copyBtn.centerYAnchor.constraint(equalTo: statsLabel.centerYAnchor),
            copyBtn.trailingAnchor.constraint(equalTo: effect.trailingAnchor, constant: -18.0),

            scroll.topAnchor.constraint(equalTo: statsLabel.bottomAnchor, constant: 10.0),
            scroll.leadingAnchor.constraint(equalTo: effect.leadingAnchor, constant: 16.0),
            scroll.trailingAnchor.constraint(equalTo: effect.trailingAnchor, constant: -16.0),
            scroll.bottomAnchor.constraint(equalTo: effect.bottomAnchor, constant: -16.0)
        ])

        win.contentView = effect
        win.makeKeyAndOrderFront(nil)
        self.resultWindow = win
    }

    @objc private func copyResultText(_ sender: Any?) {
        guard !lastExtractedText.isEmpty else { return }
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(lastExtractedText, forType: .string)
    }
}
