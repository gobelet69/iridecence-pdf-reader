import AppKit
import Foundation

public enum HomeSidebarItem: Int, CaseIterable {
    case headerLibrary = 0
    case home = 1
    case continueReading = 2
    case recent = 3
    case favorites = 4
    case headerLocations = 5
    case localFiles = 6
    case iCloudDrive = 7
    case workspaces = 8

    public var isHeader: Bool {
        self == .headerLibrary || self == .headerLocations
    }

    public var title: String {
        switch self {
        case .headerLibrary: return "LIBRARY"
        case .home: return "Home"
        case .continueReading: return "Continue Reading"
        case .recent: return "Recent"
        case .favorites: return "Favorites"
        case .headerLocations: return "LOCATIONS"
        case .localFiles: return "Local Files"
        case .iCloudDrive: return "iCloud Drive"
        case .workspaces: return "Workspaces"
        }
    }

    public var symbolName: String {
        switch self {
        case .home: return "sparkles.rectangle.stack.fill"
        case .continueReading: return "book.pages.fill"
        case .recent: return "clock.fill"
        case .favorites: return "star.fill"
        case .localFiles: return "folder.fill"
        case .iCloudDrive: return "icloud.fill"
        case .workspaces: return "square.grid.2x2.fill"
        default: return ""
        }
    }

    public var iconColor: NSColor {
        switch self {
        case .home: return IridescenceDesign.primaryViolet
        case .continueReading: return IridescenceDesign.secondaryIndigo
        case .recent: return IridescenceDesign.accentCyan
        case .favorites: return IridescenceDesign.accentAmber
        case .localFiles: return IridescenceDesign.secondaryIndigo
        case .iCloudDrive: return IridescenceDesign.accentCyan
        case .workspaces: return IridescenceDesign.accentRose
        default: return .tertiaryLabelColor
        }
    }

    public var subtitleDescription: String {
        switch self {
        case .home: return "Recent documents, reading sessions, and quick workspace access"
        case .continueReading: return "Pick up right where you left off in your active PDFs"
        case .recent: return "Recently opened PDF documents"
        case .favorites: return "Starred documents for one-click access"
        case .localFiles: return "Documents stored locally on your Mac"
        case .iCloudDrive: return "Documents synced via iCloud Drive"
        case .workspaces: return "Recent folders and project workspaces"
        default: return ""
        }
    }
}

public struct HomeDocumentEntry: Sendable, Equatable {
    public let path: String
    public let title: String
    public let subtitle: String
    public let isWorkspaceFolder: Bool
    public let isFavorite: Bool
    public let progressBadge: String
    public let iCloudStatus: ICloudStatus
    public let lastReadTime: Double
}

/// Tinted rounded-square icon badge used in Home Screen document rows.
@MainActor
private final class HomeRowIconBadgeView: NSView {
    let imageView = NSImageView()
    var tintColor: NSColor = IridescenceDesign.primaryViolet {
        didSet { needsDisplay = true; imageView.contentTintColor = tintColor }
    }

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        translatesAutoresizingMaskIntoConstraints = false
        imageView.translatesAutoresizingMaskIntoConstraints = false
        addSubview(imageView)
        NSLayoutConstraint.activate([
            imageView.centerXAnchor.constraint(equalTo: centerXAnchor),
            imageView.centerYAnchor.constraint(equalTo: centerYAnchor),
            imageView.widthAnchor.constraint(equalToConstant: 17.0),
            imageView.heightAnchor.constraint(equalToConstant: 17.0)
        ])
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func draw(_ dirtyRect: NSRect) {
        let b = bounds.insetBy(dx: 0.5, dy: 0.5)
        let path = NSBezierPath(roundedRect: b, xRadius: 7.5, yRadius: 7.5)
        let isDark = effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        tintColor.withAlphaComponent(isDark ? 0.20 : 0.11).setFill()
        path.fill()
        tintColor.withAlphaComponent(isDark ? 0.35 : 0.22).setStroke()
        path.lineWidth = 0.75
        path.stroke()
    }
}

/// Manages the Home / Welcome Screen experience:
/// Left spectral source-list sidebar, right iridescent hero banner card with emblem,
/// search filter, `Open Folder…` and `Open PDF…` buttons, and rich document/workspace rows.
@MainActor
public final class HomeScreenController: NSObject, NSOutlineViewDataSource, NSOutlineViewDelegate, NSTableViewDataSource, NSTableViewDelegate, NSTextFieldDelegate, NSMenuDelegate {
    public let splitViewController: NSSplitViewController
    public let sidebarOutlineView: NSOutlineView
    public let documentTableView: NSTableView
    public let documentScrollView: NSScrollView
    public let sectionTitleLabel: NSTextField
    public let sectionSubtitleLabel: NSTextField
    public let filterField: NSTextField
    public let emptyStateView: EmptyStateView

    public private(set) var activeCategory: HomeSidebarItem = .home
    public private(set) var displayedEntries: [HomeDocumentEntry] = []

    public var onOpenPDF: (() -> Void)?
    public var onOpenFolder: (() -> Void)?
    public var onSelectFileURL: ((URL) -> Void)?
    public var onSelectWorkspaceURL: ((URL) -> Void)?

