import AppKit
import Foundation

/// Native 5-tab macOS Settings window (`General`, `Reading`, `Workspace`, `Appearance`, `Advanced`)
/// with vertical icon-above-label tab pills, live `SettingsManager` synchronization, and `Restore Defaults`.
@MainActor
public final class SettingsWindowController: NSObject {
    public static let shared = SettingsWindowController()

    public private(set) var window: NSWindow?
    public private(set) var activeTabIndex: Int = 0
    public var onSettingsChanged: (() -> Void)?

    private var tabButtons: [NSButton] = []
    private var tabPanes: [NSView] = []

    // Controls kept for refreshing on Restore Defaults
    private var chkRestoreSession: NSButton?
    private var chkRememberRecents: NSButton?
    private var chkReopenClosed: NSButton?
    private var chkOpenNewTab: NSButton?

    private var segDisplayMode: NSSegmentedControl?
    private var segZoomMode: NSSegmentedControl?
    private var segHighlightColor: NSSegmentedControl?
    private var chkRestorePos: NSButton?

    private var chkAutoReveal: NSButton?
    private var chkShowHidden: NSButton?
    private var chkDeepSearch: NSButton?

    private var segAppearance: NSSegmentedControl?
    private var chkLiquidGlass: NSButton?

    private var chkAutoCopyOcr: NSButton?

    public override init() {
        super.init()
    }

    public func showWindow(tabIndex: Int? = nil) {
        if window == nil {
            buildWindow()
        }
        if let idx = tabIndex {
            selectTab(index: idx)
        }
        syncControlsFromSettings()
        if let win = window {
            WindowGeometry.sanitizeWindow(win, kind: .utility)
            win.makeKeyAndOrderFront(nil)
        }
    }

