import AppKit
import Foundation
import PDFKit

public final class OutlineNodeItem: NSObject {
    public let title: String
    public let pageIndex: Int
    public let children: [OutlineNodeItem]

    public init(title: String, pageIndex: Int, children: [OutlineNodeItem] = []) {
        self.title = title
        self.pageIndex = pageIndex
        self.children = children
        super.init()
    }
}

/// Right-hand inspector controller for Segment 1 (Document Outline).
/// Enforces recursion depth (`<= 10`) and node count (`<= 5,000`) bounds,
/// and synchronizes selection with current page during PDF scrolling.
@MainActor
public final class PDFOutlineController: NSObject, NSOutlineViewDataSource, NSOutlineViewDelegate, NSMenuDelegate {
    public static let maxDepth: Int = 10
    public static let maxNodes: Int = 5_000

    public let outlineView: NSOutlineView
    public let scrollView: NSScrollView
    public let emptyStateView: EmptyStateView

    public weak var pdfView: PDFView?
    public private(set) var rootItems: [OutlineNodeItem] = []

    private let hoverCoordinator = HoverCoordinator()
    private var isSyncingPage = false

    public init(pdfView: PDFView?) {
        self.pdfView = pdfView
        self.outlineView = NSOutlineView(frame: NSRect(x: 0, y: 0, width: 230, height: 660))
        self.scrollView = NSScrollView(frame: NSRect(x: 0, y: 0, width: 230, height: 660))
        self.emptyStateView = EmptyStateView(
            symbolName: "list.bullet.indent",
            title: "No document outline",
            subtitle: "This PDF does not contain a table of contents"
        )
        super.init()
        setupViews()
    }

    private func setupViews() {
        let col = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("OutlineColumn"))
        col.width = 210.0
        col.resizingMask = .autoresizingMask
        outlineView.addTableColumn(col)
        outlineView.outlineTableColumn = col
        outlineView.headerView = nil
        outlineView.backgroundColor = .clear
        outlineView.columnAutoresizingStyle = .uniformColumnAutoresizingStyle
        outlineView.rowHeight = UIConstants.rowHeightCompact
        outlineView.style = .sourceList
        outlineView.delegate = self
        outlineView.dataSource = self
        outlineView.target = self
        outlineView.action = #selector(outlineClicked(_:))

