import AppKit
import Foundation
import PDFKit

public enum AppContentMode: Int, Sendable, Equatable {
    case home = 0
    case standaloneDocument = 1
    case workspace = 2
}

public struct ClosedTabInfo: Sendable, Equatable {
    public let path: String
    public let pageIndex: Int
    public let scaleFactor: CGFloat
    public let mode: AppContentMode
    public let workspaceRoot: String?

    public init(
        path: String,
        pageIndex: Int,
        scaleFactor: CGFloat,
        mode: AppContentMode,
        workspaceRoot: String? = nil
    ) {
        self.path = path
        self.pageIndex = pageIndex
        self.scaleFactor = scaleFactor
        self.mode = mode
        self.workspaceRoot = workspaceRoot
    }
}

public final class TabSession {
    public var path: String
    public var title: String
    public var document: PDFDocument
    public var pageIndex: Int
    public var scaleFactor: CGFloat
    public var isEdited: Bool
    public var mode: AppContentMode
    public var workspaceRoot: String?
    public var lastKnownMtime: UInt64?

    public init(
        path: String,
        title: String,
        document: PDFDocument,
        pageIndex: Int = 0,
        scaleFactor: CGFloat = 1.0,
        isEdited: Bool = false,
        mode: AppContentMode = .standaloneDocument,
        workspaceRoot: String? = nil,
        lastKnownMtime: UInt64? = nil
    ) {
        self.path = path
        self.title = title
        self.document = document
        self.pageIndex = pageIndex
        self.scaleFactor = scaleFactor
        self.isEdited = isEdited
        self.mode = mode
        self.workspaceRoot = workspaceRoot
        self.lastKnownMtime = lastKnownMtime
    }
}

/// Manages open document tabs, active tab state, closed-tab history (`Cmd+Shift+T`),
/// and physical keycode mapping (`Cmd+1..9` for both QWERTY and French AZERTY layouts).
public final class TabManager {
    public private(set) var tabs: [TabSession] = []
    public var activeIndex: Int = 0
    public private(set) var closedTabs: [ClosedTabInfo] = []

    public static let maxClosedTabs: Int = 25

    public init() {}

    public var tabCount: Int {
        tabs.count
    }

    public var activeTab: TabSession? {
        guard activeIndex >= 0, activeIndex < tabs.count else { return nil }
        return tabs[activeIndex]
    }

    public func indexOfPath(_ path: String) -> Int? {
        let target = UIConstants.canonicalPath(path)
        guard !target.isEmpty else { return nil }
        return tabs.firstIndex { UIConstants.canonicalPath($0.path) == target }
    }

    @discardableResult
    public func addTab(
        path: String,
        title: String,
        document: PDFDocument,
        mode: AppContentMode,
        workspaceRoot: String? = nil
    ) -> Int {
        if let existing = indexOfPath(path) {
            activeIndex = existing
            return existing
        }
        let mtime = Self.fileMtime(forPath: path)
        let session = TabSession(
            path: path,
            title: title,
            document: document,
            pageIndex: 0,
            scaleFactor: 1.0,
            isEdited: false,
            mode: mode,
            workspaceRoot: workspaceRoot,
            lastKnownMtime: mtime
        )
        tabs.append(session)
        activeIndex = tabs.count - 1
        return activeIndex
    }

    @discardableResult
    public func removeTab(at index: Int, recordClosed: Bool = true) -> TabSession? {
        guard index >= 0, index < tabs.count else { return nil }
        let removed = tabs.remove(at: index)

        if recordClosed && !removed.path.isEmpty {
            let info = ClosedTabInfo(
                path: removed.path,
                pageIndex: removed.pageIndex,
                scaleFactor: removed.scaleFactor,
                mode: removed.mode,
                workspaceRoot: removed.workspaceRoot
            )
            closedTabs.removeAll { UIConstants.canonicalPath($0.path) == UIConstants.canonicalPath(info.path) }
            closedTabs.append(info)
            if closedTabs.count > Self.maxClosedTabs {
                closedTabs.removeFirst(closedTabs.count - Self.maxClosedTabs)
            }
        }

        if tabs.isEmpty {
            activeIndex = 0
        } else if activeIndex >= tabs.count {
            activeIndex = tabs.count - 1
        } else if index < activeIndex {
            activeIndex = max(0, activeIndex - 1)
        }
        return removed
    }

    public func popLastClosedTab() -> ClosedTabInfo? {
        closedTabs.popLast()
    }

    public func nextIndex() -> Int {
        guard !tabs.isEmpty else { return 0 }
        return (activeIndex + 1) % tabs.count
    }

