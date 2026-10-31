import AppKit
import Foundation
@preconcurrency import Dispatch

@MainActor
public final class WorkspaceOutlineController: NSObject, NSOutlineViewDataSource, NSOutlineViewDelegate, NSMenuDelegate {
    public let outlineView: NSOutlineView
    public let scrollView: NSScrollView
    public let containerView: NSVisualEffectView
    public let headerLabel: NSTextField
    public let countBadge: IridescentPillBadge
    public let emptyStateView: EmptyStateView

    public private(set) var rootNode: WorkspaceNode?
    public var onSelectPDF: ((URL) -> Void)?
    public var onRequestICloudDownload: ((URL) -> Void)?

    private let hoverCoordinator = HoverCoordinator()
    private var suppressSelectionCallback = false
    private var directoryWatchSource: DispatchSourceFileSystemObject?
    private var directoryFileDescriptor: Int32 = -1
    private var debounceReloadWorkItem: DispatchWorkItem?

    public override init() {
        self.outlineView = NSOutlineView(frame: NSRect(x: 0, y: 0, width: 230, height: 680))
        self.scrollView = NSScrollView(frame: NSRect(x: 0, y: 0, width: 230, height: 680))
        self.containerView = NSVisualEffectView(frame: NSRect(x: 0, y: 0, width: 230, height: 740))
        self.headerLabel = NSTextField(labelWithString: "WORKSPACE")
        self.countBadge = IridescentPillBadge(text: "", style: .violet)
        self.emptyStateView = EmptyStateView(
            symbolName: "folder.badge.questionmark",
            title: "No PDFs in Workspace",
            subtitle: "Add PDF files to this folder in Finder to browse them here"
        )
        super.init()
        setupViews()
    }

    deinit {
        directoryWatchSource?.cancel()
    }

    private func setupViews() {
        let wash = IridescentSurfaceWashView(role: .sidebar)

        let col = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("WorkspaceColumn"))
        col.width = 210.0
        col.resizingMask = .autoresizingMask
        outlineView.addTableColumn(col)
        outlineView.outlineTableColumn = col
        outlineView.columnAutoresizingStyle = .uniformColumnAutoresizingStyle
        outlineView.headerView = nil
        outlineView.backgroundColor = .clear
        outlineView.rowHeight = UIConstants.rowHeightCompact
        outlineView.indentationPerLevel = 13.0
        outlineView.style = .sourceList
        outlineView.delegate = self
        outlineView.dataSource = self
        outlineView.target = self
        outlineView.action = #selector(workspaceClicked(_:))
        outlineView.doubleAction = #selector(workspaceDoubleClicked(_:))

        let menu = NSMenu(title: "WorkspaceContext")
        menu.delegate = self
        outlineView.menu = menu

        let trackingArea = NSTrackingArea(
            rect: outlineView.bounds,
            options: [.mouseEnteredAndExited, .mouseMoved, .activeInKeyWindow, .inVisibleRect],
            owner: self,
            userInfo: nil
        )
        outlineView.addTrackingArea(trackingArea)