        let menu = NSMenu(title: "OutlineContext")
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
        scrollView.contentView.postsBoundsChangedNotifications = true

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(scrollBoundsChanged(_:)),
            name: NSView.boundsDidChangeNotification,
            object: scrollView.contentView
        )
    }

    public func loadOutline(from document: PDFDocument?) {
        hoverCoordinator.clear(in: outlineView)
        guard let document, let root = document.outlineRoot else {
            rootItems = []
            outlineView.reloadData()
            return
        }

        var totalVisited = 0
        func convertChildren(of parent: PDFOutline, depth: Int) -> [OutlineNodeItem] {
            guard depth < Self.maxDepth, totalVisited < Self.maxNodes else { return [] }
            let count = parent.numberOfChildren
            var items: [OutlineNodeItem] = []

            for i in 0..<count {
                if totalVisited >= Self.maxNodes { break }
                guard let child = parent.child(at: i) else { continue }
                totalVisited += 1

                let rawLabel = (child.label ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
                let title = rawLabel.isEmpty ? "Untitled Section" : rawLabel

                var pageIdx = 0
                if let page = child.destination?.page {
                    let idx = document.index(for: page)
                    if idx != NSNotFound && idx >= 0 {
                        pageIdx = idx
                    }
                } else if let action = child.action as? PDFActionGoTo, let page = action.destination.page {
                    let idx = document.index(for: page)
                    if idx != NSNotFound && idx >= 0 {
                        pageIdx = idx
                    }
                }

                let subItems = convertChildren(of: child, depth: depth + 1)
                items.append(OutlineNodeItem(title: title, pageIndex: pageIdx, children: subItems))
            }
            return items
        }

        rootItems = convertChildren(of: root, depth: 0)
        outlineView.reloadData()
        // Expand top-level items by default for readability
        for item in rootItems {
            outlineView.expandItem(item)
        }
    }

    public var isEmpty: Bool {
        rootItems.isEmpty
    }

    /// Flattens all outline entries for the `@` Command Palette mode.
    public func flattenedEntries() -> [(title: String, pageIndex: Int)] {
        var flat: [(title: String, pageIndex: Int)] = []
        func walk(_ nodes: [OutlineNodeItem]) {
            for node in nodes {
                flat.append((node.title, node.pageIndex))
                walk(node.children)
            }
        }
        walk(rootItems)
        return flat
    }

    /// Synchronizes the highlighted row in the outline view with the current PDF page index without triggering a recursive jump.
    public func syncActivePage(_ currentPageIndex: Int) {
        guard !rootItems.isEmpty else { return }
        var bestMatch: OutlineNodeItem?

        func search(_ nodes: [OutlineNodeItem]) {
            for node in nodes {
                if node.pageIndex <= currentPageIndex {
                    bestMatch = node
                }
                if !node.children.isEmpty {
                    search(node.children)
                }
            }
        }
        search(rootItems)

        guard let target = bestMatch else { return }
        let row = outlineView.row(forItem: target)
        guard row >= 0, outlineView.selectedRow != row else { return }

        isSyncingPage = true
        outlineView.selectRowIndexes(IndexSet(integer: row), byExtendingSelection: false)
        outlineView.scrollRowToVisible(row)
        isSyncingPage = false
    }

    // MARK: - NSOutlineViewDataSource & Delegate

    public func outlineView(_ outlineView: NSOutlineView, numberOfChildrenOfItem item: Any?) -> Int {
        if item == nil {
            return rootItems.count
        }
        guard let node = item as? OutlineNodeItem else { return 0 }
        return node.children.count
    }

    public func outlineView(_ outlineView: NSOutlineView, isItemExpandable item: Any) -> Bool {
        guard let node = item as? OutlineNodeItem else { return false }
        return !node.children.isEmpty
    }

    public func outlineView(_ outlineView: NSOutlineView, child index: Int, ofItem item: Any?) -> Any {
        if item == nil {
            guard index >= 0, index < rootItems.count else { return NSObject() }
            return rootItems[index]
        }
        guard let node = item as? OutlineNodeItem, index >= 0, index < node.children.count else {
            return NSObject()
        }
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
        guard let node = item as? OutlineNodeItem else { return nil }

        let cell = NSView(frame: NSRect(x: 0, y: 0, width: 210, height: 28))
        let isTopLevel = !node.children.isEmpty

        let titleField = NSTextField(labelWithString: node.title)
        titleField.font = .systemFont(ofSize: 12.0, weight: isTopLevel ? .semibold : .regular)
        titleField.textColor = .labelColor
        titleField.lineBreakMode = .byTruncatingTail
        titleField.maximumNumberOfLines = 1
        titleField.usesSingleLineMode = true
        titleField.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        titleField.translatesAutoresizingMaskIntoConstraints = false

        let pageBadge = NSTextField(labelWithString: "\(node.pageIndex + 1)")
        pageBadge.font = .monospacedDigitSystemFont(ofSize: 10.5, weight: .regular)
        pageBadge.textColor = .tertiaryLabelColor
        pageBadge.alignment = .right
        pageBadge.setContentCompressionResistancePriority(.required, for: .horizontal)
        pageBadge.setContentHuggingPriority(.required, for: .horizontal)
        pageBadge.translatesAutoresizingMaskIntoConstraints = false

        cell.addSubview(titleField)
        cell.addSubview(pageBadge)

        NSLayoutConstraint.activate([
            titleField.leadingAnchor.constraint(equalTo: cell.leadingAnchor, constant: 6.0),
            titleField.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
            titleField.trailingAnchor.constraint(lessThanOrEqualTo: pageBadge.leadingAnchor, constant: -6.0),

            pageBadge.trailingAnchor.constraint(equalTo: cell.trailingAnchor, constant: -12.0),
            pageBadge.centerYAnchor.constraint(equalTo: cell.centerYAnchor)
        ])

        return cell
    }

    @objc private func outlineClicked(_ sender: Any?) {
        let row = outlineView.clickedRow >= 0 ? outlineView.clickedRow : outlineView.selectedRow
        jumpToRow(row)
    }

    public func outlineViewSelectionDidChange(_ notification: Notification) {
        guard !isSyncingPage else { return }
        jumpToRow(outlineView.selectedRow)
    }

    private func jumpToRow(_ row: Int) {
        guard row >= 0,
              let node = outlineView.item(atRow: row) as? OutlineNodeItem,
              let pdfView,
              let doc = pdfView.document,
              let page = doc.page(at: node.pageIndex) else {
            return
        }
        pdfView.go(to: page)
    }

    // MARK: - Context Menu

    public func menuNeedsUpdate(_ menu: NSMenu) {
        menu.removeAllItems()
        let expandAll = NSMenuItem(title: "Expand All", action: #selector(expandAllItems(_:)), keyEquivalent: "")
        expandAll.target = self
        menu.addItem(expandAll)

        let collapseAll = NSMenuItem(title: "Collapse All", action: #selector(collapseAllItems(_:)), keyEquivalent: "")
        collapseAll.target = self
        menu.addItem(collapseAll)
    }

    @objc private func expandAllItems(_ sender: Any?) {
        outlineView.expandItem(nil, expandChildren: true)
    }

    @objc private func collapseAllItems(_ sender: Any?) {
        outlineView.collapseItem(nil, collapseChildren: true)
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
