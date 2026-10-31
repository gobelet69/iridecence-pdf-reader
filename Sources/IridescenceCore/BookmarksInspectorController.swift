import AppKit
import Foundation
import PDFKit

private final class InspectorItemNode: NSObject {
    enum Payload {
        case bookmark(BookmarkEntry)
        case annotation(AnnotationSummary)
    }

    let payload: Payload
    init(_ payload: Payload) {
        self.payload = payload
    }
}

/// Right-hand inspector controller for Segment 3 (Bookmarks, `filterMode = 0`)
/// and Segment 4 (Annotations, `filterMode = 1`).
@MainActor
public final class BookmarksInspectorController: NSObject, NSOutlineViewDataSource, NSOutlineViewDelegate, NSMenuDelegate {
    public let outlineView: NSOutlineView
    public let scrollView: NSScrollView
    public let emptyStateView: EmptyStateView

    public weak var pdfView: PDFView?
    public var onAnnotationDeleted: (() -> Void)?

    /// `0` = Bookmarks only, `1` = Annotations only
    public var filterMode: Int = 0 {
        didSet {
            reload()
        }
    }

    private var currentPath: String = ""
    private var items: [InspectorItemNode] = []
    private let hoverCoordinator = HoverCoordinator()

    public var isEmpty: Bool {
        items.isEmpty
    }

    public init(pdfView: PDFView?) {
        self.pdfView = pdfView
        self.outlineView = NSOutlineView(frame: NSRect(x: 0, y: 0, width: 230, height: 660))
        self.scrollView = NSScrollView(frame: NSRect(x: 0, y: 0, width: 230, height: 660))
        self.emptyStateView = EmptyStateView(
            symbolName: "bookmark",
            title: "No bookmarks in this document",
            subtitle: "Press ⌘D to bookmark the current page"
        )
        super.init()
        setupViews()
    }

    private func setupViews() {
        let col = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("BookmarksColumn"))
        col.width = 210.0
        outlineView.addTableColumn(col)
        outlineView.outlineTableColumn = col
        outlineView.headerView = nil
        outlineView.backgroundColor = .clear
        outlineView.rowHeight = UIConstants.rowHeightSidebar
        outlineView.style = .sourceList
        outlineView.delegate = self
        outlineView.dataSource = self
        outlineView.target = self
        outlineView.action = #selector(rowClicked(_:))

        let menu = NSMenu(title: "BookmarksContext")
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

    public func reload(forPath path: String? = nil) {
        if let path {
            currentPath = path
        }
        hoverCoordinator.clear(in: outlineView)

        var newItems: [InspectorItemNode] = []
        if filterMode == 0 {
            if !currentPath.isEmpty {
                let bookmarks = BookmarksManager.shared.getBookmarks(forDocument: currentPath)
                newItems = bookmarks.map { InspectorItemNode(.bookmark($0)) }
            }
            emptyStateView.updateContent(
                symbolName: "bookmark",
                title: "No bookmarks in this document",
                subtitle: "Press ⌘D to bookmark the current page"
            )
        } else {
            if let doc = pdfView?.document {
                let annotations = AnnotationsManager.getDocumentAnnotations(in: doc)
                newItems = annotations.map { InspectorItemNode(.annotation($0)) }
            }
            emptyStateView.updateContent(
                symbolName: "highlighter",
                title: "No annotations yet",
                subtitle: "Select text in the PDF to highlight, underline, or add notes"
            )
        }

        items = newItems
        outlineView.reloadData()

        let empty = items.isEmpty
        scrollView.isHidden = empty
        emptyStateView.isHidden = !empty
    }

    // MARK: - NSOutlineViewDataSource & Delegate

    public func outlineView(_ outlineView: NSOutlineView, numberOfChildrenOfItem item: Any?) -> Int {
        item == nil ? items.count : 0
    }

    public func outlineView(_ outlineView: NSOutlineView, isItemExpandable item: Any) -> Bool {
        false
    }

