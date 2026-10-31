import AppKit
import Foundation
import PDFKit
import UniformTypeIdentifiers

/// Main application window supporting physical keycode `Cmd+1..9` tab switching (QWERTY + French AZERTY)
/// and Finder file/folder drag-and-drop.
@MainActor
public final class MainWindow: NSWindow {
    public var onSelectTabNumber: ((Int) -> Void)?
    public var onDropFileURLs: (([URL]) -> Void)?

    public override init(
        contentRect: NSRect,
        styleMask style: NSWindow.StyleMask,
        backing backingStoreType: NSWindow.BackingStoreType,
        defer flag: Bool
    ) {
        super.init(contentRect: contentRect, styleMask: style, backing: backingStoreType, defer: flag)
        registerForDraggedTypes([.fileURL])
    }

    public override func performKeyEquivalent(with event: NSEvent) -> Bool {
        let flags = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        if flags == .command {
            if let tabNum = TabManager.tabNumberForKeyCode(event.keyCode) {
                onSelectTabNumber?(tabNum)
                return true
            }
        }
        return super.performKeyEquivalent(with: event)
    }

    @objc public func draggingEntered(_ sender: NSDraggingInfo) -> NSDragOperation {
        .copy
    }

    @objc public func performDragOperation(_ sender: NSDraggingInfo) -> Bool {
        let pboard = sender.draggingPasteboard
        guard let urls = pboard.readObjects(forClasses: [NSURL.self], options: [.urlReadingFileURLsOnly: true]) as? [URL],
              !urls.isEmpty else {
            return false
        }
        onDropFileURLs?(urls)
        return true
    }
}

/// Primary `@MainActor` coordinator for Iridescence PDF Reader:
/// manages application lifecycle, main window, unified toolbar, 3-pane split layout,
/// 5-segment right inspector, multi-document tabs, annotations, OCR, and persistence.
@MainActor
public final class AppDelegate: NSObject, NSApplicationDelegate, NSToolbarDelegate, NSWindowDelegate {
    public private(set) var window: MainWindow!
    public private(set) var splitViewController: NSSplitViewController!
    public private(set) var explorerSplitItem: NSSplitViewItem!
    public private(set) var readerSplitItem: NSSplitViewItem!
    public private(set) var inspectorSplitItem: NSSplitViewItem!

    public private(set) var homeScreenController: HomeScreenController!
    public private(set) var workspaceController: WorkspaceOutlineController!
    public private(set) var pdfOutlineController: PDFOutlineController!
    public private(set) var pdfSearchController: PDFSearchController!
    public private(set) var bookmarksInspectorController: BookmarksInspectorController!
    public private(set) var commandPaletteController: CommandPaletteController!

    public private(set) var pdfView: PDFView!
    public private(set) var pdfPageThumbnailController: PDFPageThumbnailController!
    public var thumbnailView: NSView { pdfPageThumbnailController.scrollView }
    public private(set) var tabBarView: TabBarView!
    public private(set) var tabBarHeightConstraint: NSLayoutConstraint!
    public private(set) var rightSegmentedControl: NSSegmentedControl!
    public private(set) var rightSidebarModeLabel: NSTextField!
    public private(set) var zoomPercentButton: NSButton?

    public let tabManager = TabManager()
    public private(set) var contentMode: AppContentMode = .home
    public private(set) var isDocumentEdited: Bool = false
    public private(set) var isFocusModeActive: Bool = false
    public private(set) var isPresentationModeActive: Bool = false

    private var savedLeftCollapsedBeforeFocus = false
    private var savedRightCollapsedBeforeFocus = false
    private var backHistory: [Int] = []
    private var forwardHistory: [Int] = []
    private var isNavigatingHistory = false
    private var pendingOpenURLs: [URL] = []

    // Toolbar Item Identifiers
    private static let tbSidebarToggle = NSToolbarItem.Identifier("SidebarToggle")
    private static let tbHistoryNav = NSToolbarItem.Identifier("HistoryNav")
    private static let tbZoomGroup = NSToolbarItem.Identifier("ZoomGroup")
    private static let tbQuickOpen = NSToolbarItem.Identifier("QuickOpen")
    private static let tbInspectorToggle = NSToolbarItem.Identifier("InspectorToggle")

    public override init() {
        super.init()
    }

    // MARK: - NSApplicationDelegate Lifecycle

    public func applicationDidFinishLaunching(_ notification: Notification) {
        AppLogger.shared.log(.ui, "Iridescence PDF Reader (Native Swift) launching")
        SettingsManager.shared.registerDefaults()
        SettingsManager.shared.applySavedAppearance()
        WorkspaceIndex.shared.loadFromDisk()

        buildMainMenu()
        buildMainWindowAndViews()
        configureCallbacks()

        // Handle pending URLs or CLI arguments
        if !pendingOpenURLs.isEmpty {
            let urls = pendingOpenURLs
            pendingOpenURLs.removeAll()
            handleOpenURLs(urls)
        } else {
            let args = CommandLine.arguments
            if args.count > 1 {
                let firstArg = args[1]
                if !firstArg.hasPrefix("-") && FileManager.default.fileExists(atPath: firstArg) {
                    handleOpenURLs([URL(fileURLWithPath: firstArg)])
                }
            }
        }

        // Trigger automated Visual QA suites when environment variables are present
        let env = ProcessInfo.processInfo.environment
        if env["IRIDESCENCE_SWIFT_VISUAL_QA"] != nil {
            VisualQAHarness.runComplete22StateSwiftVisualQA(app: self)
        } else if env["IRIDESCENCE_QA_FIX_SUITE"] != nil {
            VisualQAHarness.runComplete22StateSwiftVisualQA(app: self)
        } else if env["IRIDESCENCE_VISUAL_QA"] != nil {
            VisualQAHarness.runCanonical19VisualQA(app: self)
        } else if env["IRIDESCENCE_VISUAL_QA_ANNOTATIONS"] != nil {
            VisualQAHarness.runAnnotationsVisualQA(app: self)
        } else if env["IRIDESCENCE_VISUAL_QA_TABS"] != nil {
            VisualQAHarness.runTabsVisualQA(app: self)
        } else if env["IRIDESCENCE_VISUAL_QA_OCR"] != nil {
            VisualQAHarness.runOCRVisualQA(app: self)
        }
    }

    public func application(_ application: NSApplication, open urls: [URL]) {
        if window == nil {
            pendingOpenURLs.append(contentsOf: urls)
            return
        }
        handleOpenURLs(urls)
    }