    private let sidebarHover = HoverCoordinator()
    private let tableHover = HoverCoordinator()
    private var filterText: String = ""

    public override init() {
        self.splitViewController = NSSplitViewController()
        self.sidebarOutlineView = NSOutlineView(frame: NSRect(x: 0, y: 0, width: 210, height: 560))
        self.documentTableView = NSTableView(frame: NSRect(x: 0, y: 0, width: 660, height: 480))
        self.documentScrollView = NSScrollView(frame: NSRect(x: 0, y: 0, width: 660, height: 480))
        self.sectionTitleLabel = NSTextField(labelWithString: "Iridescence")
        self.sectionSubtitleLabel = NSTextField(labelWithString: HomeSidebarItem.home.subtitleDescription)
        self.filterField = NSTextField(string: "")
        self.emptyStateView = EmptyStateView(
            symbolName: "sparkles.rectangle.stack",
            title: "Welcome to Iridescence PDF Reader",
            subtitle: "Click “Open PDF…” or “Open Folder…” above, or drag a PDF or workspace folder into this window"
        )
        super.init()
        setupLayout()
        refreshContent()
    }

    public var viewController: NSViewController {
        splitViewController
    }

    private func setupLayout() {
        // 1. Left Sidebar with Iridescent Surface Wash
        let sidebarCol = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("HomeSidebarCol"))
        sidebarCol.width = 190.0
        sidebarCol.resizingMask = .autoresizingMask
        sidebarOutlineView.addTableColumn(sidebarCol)
        sidebarOutlineView.outlineTableColumn = sidebarCol
        sidebarOutlineView.columnAutoresizingStyle = .uniformColumnAutoresizingStyle
        sidebarOutlineView.headerView = nil
        sidebarOutlineView.backgroundColor = .clear
        sidebarOutlineView.rowHeight = UIConstants.rowHeightSidebar
        sidebarOutlineView.style = .sourceList
        sidebarOutlineView.indentationPerLevel = 0.0
        sidebarOutlineView.delegate = self
        sidebarOutlineView.dataSource = self
        sidebarOutlineView.target = self
        sidebarOutlineView.action = #selector(sidebarClicked(_:))

        let sidebarScroll = NSScrollView(frame: NSRect(x: 0, y: 0, width: 210, height: 560))
        sidebarScroll.hasVerticalScroller = false
        sidebarScroll.documentView = sidebarOutlineView
        sidebarScroll.drawsBackground = false
        sidebarScroll.translatesAutoresizingMaskIntoConstraints = false

        let sidebarEffect = NSVisualEffectView(frame: NSRect(x: 0, y: 0, width: 210, height: 600))
        sidebarEffect.material = .sidebar
        sidebarEffect.blendingMode = .behindWindow
        sidebarEffect.state = .followsWindowActiveState

        let sidebarWash = IridescentSurfaceWashView(role: .sidebar)
        sidebarEffect.addSubview(sidebarWash)
        sidebarEffect.addSubview(sidebarScroll)

        NSLayoutConstraint.activate([
            sidebarWash.topAnchor.constraint(equalTo: sidebarEffect.topAnchor),
            sidebarWash.leadingAnchor.constraint(equalTo: sidebarEffect.leadingAnchor),
            sidebarWash.trailingAnchor.constraint(equalTo: sidebarEffect.trailingAnchor),
            sidebarWash.bottomAnchor.constraint(equalTo: sidebarEffect.bottomAnchor),

            sidebarScroll.topAnchor.constraint(equalTo: sidebarEffect.topAnchor, constant: 40.0),
            sidebarScroll.leadingAnchor.constraint(equalTo: sidebarEffect.leadingAnchor),
            sidebarScroll.trailingAnchor.constraint(equalTo: sidebarEffect.trailingAnchor),
            sidebarScroll.bottomAnchor.constraint(equalTo: sidebarEffect.bottomAnchor)
        ])

        let sidebarVC = NSViewController()
        sidebarVC.view = sidebarEffect
        let sidebarItem = NSSplitViewItem(sidebarWithViewController: sidebarVC)
        sidebarItem.canCollapse = false
        sidebarItem.minimumThickness = UIConstants.homeSidebarWidthMin
        sidebarItem.maximumThickness = UIConstants.homeSidebarWidthMax

        // 2. Right Content Area with Iridescent Hero Banner Card
        let rightEffect = NSVisualEffectView(frame: NSRect(x: 0, y: 0, width: 690, height: 600))
        rightEffect.material = .underWindowBackground
        rightEffect.blendingMode = .behindWindow
        rightEffect.state = .followsWindowActiveState

        let canvasWash = IridescentSurfaceWashView(role: .welcomeCanvas)
        rightEffect.addSubview(canvasWash)

        let heroCard = IridescentSurfaceWashView(role: .welcomeHero)
        let emblem = IridescentEmblemView(size: 38.0)

        sectionTitleLabel.font = .systemFont(ofSize: 19.5, weight: .bold)
        sectionTitleLabel.textColor = .labelColor
        sectionTitleLabel.translatesAutoresizingMaskIntoConstraints = false