    public func outlineView(_ outlineView: NSOutlineView, child index: Int, ofItem item: Any?) -> Any {
        guard item == nil, index >= 0, index < items.count else {
            return NSObject()
        }
        return items[index]
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
        guard let node = item as? InspectorItemNode else { return nil }

        let cell = NSView(frame: NSRect(x: 0, y: 0, width: 210, height: 32))
        let iconView = NSImageView(frame: NSRect(x: 4, y: 8, width: 16, height: 16))
        iconView.translatesAutoresizingMaskIntoConstraints = false

        let titleField = NSTextField(labelWithString: "")
        titleField.font = .systemFont(ofSize: 12.0, weight: .medium)
        titleField.textColor = .labelColor
        titleField.lineBreakMode = .byTruncatingTail
        titleField.maximumNumberOfLines = 1
        titleField.usesSingleLineMode = true
        titleField.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        titleField.translatesAutoresizingMaskIntoConstraints = false

        let pageBadge = NSTextField(labelWithString: "")
        pageBadge.font = .monospacedDigitSystemFont(ofSize: 10.5, weight: .medium)
        pageBadge.textColor = .secondaryLabelColor
        pageBadge.alignment = .right
        pageBadge.setContentCompressionResistancePriority(.required, for: .horizontal)
        pageBadge.setContentHuggingPriority(.required, for: .horizontal)
        pageBadge.translatesAutoresizingMaskIntoConstraints = false

        switch node.payload {
        case .bookmark(let bm):
            if let img = NSImage(systemSymbolName: "bookmark.fill", accessibilityDescription: nil) {
                img.isTemplate = true
                iconView.image = img
                iconView.contentTintColor = .controlAccentColor
            }
            titleField.stringValue = bm.label
            pageBadge.stringValue = "p.\(bm.pageIndex + 1)"

        case .annotation(let ann):
            let (symName, tint): (String, NSColor) = {
                switch ann.subtype {
                case "Highlight":
                    return ("highlighter", .systemYellow)
                case "Underline":
                    return ("underline", .controlAccentColor)
                case "StrikeOut":
                    return ("strikethrough", .systemRed)
                default:
                    return ("note.text", .systemOrange)
                }
            }()
            if let img = NSImage(systemSymbolName: symName, accessibilityDescription: nil) {
                img.isTemplate = true
                iconView.image = img
                iconView.contentTintColor = tint
            }
            titleField.stringValue = ann.textPreview.isEmpty ? ann.subtype : ann.textPreview
            pageBadge.stringValue = "p.\(ann.pageIndex + 1)"
        }

        cell.addSubview(iconView)
        cell.addSubview(titleField)
        cell.addSubview(pageBadge)

        NSLayoutConstraint.activate([
            iconView.leadingAnchor.constraint(equalTo: cell.leadingAnchor, constant: 6.0),
            iconView.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: 16.0),
            iconView.heightAnchor.constraint(equalToConstant: 16.0),

            titleField.leadingAnchor.constraint(equalTo: iconView.trailingAnchor, constant: 6.0),
            titleField.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
            titleField.trailingAnchor.constraint(lessThanOrEqualTo: pageBadge.leadingAnchor, constant: -6.0),

            pageBadge.trailingAnchor.constraint(equalTo: cell.trailingAnchor, constant: -8.0),
            pageBadge.centerYAnchor.constraint(equalTo: cell.centerYAnchor)
        ])

        return cell
    }

    // MARK: - Actions & Context Menu

    @objc private func rowClicked(_ sender: Any?) {
        let row = outlineView.clickedRow >= 0 ? outlineView.clickedRow : outlineView.selectedRow
        jumpToRow(row)
    }

    public func outlineViewSelectionDidChange(_ notification: Notification) {
        jumpToRow(outlineView.selectedRow)
    }

    private func jumpToRow(_ row: Int) {
        guard row >= 0, row < items.count,
              let pdfView,
              let doc = pdfView.document else { return }

        let pageIdx: Int
        switch items[row].payload {
        case .bookmark(let bm):
            pageIdx = bm.pageIndex
        case .annotation(let ann):
            pageIdx = ann.pageIndex
        }

        if let page = doc.page(at: pageIdx) {
            pdfView.go(to: page)
        }
    }

    public func menuNeedsUpdate(_ menu: NSMenu) {
        menu.removeAllItems()
        let row = outlineView.clickedRow >= 0 ? outlineView.clickedRow : outlineView.selectedRow
        guard row >= 0, row < items.count else { return }

        let jumpItem = NSMenuItem(title: "Jump to Page", action: #selector(contextJump(_:)), keyEquivalent: "")
        jumpItem.target = self
        jumpItem.tag = row
        menu.addItem(jumpItem)

        switch items[row].payload {
        case .bookmark:
            menu.addItem(.separator())
            let removeItem = NSMenuItem(title: "Remove Bookmark", action: #selector(contextDelete(_:)), keyEquivalent: "")
            removeItem.target = self
            removeItem.tag = row
            menu.addItem(removeItem)

        case .annotation(let ann):
            if !ann.textPreview.isEmpty {
                let copyItem = NSMenuItem(title: "Copy Annotation Text", action: #selector(contextCopy(_:)), keyEquivalent: "")
                copyItem.target = self
                copyItem.tag = row
                menu.addItem(copyItem)
            }
            menu.addItem(.separator())
            let deleteItem = NSMenuItem(title: "Remove Annotation", action: #selector(contextDelete(_:)), keyEquivalent: "")
            deleteItem.target = self
            deleteItem.tag = row
            menu.addItem(deleteItem)
        }
    }

    @objc private func contextJump(_ sender: NSMenuItem) {
        jumpToRow(sender.tag)
    }

    @objc private func contextCopy(_ sender: NSMenuItem) {
        let row = sender.tag
        guard row >= 0, row < items.count else { return }
        if case .annotation(let ann) = items[row].payload, !ann.textPreview.isEmpty {
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(ann.textPreview, forType: .string)
        }
    }

    @objc private func contextDelete(_ sender: NSMenuItem) {
        let row = sender.tag
        guard row >= 0, row < items.count else { return }

        switch items[row].payload {
        case .bookmark(let bm):
            BookmarksManager.shared.removeBookmark(pdfPath: currentPath, pageIndex: bm.pageIndex)
            reload()

        case .annotation(let ann):
            if let pdfView, let doc = pdfView.document {
                if AnnotationsManager.removeAnnotation(
                    in: doc,
                    pageIndex: ann.pageIndex,
                    targetBounds: ann.bounds,
                    targetSubtype: ann.subtype
                ) {
                    pdfView.needsDisplay = true
                    onAnnotationDeleted?()
                    reload()
                }
            }
        }
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