    public func prevIndex() -> Int {
        guard !tabs.isEmpty else { return 0 }
        return (activeIndex + tabs.count - 1) % tabs.count
    }

    /// Maps physical top-row number keycodes (`18, 19, 20, 21, 23, 22, 26, 28, 25`)
    /// so `Cmd+1..9` works identically on QWERTY and French AZERTY keyboards.
    public static func tabNumberForKeyCode(_ keyCode: UInt16) -> Int? {
        switch keyCode {
        case 18: return 1
        case 19: return 2
        case 20: return 3
        case 21: return 4
        case 23: return 5
        case 22: return 6
        case 26: return 7
        case 28: return 8
        case 25: return 9
        default: return nil
        }
    }

    public static func fileMtime(forPath path: String) -> UInt64? {
        guard let attrs = try? FileManager.default.attributesOfItem(atPath: path),
              let date = attrs[.modificationDate] as? Date else {
            return nil
        }
        return UInt64(max(0, date.timeIntervalSince1970))
    }
}

/// Custom Liquid Glass multi-document tab bar displaying up to 5 visible tab pills,
/// an overflow `+N ▾` menu button, a `+` new tab button, dirty dot `●`, and right-click context menu.
@MainActor
public final class TabBarView: NSVisualEffectView {
    public static let maxVisiblePills: Int = 5
    public static let barHeight: CGFloat = 36.0

    public var onSelectTab: ((Int) -> Void)?
    public var onCloseTab: ((Int) -> Void)?
    public var onCloseOtherTabs: ((Int) -> Void)?
    public var onNewTab: (() -> Void)?

    private let stackView = NSStackView()
    private let bottomSeparator = NSView()
    private var currentTabsSnapshot: [(title: String, path: String, isEdited: Bool)] = []
    private var currentActiveIndex: Int = 0