        sectionSubtitleLabel.font = .systemFont(ofSize: 12.0, weight: .regular)
        sectionSubtitleLabel.textColor = .secondaryLabelColor
        sectionSubtitleLabel.lineBreakMode = .byTruncatingTail
        sectionSubtitleLabel.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        sectionSubtitleLabel.translatesAutoresizingMaskIntoConstraints = false

        filterField.placeholderString = "Filter documents…"
        filterField.controlSize = .regular
        filterField.font = .systemFont(ofSize: 12.0)
        filterField.bezelStyle = .roundedBezel
        filterField.delegate = self
        filterField.translatesAutoresizingMaskIntoConstraints = false

        let openFolderBtn = NSButton(title: " Open Folder…", target: self, action: #selector(openFolderClicked(_:)))
        openFolderBtn.bezelStyle = .rounded
        openFolderBtn.imagePosition = .imageLeading
        if let fImg = NSImage(systemSymbolName: "folder.badge.plus", accessibilityDescription: "Open Workspace Folder") {
            openFolderBtn.image = fImg
            openFolderBtn.contentTintColor = IridescenceDesign.secondaryIndigo
        }
        openFolderBtn.translatesAutoresizingMaskIntoConstraints = false

        let openPDFBtn = NSButton(title: " Open PDF…", target: self, action: #selector(openPDFClicked(_:)))
        openPDFBtn.bezelStyle = .rounded
        openPDFBtn.keyEquivalent = "\r"
        openPDFBtn.imagePosition = .imageLeading
        openPDFBtn.bezelColor = IridescenceDesign.primaryViolet
        if let pImg = NSImage(systemSymbolName: "doc.badge.plus", accessibilityDescription: "Open PDF Document") {
            openPDFBtn.image = pImg
        }
        openPDFBtn.translatesAutoresizingMaskIntoConstraints = false

        let docCol = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("HomeDocCol"))
        docCol.width = 640.0
        docCol.resizingMask = .autoresizingMask
        documentTableView.addTableColumn(docCol)
        documentTableView.columnAutoresizingStyle = .uniformColumnAutoresizingStyle
        documentTableView.headerView = nil
        documentTableView.backgroundColor = .clear
        documentTableView.rowHeight = 46.0
        documentTableView.style = .inset
        documentTableView.intercellSpacing = NSSize(width: 0, height: 3)
        documentTableView.delegate = self
        documentTableView.dataSource = self
        documentTableView.target = self
        documentTableView.action = #selector(documentRowClicked(_:))
        documentTableView.doubleAction = #selector(documentRowDoubleClicked(_:))

        let menu = NSMenu(title: "HomeContext")
        menu.delegate = self
        documentTableView.menu = menu

        let trackingArea = NSTrackingArea(
            rect: documentTableView.bounds,
            options: [.mouseEnteredAndExited, .mouseMoved, .activeInKeyWindow, .inVisibleRect],
            owner: self,
            userInfo: nil
        )
        documentTableView.addTrackingArea(trackingArea)

        documentScrollView.hasVerticalScroller = true
        documentScrollView.documentView = documentTableView
        documentScrollView.drawsBackground = false
        documentScrollView.translatesAutoresizingMaskIntoConstraints = false
        documentScrollView.contentView.postsBoundsChangedNotifications = true

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(scrollBoundsChanged(_:)),
            name: NSView.boundsDidChangeNotification,
            object: documentScrollView.contentView
        )

        rightEffect.addSubview(heroCard)
        rightEffect.addSubview(emblem)
        rightEffect.addSubview(sectionTitleLabel)
        rightEffect.addSubview(sectionSubtitleLabel)
        rightEffect.addSubview(filterField)
        rightEffect.addSubview(openFolderBtn)
        rightEffect.addSubview(openPDFBtn)
        rightEffect.addSubview(documentScrollView)
        rightEffect.addSubview(emptyStateView)