    private func buildWindow() {
        let rect = NSRect(x: 0, y: 0, width: 540, height: 420)
        let win = NSWindow(
            contentRect: rect,
            styleMask: [.titled, .closable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        win.title = "Iridescence Settings"
        win.titlebarAppearsTransparent = true
        win.isReleasedWhenClosed = false
        win.isOpaque = false
        win.backgroundColor = .clear
        win.center()

        let rootEffect = NSVisualEffectView(frame: rect)
        rootEffect.material = .underWindowBackground
        rootEffect.blendingMode = .behindWindow
        rootEffect.state = .followsWindowActiveState
        rootEffect.autoresizingMask = [.width, .height]

        // Top tab bar with vertical icon-above-label pills
        let tabBar = NSStackView()
        tabBar.orientation = .horizontal
        tabBar.distribution = .fillEqually
        tabBar.spacing = 8.0
        tabBar.translatesAutoresizingMaskIntoConstraints = false

        let tabDefs: [(title: String, symbol: String)] = [
            ("General", "gearshape"),
            ("Reading", "book"),
            ("Workspace", "folder"),
            ("Appearance", "paintbrush"),
            ("Advanced", "slider.horizontal.3")
        ]

        tabButtons.removeAll()
        for (idx, def) in tabDefs.enumerated() {
            let btn = NSButton(title: def.title, target: self, action: #selector(tabButtonClicked(_:)))
            btn.tag = idx
            btn.bezelStyle = .recessed
            btn.isBordered = false
            btn.imagePosition = .imageAbove
            btn.font = .systemFont(ofSize: 11.0, weight: .medium)
            if let img = NSImage(systemSymbolName: def.symbol, accessibilityDescription: def.title) {
                let cfg = NSImage.SymbolConfiguration(pointSize: 16.0, weight: .medium)
                btn.image = img.withSymbolConfiguration(cfg) ?? img
            }
            btn.wantsLayer = true
            btn.layer?.cornerRadius = 8.0
            btn.translatesAutoresizingMaskIntoConstraints = false
            NSLayoutConstraint.activate([
                btn.heightAnchor.constraint(equalToConstant: 48.0),
                btn.widthAnchor.constraint(greaterThanOrEqualToConstant: 84.0)
            ])
            tabButtons.append(btn)
            tabBar.addArrangedSubview(btn)
        }

        let divider = NSView()
        divider.wantsLayer = true
        divider.layer?.backgroundColor = NSColor.separatorColor.cgColor
        divider.translatesAutoresizingMaskIntoConstraints = false

        let contentHost = NSView()
        contentHost.translatesAutoresizingMaskIntoConstraints = false

        let footerDivider = NSView()
        footerDivider.wantsLayer = true
        footerDivider.layer?.backgroundColor = NSColor.separatorColor.withAlphaComponent(0.6).cgColor
        footerDivider.translatesAutoresizingMaskIntoConstraints = false

        let restoreBtn = NSButton(title: "Restore Defaults", target: self, action: #selector(restoreDefaultsClicked(_:)))
        restoreBtn.bezelStyle = .rounded
        restoreBtn.controlSize = .small
        restoreBtn.translatesAutoresizingMaskIntoConstraints = false

        tabPanes = [
            buildGeneralPane(),
            buildReadingPane(),
            buildWorkspacePane(),
            buildAppearancePane(),
            buildAdvancedPane()
        ]

        for pane in tabPanes {
            pane.translatesAutoresizingMaskIntoConstraints = false
            contentHost.addSubview(pane)
            NSLayoutConstraint.activate([
                pane.topAnchor.constraint(equalTo: contentHost.topAnchor, constant: 18.0),
                pane.leadingAnchor.constraint(equalTo: contentHost.leadingAnchor, constant: 32.0),
                pane.trailingAnchor.constraint(equalTo: contentHost.trailingAnchor, constant: -32.0),
                pane.bottomAnchor.constraint(lessThanOrEqualTo: contentHost.bottomAnchor, constant: -12.0)
            ])
        }

        rootEffect.addSubview(tabBar)
        rootEffect.addSubview(divider)
        rootEffect.addSubview(contentHost)
        rootEffect.addSubview(footerDivider)
        rootEffect.addSubview(restoreBtn)

        NSLayoutConstraint.activate([
            tabBar.topAnchor.constraint(equalTo: rootEffect.topAnchor, constant: 36.0),
            tabBar.centerXAnchor.constraint(equalTo: rootEffect.centerXAnchor),

            divider.topAnchor.constraint(equalTo: tabBar.bottomAnchor, constant: 10.0),
            divider.leadingAnchor.constraint(equalTo: rootEffect.leadingAnchor),
            divider.trailingAnchor.constraint(equalTo: rootEffect.trailingAnchor),
            divider.heightAnchor.constraint(equalToConstant: 1.0),

            contentHost.topAnchor.constraint(equalTo: divider.bottomAnchor),
            contentHost.leadingAnchor.constraint(equalTo: rootEffect.leadingAnchor),
            contentHost.trailingAnchor.constraint(equalTo: rootEffect.trailingAnchor),
            contentHost.bottomAnchor.constraint(equalTo: footerDivider.topAnchor),

            footerDivider.leadingAnchor.constraint(equalTo: rootEffect.leadingAnchor),
            footerDivider.trailingAnchor.constraint(equalTo: rootEffect.trailingAnchor),
            footerDivider.bottomAnchor.constraint(equalTo: rootEffect.bottomAnchor, constant: -44.0),
            footerDivider.heightAnchor.constraint(equalToConstant: 1.0),

            restoreBtn.trailingAnchor.constraint(equalTo: rootEffect.trailingAnchor, constant: -20.0),
            restoreBtn.bottomAnchor.constraint(equalTo: rootEffect.bottomAnchor, constant: -12.0)
        ])

        win.contentView = rootEffect
        self.window = win
        selectTab(index: 0)
    }

    public func selectTab(index: Int) {
        let clamped = min(max(index, 0), max(0, tabPanes.count - 1))
        activeTabIndex = clamped
        for (i, pane) in tabPanes.enumerated() {
            pane.isHidden = (i != clamped)
        }
        for (i, btn) in tabButtons.enumerated() {
            let selected = (i == clamped)
            btn.contentTintColor = selected ? .controlAccentColor : .secondaryLabelColor
            btn.layer?.backgroundColor = selected
                ? NSColor.controlAccentColor.withAlphaComponent(0.16).cgColor
                : NSColor.clear.cgColor
        }
    }

    @objc private func tabButtonClicked(_ sender: NSButton) {
        selectTab(index: sender.tag)
    }

    // MARK: - Tab Pane Builders

    private func makeSectionHeader(_ title: String) -> NSTextField {
        let lbl = NSTextField(labelWithString: title)
        lbl.font = .systemFont(ofSize: 13.0, weight: .semibold)
        lbl.textColor = .labelColor
        return lbl
    }

    private func makeDescriptionLabel(_ text: String) -> NSTextField {
        let lbl = NSTextField(wrappingLabelWithString: text)
        lbl.font = .systemFont(ofSize: 11.5, weight: .regular)
        lbl.textColor = .secondaryLabelColor
        lbl.preferredMaxLayoutWidth = 440.0
        return lbl
    }

    private func buildGeneralPane() -> NSView {
        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .leading
        stack.spacing = 12.0

        stack.addArrangedSubview(makeSectionHeader("Startup & Documents"))

        let c1 = NSButton(checkboxWithTitle: "Restore previous session on launch", target: self, action: #selector(settingChanged(_:)))
        let c2 = NSButton(checkboxWithTitle: "Remember recent files and workspaces", target: self, action: #selector(settingChanged(_:)))
        let c3 = NSButton(checkboxWithTitle: "Allow reopening recently closed tabs (⇧⌘T)", target: self, action: #selector(settingChanged(_:)))
        let c4 = NSButton(checkboxWithTitle: "Open documents in a new tab by default", target: self, action: #selector(settingChanged(_:)))

        chkRestoreSession = c1
        chkRememberRecents = c2
        chkReopenClosed = c3
        chkOpenNewTab = c4

        stack.addArrangedSubview(c1)
        stack.addArrangedSubview(c2)
        stack.addArrangedSubview(c3)
        stack.addArrangedSubview(c4)
        stack.addArrangedSubview(makeDescriptionLabel("Iridescence preserves your reading position, active tabs, and workspace hierarchy across launches."))
        return stack
    }

    private func buildReadingPane() -> NSView {
        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .leading
        stack.spacing = 12.0

        stack.addArrangedSubview(makeSectionHeader("Default PDF Layout & Markup"))

        let dmRow = NSStackView()
        dmRow.orientation = .horizontal
        dmRow.spacing = 12.0
        dmRow.addArrangedSubview(NSTextField(labelWithString: "Default Display Mode:"))
        let segDM = NSSegmentedControl(labels: ["Single Page", "Continuous", "Two-Up"], trackingMode: .selectOne, target: self, action: #selector(settingChanged(_:)))
        segDisplayMode = segDM
        dmRow.addArrangedSubview(segDM)
        stack.addArrangedSubview(dmRow)

        let zmRow = NSStackView()
        zmRow.orientation = .horizontal
        zmRow.spacing = 12.0
        zmRow.addArrangedSubview(NSTextField(labelWithString: "Default Zoom:"))
        let segZM = NSSegmentedControl(labels: ["Fit Width", "Actual Size (100%)"], trackingMode: .selectOne, target: self, action: #selector(settingChanged(_:)))
        segZoomMode = segZM
        zmRow.addArrangedSubview(segZM)
        stack.addArrangedSubview(zmRow)

        let hlRow = NSStackView()
        hlRow.orientation = .horizontal
        hlRow.spacing = 12.0
        hlRow.addArrangedSubview(NSTextField(labelWithString: "Highlight Color:"))
        let segHL = NSSegmentedControl(labels: ["Yellow", "Green", "Blue", "Pink"], trackingMode: .selectOne, target: self, action: #selector(settingChanged(_:)))
        segHighlightColor = segHL
        hlRow.addArrangedSubview(segHL)
        stack.addArrangedSubview(hlRow)

        let cPos = NSButton(checkboxWithTitle: "Restore last reading page and zoom factor per document", target: self, action: #selector(settingChanged(_:)))
        chkRestorePos = cPos
        stack.addArrangedSubview(cPos)

        return stack
    }

    private func buildWorkspacePane() -> NSView {
        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .leading
        stack.spacing = 12.0

        stack.addArrangedSubview(makeSectionHeader("Workspace Explorer & Indexing"))

        let c1 = NSButton(checkboxWithTitle: "Automatically reveal active PDF in Workspace sidebar", target: self, action: #selector(settingChanged(_:)))
        let c2 = NSButton(checkboxWithTitle: "Show hidden files and dot-directories in Explorer", target: self, action: #selector(settingChanged(_:)))
        let c3 = NSButton(checkboxWithTitle: "Enable background full-text workspace indexing (IRIDX1)", target: self, action: #selector(settingChanged(_:)))

        chkAutoReveal = c1
        chkShowHidden = c2
        chkDeepSearch = c3

        stack.addArrangedSubview(c1)
        stack.addArrangedSubview(c2)
        stack.addArrangedSubview(c3)
        stack.addArrangedSubview(makeDescriptionLabel("Full-text indexing powers instant // searches across all PDFs in your mounted workspace."))
        return stack
    }

    private func buildAppearancePane() -> NSView {
        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .leading
        stack.spacing = 12.0

        stack.addArrangedSubview(makeSectionHeader("Theme & Liquid Glass Materials"))

        let appRow = NSStackView()
        appRow.orientation = .horizontal
        appRow.spacing = 12.0
        appRow.addArrangedSubview(NSTextField(labelWithString: "Appearance:"))
        let segApp = NSSegmentedControl(labels: ["System", "Light", "Dark"], trackingMode: .selectOne, target: self, action: #selector(settingChanged(_:)))
        segAppearance = segApp
        appRow.addArrangedSubview(segApp)
        stack.addArrangedSubview(appRow)

        let cGlass = NSButton(checkboxWithTitle: "Enable native Liquid Glass translucency in sidebars and panels", target: self, action: #selector(settingChanged(_:)))
        chkLiquidGlass = cGlass
        stack.addArrangedSubview(cGlass)
        stack.addArrangedSubview(makeDescriptionLabel("Automatically respects macOS Reduce Transparency and Increase Contrast accessibility settings."))
        return stack
    }

    private func buildAdvancedPane() -> NSView {
        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .leading
        stack.spacing = 12.0

        stack.addArrangedSubview(makeSectionHeader("Vision OCR & Diagnostics"))

        let cOcr = NSButton(checkboxWithTitle: "Automatically copy extracted OCR text to clipboard", target: self, action: #selector(settingChanged(_:)))
        chkAutoCopyOcr = cOcr
        stack.addArrangedSubview(cOcr)

        let clearCacheBtn = NSButton(title: "Clear Workspace Search Index Cache", target: self, action: #selector(clearIndexCacheClicked(_:)))
        clearCacheBtn.bezelStyle = .rounded
        stack.addArrangedSubview(clearCacheBtn)
        stack.addArrangedSubview(makeDescriptionLabel("Safety Invariant: /Documents/MASTER read-only protection is permanently active."))
        return stack
    }

    // MARK: - Sync & Callbacks

    public func syncControlsFromSettings() {
        let s = SettingsManager.shared
        chkRestoreSession?.state = s.restoreSession ? .on : .off
        chkRememberRecents?.state = s.rememberRecents ? .on : .off
        chkReopenClosed?.state = s.reopenClosedTabs ? .on : .off
        chkOpenNewTab?.state = s.openInNewTab ? .on : .off

        segDisplayMode?.selectedSegment = s.defaultDisplayMode
        segZoomMode?.selectedSegment = s.defaultZoomMode
        segHighlightColor?.selectedSegment = s.defaultHighlightColorIndex
        chkRestorePos?.state = s.restoreReadingPos ? .on : .off

        chkAutoReveal?.state = s.autoRevealSidebar ? .on : .off
        chkShowHidden?.state = s.showHiddenFiles ? .on : .off
        chkDeepSearch?.state = s.deepWorkspaceSearch ? .on : .off

        segAppearance?.selectedSegment = s.appearanceMode
        chkLiquidGlass?.state = s.liquidGlass ? .on : .off
        chkAutoCopyOcr?.state = s.autoCopyOcr ? .on : .off
    }

    @objc private func settingChanged(_ sender: Any?) {
        let s = SettingsManager.shared
        if let c = chkRestoreSession { s.restoreSession = (c.state == .on) }
        if let c = chkRememberRecents { s.rememberRecents = (c.state == .on) }
        if let c = chkReopenClosed { s.reopenClosedTabs = (c.state == .on) }
        if let c = chkOpenNewTab { s.openInNewTab = (c.state == .on) }

        if let seg = segDisplayMode, seg.selectedSegment >= 0 { s.defaultDisplayMode = seg.selectedSegment }
        if let seg = segZoomMode, seg.selectedSegment >= 0 { s.defaultZoomMode = seg.selectedSegment }
        if let seg = segHighlightColor, seg.selectedSegment >= 0 { s.defaultHighlightColorIndex = seg.selectedSegment }
        if let c = chkRestorePos { s.restoreReadingPos = (c.state == .on) }

        if let c = chkAutoReveal { s.autoRevealSidebar = (c.state == .on) }
        if let c = chkShowHidden { s.showHiddenFiles = (c.state == .on) }
        if let c = chkDeepSearch { s.deepWorkspaceSearch = (c.state == .on) }

        if let seg = segAppearance, seg.selectedSegment >= 0 { s.appearanceMode = seg.selectedSegment }
        if let c = chkLiquidGlass { s.liquidGlass = (c.state == .on) }
        if let c = chkAutoCopyOcr { s.autoCopyOcr = (c.state == .on) }

        s.applySavedAppearance()
        onSettingsChanged?()
    }

    @objc private func restoreDefaultsClicked(_ sender: Any?) {
        SettingsManager.shared.restoreAllDefaults()
        SettingsManager.shared.applySavedAppearance()
        syncControlsFromSettings()
        onSettingsChanged?()
    }

    @objc private func clearIndexCacheClicked(_ sender: Any?) {
        WorkspaceIndex.shared.clear()
        try? FileManager.default.removeItem(at: WorkspaceIndex.defaultCacheURL())
        AppLogger.shared.log(.workspace, "Cleared workspace search index cache")
    }
}