    public override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        setupView()
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupView() {
        material = .headerView
        blendingMode = .withinWindow
        state = .followsWindowActiveState
        translatesAutoresizingMaskIntoConstraints = false

        stackView.orientation = .horizontal
        stackView.alignment = .centerY
        stackView.spacing = 6.0
        stackView.translatesAutoresizingMaskIntoConstraints = false

        bottomSeparator.wantsLayer = true
        bottomSeparator.layer?.backgroundColor = NSColor.separatorColor.cgColor
        bottomSeparator.translatesAutoresizingMaskIntoConstraints = false

        addSubview(stackView)
        addSubview(bottomSeparator)

        NSLayoutConstraint.activate([
            stackView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 10.0),
            stackView.trailingAnchor.constraint(lessThanOrEqualTo: trailingAnchor, constant: -10.0),
            stackView.centerYAnchor.constraint(equalTo: centerYAnchor),

            bottomSeparator.leadingAnchor.constraint(equalTo: leadingAnchor),
            bottomSeparator.trailingAnchor.constraint(equalTo: trailingAnchor),
            bottomSeparator.bottomAnchor.constraint(equalTo: bottomAnchor),
            bottomSeparator.heightAnchor.constraint(equalToConstant: 1.0)
        ])
    }

    public func rebuild(from manager: TabManager) {
        for sub in stackView.arrangedSubviews {
            stackView.removeArrangedSubview(sub)
            sub.removeFromSuperview()
        }

        let tabs = manager.tabs
        currentTabsSnapshot = tabs.map { ($0.title, $0.path, $0.isEdited) }
        currentActiveIndex = manager.activeIndex

        guard !tabs.isEmpty else { return }

        // Ensure active tab is visible among the 5 pills
        var visibleIndices = Array(0..<min(tabs.count, Self.maxVisiblePills))
        if manager.activeIndex >= Self.maxVisiblePills && manager.activeIndex < tabs.count {
            visibleIndices[Self.maxVisiblePills - 1] = manager.activeIndex
        }

        for idx in visibleIndices {
            let tab = tabs[idx]
            let isSelected = (idx == manager.activeIndex)
            let pill = buildTabPill(index: idx, title: tab.title, isEdited: tab.isEdited, isSelected: isSelected)
            stackView.addArrangedSubview(pill)
        }

        if tabs.count > Self.maxVisiblePills {
            let overflowCount = tabs.count - Self.maxVisiblePills
            let overflowBtn = NSButton(title: "+\(overflowCount) ▾", target: self, action: #selector(overflowClicked(_:)))
            overflowBtn.bezelStyle = .recessed
            overflowBtn.isBordered = true
            overflowBtn.font = .systemFont(ofSize: 11.0, weight: .semibold)
            overflowBtn.toolTip = "Show all \(tabs.count) open tabs"
            stackView.addArrangedSubview(overflowBtn)
        }

        let plusBtn = NSButton(title: "", target: self, action: #selector(newTabClicked(_:)))
        plusBtn.bezelStyle = .recessed
        plusBtn.isBordered = false
        if let img = NSImage(systemSymbolName: "plus", accessibilityDescription: "Open PDF in New Tab") {
            plusBtn.image = img
        }
        plusBtn.toolTip = "Open PDF in New Tab (⌘O)"
        stackView.addArrangedSubview(plusBtn)
    }

    private func buildTabPill(index: Int, title: String, isEdited: Bool, isSelected: Bool) -> NSView {
        let pill = TabPillContainerView(tabIndex: index, isSelected: isSelected)
        pill.onLeftClick = { [weak self] idx in
            self?.onSelectTab?(idx)
        }
        pill.onRightClick = { [weak self] idx, event, view in
            self?.presentContextMenu(forTab: idx, with: event, in: view)
        }
        pill.translatesAutoresizingMaskIntoConstraints = false

        let docIcon = NSImageView()
        docIcon.translatesAutoresizingMaskIntoConstraints = false
        if let dImg = NSImage(systemSymbolName: isEdited ? "doc.badge.gearshape.fill" : "doc.richtext.fill", accessibilityDescription: nil) {
            dImg.isTemplate = true
            docIcon.image = dImg
            docIcon.contentTintColor = isSelected ? IridescenceDesign.primaryViolet : .tertiaryLabelColor
        }

        let displayTitle = isEdited ? "● \(title)" : title
        let titleBtn = NSButton(title: displayTitle, target: self, action: #selector(tabPillClicked(_:)))
        titleBtn.tag = index
        titleBtn.isBordered = false
        titleBtn.font = .systemFont(ofSize: 11.5, weight: isSelected ? .semibold : .regular)
        titleBtn.contentTintColor = isSelected ? .labelColor : .secondaryLabelColor
        titleBtn.lineBreakMode = .byTruncatingTail
        titleBtn.translatesAutoresizingMaskIntoConstraints = false

        let closeBtn = NSButton(title: "", target: self, action: #selector(closePillClicked(_:)))
        closeBtn.tag = index
        closeBtn.isBordered = false
        if let xImg = NSImage(systemSymbolName: "xmark", accessibilityDescription: "Close Tab") {
            let cfg = NSImage.SymbolConfiguration(pointSize: 9.0, weight: .bold)
            closeBtn.image = xImg.withSymbolConfiguration(cfg) ?? xImg
        }
        closeBtn.contentTintColor = isSelected ? IridescenceDesign.primaryViolet : .secondaryLabelColor
        closeBtn.toolTip = "Close Tab (⌘W)"
        closeBtn.translatesAutoresizingMaskIntoConstraints = false

        pill.addSubview(docIcon)
        pill.addSubview(titleBtn)
        pill.addSubview(closeBtn)

        NSLayoutConstraint.activate([
            pill.heightAnchor.constraint(equalToConstant: 26.0),
            pill.widthAnchor.constraint(greaterThanOrEqualToConstant: 108.0),
            pill.widthAnchor.constraint(lessThanOrEqualToConstant: 215.0),

            docIcon.leadingAnchor.constraint(equalTo: pill.leadingAnchor, constant: 8.0),
            docIcon.centerYAnchor.constraint(equalTo: pill.centerYAnchor),
            docIcon.widthAnchor.constraint(equalToConstant: 12.0),
            docIcon.heightAnchor.constraint(equalToConstant: 12.0),

            titleBtn.leadingAnchor.constraint(equalTo: docIcon.trailingAnchor, constant: 5.0),
            titleBtn.centerYAnchor.constraint(equalTo: pill.centerYAnchor),
            titleBtn.trailingAnchor.constraint(equalTo: closeBtn.leadingAnchor, constant: -4.0),

            closeBtn.trailingAnchor.constraint(equalTo: pill.trailingAnchor, constant: -6.0),
            closeBtn.centerYAnchor.constraint(equalTo: pill.centerYAnchor),
            closeBtn.widthAnchor.constraint(equalToConstant: 14.0),
            closeBtn.heightAnchor.constraint(equalToConstant: 14.0)
        ])

        return pill
    }

    @objc private func tabPillClicked(_ sender: NSButton) {
        onSelectTab?(sender.tag)
    }

    @objc private func closePillClicked(_ sender: NSButton) {
        onCloseTab?(sender.tag)
    }

    @objc private func newTabClicked(_ sender: Any?) {
        onNewTab?()
    }

    @objc private func overflowClicked(_ sender: NSButton) {
        let menu = NSMenu(title: "All Open Tabs")
        for (idx, info) in currentTabsSnapshot.enumerated() {
            let label = info.isEdited ? "● \(info.title)" : info.title
            let item = NSMenuItem(title: label, action: #selector(overflowMenuSelect(_:)), keyEquivalent: "")
            item.target = self
            item.tag = idx
            item.state = (idx == currentActiveIndex) ? .on : .off
            menu.addItem(item)
        }
        let point = NSPoint(x: 0, y: sender.bounds.height + 4)
        menu.popUp(positioning: nil, at: point, in: sender)
    }

    @objc private func overflowMenuSelect(_ sender: NSMenuItem) {
        onSelectTab?(sender.tag)
    }

    private func presentContextMenu(forTab index: Int, with event: NSEvent, in view: NSView) {
        guard index >= 0, index < currentTabsSnapshot.count else { return }
        let menu = NSMenu(title: "TabActions")

        let closeItem = NSMenuItem(title: "Close Tab", action: #selector(ctxCloseTab(_:)), keyEquivalent: "")
        closeItem.target = self
        closeItem.tag = index
        menu.addItem(closeItem)

        let closeOthers = NSMenuItem(title: "Close Other Tabs", action: #selector(ctxCloseOthers(_:)), keyEquivalent: "")
        closeOthers.target = self
        closeOthers.tag = index
        closeOthers.isEnabled = currentTabsSnapshot.count > 1
        menu.addItem(closeOthers)

        menu.addItem(.separator())

        let copyPath = NSMenuItem(title: "Copy File Path", action: #selector(ctxCopyPath(_:)), keyEquivalent: "")
        copyPath.target = self
        copyPath.tag = index
        menu.addItem(copyPath)

        let reveal = NSMenuItem(title: "Reveal in Finder", action: #selector(ctxRevealInFinder(_:)), keyEquivalent: "")
        reveal.target = self
        reveal.tag = index
        menu.addItem(reveal)

        NSMenu.popUpContextMenu(menu, with: event, for: view)
    }

    @objc private func ctxCloseTab(_ sender: NSMenuItem) {
        onCloseTab?(sender.tag)
    }

    @objc private func ctxCloseOthers(_ sender: NSMenuItem) {
        onCloseOtherTabs?(sender.tag)
    }

    @objc private func ctxCopyPath(_ sender: NSMenuItem) {
        let idx = sender.tag
        guard idx >= 0, idx < currentTabsSnapshot.count else { return }
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(currentTabsSnapshot[idx].path, forType: .string)
    }

    @objc private func ctxRevealInFinder(_ sender: NSMenuItem) {
        let idx = sender.tag
        guard idx >= 0, idx < currentTabsSnapshot.count else { return }
        let url = URL(fileURLWithPath: currentTabsSnapshot[idx].path)
        NSWorkspace.shared.activateFileViewerSelecting([url])
    }
}

private final class TabPillContainerView: NSView {
    let tabIndex: Int
    let isSelected: Bool
    var onLeftClick: ((Int) -> Void)?
    var onRightClick: ((Int, NSEvent, NSView) -> Void)?

    init(tabIndex: Int, isSelected: Bool) {
        self.tabIndex = tabIndex
        self.isSelected = isSelected
        super.init(frame: .zero)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func draw(_ dirtyRect: NSRect) {
        let b = bounds.insetBy(dx: 0.5, dy: 0.5)
        guard b.width > 4, b.height > 4 else { return }
        let path = NSBezierPath(roundedRect: b, xRadius: UIConstants.radiusRow, yRadius: UIConstants.radiusRow)
        let isDark = effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua

        if isSelected {
            let c1 = IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.28 : 0.16)
            let c2 = IridescenceDesign.secondaryIndigo.withAlphaComponent(isDark ? 0.22 : 0.12)
            let c3 = IridescenceDesign.accentCyan.withAlphaComponent(isDark ? 0.20 : 0.11)
            if let grad = NSGradient(colors: [c1, c2, c3], atLocations: [0.0, 0.55, 1.0], colorSpace: .sRGB) {
                grad.draw(in: path, angle: 15.0)
            } else {
                c1.setFill()
                path.fill()
            }
            IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.55 : 0.40).setStroke()
            path.lineWidth = 1.0
            path.stroke()
        } else {
            NSColor.labelColor.withAlphaComponent(isDark ? 0.05 : 0.035).setFill()
            path.fill()
            NSColor.separatorColor.withAlphaComponent(0.38).setStroke()
            path.lineWidth = 0.5
            path.stroke()
        }
    }

    override func mouseDown(with event: NSEvent) {
        onLeftClick?(tabIndex)
    }

    override func rightMouseDown(with event: NSEvent) {
        onRightClick?(tabIndex, event, self)
    }
}