        NSLayoutConstraint.activate([
            canvasWash.topAnchor.constraint(equalTo: rightEffect.topAnchor),
            canvasWash.leadingAnchor.constraint(equalTo: rightEffect.leadingAnchor),
            canvasWash.trailingAnchor.constraint(equalTo: rightEffect.trailingAnchor),
            canvasWash.bottomAnchor.constraint(equalTo: rightEffect.bottomAnchor),

            heroCard.topAnchor.constraint(equalTo: rightEffect.topAnchor, constant: 24.0),
            heroCard.leadingAnchor.constraint(equalTo: rightEffect.leadingAnchor, constant: 20.0),
            heroCard.trailingAnchor.constraint(equalTo: rightEffect.trailingAnchor, constant: -20.0),
            heroCard.heightAnchor.constraint(equalToConstant: 70.0),

            emblem.leadingAnchor.constraint(equalTo: heroCard.leadingAnchor, constant: 16.0),
            emblem.centerYAnchor.constraint(equalTo: heroCard.centerYAnchor),
            emblem.widthAnchor.constraint(equalToConstant: 38.0),
            emblem.heightAnchor.constraint(equalToConstant: 38.0),

            sectionTitleLabel.topAnchor.constraint(equalTo: heroCard.topAnchor, constant: 13.0),
            sectionTitleLabel.leadingAnchor.constraint(equalTo: emblem.trailingAnchor, constant: 12.0),

            sectionSubtitleLabel.topAnchor.constraint(equalTo: sectionTitleLabel.bottomAnchor, constant: 2.0),
            sectionSubtitleLabel.leadingAnchor.constraint(equalTo: sectionTitleLabel.leadingAnchor),
            sectionSubtitleLabel.trailingAnchor.constraint(lessThanOrEqualTo: filterField.leadingAnchor, constant: -14.0),

            openPDFBtn.centerYAnchor.constraint(equalTo: heroCard.centerYAnchor),
            openPDFBtn.trailingAnchor.constraint(equalTo: heroCard.trailingAnchor, constant: -16.0),

            openFolderBtn.centerYAnchor.constraint(equalTo: heroCard.centerYAnchor),
            openFolderBtn.trailingAnchor.constraint(equalTo: openPDFBtn.leadingAnchor, constant: -8.0),

            filterField.centerYAnchor.constraint(equalTo: heroCard.centerYAnchor),
            filterField.trailingAnchor.constraint(equalTo: openFolderBtn.leadingAnchor, constant: -10.0),
            filterField.widthAnchor.constraint(equalToConstant: 148.0),

            documentScrollView.topAnchor.constraint(equalTo: heroCard.bottomAnchor, constant: 10.0),
            documentScrollView.leadingAnchor.constraint(equalTo: rightEffect.leadingAnchor, constant: 10.0),
            documentScrollView.trailingAnchor.constraint(equalTo: rightEffect.trailingAnchor, constant: -10.0),
            documentScrollView.bottomAnchor.constraint(equalTo: rightEffect.bottomAnchor, constant: -12.0),

            emptyStateView.topAnchor.constraint(equalTo: heroCard.bottomAnchor, constant: 10.0),
            emptyStateView.leadingAnchor.constraint(equalTo: rightEffect.leadingAnchor, constant: 24.0),
            emptyStateView.trailingAnchor.constraint(equalTo: rightEffect.trailingAnchor, constant: -24.0),
            emptyStateView.bottomAnchor.constraint(equalTo: rightEffect.bottomAnchor, constant: -24.0)
        ])

        let rightVC = NSViewController()
        rightVC.view = rightEffect
        let rightItem = NSSplitViewItem(viewController: rightVC)
        rightItem.minimumThickness = 500.0

        splitViewController.addSplitViewItem(sidebarItem)
        splitViewController.addSplitViewItem(rightItem)

