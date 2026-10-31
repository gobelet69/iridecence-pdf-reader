import AppKit
import Foundation

/// Live diagnostic console window (`Live Debug Console`) with Clear, Pause/Resume, and Auto-scroll controls.
@MainActor
public final class DebugConsoleController: NSObject {
    public static let shared = DebugConsoleController()

    public private(set) var window: NSWindow?
    private var textView: NSTextView?
    private var pauseButton: NSButton?
    private var autoScrollCheckbox: NSButton?
    private var statusCountLabel: NSTextField?

    public override init() {
        super.init()
    }

    public func showWindow() {
        if window == nil {
            buildWindow()
        }
        refreshLogs()
        if let win = window {
            WindowGeometry.sanitizeWindow(win, kind: .debugConsole)
            win.makeKeyAndOrderFront(nil)
        }
    }

    private func buildWindow() {
        let rect = WindowGeometry.initialFrame(for: .debugConsole)
        let win = NSWindow(
            contentRect: rect,
            styleMask: [.titled, .closable, .resizable, .miniaturizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        win.title = "Live Debug Console"
        win.titlebarAppearsTransparent = true
        win.isReleasedWhenClosed = false
        win.isOpaque = false
        win.backgroundColor = .clear

        let effect = NSVisualEffectView(frame: NSRect(x: 0, y: 0, width: rect.width, height: rect.height))
        effect.material = .underWindowBackground
        effect.blendingMode = .behindWindow
        effect.state = .followsWindowActiveState
        effect.autoresizingMask = [.width, .height]

        let headerBar = NSStackView()
        headerBar.orientation = .horizontal
        headerBar.alignment = .centerY
        headerBar.spacing = 10.0
        headerBar.translatesAutoresizingMaskIntoConstraints = false

        let countLabel = NSTextField(labelWithString: "0 entries")
        countLabel.font = .monospacedDigitSystemFont(ofSize: 11.5, weight: .medium)
        countLabel.textColor = .secondaryLabelColor
        self.statusCountLabel = countLabel

        let autoChk = NSButton(checkboxWithTitle: "Auto-scroll", target: self, action: nil)
        autoChk.state = .on
        autoChk.controlSize = .small
        autoChk.font = .systemFont(ofSize: 11.5)
        self.autoScrollCheckbox = autoChk

        let pauseBtn = NSButton(title: "Pause", target: self, action: #selector(togglePauseClicked(_:)))
        pauseBtn.bezelStyle = .rounded
        pauseBtn.controlSize = .small
        self.pauseButton = pauseBtn

        let clearBtn = NSButton(title: "Clear", target: self, action: #selector(clearClicked(_:)))
        clearBtn.bezelStyle = .rounded
        clearBtn.controlSize = .small

        headerBar.addArrangedSubview(countLabel)
        headerBar.addArrangedSubview(autoChk)
        headerBar.addArrangedSubview(pauseBtn)
        headerBar.addArrangedSubview(clearBtn)

        let scroll = NSScrollView()
        scroll.hasVerticalScroller = true
        scroll.drawsBackground = true
        scroll.backgroundColor = NSColor.textBackgroundColor.withAlphaComponent(0.55)
        scroll.wantsLayer = true
        scroll.layer?.cornerRadius = UIConstants.radiusRow
        scroll.translatesAutoresizingMaskIntoConstraints = false

        let tv = NSTextView(frame: NSRect(x: 0, y: 0, width: rect.width - 40, height: rect.height - 90))
        tv.isEditable = false
        tv.isSelectable = true
        tv.drawsBackground = false
        tv.font = .monospacedSystemFont(ofSize: 11.5, weight: .regular)
        tv.textColor = .labelColor
        tv.textContainerInset = NSSize(width: 8, height: 8)
        tv.autoresizingMask = [.width]
        scroll.documentView = tv
        self.textView = tv

        effect.addSubview(headerBar)
        effect.addSubview(scroll)

        NSLayoutConstraint.activate([
            headerBar.topAnchor.constraint(equalTo: effect.topAnchor, constant: 36.0),
            headerBar.trailingAnchor.constraint(equalTo: effect.trailingAnchor, constant: -16.0),
            headerBar.leadingAnchor.constraint(greaterThanOrEqualTo: effect.leadingAnchor, constant: 16.0),

            scroll.topAnchor.constraint(equalTo: headerBar.bottomAnchor, constant: 8.0),
            scroll.leadingAnchor.constraint(equalTo: effect.leadingAnchor, constant: 14.0),
            scroll.trailingAnchor.constraint(equalTo: effect.trailingAnchor, constant: -14.0),
            scroll.bottomAnchor.constraint(equalTo: effect.bottomAnchor, constant: -14.0)
        ])

        win.contentView = effect
        self.window = win

        AppLogger.shared.setObserver { [weak self] in
            Task { @MainActor in
                self?.refreshLogs()
            }
        }
    }

    public func refreshLogs() {
        guard let tv = textView else { return }
        let entries = AppLogger.shared.entries()
        statusCountLabel?.stringValue = "\(entries.count) / \(AppLogger.maxBufferSize) entries"
        let combined = entries.map(\.formattedLine).joined(separator: "\n")
        tv.string = combined
        if autoScrollCheckbox?.state == .on && !combined.isEmpty {
            tv.scrollToEndOfDocument(nil)
        }
    }

    @objc private func togglePauseClicked(_ sender: NSButton) {
        let next = !AppLogger.shared.isPaused
        AppLogger.shared.isPaused = next
        sender.title = next ? "Resume" : "Pause"
    }

    @objc private func clearClicked(_ sender: Any?) {
        AppLogger.shared.clear()
        refreshLogs()
    }
}