    public func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        false
    }

    public func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if !flag {
            window?.makeKeyAndOrderFront(nil)
        }
        return true
    }

    public func applicationDidBecomeActive(_ notification: Notification) {
        checkActiveDocumentExternalModification()
    }

    // MARK: - Window & 3-Pane Split Hierarchy Construction

    public func buildMainWindowAndViews() {
        let initialFrame = WindowGeometry.initialFrame(for: .home)
        window = MainWindow(
            contentRect: initialFrame,
            styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        window.title = "Iridescence PDF Reader"
        window.titlebarAppearsTransparent = true
        window.toolbarStyle = .unified
        window.isOpaque = false
        window.backgroundColor = .clear
        window.hasShadow = true
        window.isReleasedWhenClosed = false
        window.delegate = self

        let toolbar = NSToolbar(identifier: "IridescenceMainToolbar")
        toolbar.delegate = self
        toolbar.displayMode = .iconOnly
        toolbar.allowsUserCustomization = false
        window.toolbar = toolbar

        // 1. Left Workspace Explorer
        workspaceController = WorkspaceOutlineController()
        let leftVC = NSViewController()
        leftVC.view = workspaceController.containerView
        explorerSplitItem = NSSplitViewItem(sidebarWithViewController: leftVC)
        explorerSplitItem.canCollapse = true
        explorerSplitItem.minimumThickness = UIConstants.sidebarWidthMin
        explorerSplitItem.maximumThickness = UIConstants.sidebarWidthMax
        explorerSplitItem.holdingPriority = NSLayoutConstraint.Priority(251)

        // 2. Center PDF Reader + Liquid Glass Tab Bar
        let centerEffect = NSVisualEffectView(frame: NSRect(x: 0, y: 0, width: 780, height: 740))
        centerEffect.material = .underWindowBackground
        centerEffect.blendingMode = .behindWindow
        centerEffect.state = .followsWindowActiveState
        centerEffect.autoresizingMask = [.width, .height]

        tabBarView = TabBarView(frame: NSRect(x: 0, y: 0, width: 780, height: TabBarView.barHeight))
        tabBarView.isHidden = true

        pdfView = PDFView(frame: NSRect(x: 0, y: 0, width: 780, height: 700))
        pdfView.autoScales = true
        pdfView.displayMode = .singlePageContinuous
        pdfView.displaysPageBreaks = true
        pdfView.backgroundColor = NSColor.windowBackgroundColor.withAlphaComponent(0.35)
        pdfView.translatesAutoresizingMaskIntoConstraints = false

        centerEffect.addSubview(tabBarView)
        centerEffect.addSubview(pdfView)

        tabBarHeightConstraint = tabBarView.heightAnchor.constraint(equalToConstant: 0.0)
        NSLayoutConstraint.activate([
            tabBarView.topAnchor.constraint(equalTo: centerEffect.topAnchor, constant: 52.0),
            tabBarView.leadingAnchor.constraint(equalTo: centerEffect.leadingAnchor),
            tabBarView.trailingAnchor.constraint(equalTo: centerEffect.trailingAnchor),
            tabBarHeightConstraint,

            pdfView.topAnchor.constraint(equalTo: tabBarView.bottomAnchor),
            pdfView.leadingAnchor.constraint(equalTo: centerEffect.leadingAnchor),
            pdfView.trailingAnchor.constraint(equalTo: centerEffect.trailingAnchor),
            pdfView.bottomAnchor.constraint(equalTo: centerEffect.bottomAnchor)
        ])

        let centerVC = NSViewController()
        centerVC.view = centerEffect
        readerSplitItem = NSSplitViewItem(viewController: centerVC)
        readerSplitItem.minimumThickness = UIConstants.readerCenterMinWidth
        readerSplitItem.holdingPriority = NSLayoutConstraint.Priority(200)

        // 3. Right 5-Segment Inspector (Pages | Outline | Search | Bookmarks | Annotations)
        pdfPageThumbnailController = PDFPageThumbnailController(pdfView: pdfView)

        pdfOutlineController = PDFOutlineController(pdfView: pdfView)
        pdfSearchController = PDFSearchController(pdfView: pdfView)
        bookmarksInspectorController = BookmarksInspectorController(pdfView: pdfView)

        let rightContainer = NSView(frame: NSRect(x: 0, y: 0, width: 230, height: 700))
        rightContainer.translatesAutoresizingMaskIntoConstraints = false

        rightSegmentedControl = NSSegmentedControl()
        rightSegmentedControl.segmentCount = 5
        rightSegmentedControl.trackingMode = .selectOne
        rightSegmentedControl.target = self
        rightSegmentedControl.action = #selector(switchSidebar(_:))
        rightSegmentedControl.translatesAutoresizingMaskIntoConstraints = false

        let segDefs: [(symbol: String, tooltip: String)] = [
            ("doc.richtext", "Pages"),
            ("list.bullet.indent", "Outline"),
            ("magnifyingglass", "Search"),
            ("bookmark", "Bookmarks"),
            ("highlighter", "Annotations")
        ]
        let symConfig = NSImage.SymbolConfiguration(pointSize: 11.5, weight: .regular, scale: .medium)
        for (i, def) in segDefs.enumerated() {
            if let img = NSImage(systemSymbolName: def.symbol, accessibilityDescription: def.tooltip) {
                rightSegmentedControl.setImage(img.withSymbolConfiguration(symConfig) ?? img, forSegment: i)
            }
            rightSegmentedControl.setToolTip(def.tooltip, forSegment: i)
        }

        rightSidebarModeLabel = NSTextField(labelWithString: "PAGES")
        rightSidebarModeLabel.font = .systemFont(ofSize: 10.0, weight: .semibold)
        rightSidebarModeLabel.textColor = .secondaryLabelColor
        rightSidebarModeLabel.translatesAutoresizingMaskIntoConstraints = false

        let segHeaderWrapper = NSView(frame: NSRect(x: 0, y: 0, width: 230, height: 58))
        segHeaderWrapper.translatesAutoresizingMaskIntoConstraints = false
        segHeaderWrapper.addSubview(rightSegmentedControl)
        segHeaderWrapper.addSubview(rightSidebarModeLabel)

        let rightSeparator = NSView()
        rightSeparator.wantsLayer = true
        rightSeparator.layer?.backgroundColor = NSColor.separatorColor.cgColor
        rightSeparator.translatesAutoresizingMaskIntoConstraints = false

        let contentWrapper = NSView()
        contentWrapper.translatesAutoresizingMaskIntoConstraints = false

        let viewsToStack: [NSView] = [
            pdfPageThumbnailController.scrollView,
            pdfOutlineController.scrollView,
            pdfOutlineController.emptyStateView,
            pdfSearchController.containerView,
            bookmarksInspectorController.scrollView,
            bookmarksInspectorController.emptyStateView
        ]
        for v in viewsToStack {
            v.translatesAutoresizingMaskIntoConstraints = false
            contentWrapper.addSubview(v)
            NSLayoutConstraint.activate([
                v.topAnchor.constraint(equalTo: contentWrapper.topAnchor),
                v.leadingAnchor.constraint(equalTo: contentWrapper.leadingAnchor),
                v.trailingAnchor.constraint(equalTo: contentWrapper.trailingAnchor),
                v.bottomAnchor.constraint(equalTo: contentWrapper.bottomAnchor)
            ])
        }

        rightContainer.addSubview(segHeaderWrapper)
        rightContainer.addSubview(rightSeparator)
        rightContainer.addSubview(contentWrapper)

        NSLayoutConstraint.activate([
            rightSegmentedControl.topAnchor.constraint(equalTo: segHeaderWrapper.topAnchor, constant: 8.0),
            rightSegmentedControl.leadingAnchor.constraint(equalTo: segHeaderWrapper.leadingAnchor, constant: 10.0),
            rightSegmentedControl.trailingAnchor.constraint(equalTo: segHeaderWrapper.trailingAnchor, constant: -10.0),

            rightSidebarModeLabel.topAnchor.constraint(equalTo: rightSegmentedControl.bottomAnchor, constant: 5.0),
            rightSidebarModeLabel.leadingAnchor.constraint(equalTo: segHeaderWrapper.leadingAnchor, constant: 14.0),
            rightSidebarModeLabel.trailingAnchor.constraint(equalTo: segHeaderWrapper.trailingAnchor, constant: -14.0),

            segHeaderWrapper.topAnchor.constraint(equalTo: rightContainer.topAnchor),
            segHeaderWrapper.leadingAnchor.constraint(equalTo: rightContainer.leadingAnchor),
            segHeaderWrapper.trailingAnchor.constraint(equalTo: rightContainer.trailingAnchor),
            segHeaderWrapper.heightAnchor.constraint(equalToConstant: 58.0),

            rightSeparator.topAnchor.constraint(equalTo: segHeaderWrapper.bottomAnchor),
            rightSeparator.leadingAnchor.constraint(equalTo: rightContainer.leadingAnchor),
            rightSeparator.trailingAnchor.constraint(equalTo: rightContainer.trailingAnchor),
            rightSeparator.heightAnchor.constraint(equalToConstant: 1.0),

            contentWrapper.topAnchor.constraint(equalTo: rightSeparator.bottomAnchor),
            contentWrapper.leadingAnchor.constraint(equalTo: rightContainer.leadingAnchor),
            contentWrapper.trailingAnchor.constraint(equalTo: rightContainer.trailingAnchor),
            contentWrapper.bottomAnchor.constraint(equalTo: rightContainer.bottomAnchor)
        ])

        let rightEffect = NSVisualEffectView(frame: NSRect(x: 0, y: 0, width: 230, height: 740))
        rightEffect.material = .sidebar
        rightEffect.blendingMode = .behindWindow
        rightEffect.state = .followsWindowActiveState
        rightEffect.autoresizingMask = [.width, .height]

        let inspectorWash = IridescentSurfaceWashView(role: .inspector)
        inspectorWash.frame = rightEffect.bounds
        inspectorWash.autoresizingMask = [.width, .height]
        rightEffect.addSubview(inspectorWash)

        rightEffect.addSubview(rightContainer)

        NSLayoutConstraint.activate([
            rightContainer.topAnchor.constraint(equalTo: rightEffect.topAnchor, constant: 52.0),
            rightContainer.leadingAnchor.constraint(equalTo: rightEffect.leadingAnchor),
            rightContainer.trailingAnchor.constraint(equalTo: rightEffect.trailingAnchor),
            rightContainer.bottomAnchor.constraint(equalTo: rightEffect.bottomAnchor)
        ])

        let rightVC = NSViewController()
        rightVC.view = rightEffect
        inspectorSplitItem = NSSplitViewItem(inspectorWithViewController: rightVC)
        inspectorSplitItem.canCollapse = true
        inspectorSplitItem.minimumThickness = UIConstants.sidebarWidthMin
        inspectorSplitItem.maximumThickness = UIConstants.sidebarWidthMax
        inspectorSplitItem.holdingPriority = NSLayoutConstraint.Priority(250)

        splitViewController = NSSplitViewController()
        splitViewController.addSplitViewItem(explorerSplitItem)
        splitViewController.addSplitViewItem(readerSplitItem)
        splitViewController.addSplitViewItem(inspectorSplitItem)

        // Restore saved right sidebar segment
        let savedRightMode = min(max(UserDefaults.standard.integer(forKey: SettingsKeys.rightSidebarMode), 0), 4)
        switchSidebarToIndex(savedRightMode)

        // 4. Home Screen & Command Palette
        homeScreenController = HomeScreenController()
        commandPaletteController = CommandPaletteController()

        // Default to Home screen on cold launch unless a document is opened
        showHomeScreen(nil)
        window.makeKeyAndOrderFront(nil)
    }

    private func configureCallbacks() {
        window.onSelectTabNumber = { [weak self] tabNum in
            guard let self else { return }
            let count = self.tabManager.tabCount
            guard count > 0 else { return }
            let targetIdx = (tabNum == 9) ? (count - 1) : (tabNum - 1)
            if targetIdx >= 0 && targetIdx < count {
                self.switchToTab(at: targetIdx)
            }
        }

        window.onDropFileURLs = { [weak self] urls in
            self?.handleOpenURLs(urls)
        }

        homeScreenController.onOpenPDF = { [weak self] in
            self?.openDocument(nil)
        }
        homeScreenController.onOpenFolder = { [weak self] in
            self?.openFolder(nil)
        }
        homeScreenController.onSelectFileURL = { [weak self] url in
            self?.openFile(at: url)
        }
        homeScreenController.onSelectWorkspaceURL = { [weak self] url in
            self?.openFolder(at: url)
        }

        workspaceController.onSelectPDF = { [weak self] url in
            self?.openFile(at: url, fromWorkspaceSidebar: true)
        }

        ICloudDownloadManager.shared.onAutoOpenReady = { [weak self] verifiedURL in
            guard let self = self else { return }
            self.openFile(at: verifiedURL, fromWorkspaceSidebar: true)
        }

        ICloudDownloadManager.shared.onBackgroundDownloadReady = { [weak self] _ in
            guard let self = self else { return }
            self.homeScreenController.refreshContent()
        }

        ICloudDownloadManager.shared.onStateChanged = { [weak self] session in
            guard let self = self else { return }
            self.workspaceController.updateNode(for: session)
        }

        bookmarksInspectorController.onAnnotationDeleted = { [weak self] in
            self?.markDocumentEdited(true)
        }

        tabBarView.onSelectTab = { [weak self] idx in
            self?.switchToTab(at: idx)
        }
        tabBarView.onCloseTab = { [weak self] idx in
            self?.closeTab(at: idx)
        }
        tabBarView.onCloseOtherTabs = { [weak self] keepIdx in
            self?.closeOtherTabs(keeping: keepIdx)
        }
        tabBarView.onNewTab = { [weak self] in
            self?.openDocument(nil)
        }

        commandPaletteController.workspaceFilesProvider = { [weak self] in
            guard let root = self?.workspaceController.rootNode?.path else { return [] }
            return WorkspaceNode.collectAllPDFPaths(in: root)
        }
        commandPaletteController.recentFilesProvider = {
            UserDefaults.standard.stringArray(forKey: SettingsKeys.recentFiles) ?? []
        }
        commandPaletteController.outlineEntriesProvider = { [weak self] in
            self?.pdfOutlineController.flattenedEntries() ?? []
        }
        commandPaletteController.currentDocumentBookmarksProvider = { [weak self] in
            guard let activePath = self?.tabManager.activeTab?.path, !activePath.isEmpty else {
                return BookmarksManager.shared.loadAllBookmarks()
            }
            return BookmarksManager.shared.getBookmarks(forDocument: activePath)
        }
        commandPaletteController.onTriggerAction = { [weak self] action in
            self?.handlePaletteAction(action)
        }

        SettingsWindowController.shared.onSettingsChanged = { [weak self] in
            self?.applyRuntimeSettings()
        }

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(pdfPageChanged(_:)),
            name: .PDFViewPageChanged,
            object: pdfView
        )
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(pdfScaleChanged(_:)),
            name: .PDFViewScaleChanged,
            object: pdfView
        )
    }

    // MARK: - Unified NSToolbarDelegate

    public func toolbarAllowedItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] {
        [
            Self.tbSidebarToggle,
            Self.tbHistoryNav,
            .flexibleSpace,
            Self.tbZoomGroup,
            Self.tbQuickOpen,
            Self.tbInspectorToggle
        ]
    }

    public func toolbarDefaultItemIdentifiers(_ toolbar: NSToolbar) -> [NSToolbarItem.Identifier] {
        [
            Self.tbSidebarToggle,
            Self.tbHistoryNav,
            .flexibleSpace,
            Self.tbZoomGroup,
            Self.tbQuickOpen,
            Self.tbInspectorToggle
        ]
    }

    public func toolbar(
        _ toolbar: NSToolbar,
        itemForItemIdentifier itemIdentifier: NSToolbarItem.Identifier,
        willBeInsertedIntoToolbar flag: Bool
    ) -> NSToolbarItem? {
        switch itemIdentifier {
        case Self.tbSidebarToggle:
            let item = NSToolbarItem(itemIdentifier: itemIdentifier)
            item.label = "Sidebar"
            item.toolTip = "Toggle Left Sidebar (⌘B)"
            let btn = NSButton(title: "", target: self, action: #selector(toggleLeftSidebar(_:)))
            btn.bezelStyle = .texturedRounded
            btn.image = NSImage(systemSymbolName: "sidebar.left", accessibilityDescription: "Sidebar")
            item.view = btn
            return item

        case Self.tbHistoryNav:
            let item = NSToolbarItem(itemIdentifier: itemIdentifier)
            item.label = "History"
            item.toolTip = "Navigate Back / Forward"
            let seg = NSSegmentedControl()
            seg.segmentCount = 2
            seg.trackingMode = .momentary
            seg.segmentStyle = .separated
            if let bImg = NSImage(systemSymbolName: "chevron.left", accessibilityDescription: "Back") {
                seg.setImage(bImg, forSegment: 0)
            }
            if let fImg = NSImage(systemSymbolName: "chevron.right", accessibilityDescription: "Forward") {
                seg.setImage(fImg, forSegment: 1)
            }
            seg.setWidth(28, forSegment: 0)
            seg.setWidth(28, forSegment: 1)
            seg.target = self
            seg.action = #selector(historySegmentClicked(_:))
            item.view = seg
            return item

        case Self.tbZoomGroup:
            let item = NSToolbarItem(itemIdentifier: itemIdentifier)
            item.label = "Zoom"
            item.toolTip = "Zoom Out / Reset / Zoom In"

            let container = NSStackView()
            container.orientation = .horizontal
            container.spacing = 2.0

            let outBtn = NSButton(title: "", target: self, action: #selector(zoomOut(_:)))
            outBtn.bezelStyle = .texturedRounded
            outBtn.image = NSImage(systemSymbolName: "minus.magnifyingglass", accessibilityDescription: "Zoom Out")

            let pctBtn = NSButton(title: "100%", target: self, action: #selector(zoomActualSize(_:)))
            pctBtn.bezelStyle = .texturedRounded
            pctBtn.font = .monospacedDigitSystemFont(ofSize: 11.0, weight: .medium)
            self.zoomPercentButton = pctBtn

            let inBtn = NSButton(title: "", target: self, action: #selector(zoomIn(_:)))
            inBtn.bezelStyle = .texturedRounded
            inBtn.image = NSImage(systemSymbolName: "plus.magnifyingglass", accessibilityDescription: "Zoom In")

            container.addArrangedSubview(outBtn)
            container.addArrangedSubview(pctBtn)
            container.addArrangedSubview(inBtn)
            item.view = container
            return item

        case Self.tbQuickOpen:
            let item = NSToolbarItem(itemIdentifier: itemIdentifier)
            item.label = "Command Palette"
            item.toolTip = "Quick Open & Command Palette (⌘P)"
            let btn = NSButton(title: "", target: self, action: #selector(quickOpen(_:)))
            btn.bezelStyle = .texturedRounded
            btn.image = NSImage(systemSymbolName: "command", accessibilityDescription: "Command Palette")
            item.view = btn
            return item

        case Self.tbInspectorToggle:
            let item = NSToolbarItem(itemIdentifier: itemIdentifier)
            item.label = "Inspector"
            item.toolTip = "Toggle Right Inspector (⌥⌘B)"
            let btn = NSButton(title: "", target: self, action: #selector(toggleRightSidebar(_:)))
            btn.bezelStyle = .texturedRounded
            btn.image = NSImage(systemSymbolName: "sidebar.right", accessibilityDescription: "Inspector")
            item.view = btn
            return item

        default:
            return nil
        }
    }

    // MARK: - Opening Files, Folders & Mode Transitions

    public func handleOpenURLs(_ urls: [URL]) {
        for url in urls {
            var isDir: ObjCBool = false
            if FileManager.default.fileExists(atPath: url.path, isDirectory: &isDir), isDir.boolValue {
                openFolder(at: url)
            } else {
                openFile(at: url)
            }
        }
    }

    public func openFile(at url: URL, fromWorkspaceSidebar: Bool = false) {
        let logicalURL = ICloudCoordinator.logicalURL(for: url)
        let resolvedURL: URL
        if FileManager.default.fileExists(atPath: url.path) {
            resolvedURL = url
        } else if FileManager.default.fileExists(atPath: logicalURL.path) {
            resolvedURL = logicalURL
        } else {
            let placeholderURL = ICloudCoordinator.placeholderURL(for: logicalURL)
            if FileManager.default.fileExists(atPath: placeholderURL.path) {
                resolvedURL = placeholderURL
            } else {
                UtilityWindows.shared.showError(title: "File Not Found", message: "The file at “\(url.path)” could not be found on disk.")
                return
            }
        }

        let cloudStatus = ICloudCoordinator.status(for: resolvedURL)
        if cloudStatus == .cloudOnly || cloudStatus == .downloadRequested || cloudStatus == .downloading {
            ICloudDownloadManager.shared.requestDownload(for: resolvedURL, autoOpenWhenReady: true)
            return
        }

        let path = resolvedURL.path

        // Check if already open in a tab
        if let existingTabIdx = tabManager.indexOfPath(path) {
            if fromWorkspaceSidebar || contentMode == .workspace {
                tabManager.tabs[existingTabIdx].mode = .workspace
                if tabManager.tabs[existingTabIdx].workspaceRoot == nil {
                    tabManager.tabs[existingTabIdx].workspaceRoot = workspaceController.rootNode?.path
                }
            }
            if contentMode == .home {
                transitionToReaderLayout(mode: tabManager.tabs[existingTabIdx].mode)
            }
            switchToTab(at: existingTabIdx)
            if SettingsManager.shared.autoRevealSidebar {
                workspaceController.revealFile(atPath: path)
            }
            return
        }

        guard let doc = PDFDocument(url: resolvedURL) else {
            UtilityWindows.shared.showError(title: "Cannot Open PDF", message: "“\(resolvedURL.lastPathComponent)” is not a valid or readable PDF document.")
            return
        }

        recordRecentFile(path)

        let targetMode: AppContentMode = (fromWorkspaceSidebar || contentMode == .workspace) ? .workspace : .standaloneDocument
        let title = url.deletingPathExtension().lastPathComponent
        let wsRoot = workspaceController.rootNode?.path

        if !SettingsManager.shared.openInNewTab && tabManager.tabCount > 0 {
            if let active = tabManager.activeTab {
                saveReadingPosition(for: active.path)
                active.path = path
                active.title = title
                active.document = doc
                active.pageIndex = 0
                active.isEdited = false
                active.mode = targetMode
                active.workspaceRoot = wsRoot
                active.lastKnownMtime = TabManager.fileMtime(forPath: path)
            }
        } else {
            if let active = tabManager.activeTab {
                saveCurrentTabViewportState(into: active)
            }
            _ = tabManager.addTab(
                path: path,
                title: title,
                document: doc,
                mode: targetMode,
                workspaceRoot: wsRoot
            )
        }

        transitionToReaderLayout(mode: targetMode)
        applyActiveTabToReader(restoreSavedPosition: SettingsManager.shared.restoreReadingPos)

        if targetMode == .workspace && SettingsManager.shared.autoRevealSidebar {
            workspaceController.revealFile(atPath: path)
        }
    }

    public func openFolder(at url: URL) {
        var isDir: ObjCBool = false
        guard FileManager.default.fileExists(atPath: url.path, isDirectory: &isDir), isDir.boolValue else {
            openFile(at: url)
            return
        }

        recordRecentWorkspace(url.path)
        workspaceController.loadWorkspace(rootURL: url)

        if SettingsManager.shared.deepWorkspaceSearch {
            let rootPath = url.path
            Task.detached(priority: .utility) {
                WorkspaceIndex.shared.indexWorkspaceDirectory(rootPath)
            }
        }

        transitionToReaderLayout(mode: .workspace)
        explorerSplitItem.isCollapsed = false

        // Automatically open the first PDF in the workspace if no document is loaded or if switching workspaces
        let allPDFs = WorkspaceNode.collectAllPDFPaths(in: url.path, onlyLocallyAvailable: true)
        if let firstPDF = allPDFs.first {
            openFile(at: URL(fileURLWithPath: firstPDF), fromWorkspaceSidebar: true)
        } else {
            window.title = url.lastPathComponent
            window.subtitle = "Empty Workspace"
        }
    }

    private func transitionToReaderLayout(mode: AppContentMode) {
        let wasHome = (contentMode == .home)
        contentMode = mode

        if window.contentViewController !== splitViewController {
            window.contentViewController = splitViewController
        }

        if mode == .standaloneDocument {
            explorerSplitItem.isCollapsed = true
        } else if mode == .workspace {
            explorerSplitItem.isCollapsed = false
        }

        if wasHome {
            window.setFrameAutosaveName("ReaderWindowGeometry")
            if window.frame.width < 1000 || window.frame.height < 680 {
                let readerFrame = WindowGeometry.initialFrame(for: .reader)
                window.setFrame(readerFrame, display: true)
            }
            WindowGeometry.sanitizeWindow(window, kind: .reader)
        }
        window.toolbar?.isVisible = true
    }

    @objc public func showHomeScreen(_ sender: Any?) {
        if let active = tabManager.activeTab {
            saveCurrentTabViewportState(into: active)
        }
        contentMode = .home
        homeScreenController.refreshContent()
        window.contentViewController = homeScreenController.viewController
        window.title = "Iridescence PDF Reader"
        window.subtitle = ""
        window.setFrameAutosaveName("HomeWindowGeometry")
        WindowGeometry.sanitizeWindow(window, kind: .home)
        window.toolbar?.isVisible = false
    }

    // MARK: - Multi-Document Tab Operations

    public func switchToTab(at index: Int) {
        guard index >= 0, index < tabManager.tabCount else { return }
        if let current = tabManager.activeTab {
            saveCurrentTabViewportState(into: current)
        }
        tabManager.activeIndex = index
        if let nextTab = tabManager.activeTab {
            if nextTab.mode == .workspace, let wsRoot = nextTab.workspaceRoot {
                if workspaceController.rootNode?.path != wsRoot {
                    workspaceController.loadWorkspace(rootURL: URL(fileURLWithPath: wsRoot))
                }
                explorerSplitItem.isCollapsed = false
                contentMode = .workspace
                if SettingsManager.shared.autoRevealSidebar {
                    workspaceController.revealFile(atPath: nextTab.path)
                }
            }
            applyActiveTabToReader(restoreSavedPosition: false)
        }
    }

    public func closeTab(at index: Int) {
        guard index >= 0, index < tabManager.tabCount else { return }
        let closingTab = tabManager.tabs[index]
        saveReadingPosition(for: closingTab.path)

        _ = tabManager.removeTab(at: index, recordClosed: SettingsManager.shared.reopenClosedTabs)
        if tabManager.tabCount == 0 {
            pdfView.document = nil
            pdfPageThumbnailController.reload(for: nil)
            pdfOutlineController.loadOutline(from: nil)
            pdfSearchController.cancelAndReset()
            isDocumentEdited = false
            window.isDocumentEdited = false
            updateTabBarVisibility()
            showHomeScreen(nil)
        } else {
            applyActiveTabToReader(restoreSavedPosition: false)
        }
    }

    public func closeOtherTabs(keeping index: Int) {
        guard index >= 0, index < tabManager.tabCount else { return }
        let targetPath = tabManager.tabs[index].path
        for i in (0..<tabManager.tabCount).reversed() {
            if tabManager.tabs[i].path != targetPath {
                _ = tabManager.removeTab(at: i, recordClosed: true)
            }
        }
        tabManager.activeIndex = 0
        applyActiveTabToReader(restoreSavedPosition: false)
    }

    public func selectNextTab() {
        guard tabManager.tabCount > 1 else { return }
        switchToTab(at: tabManager.nextIndex())
    }

    public func selectPreviousTab() {
        guard tabManager.tabCount > 1 else { return }
        switchToTab(at: tabManager.prevIndex())
    }

    @objc public func reopenClosedTab(_ sender: Any?) {
        guard SettingsManager.shared.reopenClosedTabs,
              let closed = tabManager.popLastClosedTab() else { return }
        let url = URL(fileURLWithPath: closed.path)
        openFile(at: url, fromWorkspaceSidebar: closed.mode == .workspace)
        if let doc = pdfView.document, let page = doc.page(at: closed.pageIndex) {
            pdfView.go(to: page)
        }
    }

    private func saveCurrentTabViewportState(into session: TabSession) {
        if let doc = pdfView.document, let curPage = pdfView.currentPage {
            let idx = doc.index(for: curPage)
            if idx != NSNotFound && idx >= 0 {
                session.pageIndex = idx
            }
        }
        session.scaleFactor = pdfView.scaleFactor
        saveReadingPosition(for: session.path)
    }

    private func applyActiveTabToReader(restoreSavedPosition: Bool) {
        guard let active = tabManager.activeTab else { return }

        pdfSearchController.cancelAndReset()
        backHistory.removeAll()
        forwardHistory.removeAll()

        pdfView.document = active.document
        applyDefaultDisplayAndZoomSettings(to: pdfView)

        if restoreSavedPosition {
            restoreReadingPosition(for: active.path)
        } else if let page = active.document.page(at: active.pageIndex) {
            pdfView.go(to: page)
            if active.scaleFactor > 0.1 {
                pdfView.scaleFactor = active.scaleFactor
            }
        }

        isDocumentEdited = active.isEdited
        window.isDocumentEdited = active.isEdited

        pdfOutlineController.loadOutline(from: active.document)
        bookmarksInspectorController.reload(forPath: active.path)
        pdfPageThumbnailController.reload(for: active.document, documentPath: active.path)
        updateRightSidebarVisibility(forSegment: rightSegmentedControl.selectedSegment)
        updateTabBarVisibility()
        updateWindowTitleAndToolbar()
    }

    private func updateTabBarVisibility() {
        tabBarView.rebuild(from: tabManager)
        let shouldShow = tabManager.tabCount >= 1
        tabBarView.isHidden = !shouldShow
        tabBarHeightConstraint.constant = shouldShow ? TabBarView.barHeight : 0.0
    }

    public func markDocumentEdited(_ edited: Bool) {
        isDocumentEdited = edited
        window?.isDocumentEdited = edited
        tabManager.activeTab?.isEdited = edited
        tabBarView?.rebuild(from: tabManager)
        updateWindowTitleAndToolbar()
    }

    // MARK: - Right Inspector Switching

    @objc public func switchSidebar(_ sender: NSSegmentedControl) {
        switchSidebarToIndex(sender.selectedSegment)
    }

    public func switchSidebarToIndex(_ index: Int) {
        let clamped = min(max(index, 0), 4)
        rightSegmentedControl.setSelected(true, forSegment: clamped)
        UserDefaults.standard.set(clamped, forKey: SettingsKeys.rightSidebarMode)
        updateRightSidebarVisibility(forSegment: clamped)
    }

    private func updateRightSidebarVisibility(forSegment index: Int) {
        let modeTitle: String = {
            switch index {
            case 1: return "OUTLINE"
            case 2: return "SEARCH"
            case 3: return "BOOKMARKS"
            case 4: return "ANNOTATIONS"
            default: return "PAGES"
            }
        }()
        rightSidebarModeLabel.stringValue = modeTitle

        pdfPageThumbnailController.scrollView.isHidden = (index != 0)
        pdfOutlineController.scrollView.isHidden = (index != 1) || pdfOutlineController.isEmpty
        pdfOutlineController.emptyStateView.isHidden = (index != 1) || !pdfOutlineController.isEmpty
        pdfSearchController.containerView.isHidden = (index != 2)

        if index == 1 {
            if let doc = pdfView.document, let cur = pdfView.currentPage {
                let idx = doc.index(for: cur)
                if idx != NSNotFound && idx >= 0 {
                    pdfOutlineController.syncActivePage(idx)
                }
            }
        }

        if index == 3 {
            bookmarksInspectorController.filterMode = 0
            bookmarksInspectorController.reload(forPath: tabManager.activeTab?.path)
            let empty = bookmarksInspectorController.isEmpty
            bookmarksInspectorController.scrollView.isHidden = empty
            bookmarksInspectorController.emptyStateView.isHidden = !empty
        } else if index == 4 {
            bookmarksInspectorController.filterMode = 1
            bookmarksInspectorController.reload(forPath: tabManager.activeTab?.path)
            let empty = bookmarksInspectorController.isEmpty
            bookmarksInspectorController.scrollView.isHidden = empty
            bookmarksInspectorController.emptyStateView.isHidden = !empty
        } else {
            bookmarksInspectorController.scrollView.isHidden = true
            bookmarksInspectorController.emptyStateView.isHidden = true
        }
    }

    public func refreshBookmarksView() {
        bookmarksInspectorController.reload(forPath: tabManager.activeTab?.path)
        let seg = rightSegmentedControl.selectedSegment
        if seg == 3 || seg == 4 {
            updateRightSidebarVisibility(forSegment: seg)
        }
    }

    // MARK: - PDF Notifications, Position Persistence & External Modification

    @objc private func pdfPageChanged(_ notification: Notification) {
        guard let doc = pdfView.document, let curPage = pdfView.currentPage else { return }
        let pageIdx = doc.index(for: curPage)
        guard pageIdx != NSNotFound && pageIdx >= 0 else { return }

        if let active = tabManager.activeTab {
            if !isNavigatingHistory && active.pageIndex != pageIdx {
                backHistory.append(active.pageIndex)
                if backHistory.count > 100 { backHistory.removeFirst() }
                forwardHistory.removeAll()
            }
            active.pageIndex = pageIdx
            saveReadingPosition(for: active.path)
        }

        pdfOutlineController.syncActivePage(pageIdx)
        pdfPageThumbnailController.updateActivePageIndex(pageIdx)
        updateWindowTitleAndToolbar()
    }

    @objc private func pdfScaleChanged(_ notification: Notification) {
        tabManager.activeTab?.scaleFactor = pdfView.scaleFactor
        updateWindowTitleAndToolbar()
    }

    private func updateWindowTitleAndToolbar() {
        guard contentMode != .home else { return }
        if let active = tabManager.activeTab, let doc = pdfView.document {
            let prefix = active.isEdited ? "● " : ""
            window.title = "\(prefix)\(active.title)"
            let curPageIdx = pdfView.currentPage.map { doc.index(for: $0) } ?? 0
            let safeIdx = (curPageIdx == NSNotFound || curPageIdx < 0) ? 0 : curPageIdx
            window.subtitle = "Page \(safeIdx + 1) of \(doc.pageCount)"
        }
        let pct = Int((pdfView.scaleFactor * 100.0).rounded())
        zoomPercentButton?.title = "\(min(max(pct, 10), 800))%"
    }

    private func saveReadingPosition(for rawPath: String) {
        guard SettingsManager.shared.restoreReadingPos, !rawPath.isEmpty,
              let doc = pdfView.document else { return }
        let norm = UIConstants.canonicalPath(rawPath)
        var pageIdx = 0
        if let curPage = pdfView.currentPage {
            let idx = doc.index(for: curPage)
            if idx != NSNotFound && idx >= 0 { pageIdx = idx }
        }
        let payload: [String: Any] = [
            "PageIndex": pageIdx,
            "Zoom": Double(pdfView.scaleFactor),
            "TotalPages": doc.pageCount,
            "LastReadTime": Date().timeIntervalSince1970
        ]
        UserDefaults.standard.set(payload, forKey: "PDFPos_\(norm)")
    }

    private func restoreReadingPosition(for rawPath: String) {
        guard SettingsManager.shared.restoreReadingPos, !rawPath.isEmpty,
              let doc = pdfView.document else { return }
        let norm = UIConstants.canonicalPath(rawPath)
        guard let dict = UserDefaults.standard.dictionary(forKey: "PDFPos_\(norm)") else { return }
        if let pageIdx = dict["PageIndex"] as? Int,
           pageIdx >= 0, pageIdx < doc.pageCount,
           let page = doc.page(at: pageIdx) {
            pdfView.go(to: page)
        }
        if let zoom = dict["Zoom"] as? Double, zoom >= 0.25 && zoom <= 6.0 {
            pdfView.scaleFactor = CGFloat(zoom)
        }
    }

    private func recordRecentFile(_ path: String) {
        guard SettingsManager.shared.rememberRecents else { return }
        let norm = UIConstants.canonicalPath(path)
        let defaults = UserDefaults.standard
        var list = defaults.stringArray(forKey: SettingsKeys.recentFiles) ?? []
        list.removeAll { UIConstants.canonicalPath($0) == norm }
        list.insert(path, at: 0)
        if list.count > 20 { list = Array(list.prefix(20)) }
        defaults.set(list, forKey: SettingsKeys.recentFiles)
        defaults.set(list, forKey: SettingsKeys.recentItems)
    }

    private func recordRecentWorkspace(_ path: String) {
        guard SettingsManager.shared.rememberRecents else { return }
        let norm = UIConstants.canonicalPath(path)
        let defaults = UserDefaults.standard
        var list = defaults.stringArray(forKey: SettingsKeys.recentWorkspaces) ?? []
        list.removeAll { UIConstants.canonicalPath($0) == norm }
        list.insert(path, at: 0)
        if list.count > 10 { list = Array(list.prefix(10)) }
        defaults.set(list, forKey: SettingsKeys.recentWorkspaces)
    }

    private func checkActiveDocumentExternalModification() {
        guard let active = tabManager.activeTab,
              !active.isEdited,
              !active.path.isEmpty,
              let currentMtime = TabManager.fileMtime(forPath: active.path) else {
            return
        }
        if let prevMtime = active.lastKnownMtime, currentMtime > prevMtime {
            active.lastKnownMtime = currentMtime
            if let reloaded = PDFDocument(url: URL(fileURLWithPath: active.path)) {
                let savedPage = active.pageIndex
                active.document = reloaded
                pdfView.document = reloaded
                if let page = reloaded.page(at: min(savedPage, max(0, reloaded.pageCount - 1))) {
                    pdfView.go(to: page)
                }
                AppLogger.shared.log(.pdf, "Reloaded externally modified PDF: \(active.path)")
            }
        }
    }

    private func applyDefaultDisplayAndZoomSettings(to view: PDFView) {
        let s = SettingsManager.shared
        switch s.defaultDisplayMode {
        case 0: view.displayMode = .singlePage
        case 2: view.displayMode = .twoUpContinuous
        default: view.displayMode = .singlePageContinuous
        }
        if s.defaultZoomMode == 1 {
            view.autoScales = false
            view.scaleFactor = 1.0
        } else {
            view.autoScales = true
        }
    }

    public func applyRuntimeSettings() {
        SettingsManager.shared.applySavedAppearance()
        if contentMode == .workspace {
            workspaceController.reloadCurrentWorkspace()
        }
        homeScreenController.refreshContent()
    }

    // MARK: - Responsive Window Resize Handling (BUG-06 Fix)

    public func windowDidResize(_ notification: Notification) {
        guard contentMode != .home else { return }
        let width = window.frame.width
        if width < UIConstants.responsiveCollapseInspectorWidth && !inspectorSplitItem.isCollapsed {
            inspectorSplitItem.isCollapsed = true
        }
        if width < UIConstants.responsiveCollapseSidebarWidth && !explorerSplitItem.isCollapsed {
            explorerSplitItem.isCollapsed = true
        }
    }

    // MARK: - Command Palette Routing

    private func handlePaletteAction(_ action: PaletteItemAction) {
        switch action {
        case .openFile(let path):
            openFile(at: URL(fileURLWithPath: path))

        case .openFileAtPage(let path, let pageIdx):
            openFile(at: URL(fileURLWithPath: path))
            if let doc = pdfView.document, let page = doc.page(at: pageIdx) {
                pdfView.go(to: page)
            }

        case .goToPage(let pageNum):
            guard let doc = pdfView.document else { return }
            let targetIdx = min(max(pageNum - 1, 0), max(0, doc.pageCount - 1))
            if let page = doc.page(at: targetIdx) {
                pdfView.go(to: page)
            }

        case .searchInPDF(let query):
            inspectorSplitItem.isCollapsed = false
            switchSidebarToIndex(2)
            pdfSearchController.performSearch(query)

        case .executeCommand(let cmdId):
            switch cmdId {
            case "open_pdf": openDocument(nil)
            case "open_folder": openFolder(nil)
            case "save": saveDocument(nil)
            case "save_as": saveDocumentAs(nil)
            case "export_markdown": exportNotesAndAnnotations(nil)
            case "highlight": highlightSelection(nil)
            case "underline": underlineSelection(nil)
            case "strikeout": strikeoutSelection(nil)
            case "add_note": addNoteToSelection(nil)
            case "bookmark_toggle": toggleBookmark(nil)
            case "ocr_page": ocrCurrentPage(nil)
            case "zoom_in": zoomIn(nil)
            case "zoom_out": zoomOut(nil)
            case "zoom_actual": zoomActualSize(nil)
            case "zoom_fit": zoomToFit(nil)
            case "mode_continuous": setContinuousMode(nil)
            case "mode_single": setSinglePageMode(nil)
            case "mode_twoup": setTwoUpMode(nil)
            case "toggle_left_sidebar": toggleLeftSidebar(nil)
            case "toggle_right_sidebar": toggleRightSidebar(nil)
            case "focus_mode": toggleFocusMode(nil)
            case "presentation_mode": togglePresentationMode(nil)
            case "tab_next": selectNextTab()
            case "tab_prev": selectPreviousTab()
            case "tab_reopen": reopenClosedTab(nil)
            case "tab_close": closeActiveTabOrWindow(nil)
            case "doc_info": showDocumentInfoWindow()
            case "debug_console": openDebugConsole(nil)
            case "settings": openSettings(nil)
            case "go_home": showHomeScreen(nil)
            default: break
            }
        }
    }

    // MARK: - Menu & Toolbar Actions (@objc Selectors)

    @objc public func openDocument(_ sender: Any?) {
        let panel = NSOpenPanel()
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = true
        panel.allowedContentTypes = [.pdf]
        if panel.runModal() == .OK {
            handleOpenURLs(panel.urls)
        }
    }

    @objc public func openFolder(_ sender: Any?) {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = false
        if panel.runModal() == .OK, let url = panel.url {
            openFolder(at: url)
        }
    }

    @objc public func saveDocument(_ sender: Any?) {
        guard let active = tabManager.activeTab, let doc = pdfView.document else { return }
        if AnnotationsManager.isMasterProtectedPath(active.path) {
            saveDocumentAs(sender)
            return
        }
        do {
            try AnnotationsManager.saveDocumentSafely(doc, to: active.path)
            active.lastKnownMtime = TabManager.fileMtime(forPath: active.path)
            markDocumentEdited(false)
        } catch {
            UtilityWindows.shared.showError(title: "Save Failed", message: error.localizedDescription)
        }
    }

    @objc public func saveDocumentAs(_ sender: Any?) {
        guard let active = tabManager.activeTab, let doc = pdfView.document else { return }
        let defaultName = URL(fileURLWithPath: active.path).lastPathComponent
        guard let destPath = AnnotationsManager.runSavePanel(defaultFilename: defaultName.isEmpty ? "Document.pdf" : defaultName) else {
            return
        }
        do {
            try AnnotationsManager.saveDocumentSafely(doc, to: destPath)
            BookmarksManager.shared.migrateBookmarks(from: active.path, to: destPath)
            active.path = destPath
            active.title = URL(fileURLWithPath: destPath).deletingPathExtension().lastPathComponent
            active.lastKnownMtime = TabManager.fileMtime(forPath: destPath)
            recordRecentFile(destPath)
            markDocumentEdited(false)
            bookmarksInspectorController.reload(forPath: destPath)
        } catch {
            UtilityWindows.shared.showError(title: "Save As Failed", message: error.localizedDescription)
        }
    }

    @objc public func exportNotesAndAnnotations(_ sender: Any?) {
        guard let active = tabManager.activeTab, let doc = pdfView.document else { return }
        let annots = AnnotationsManager.getDocumentAnnotations(in: doc)
        let md = BookmarksManager.shared.exportNotesAndAnnotationsMarkdown(
            pdfPath: active.path,
            title: active.title,
            annotations: annots
        )

        let panel = NSSavePanel()
        panel.canCreateDirectories = true
        panel.nameFieldStringValue = "\(active.title)-Notes.md"
        if panel.runModal() == .OK, let url = panel.url {
            if AnnotationsManager.isMasterProtectedPath(url.path) {
                UtilityWindows.shared.showError(title: "Protected Directory", message: "Cannot export files inside /Documents/MASTER.")
                return
            }
            try? md.write(to: url, atomically: true, encoding: .utf8)
        }
    }

    @objc public func highlightSelection(_ sender: Any?) {
        if AnnotationsManager.addAnnotationToSelection(in: pdfView, type: .highlight) {
            markDocumentEdited(true)
            refreshBookmarksView()
        }
    }

    @objc public func underlineSelection(_ sender: Any?) {
        if AnnotationsManager.addAnnotationToSelection(in: pdfView, type: .underline) {
            markDocumentEdited(true)
            refreshBookmarksView()
        }
    }

    @objc public func strikeoutSelection(_ sender: Any?) {
        if AnnotationsManager.addAnnotationToSelection(in: pdfView, type: .strikeout) {
            markDocumentEdited(true)
            refreshBookmarksView()
        }
    }

    @objc public func addNoteToSelection(_ sender: Any?) {
        if AnnotationsManager.addAnnotationToSelection(in: pdfView, type: .textNote, noteText: "Note") {
            markDocumentEdited(true)
            refreshBookmarksView()
        }
    }

    @objc public func toggleBookmark(_ sender: Any?) {
        guard let active = tabManager.activeTab,
              let doc = pdfView.document,
              let curPage = pdfView.currentPage else { return }
        let pageIdx = doc.index(for: curPage)
        guard pageIdx != NSNotFound && pageIdx >= 0 else { return }

        let snippet = (curPage.string ?? "")
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .split(whereSeparator: \.isNewline)
            .first
            .map { String($0.prefix(48)) } ?? "Page \(pageIdx + 1)"

        let defaultLabel = snippet.isEmpty ? "Page \(pageIdx + 1)" : snippet
        _ = BookmarksManager.shared.toggleBookmark(pdfPath: active.path, pageIndex: pageIdx, defaultLabel: defaultLabel)
        refreshBookmarksView()
    }

    @objc public func ocrCurrentPage(_ sender: Any?) {
        guard let doc = pdfView.document, let page = pdfView.currentPage else { return }
        let pageIdx = max(0, doc.index(for: page))
        let token = OCRCancellationToken()
        OCRService.shared.showRunningHUD(forPage: pageIdx, token: token)

        nonisolated(unsafe) let unsafePage = page
        Task.detached(priority: .userInitiated) {
            do {
                let res = try OCRService.recognizeText(on: unsafePage, pageIndex: pageIdx, token: token)
                await MainActor.run {
                    if !token.isCancelled {
                        OCRService.shared.showResultPanel(result: res)
                    }
                }
            } catch {
                await MainActor.run {
                    OCRService.shared.dismissRunningHUD()
                    if !token.isCancelled {
                        UtilityWindows.shared.showError(title: "Vision OCR Failed", message: error.localizedDescription)
                    }
                }
            }
        }
    }

    @objc public func quickOpen(_ sender: Any?) {
        commandPaletteController.present(prefill: "")
    }

    @objc public func commandPalette(_ sender: Any?) {
        commandPaletteController.present(prefill: ">")
    }

    @objc public func goToPagePrompt(_ sender: Any?) {
        commandPaletteController.present(prefill: "p:")
    }

    @objc public func findInDocument(_ sender: Any?) {
        guard contentMode != .home else { return }
        inspectorSplitItem.isCollapsed = false
        switchSidebarToIndex(2)
        window.makeFirstResponder(pdfSearchController.searchField)
    }

    @objc public func zoomIn(_ sender: Any?) {
        pdfView.zoomIn(sender)
    }

    @objc public func zoomOut(_ sender: Any?) {
        pdfView.zoomOut(sender)
    }

    @objc public func zoomActualSize(_ sender: Any?) {
        pdfView.autoScales = false
        pdfView.scaleFactor = 1.0
    }

    @objc public func zoomToFit(_ sender: Any?) {
        pdfView.autoScales = true
    }

    @objc public func setContinuousMode(_ sender: Any?) {
        pdfView.displayMode = .singlePageContinuous
    }

    @objc public func setSinglePageMode(_ sender: Any?) {
        pdfView.displayMode = .singlePage
    }

    @objc public func setTwoUpMode(_ sender: Any?) {
        pdfView.displayMode = .twoUpContinuous
    }

    @objc public func toggleLeftSidebar(_ sender: Any?) {
        guard contentMode != .home else { return }
        explorerSplitItem.animator().isCollapsed.toggle()
    }

    @objc public func toggleRightSidebar(_ sender: Any?) {
        guard contentMode != .home else { return }
        inspectorSplitItem.animator().isCollapsed.toggle()
    }

    @objc public func toggleFocusMode(_ sender: Any?) {
        guard contentMode != .home else { return }
        if !isFocusModeActive {
            savedLeftCollapsedBeforeFocus = explorerSplitItem.isCollapsed
            savedRightCollapsedBeforeFocus = inspectorSplitItem.isCollapsed
            explorerSplitItem.isCollapsed = true
            inspectorSplitItem.isCollapsed = true
            isFocusModeActive = true
        } else {
            explorerSplitItem.isCollapsed = (contentMode == .standaloneDocument) ? true : savedLeftCollapsedBeforeFocus
            inspectorSplitItem.isCollapsed = savedRightCollapsedBeforeFocus
            isFocusModeActive = false
        }
    }

    @objc public func togglePresentationMode(_ sender: Any?) {
        guard contentMode != .home else { return }
        if !isPresentationModeActive {
            savedLeftCollapsedBeforeFocus = explorerSplitItem.isCollapsed
            savedRightCollapsedBeforeFocus = inspectorSplitItem.isCollapsed
            explorerSplitItem.isCollapsed = true
            inspectorSplitItem.isCollapsed = true
            window.toolbar?.isVisible = false
            tabBarView.isHidden = true
            tabBarHeightConstraint.constant = 0.0
            isPresentationModeActive = true
        } else {
            explorerSplitItem.isCollapsed = (contentMode == .standaloneDocument) ? true : savedLeftCollapsedBeforeFocus
            inspectorSplitItem.isCollapsed = savedRightCollapsedBeforeFocus
            window.toolbar?.isVisible = true
            updateTabBarVisibility()
            isPresentationModeActive = false
        }
    }

    @objc private func historySegmentClicked(_ sender: NSSegmentedControl) {
        if sender.selectedSegment == 0 {
            navigateBack(nil)
        } else {
            navigateForward(nil)
        }
    }

    @objc public func navigateBack(_ sender: Any?) {
        guard let doc = pdfView.document, let targetIdx = backHistory.popLast() else { return }
        if let cur = pdfView.currentPage {
            let curIdx = doc.index(for: cur)
            if curIdx != NSNotFound && curIdx >= 0 {
                forwardHistory.append(curIdx)
            }
        }
        isNavigatingHistory = true
        if let page = doc.page(at: targetIdx) {
            pdfView.go(to: page)
        }
        isNavigatingHistory = false
    }

    @objc public func navigateForward(_ sender: Any?) {
        guard let doc = pdfView.document, let targetIdx = forwardHistory.popLast() else { return }
        if let cur = pdfView.currentPage {
            let curIdx = doc.index(for: cur)
            if curIdx != NSNotFound && curIdx >= 0 {
                backHistory.append(curIdx)
            }
        }
        isNavigatingHistory = true
        if let page = doc.page(at: targetIdx) {
            pdfView.go(to: page)
        }
        isNavigatingHistory = false
    }

    @objc public func closeActiveTabOrWindow(_ sender: Any?) {
        if contentMode != .home && tabManager.tabCount > 0 {
            closeTab(at: tabManager.activeIndex)
        } else {
            window.performClose(sender)
        }
    }

    public func showDocumentInfoWindow() {
        let path = tabManager.activeTab?.path ?? ""
        UtilityWindows.shared.showDocumentInfo(for: pdfView.document, filePath: path)
    }

    @objc public func closeDocumentInfoWindow(_ sender: Any?) {
        UtilityWindows.shared.closeDocumentInfo()
    }

    @objc public func openDebugConsole(_ sender: Any?) {
        DebugConsoleController.shared.showWindow()
    }

    @objc public func openSettings(_ sender: Any?) {
        SettingsWindowController.shared.showWindow()
    }

    @objc private func menuSelectNextTab(_ sender: Any?) {
        selectNextTab()
    }

    @objc private func menuSelectPrevTab(_ sender: Any?) {
        selectPreviousTab()
    }

    @objc private func menuShowDocInfo(_ sender: Any?) {
        showDocumentInfoWindow()
    }

    // MARK: - Native Menu Bar Construction

    private func buildMainMenu() {
        let mainMenu = NSMenu()

        // 1. Application Menu
        let appMenuItem = NSMenuItem()
        let appMenu = NSMenu(title: "Iridescence PDF Reader")
        appMenu.addItem(withTitle: "About Iridescence PDF Reader", action: #selector(NSApplication.orderFrontStandardAboutPanel(_:)), keyEquivalent: "")
        appMenu.addItem(.separator())
        let prefsItem = NSMenuItem(title: "Settings…", action: #selector(openSettings(_:)), keyEquivalent: ",")
        prefsItem.target = self
        appMenu.addItem(prefsItem)
        appMenu.addItem(.separator())
        appMenu.addItem(withTitle: "Hide Iridescence PDF Reader", action: #selector(NSApplication.hide(_:)), keyEquivalent: "h")
        let hideOthers = appMenu.addItem(withTitle: "Hide Others", action: #selector(NSApplication.hideOtherApplications(_:)), keyEquivalent: "h")
        hideOthers.keyEquivalentModifierMask = [.command, .option]
        appMenu.addItem(withTitle: "Show All", action: #selector(NSApplication.unhideAllApplications(_:)), keyEquivalent: "")
        appMenu.addItem(.separator())
        appMenu.addItem(withTitle: "Quit Iridescence PDF Reader", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        appMenuItem.submenu = appMenu
        mainMenu.addItem(appMenuItem)

        // 2. File Menu
        let fileMenuItem = NSMenuItem()
        let fileMenu = NSMenu(title: "File")
        let openItem = NSMenuItem(title: "Open PDF…", action: #selector(openDocument(_:)), keyEquivalent: "o")
        openItem.target = self
        fileMenu.addItem(openItem)

        let openFolderItem = NSMenuItem(title: "Open Workspace Folder…", action: #selector(openFolder(_:)), keyEquivalent: "O")
        openFolderItem.keyEquivalentModifierMask = [.command, .shift]
        openFolderItem.target = self
        fileMenu.addItem(openFolderItem)

        fileMenu.addItem(.separator())

        let saveItem = NSMenuItem(title: "Save", action: #selector(saveDocument(_:)), keyEquivalent: "s")
        saveItem.target = self
        fileMenu.addItem(saveItem)

        let saveAsItem = NSMenuItem(title: "Save As…", action: #selector(saveDocumentAs(_:)), keyEquivalent: "S")
        saveAsItem.keyEquivalentModifierMask = [.command, .shift]
        saveAsItem.target = self
        fileMenu.addItem(saveAsItem)

        let exportMDItem = NSMenuItem(title: "Export Notes & Annotations…", action: #selector(exportNotesAndAnnotations(_:)), keyEquivalent: "E")
        exportMDItem.keyEquivalentModifierMask = [.command, .shift]
        exportMDItem.target = self
        fileMenu.addItem(exportMDItem)

        fileMenu.addItem(.separator())

        let reopenTabItem = NSMenuItem(title: "Reopen Closed Tab", action: #selector(reopenClosedTab(_:)), keyEquivalent: "T")
        reopenTabItem.keyEquivalentModifierMask = [.command, .shift]
        reopenTabItem.target = self
        fileMenu.addItem(reopenTabItem)

        let closeItem = NSMenuItem(title: "Close Tab / Window", action: #selector(closeActiveTabOrWindow(_:)), keyEquivalent: "w")
        closeItem.target = self
        fileMenu.addItem(closeItem)

        let infoItem = NSMenuItem(title: "Get Info", action: #selector(menuShowDocInfo(_:)), keyEquivalent: "i")
        infoItem.target = self
        fileMenu.addItem(infoItem)

        fileMenuItem.submenu = fileMenu
        mainMenu.addItem(fileMenuItem)

        // 3. Edit Menu
        let editMenuItem = NSMenuItem()
        let editMenu = NSMenu(title: "Edit")
        editMenu.addItem(withTitle: "Copy", action: #selector(NSText.copy(_:)), keyEquivalent: "c")
        editMenu.addItem(withTitle: "Select All", action: #selector(NSText.selectAll(_:)), keyEquivalent: "a")
        editMenu.addItem(.separator())

        let findItem = NSMenuItem(title: "Find in Document…", action: #selector(findInDocument(_:)), keyEquivalent: "f")
        findItem.target = self
        editMenu.addItem(findItem)

        let quickOpenItem = NSMenuItem(title: "Quick Open…", action: #selector(quickOpen(_:)), keyEquivalent: "p")
        quickOpenItem.target = self
        editMenu.addItem(quickOpenItem)

        let cmdPaletteItem = NSMenuItem(title: "Command Palette…", action: #selector(commandPalette(_:)), keyEquivalent: "P")
        cmdPaletteItem.keyEquivalentModifierMask = [.command, .shift]
        cmdPaletteItem.target = self
        editMenu.addItem(cmdPaletteItem)

        editMenuItem.submenu = editMenu
        mainMenu.addItem(editMenuItem)

        // 4. Tools & Markup Menu
        let toolsMenuItem = NSMenuItem()
        let toolsMenu = NSMenu(title: "Tools")

        let hlItem = NSMenuItem(title: "Highlight Selection", action: #selector(highlightSelection(_:)), keyEquivalent: "h")
        hlItem.keyEquivalentModifierMask = [.command, .control]
        hlItem.target = self
        toolsMenu.addItem(hlItem)

        let ulItem = NSMenuItem(title: "Underline Selection", action: #selector(underlineSelection(_:)), keyEquivalent: "u")
        ulItem.keyEquivalentModifierMask = [.command, .control]
        ulItem.target = self
        toolsMenu.addItem(ulItem)

        let soItem = NSMenuItem(title: "Strikeout Selection", action: #selector(strikeoutSelection(_:)), keyEquivalent: "s")
        soItem.keyEquivalentModifierMask = [.command, .control]
        soItem.target = self
        toolsMenu.addItem(soItem)

        let noteItem = NSMenuItem(title: "Add Note to Selection", action: #selector(addNoteToSelection(_:)), keyEquivalent: "n")
        noteItem.keyEquivalentModifierMask = [.command, .control]
        noteItem.target = self
        toolsMenu.addItem(noteItem)

        toolsMenu.addItem(.separator())

        let bmItem = NSMenuItem(title: "Toggle Bookmark", action: #selector(toggleBookmark(_:)), keyEquivalent: "d")
        bmItem.target = self
        toolsMenu.addItem(bmItem)

        let ocrItem = NSMenuItem(title: "OCR Current Page (Apple Vision)", action: #selector(ocrCurrentPage(_:)), keyEquivalent: "O")
        ocrItem.keyEquivalentModifierMask = [.command, .shift]
        ocrItem.target = self
        toolsMenu.addItem(ocrItem)

        toolsMenuItem.submenu = toolsMenu
        mainMenu.addItem(toolsMenuItem)

        // 5. View Menu
        let viewMenuItem = NSMenuItem()
        let viewMenu = NSMenu(title: "View")

        let homeItem = NSMenuItem(title: "Home Screen", action: #selector(showHomeScreen(_:)), keyEquivalent: "H")
        homeItem.keyEquivalentModifierMask = [.command, .shift]
        homeItem.target = self
        viewMenu.addItem(homeItem)

        let goPageItem = NSMenuItem(title: "Go to Page…", action: #selector(goToPagePrompt(_:)), keyEquivalent: "g")
        goPageItem.keyEquivalentModifierMask = [.command, .option]
        goPageItem.target = self
        viewMenu.addItem(goPageItem)

        viewMenu.addItem(.separator())

        let zIn = NSMenuItem(title: "Zoom In", action: #selector(zoomIn(_:)), keyEquivalent: "+")
        zIn.target = self
        viewMenu.addItem(zIn)

        let zOut = NSMenuItem(title: "Zoom Out", action: #selector(zoomOut(_:)), keyEquivalent: "-")
        zOut.target = self
        viewMenu.addItem(zOut)

        let zActual = NSMenuItem(title: "Actual Size", action: #selector(zoomActualSize(_:)), keyEquivalent: "0")
        zActual.target = self
        viewMenu.addItem(zActual)

        let zFit = NSMenuItem(title: "Zoom to Fit", action: #selector(zoomToFit(_:)), keyEquivalent: "9")
        zFit.target = self
        viewMenu.addItem(zFit)

        viewMenu.addItem(.separator())

        let leftSb = NSMenuItem(title: "Toggle Left Sidebar", action: #selector(toggleLeftSidebar(_:)), keyEquivalent: "b")
        leftSb.target = self
        viewMenu.addItem(leftSb)

        let rightSb = NSMenuItem(title: "Toggle Right Inspector", action: #selector(toggleRightSidebar(_:)), keyEquivalent: "b")
        rightSb.keyEquivalentModifierMask = [.command, .option]
        rightSb.target = self
        viewMenu.addItem(rightSb)

        let focusItem = NSMenuItem(title: "Toggle Focus Mode", action: #selector(toggleFocusMode(_:)), keyEquivalent: "F")
        focusItem.keyEquivalentModifierMask = [.command, .shift]
        focusItem.target = self
        viewMenu.addItem(focusItem)

        let presItem = NSMenuItem(title: "Toggle Presentation Mode", action: #selector(togglePresentationMode(_:)), keyEquivalent: "f")
        presItem.keyEquivalentModifierMask = [.command, .control]
        presItem.target = self
        viewMenu.addItem(presItem)

        viewMenu.addItem(.separator())

        let nextTab = NSMenuItem(title: "Show Next Tab", action: #selector(menuSelectNextTab(_:)), keyEquivalent: "\t")
        nextTab.keyEquivalentModifierMask = [.control]
        nextTab.target = self
        viewMenu.addItem(nextTab)

        let prevTab = NSMenuItem(title: "Show Previous Tab", action: #selector(menuSelectPrevTab(_:)), keyEquivalent: "\t")
        prevTab.keyEquivalentModifierMask = [.control, .shift]
        prevTab.target = self
        viewMenu.addItem(prevTab)

        viewMenu.addItem(.separator())

        let dbgItem = NSMenuItem(title: "Live Debug Console", action: #selector(openDebugConsole(_:)), keyEquivalent: "l")
        dbgItem.keyEquivalentModifierMask = [.command, .option]
        dbgItem.target = self
        viewMenu.addItem(dbgItem)

        viewMenuItem.submenu = viewMenu
        mainMenu.addItem(viewMenuItem)

        NSApp.mainMenu = mainMenu
    }
}