        sidebarOutlineView.reloadData()
        sidebarOutlineView.selectRowIndexes(IndexSet(integer: 1), byExtendingSelection: false)
    }

    // MARK: - Data Loading & Filtering

    public func refreshContent() {
        tableHover.clear(in: documentTableView)
        sectionTitleLabel.stringValue = activeCategory == .home ? "Iridescence" : activeCategory.title
        sectionSubtitleLabel.stringValue = activeCategory.subtitleDescription

        // Keep active sidebar category selected visually
        if let activeRow = HomeSidebarItem.allCases.firstIndex(of: activeCategory) {
            if sidebarOutlineView.selectedRow != activeRow {
                sidebarOutlineView.selectRowIndexes(IndexSet(integer: activeRow), byExtendingSelection: false)
            }
        }

        let defaults = UserDefaults.standard
        let recentFiles = defaults.stringArray(forKey: SettingsKeys.recentFiles)
            ?? defaults.stringArray(forKey: SettingsKeys.recentItems)
            ?? []
        let recentWorkspaces = defaults.stringArray(forKey: SettingsKeys.recentWorkspaces) ?? []
        let favoriteList = defaults.stringArray(forKey: SettingsKeys.favoriteFiles) ?? []
        let favoriteSet = Set(favoriteList.map(UIConstants.canonicalPath))

        var entries: [HomeDocumentEntry] = []

        func makeFileEntry(_ rawPath: String) -> HomeDocumentEntry {
            let rawURL = URL(fileURLWithPath: rawPath)
            let logicalURL = ICloudCoordinator.logicalURL(for: rawURL)
            let title = logicalURL.deletingPathExtension().lastPathComponent
            let normPath = UIConstants.canonicalPath(logicalURL.path)
            let isFav = favoriteSet.contains(normPath)
            let cloudStatus = ICloudDownloadManager.shared.effectiveStatus(for: rawURL)

            var sizeDesc = ""
            if let attrs = try? FileManager.default.attributesOfItem(atPath: logicalURL.path),
               let bytes = attrs[.size] as? UInt64 {
                sizeDesc = ByteCountFormatter.string(fromByteCount: Int64(bytes), countStyle: .file)
            }

            let folderSubtitle = CommandPaletteController.formatPathSubtitle(logicalURL.path)
            let subtitle = sizeDesc.isEmpty ? folderSubtitle : "\(folderSubtitle) · \(sizeDesc)"

            var progressBadge = ""
            var lastRead: Double = 0
            let posKey = "PDFPos_\(normPath)"
            if let dict = defaults.dictionary(forKey: posKey) {
                let pageIdx = (dict["PageIndex"] as? Int) ?? 0
                let totalPages = (dict["TotalPages"] as? Int) ?? 0
                lastRead = (dict["LastReadTime"] as? Double) ?? 0
                if totalPages > 0 {
                    let pct = Int(((Double(pageIdx + 1) / Double(totalPages)) * 100.0).rounded())
                    progressBadge = "p.\(pageIdx + 1)/\(totalPages) · \(min(100, max(1, pct)))%"
                } else if pageIdx > 0 {
                    progressBadge = "Page \(pageIdx + 1)"
                }
            }
            if !cloudStatus.isReadyToOpen {
                progressBadge = cloudStatus.badgeText
            } else if progressBadge.isEmpty {
                progressBadge = "PDF"
            }

            return HomeDocumentEntry(
                path: rawPath,
                title: title.isEmpty ? rawPath : title,
                subtitle: subtitle,
                isWorkspaceFolder: false,
                isFavorite: isFav,
                progressBadge: progressBadge,
                iCloudStatus: cloudStatus,
                lastReadTime: lastRead
            )
        }

        func makeWorkspaceEntry(_ rawPath: String) -> HomeDocumentEntry {
            let url = URL(fileURLWithPath: rawPath)
            let title = url.lastPathComponent
            return HomeDocumentEntry(
                path: rawPath,
                title: title.isEmpty ? rawPath : title,
                subtitle: CommandPaletteController.formatPathSubtitle(rawPath),
                isWorkspaceFolder: true,
                isFavorite: false,
                progressBadge: "Workspace",
                iCloudStatus: .local,
                lastReadTime: 0
            )
        }

        switch activeCategory {
        case .home:
            for ws in recentWorkspaces.prefix(4) {
                entries.append(makeWorkspaceEntry(ws))
            }
            for f in recentFiles {
                entries.append(makeFileEntry(f))
            }

        case .continueReading:
            let allEntries = recentFiles.map(makeFileEntry).filter { $0.lastReadTime > 0 }
            entries = allEntries.sorted { $0.lastReadTime > $1.lastReadTime }
            if entries.isEmpty {
                entries = recentFiles.prefix(8).map(makeFileEntry)
            }

        case .recent:
            entries = recentFiles.map(makeFileEntry)

        case .favorites:
            entries = favoriteList.map(makeFileEntry)

        case .localFiles:
            entries = recentFiles.filter { !$0.contains("Mobile Documents") && !$0.contains("iCloud") }.map(makeFileEntry)

        case .iCloudDrive:
            entries = recentFiles.filter { $0.contains("Mobile Documents") || $0.contains("iCloud") || $0.hasSuffix(".icloud") }.map(makeFileEntry)

        case .workspaces:
            entries = recentWorkspaces.map(makeWorkspaceEntry)

        default:
            break
        }

        let query = filterText.trimmingCharacters(in: .whitespacesAndNewlines)
        if !query.isEmpty {
            entries = entries.filter {
                $0.title.localizedCaseInsensitiveContains(query) || $0.subtitle.localizedCaseInsensitiveContains(query)
            }
        }

        displayedEntries = entries
        documentTableView.reloadData()

        let isEmpty = displayedEntries.isEmpty
        documentScrollView.isHidden = isEmpty
        emptyStateView.isHidden = !isEmpty
        if isEmpty {
            switch activeCategory {
            case .favorites:
                emptyStateView.updateContent(
                    symbolName: "star.circle",
                    title: "No favorite documents yet",
                    subtitle: "Click the star icon next to any recent PDF to pin it to Favorites"
                )
            case .iCloudDrive:
                emptyStateView.updateContent(
                    symbolName: "icloud.and.arrow.down",
                    title: "No iCloud Drive PDFs in recent list",
                    subtitle: "Open a PDF or workspace stored in iCloud Drive to see it here"
                )
            case .workspaces:
                emptyStateView.updateContent(
                    symbolName: "square.grid.2x2",
                    title: "No workspaces opened yet",
                    subtitle: "Click “Open Folder…” above to mount a project directory"
                )
            default:
                emptyStateView.updateContent(
                    symbolName: "sparkles.rectangle.stack",
                    title: query.isEmpty ? "Ready to Read" : "No matching documents",
                    subtitle: query.isEmpty
                        ? "Click “Open PDF…” or “Open Folder…” above, or drag a PDF or folder into this window"
                        : "Try clearing the filter text above"
                )
            }
        }
    }

    // MARK: - Sidebar NSOutlineViewDataSource & Delegate

    public func outlineView(_ outlineView: NSOutlineView, numberOfChildrenOfItem item: Any?) -> Int {
        item == nil ? HomeSidebarItem.allCases.count : 0
    }

    public func outlineView(_ outlineView: NSOutlineView, isItemExpandable item: Any) -> Bool {
        false
    }

    public func outlineView(_ outlineView: NSOutlineView, child index: Int, ofItem item: Any?) -> Any {
        let all = HomeSidebarItem.allCases
        guard index >= 0, index < all.count else { return NSNumber(value: 1) }
        return NSNumber(value: all[index].rawValue)
    }

    public func outlineView(_ outlineView: NSOutlineView, shouldSelectItem item: Any) -> Bool {
        guard let num = item as? NSNumber, let cat = HomeSidebarItem(rawValue: num.intValue) else { return false }
        return !cat.isHeader
    }

    public func outlineView(_ outlineView: NSOutlineView, heightOfRowByItem item: Any) -> CGFloat {
        guard let num = item as? NSNumber, let cat = HomeSidebarItem(rawValue: num.intValue) else {
            return UIConstants.rowHeightSidebar
        }
        return cat.isHeader ? 26.0 : UIConstants.rowHeightSidebar
    }

    public func outlineView(_ outlineView: NSOutlineView, rowViewForItem item: Any) -> NSTableRowView? {
        GlassTableRowView()
    }

    public func outlineView(_ outlineView: NSOutlineView, viewFor tableColumn: NSTableColumn?, item: Any) -> NSView? {
        guard let num = item as? NSNumber, let cat = HomeSidebarItem(rawValue: num.intValue) else { return nil }

        if cat.isHeader {
            let headerCell = NSView(frame: NSRect(x: 0, y: 0, width: 190, height: 26))
            let label = NSTextField(labelWithString: cat.title)
            label.font = .systemFont(ofSize: 10.0, weight: .bold)
            label.textColor = IridescenceDesign.primaryViolet.withAlphaComponent(0.75)
            label.translatesAutoresizingMaskIntoConstraints = false
            headerCell.addSubview(label)
            NSLayoutConstraint.activate([
                label.leadingAnchor.constraint(equalTo: headerCell.leadingAnchor, constant: 10.0),
                label.bottomAnchor.constraint(equalTo: headerCell.bottomAnchor, constant: -3.0)
            ])
            return headerCell
        }

        let cell = NSView(frame: NSRect(x: 0, y: 0, width: 190, height: 32))
        let iconView = NSImageView()
        iconView.translatesAutoresizingMaskIntoConstraints = false
        if let img = NSImage(systemSymbolName: cat.symbolName, accessibilityDescription: cat.title) {
            img.isTemplate = true
            iconView.image = img
            iconView.contentTintColor = cat.iconColor
        }

        let isSelected = (cat == activeCategory)
        let titleField = NSTextField(labelWithString: cat.title)
        titleField.font = .systemFont(ofSize: 13.0, weight: isSelected ? .semibold : .medium)
        titleField.textColor = .labelColor
        titleField.translatesAutoresizingMaskIntoConstraints = false

        cell.addSubview(iconView)
        cell.addSubview(titleField)

        NSLayoutConstraint.activate([
            iconView.leadingAnchor.constraint(equalTo: cell.leadingAnchor, constant: 10.0),
            iconView.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: 16.0),
            iconView.heightAnchor.constraint(equalToConstant: 16.0),

            titleField.leadingAnchor.constraint(equalTo: iconView.trailingAnchor, constant: 8.0),
            titleField.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
            titleField.trailingAnchor.constraint(lessThanOrEqualTo: cell.trailingAnchor, constant: -8.0)
        ])

        return cell
    }

    @objc private func sidebarClicked(_ sender: Any?) {
        let row = sidebarOutlineView.clickedRow >= 0 ? sidebarOutlineView.clickedRow : sidebarOutlineView.selectedRow
        guard row >= 0,
              let num = sidebarOutlineView.item(atRow: row) as? NSNumber,
              let cat = HomeSidebarItem(rawValue: num.intValue),
              !cat.isHeader else { return }
        activeCategory = cat
        sidebarOutlineView.reloadData()
        sidebarOutlineView.selectRowIndexes(IndexSet(integer: row), byExtendingSelection: false)
        refreshContent()
    }

    public func outlineViewSelectionDidChange(_ notification: Notification) {
        let row = sidebarOutlineView.selectedRow
        guard row >= 0,
              let num = sidebarOutlineView.item(atRow: row) as? NSNumber,
              let cat = HomeSidebarItem(rawValue: num.intValue),
              !cat.isHeader else { return }
        if activeCategory != cat {
            activeCategory = cat
            sidebarOutlineView.reloadData()
            sidebarOutlineView.selectRowIndexes(IndexSet(integer: row), byExtendingSelection: false)
            refreshContent()
        }
    }

    // MARK: - Document Table NSTableViewDataSource & Delegate

    public func numberOfRows(in tableView: NSTableView) -> Int {
        displayedEntries.count
    }

    public func tableView(_ tableView: NSTableView, rowViewForRow row: Int) -> NSTableRowView? {
        let rowView = GlassTableRowView()
        if row == tableHover.currentRow {
            rowView.isHovered = true
        }
        return rowView
    }

    public func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        guard row >= 0, row < displayedEntries.count else { return nil }
        let entry = displayedEntries[row]

        let cell = NSView(frame: NSRect(x: 0, y: 0, width: 640, height: 46))

        let iconBadge = HomeRowIconBadgeView(frame: NSRect(x: 0, y: 0, width: 32, height: 32))
        let sym: String
        let tint: NSColor
        if entry.isWorkspaceFolder {
            sym = "folder.fill"
            tint = IridescenceDesign.accentRose
        } else if !entry.iCloudStatus.isReadyToOpen {
            sym = "icloud.and.arrow.down"
            tint = IridescenceDesign.accentCyan
        } else {
            sym = "doc.richtext.fill"
            tint = IridescenceDesign.primaryViolet
        }
        iconBadge.tintColor = tint
        if let img = NSImage(systemSymbolName: sym, accessibilityDescription: nil) {
            img.isTemplate = true
            iconBadge.imageView.image = img
        }

        let titleField = NSTextField(labelWithString: entry.title)
        titleField.font = .systemFont(ofSize: 13.5, weight: .semibold)
        titleField.textColor = .labelColor
        titleField.lineBreakMode = .byTruncatingTail
        titleField.maximumNumberOfLines = 1
        titleField.usesSingleLineMode = true
        titleField.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        titleField.translatesAutoresizingMaskIntoConstraints = false

        let subtitleField = NSTextField(labelWithString: entry.subtitle)
        subtitleField.font = .systemFont(ofSize: 11.5, weight: .regular)
        subtitleField.textColor = .secondaryLabelColor
        subtitleField.lineBreakMode = .byTruncatingMiddle
        subtitleField.maximumNumberOfLines = 1
        subtitleField.usesSingleLineMode = true
        subtitleField.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        subtitleField.translatesAutoresizingMaskIntoConstraints = false

        let badgeStyle: IridescentPillBadge.Style = {
            if entry.isWorkspaceFolder { return .rose }
            if entry.iCloudStatus == .failed { return .error }
            if !entry.iCloudStatus.isReadyToOpen { return .cyan }
            if entry.progressBadge.contains("%") { return .violet }
            return .neutral
        }()
        let pillBadge = IridescentPillBadge(text: entry.progressBadge, style: badgeStyle)

        let favBtn = NSButton(title: "", target: self, action: #selector(toggleFavoriteClicked(_:)))
        favBtn.tag = row
        favBtn.isBordered = false
        favBtn.translatesAutoresizingMaskIntoConstraints = false
        if entry.isWorkspaceFolder {
            favBtn.isHidden = false
            if let chevImg = NSImage(systemSymbolName: "chevron.right", accessibilityDescription: "Open Workspace") {
                favBtn.image = chevImg
                favBtn.contentTintColor = .tertiaryLabelColor
            }
        } else {
            favBtn.isHidden = false
            let starName = entry.isFavorite ? "star.fill" : "star"
            if let starImg = NSImage(systemSymbolName: starName, accessibilityDescription: "Toggle Favorite") {
                favBtn.image = starImg
                favBtn.contentTintColor = entry.isFavorite ? IridescenceDesign.accentAmber : .tertiaryLabelColor
            }
        }

        cell.addSubview(iconBadge)
        cell.addSubview(titleField)
        cell.addSubview(subtitleField)
        cell.addSubview(pillBadge)
        cell.addSubview(favBtn)

        NSLayoutConstraint.activate([
            iconBadge.leadingAnchor.constraint(equalTo: cell.leadingAnchor, constant: 12.0),
            iconBadge.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
            iconBadge.widthAnchor.constraint(equalToConstant: 32.0),
            iconBadge.heightAnchor.constraint(equalToConstant: 32.0),

            titleField.leadingAnchor.constraint(equalTo: iconBadge.trailingAnchor, constant: 12.0),
            titleField.topAnchor.constraint(equalTo: cell.topAnchor, constant: 6.0),
            titleField.trailingAnchor.constraint(lessThanOrEqualTo: pillBadge.leadingAnchor, constant: -12.0),

            subtitleField.leadingAnchor.constraint(equalTo: titleField.leadingAnchor),
            subtitleField.topAnchor.constraint(equalTo: titleField.bottomAnchor, constant: 1.0),
            subtitleField.trailingAnchor.constraint(lessThanOrEqualTo: pillBadge.leadingAnchor, constant: -12.0),

            pillBadge.trailingAnchor.constraint(equalTo: favBtn.leadingAnchor, constant: -10.0),
            pillBadge.centerYAnchor.constraint(equalTo: cell.centerYAnchor),

            favBtn.trailingAnchor.constraint(equalTo: cell.trailingAnchor, constant: -12.0),
            favBtn.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
            favBtn.widthAnchor.constraint(equalToConstant: 20.0),
            favBtn.heightAnchor.constraint(equalToConstant: 20.0)
        ])

        return cell
    }

    // MARK: - Actions & Context Menu

    @objc private func openFolderClicked(_ sender: Any?) {
        onOpenFolder?()
    }

    @objc private func openPDFClicked(_ sender: Any?) {
        onOpenPDF?()
    }

    @objc private func documentRowClicked(_ sender: Any?) {
        let row = documentTableView.clickedRow >= 0 ? documentTableView.clickedRow : documentTableView.selectedRow
        openEntryAtRow(row)
    }

    @objc private func documentRowDoubleClicked(_ sender: Any?) {
        let row = documentTableView.clickedRow >= 0 ? documentTableView.clickedRow : documentTableView.selectedRow
        openEntryAtRow(row)
    }

    private func openEntryAtRow(_ row: Int) {
        guard row >= 0, row < displayedEntries.count else { return }
        let entry = displayedEntries[row]
        let url = URL(fileURLWithPath: entry.path)
        if entry.isWorkspaceFolder {
            onSelectWorkspaceURL?(url)
        } else {
            onSelectFileURL?(url)
        }
    }

    @objc private func toggleFavoriteClicked(_ sender: NSButton) {
        let row = sender.tag
        guard row >= 0, row < displayedEntries.count else { return }
        let entry = displayedEntries[row]
        if entry.isWorkspaceFolder {
            onSelectWorkspaceURL?(URL(fileURLWithPath: entry.path))
        } else {
            toggleFavorite(forPath: entry.path)
        }
    }

    public func toggleFavorite(forPath rawPath: String) {
        let normTarget = UIConstants.canonicalPath(rawPath)
        let defaults = UserDefaults.standard
        var favs = defaults.stringArray(forKey: SettingsKeys.favoriteFiles) ?? []
        if let idx = favs.firstIndex(where: { UIConstants.canonicalPath($0) == normTarget }) {
            favs.remove(at: idx)
        } else {
            favs.insert(rawPath, at: 0)
        }
        defaults.set(favs, forKey: SettingsKeys.favoriteFiles)
        refreshContent()
    }

    public func menuNeedsUpdate(_ menu: NSMenu) {
        menu.removeAllItems()
        let row = documentTableView.clickedRow >= 0 ? documentTableView.clickedRow : documentTableView.selectedRow
        guard row >= 0, row < displayedEntries.count else { return }
        let entry = displayedEntries[row]

        let openItem = NSMenuItem(title: entry.isWorkspaceFolder ? "Open Workspace" : "Open PDF", action: #selector(ctxOpen(_:)), keyEquivalent: "")
        openItem.target = self
        openItem.tag = row
        menu.addItem(openItem)

        if !entry.isWorkspaceFolder {
            let favTitle = entry.isFavorite ? "Remove from Favorites" : "Add to Favorites"
            let favItem = NSMenuItem(title: favTitle, action: #selector(ctxToggleFavorite(_:)), keyEquivalent: "")
            favItem.target = self
            favItem.tag = row
            menu.addItem(favItem)
        }

        menu.addItem(.separator())

        let revealItem = NSMenuItem(title: "Reveal in Finder", action: #selector(ctxRevealInFinder(_:)), keyEquivalent: "")
        revealItem.target = self
        revealItem.tag = row
        menu.addItem(revealItem)

        let copyItem = NSMenuItem(title: "Copy Path", action: #selector(ctxCopyPath(_:)), keyEquivalent: "")
        copyItem.target = self
        copyItem.tag = row
        menu.addItem(copyItem)

        menu.addItem(.separator())

        let removeItem = NSMenuItem(title: "Remove from Recent", action: #selector(ctxRemoveRecent(_:)), keyEquivalent: "")
        removeItem.target = self
        removeItem.tag = row
        menu.addItem(removeItem)
    }

    @objc private func ctxOpen(_ sender: NSMenuItem) {
        openEntryAtRow(sender.tag)
    }

    @objc private func ctxToggleFavorite(_ sender: NSMenuItem) {
        guard sender.tag >= 0, sender.tag < displayedEntries.count else { return }
        toggleFavorite(forPath: displayedEntries[sender.tag].path)
    }

    @objc private func ctxRevealInFinder(_ sender: NSMenuItem) {
        guard sender.tag >= 0, sender.tag < displayedEntries.count else { return }
        let url = URL(fileURLWithPath: displayedEntries[sender.tag].path)
        NSWorkspace.shared.activateFileViewerSelecting([url])
    }

    @objc private func ctxCopyPath(_ sender: NSMenuItem) {
        guard sender.tag >= 0, sender.tag < displayedEntries.count else { return }
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(displayedEntries[sender.tag].path, forType: .string)
    }

    @objc private func ctxRemoveRecent(_ sender: NSMenuItem) {
        guard sender.tag >= 0, sender.tag < displayedEntries.count else { return }
        let entry = displayedEntries[sender.tag]
        let norm = UIConstants.canonicalPath(entry.path)
        let defaults = UserDefaults.standard
        if entry.isWorkspaceFolder {
            var ws = defaults.stringArray(forKey: SettingsKeys.recentWorkspaces) ?? []
            ws.removeAll { UIConstants.canonicalPath($0) == norm }
            defaults.set(ws, forKey: SettingsKeys.recentWorkspaces)
        } else {
            var rf = defaults.stringArray(forKey: SettingsKeys.recentFiles) ?? []
            rf.removeAll { UIConstants.canonicalPath($0) == norm }
            defaults.set(rf, forKey: SettingsKeys.recentFiles)
            defaults.set(rf, forKey: SettingsKeys.recentItems)
        }
        refreshContent()
    }

    // MARK: - NSTextFieldDelegate & Hover Tracking

    public func controlTextDidChange(_ obj: Notification) {
        filterText = filterField.stringValue
        refreshContent()
    }

    @objc private func scrollBoundsChanged(_ notification: Notification) {
        tableHover.clear(in: documentTableView)
    }

    @objc public func mouseMoved(with event: NSEvent) {
        tableHover.handleMouseMoved(with: event, in: documentTableView)
    }

    @objc public func mouseExited(with event: NSEvent) {
        tableHover.clear(in: documentTableView)
    }
}