        scrollView.hasVerticalScroller = true
        scrollView.documentView = outlineView
        scrollView.drawsBackground = false
        scrollView.translatesAutoresizingMaskIntoConstraints = false
        scrollView.contentView.postsBoundsChangedNotifications = true

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(scrollBoundsChanged(_:)),
            name: NSView.boundsDidChangeNotification,
            object: scrollView.contentView
        )

        let headerIcon = NSImageView()
        headerIcon.translatesAutoresizingMaskIntoConstraints = false
        if let fImg = NSImage(systemSymbolName: "folder.fill.badge.gearshape", accessibilityDescription: nil) {
            fImg.isTemplate = true
            headerIcon.image = fImg
            headerIcon.contentTintColor = IridescenceDesign.primaryViolet
        }

        headerLabel.font = .systemFont(ofSize: 11.0, weight: .bold)
        headerLabel.textColor = IridescenceDesign.primaryViolet
        headerLabel.lineBreakMode = .byTruncatingTail
        headerLabel.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        headerLabel.translatesAutoresizingMaskIntoConstraints = false

        countBadge.isHidden = true
        emptyStateView.isHidden = true

        containerView.material = .sidebar
        containerView.blendingMode = .behindWindow
        containerView.state = .followsWindowActiveState
        containerView.autoresizingMask = [.width, .height]

        containerView.addSubview(wash)
        containerView.addSubview(headerIcon)
        containerView.addSubview(headerLabel)
        containerView.addSubview(countBadge)
        containerView.addSubview(scrollView)
        containerView.addSubview(emptyStateView)

        NSLayoutConstraint.activate([
            wash.topAnchor.constraint(equalTo: containerView.topAnchor),
            wash.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            wash.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),
            wash.bottomAnchor.constraint(equalTo: containerView.bottomAnchor),

            headerIcon.topAnchor.constraint(equalTo: containerView.topAnchor, constant: 54.0),
            headerIcon.leadingAnchor.constraint(equalTo: containerView.leadingAnchor, constant: 14.0),
            headerIcon.widthAnchor.constraint(equalToConstant: 14.0),
            headerIcon.heightAnchor.constraint(equalToConstant: 14.0),

            headerLabel.centerYAnchor.constraint(equalTo: headerIcon.centerYAnchor),
            headerLabel.leadingAnchor.constraint(equalTo: headerIcon.trailingAnchor, constant: 6.0),
            headerLabel.trailingAnchor.constraint(lessThanOrEqualTo: countBadge.leadingAnchor, constant: -6.0),

            countBadge.centerYAnchor.constraint(equalTo: headerIcon.centerYAnchor),
            countBadge.trailingAnchor.constraint(equalTo: containerView.trailingAnchor, constant: -12.0),

            scrollView.topAnchor.constraint(equalTo: headerIcon.bottomAnchor, constant: 8.0),
            scrollView.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),
            scrollView.bottomAnchor.constraint(equalTo: containerView.bottomAnchor),

            emptyStateView.topAnchor.constraint(equalTo: headerIcon.bottomAnchor, constant: 8.0),
            emptyStateView.leadingAnchor.constraint(equalTo: containerView.leadingAnchor, constant: 8.0),
            emptyStateView.trailingAnchor.constraint(equalTo: containerView.trailingAnchor, constant: -8.0),
            emptyStateView.bottomAnchor.constraint(equalTo: containerView.bottomAnchor, constant: -8.0)
        ])
    }

    public func loadWorkspace(rootURL: URL) {
        hoverCoordinator.clear(in: outlineView)
        let folderName = rootURL.lastPathComponent.isEmpty ? "WORKSPACE" : rootURL.lastPathComponent.uppercased()
        headerLabel.stringValue = folderName

        let root = WorkspaceNode(
            name: rootURL.lastPathComponent,
            path: rootURL.path,
            isDirectory: true
        )
        root.loadChildrenSynchronously()
        applyActiveDownloadStates(in: root)
        self.rootNode = root
        outlineView.reloadData()

        let isEmpty = root.children.isEmpty
        scrollView.isHidden = isEmpty
        emptyStateView.isHidden = !isEmpty
        countBadge.stringValue = isEmpty ? "0" : "\(root.children.count)"

        startDirectoryWatcher(for: rootURL.path)
        AppLogger.shared.log(.workspace, "Loaded workspace tree at \(rootURL.path) (\(root.children.count) top-level items)")
    }

    public func reloadCurrentWorkspace() {
        guard let root = rootNode else { return }
        let expandedPaths = collectExpandedDirectoryPaths()
        let selectedKey = (outlineView.item(atRow: outlineView.selectedRow) as? WorkspaceNode).map {
            ICloudCoordinator.canonicalLogicalKey(for: URL(fileURLWithPath: $0.path))
        }

        root.loadChildrenSynchronously()
        applyActiveDownloadStates(in: root)
        outlineView.reloadData()

        let isEmpty = root.children.isEmpty
        scrollView.isHidden = isEmpty
        emptyStateView.isHidden = !isEmpty
        countBadge.stringValue = isEmpty ? "0" : "\(root.children.count)"

        suppressSelectionCallback = true
        defer { suppressSelectionCallback = false }

        restoreExpandedDirectories(expandedPaths, in: root)
        if let key = selectedKey {
            selectNodeWithLogicalKey(key)
        }
    }

    /// Synchronizes any active or completed `ICloudDownloadManager` sessions into a newly scanned `WorkspaceNode` tree.
    private func applyActiveDownloadStates(in parent: WorkspaceNode) {
        for child in parent.children {
            let key = ICloudCoordinator.canonicalLogicalKey(for: URL(fileURLWithPath: child.path))
            if let session = ICloudDownloadManager.shared.sessions[key] {
                child.iCloudStatus = session.status
                if session.status == .downloadedVerified {
                    child.path = session.logicalURL.path
                }
            }
            if child.isDirectory && child.isLoaded {
                applyActiveDownloadStates(in: child)
            }
        }
    }

    /// Updates matching workspace nodes in-place when `ICloudDownloadManager` reports a state transition.
    public func updateNode(for session: ICloudDownloadSession) {
        guard let root = rootNode else { return }

        func updateInTree(_ parent: WorkspaceNode) -> Bool {
            var found = false
            for child in parent.children {
                let childKey = ICloudCoordinator.canonicalLogicalKey(for: URL(fileURLWithPath: child.path))
                if childKey == session.canonicalKey {
                    child.iCloudStatus = session.status
                    if session.status == .downloadedVerified || session.status == .local {
                        child.path = session.logicalURL.path
                    }
                    outlineView.reloadItem(child)
                    found = true
                }
                if child.isDirectory && child.isLoaded {
                    if updateInTree(child) {
                        found = true
                    }
                }
            }
            return found
        }

        _ = updateInTree(root)
    }

    // MARK: - External File-System Synchronization (`DispatchSourceFileSystemObject`)

    private func startDirectoryWatcher(for dirPath: String) {
        directoryWatchSource?.cancel()
        directoryWatchSource = nil

        let fd = open(dirPath, O_EVTONLY)
        guard fd >= 0 else { return }
        directoryFileDescriptor = fd

        let source = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: [.write, .rename, .delete, .attrib, .extend],
            queue: .main
        )
        source.setEventHandler { [weak self] in
            Task { @MainActor [weak self] in
                self?.scheduleDebouncedReload()
            }
        }
        source.setCancelHandler {
            close(fd)
        }
        source.resume()
        directoryWatchSource = source
    }

    private func scheduleDebouncedReload() {
        debounceReloadWorkItem?.cancel()
        let item = DispatchWorkItem { [weak self] in
            Task { @MainActor [weak self] in
                self?.reloadCurrentWorkspace()
            }
        }
        debounceReloadWorkItem = item
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.18, execute: item)
    }

    private func collectExpandedDirectoryPaths() -> Set<String> {
        var expanded: Set<String> = []
        let rowCount = outlineView.numberOfRows
        for r in 0..<rowCount {
            if let node = outlineView.item(atRow: r) as? WorkspaceNode,
               node.isDirectory,
               outlineView.isItemExpanded(node) {
                expanded.insert(UIConstants.canonicalPath(node.path))
            }
        }
        return expanded
    }

    private func restoreExpandedDirectories(_ expandedPaths: Set<String>, in parent: WorkspaceNode) {
        for child in parent.children where child.isDirectory {
            let canon = UIConstants.canonicalPath(child.path)
            if expandedPaths.contains(canon) {
                if !child.isLoaded {
                    child.loadChildrenSynchronously()
                    applyActiveDownloadStates(in: child)
                }
                outlineView.expandItem(child)
                restoreExpandedDirectories(expandedPaths, in: child)
            }
        }
    }

    private func selectNodeWithLogicalKey(_ targetKey: String) {
        let rowCount = outlineView.numberOfRows
        for r in 0..<rowCount {
            if let node = outlineView.item(atRow: r) as? WorkspaceNode {
                let key = ICloudCoordinator.canonicalLogicalKey(for: URL(fileURLWithPath: node.path))
                if key == targetKey {
                    outlineView.selectRowIndexes(IndexSet(integer: r), byExtendingSelection: false)
                    return
                }
            }
        }
    }

    /// Expands parent folders and highlights the row corresponding to `filePath` in the workspace tree.
    public func revealFile(atPath filePath: String) {
        guard let root = rootNode else { return }
        let targetCanon = ICloudCoordinator.canonicalLogicalKey(for: URL(fileURLWithPath: filePath))
        guard !targetCanon.isEmpty else { return }

        func findChain(in current: WorkspaceNode, depth: Int) -> [WorkspaceNode]? {
            guard depth <= WorkspaceNode.maxRecursionDepth else { return nil }
            if !current.isLoaded && current.isDirectory {
                current.loadChildrenSynchronously()
                applyActiveDownloadStates(in: current)
            }
            for child in current.children {
                let childCanon = ICloudCoordinator.canonicalLogicalKey(for: URL(fileURLWithPath: child.path))
                if childCanon == targetCanon {
                    return [child]
                }
                if child.isDirectory && targetCanon.hasPrefix(childCanon + "/") {
                    if let subChain = findChain(in: child, depth: depth + 1) {
                        return [child] + subChain
                    }
                }
            }
            return nil
        }

        guard let chain = findChain(in: root, depth: 0), !chain.isEmpty else { return }
        outlineView.reloadData()

        suppressSelectionCallback = true
        defer { suppressSelectionCallback = false }

        for i in 0..<(chain.count - 1) {
            outlineView.expandItem(chain[i], expandChildren: false)
        }
        outlineView.layoutSubtreeIfNeeded()
        if let leaf = chain.last {
            let row = outlineView.row(forItem: leaf)
            if row >= 0 {
                outlineView.selectRowIndexes(IndexSet(integer: row), byExtendingSelection: false)
                outlineView.scrollRowToVisible(row)
            }
        }
    }

    // MARK: - NSOutlineViewDataSource & Delegate

    public func outlineView(_ outlineView: NSOutlineView, numberOfChildrenOfItem item: Any?) -> Int {
        if item == nil {
            return rootNode?.children.count ?? 0
        }
        guard let node = item as? WorkspaceNode, node.isDirectory else { return 0 }
        if !node.isLoaded {
            node.loadChildrenSynchronously()
            applyActiveDownloadStates(in: node)
        }
        return node.children.count
    }

    public func outlineView(_ outlineView: NSOutlineView, isItemExpandable item: Any) -> Bool {
        guard let node = item as? WorkspaceNode else { return false }
        return node.isDirectory
    }

    public func outlineView(_ outlineView: NSOutlineView, child index: Int, ofItem item: Any?) -> Any {
        if item == nil {
            guard let children = rootNode?.children, index >= 0, index < children.count else {
                return NSObject()
            }
            return children[index]
        }
        guard let node = item as? WorkspaceNode else { return NSObject() }
        if !node.isLoaded && node.isDirectory {
            node.loadChildrenSynchronously()
            applyActiveDownloadStates(in: node)
        }
        guard index >= 0, index < node.children.count else { return NSObject() }
        return node.children[index]
    }

    public func outlineView(_ outlineView: NSOutlineView, rowViewForItem item: Any) -> NSTableRowView? {
        let rowView = GlassTableRowView()
        let idx = outlineView.row(forItem: item)
        if idx >= 0 && idx == hoverCoordinator.currentRow {
            rowView.isHovered = true
        }
        return rowView
    }

    public func outlineView(_ outlineView: NSOutlineView, viewFor tableColumn: NSTableColumn?, item: Any) -> NSView? {
        guard let node = item as? WorkspaceNode else { return nil }

        let cell = NSView(frame: NSRect(x: 0, y: 0, width: 210, height: 28))

        let iconView = NSImageView()
        iconView.translatesAutoresizingMaskIntoConstraints = false
        let symbolName = node.isDirectory ? "folder.fill" : "doc.richtext.fill"
        if let img = NSImage(systemSymbolName: symbolName, accessibilityDescription: nil) {
            img.isTemplate = true
            iconView.image = img
            if node.isDirectory {
                iconView.contentTintColor = IridescenceDesign.primaryViolet
            } else if node.iCloudStatus == .cloudOnly {
                iconView.contentTintColor = IridescenceDesign.accentCyan
            } else if node.iCloudStatus == .failed || node.iCloudStatus == .unavailable {
                iconView.contentTintColor = .systemRed
            } else {
                iconView.contentTintColor = IridescenceDesign.secondaryIndigo.withAlphaComponent(0.85)
            }
        }

        let displayName = node.isDirectory ? node.name : URL(fileURLWithPath: node.name).deletingPathExtension().lastPathComponent
        let label = NSTextField(labelWithString: displayName.isEmpty ? node.name : displayName)
        label.font = .systemFont(ofSize: 12.5, weight: node.isDirectory ? .semibold : .regular)
        label.textColor = (node.iCloudStatus == .cloudOnly) ? .secondaryLabelColor : .labelColor
        label.lineBreakMode = .byTruncatingTail
        label.maximumNumberOfLines = 1
        label.usesSingleLineMode = true
        label.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        label.toolTip = node.name
        label.translatesAutoresizingMaskIntoConstraints = false

        cell.addSubview(iconView)
        cell.addSubview(label)

        var trailingConstraint = label.trailingAnchor.constraint(lessThanOrEqualTo: cell.trailingAnchor, constant: -8.0)

        if node.iCloudStatus.isInFlight {
            let spinner = NSProgressIndicator()
            spinner.style = .spinning
            spinner.controlSize = .small
            spinner.isIndeterminate = true
            spinner.startAnimation(nil)
            spinner.toolTip = "Downloading “\(node.name)” from iCloud Drive…"
            spinner.translatesAutoresizingMaskIntoConstraints = false
            cell.addSubview(spinner)

            trailingConstraint = label.trailingAnchor.constraint(lessThanOrEqualTo: spinner.leadingAnchor, constant: -6.0)
            NSLayoutConstraint.activate([
                spinner.trailingAnchor.constraint(equalTo: cell.trailingAnchor, constant: -8.0),
                spinner.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
                spinner.widthAnchor.constraint(equalToConstant: 14.0),
                spinner.heightAnchor.constraint(equalToConstant: 14.0)
            ])
        } else if node.iCloudStatus != .local && node.iCloudStatus != .downloadedVerified {
            let statusIcon = NSImageView()
            statusIcon.translatesAutoresizingMaskIntoConstraints = false
            let (cloudSymbol, tint, tip): (String, NSColor, String) = {
                switch node.iCloudStatus {
                case .cloudOnly:
                    return ("icloud.and.arrow.down", IridescenceDesign.accentCyan, "Stored in iCloud Drive — Click to download and open")
                case .failed:
                    return ("exclamationmark.icloud.fill", .systemRed, "iCloud download failed — Click to retry")
                case .unavailable:
                    return ("xmark.icloud.fill", .systemOrange, "File unavailable")
                default:
                    return ("icloud", .secondaryLabelColor, "")
                }
            }()
            if let cImg = NSImage(systemSymbolName: cloudSymbol, accessibilityDescription: tip) {
                cImg.isTemplate = true
                statusIcon.image = cImg
                statusIcon.contentTintColor = tint
            }
            statusIcon.toolTip = tip
            cell.addSubview(statusIcon)
            trailingConstraint = label.trailingAnchor.constraint(lessThanOrEqualTo: statusIcon.leadingAnchor, constant: -6.0)
            NSLayoutConstraint.activate([
                statusIcon.trailingAnchor.constraint(equalTo: cell.trailingAnchor, constant: -8.0),
                statusIcon.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
                statusIcon.widthAnchor.constraint(equalToConstant: 15.0),
                statusIcon.heightAnchor.constraint(equalToConstant: 15.0)
            ])
        }

        NSLayoutConstraint.activate([
            iconView.leadingAnchor.constraint(equalTo: cell.leadingAnchor, constant: 4.0),
            iconView.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: 15.0),
            iconView.heightAnchor.constraint(equalToConstant: 15.0),

            label.leadingAnchor.constraint(equalTo: iconView.trailingAnchor, constant: 6.0),
            label.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
            trailingConstraint
        ])

        return cell
    }

    // MARK: - Click, Selection & iCloud Auto-Open Handling

    @objc private func workspaceClicked(_ sender: Any?) {
        let row = outlineView.clickedRow >= 0 ? outlineView.clickedRow : outlineView.selectedRow
        activateRow(row, toggleFolders: true)
    }

    @objc private func workspaceDoubleClicked(_ sender: Any?) {
        let row = outlineView.clickedRow >= 0 ? outlineView.clickedRow : outlineView.selectedRow
        activateRow(row, toggleFolders: true)
    }

    public func outlineViewSelectionDidChange(_ notification: Notification) {
        guard !suppressSelectionCallback else { return }
        activateRow(outlineView.selectedRow, toggleFolders: false)
    }

    private func activateRow(_ row: Int, toggleFolders: Bool) {
        guard row >= 0, let node = outlineView.item(atRow: row) as? WorkspaceNode else { return }
        if node.isDirectory {
            if toggleFolders {
                if outlineView.isItemExpanded(node) {
                    outlineView.collapseItem(node)
                } else {
                    outlineView.expandItem(node)
                }
            }
            return
        }

        let url = URL(fileURLWithPath: node.path)
        if !node.iCloudStatus.isReadyToOpen {
            // Initiate or retry iCloud download with automatic opening upon completion
            let session = ICloudDownloadManager.shared.requestDownload(for: url, autoOpenWhenReady: true)
            node.iCloudStatus = session.status
            if session.status.isReadyToOpen {
                node.path = session.logicalURL.path
                outlineView.reloadItem(node)
                onSelectPDF?(session.logicalURL)
            } else {
                outlineView.reloadItem(node)
                onRequestICloudDownload?(url)
            }
            return
        }

        let targetURL = ICloudCoordinator.logicalURL(for: url)
        ICloudDownloadManager.shared.userDidNavigateToDocument(at: targetURL)
        onSelectPDF?(targetURL)
    }

    // MARK: - Context Menu

    public func menuNeedsUpdate(_ menu: NSMenu) {
        menu.removeAllItems()
        let row = outlineView.clickedRow >= 0 ? outlineView.clickedRow : outlineView.selectedRow
        guard row >= 0, let node = outlineView.item(atRow: row) as? WorkspaceNode else { return }

        if !node.isDirectory {
            if node.iCloudStatus.isReadyToOpen {
                let openItem = NSMenuItem(title: "Open PDF", action: #selector(ctxOpenPDF(_:)), keyEquivalent: "")
                openItem.target = self
                openItem.tag = row
                menu.addItem(openItem)
            } else {
                let dlTitle = (node.iCloudStatus == .failed) ? "Retry iCloud Download" : "Download & Open from iCloud"
                let dlItem = NSMenuItem(title: dlTitle, action: #selector(ctxDownloadICloud(_:)), keyEquivalent: "")
                dlItem.target = self
                dlItem.tag = row
                menu.addItem(dlItem)
            }
            menu.addItem(.separator())
        }

        let revealItem = NSMenuItem(title: "Reveal in Finder", action: #selector(ctxRevealInFinder(_:)), keyEquivalent: "")
        revealItem.target = self
        revealItem.tag = row
        menu.addItem(revealItem)

        let copyItem = NSMenuItem(title: "Copy Path", action: #selector(ctxCopyPath(_:)), keyEquivalent: "")
        copyItem.target = self
        copyItem.tag = row
        menu.addItem(copyItem)
    }

    @objc private func ctxOpenPDF(_ sender: NSMenuItem) {
        activateRow(sender.tag, toggleFolders: false)
    }

    @objc private func ctxDownloadICloud(_ sender: NSMenuItem) {
        activateRow(sender.tag, toggleFolders: false)
    }

    @objc private func ctxRevealInFinder(_ sender: NSMenuItem) {
        guard sender.tag >= 0, let node = outlineView.item(atRow: sender.tag) as? WorkspaceNode else { return }
        NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: node.path)])
    }

    @objc private func ctxCopyPath(_ sender: NSMenuItem) {
        guard sender.tag >= 0, let node = outlineView.item(atRow: sender.tag) as? WorkspaceNode else { return }
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(node.logicalPath, forType: .string)
    }

    // MARK: - Hover Tracking

    @objc private func scrollBoundsChanged(_ notification: Notification) {
        hoverCoordinator.clear(in: outlineView)
    }

    @objc public func mouseMoved(with event: NSEvent) {
        hoverCoordinator.handleMouseMoved(with: event, in: outlineView)
    }

    @objc public func mouseExited(with event: NSEvent) {
        hoverCoordinator.clear(in: outlineView)
    }
}
