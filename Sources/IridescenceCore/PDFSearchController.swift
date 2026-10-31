import AppKit
import Foundation
import PDFKit

public final class PDFSearchResultItem: NSObject {
    public let selection: PDFSelection
    public let pageIndex: Int
    public let snippet: String

    public init(selection: PDFSelection, pageIndex: Int, snippet: String) {
        self.selection = selection
        self.pageIndex = pageIndex
        self.snippet = snippet
        super.init()
    }
}

/// Right-hand inspector controller for Segment 2 (In-Document Search).
/// Supports synchronous search for documents `<= 500` pages and asynchronous `beginFindString` for `> 500` pages,
/// preventing stale search notifications from previous tabs (BUG-04 fix).
@MainActor
public final class PDFSearchController: NSObject, NSOutlineViewDataSource, NSOutlineViewDelegate, NSTextFieldDelegate, PDFDocumentDelegate {
    public static let maxResultsCap: Int = 500
    public static let asyncPageThreshold: Int = 500

    public let containerView: NSView
    public let searchField: NSTextField
    public let statusLabel: NSTextField
    public let outlineView: NSOutlineView
    public let scrollView: NSScrollView
    public let emptyStateView: EmptyStateView

    public weak var pdfView: PDFView?
    public private(set) var results: [PDFSearchResultItem] = []

    private let hoverCoordinator = HoverCoordinator()
    private weak var activeSearchDocument: PDFDocument?

    public init(pdfView: PDFView?) {
        self.pdfView = pdfView
        self.containerView = NSView(frame: NSRect(x: 0, y: 0, width: 230, height: 660))
        self.searchField = NSTextField(string: "")
        self.statusLabel = NSTextField(labelWithString: "")
        self.outlineView = NSOutlineView(frame: NSRect(x: 0, y: 0, width: 230, height: 600))
        self.scrollView = NSScrollView(frame: NSRect(x: 0, y: 0, width: 230, height: 600))
        self.emptyStateView = EmptyStateView(
            symbolName: "magnifyingglass",
            title: "Search in this document",
            subtitle: "Type a word or phrase above to find matches"
        )
        super.init()
        setupViews()
    }

    private func setupViews() {
        searchField.placeholderString = "Search in document…"
        searchField.delegate = self
        searchField.translatesAutoresizingMaskIntoConstraints = false

        statusLabel.font = .systemFont(ofSize: 11.0, weight: .medium)
        statusLabel.textColor = .secondaryLabelColor
        statusLabel.translatesAutoresizingMaskIntoConstraints = false
        statusLabel.isHidden = true

        let col = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("SearchColumn"))
        col.width = 210.0
        col.resizingMask = .autoresizingMask
        outlineView.addTableColumn(col)
        outlineView.outlineTableColumn = col
        outlineView.headerView = nil
        outlineView.backgroundColor = .clear
        outlineView.columnAutoresizingStyle = .uniformColumnAutoresizingStyle
        outlineView.rowHeight = UIConstants.rowHeightStandard
        outlineView.style = .sourceList
        outlineView.delegate = self
        outlineView.dataSource = self
        outlineView.target = self
        outlineView.action = #selector(searchResultClicked(_:))

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
        scrollView.isHidden = true
        scrollView.contentView.postsBoundsChangedNotifications = true

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(scrollBoundsChanged(_:)),
            name: NSView.boundsDidChangeNotification,
            object: scrollView.contentView
        )

        containerView.addSubview(searchField)
        containerView.addSubview(statusLabel)
        containerView.addSubview(scrollView)
        containerView.addSubview(emptyStateView)

        NSLayoutConstraint.activate([
            searchField.topAnchor.constraint(equalTo: containerView.topAnchor, constant: 8.0),
            searchField.leadingAnchor.constraint(equalTo: containerView.leadingAnchor, constant: 10.0),
            searchField.trailingAnchor.constraint(equalTo: containerView.trailingAnchor, constant: -10.0),

            statusLabel.topAnchor.constraint(equalTo: searchField.bottomAnchor, constant: 4.0),
            statusLabel.leadingAnchor.constraint(equalTo: containerView.leadingAnchor, constant: 12.0),
            statusLabel.trailingAnchor.constraint(equalTo: containerView.trailingAnchor, constant: -12.0),
            statusLabel.heightAnchor.constraint(equalToConstant: 15.0),

            scrollView.topAnchor.constraint(equalTo: statusLabel.bottomAnchor, constant: 4.0),
            scrollView.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),
            scrollView.bottomAnchor.constraint(equalTo: containerView.bottomAnchor),

            emptyStateView.topAnchor.constraint(equalTo: statusLabel.bottomAnchor, constant: 4.0),
            emptyStateView.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            emptyStateView.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),
            emptyStateView.bottomAnchor.constraint(equalTo: containerView.bottomAnchor)
        ])
    }

    public func cancelAndReset() {
        if let doc = activeSearchDocument, doc.isFinding {
            doc.cancelFindString()
        }
        activeSearchDocument = nil
        hoverCoordinator.clear(in: outlineView)
        results.removeAll()
        searchField.stringValue = ""
        pdfView?.highlightedSelections = nil
        outlineView.reloadData()
        statusLabel.stringValue = ""
        statusLabel.isHidden = true
        scrollView.isHidden = true
        emptyStateView.updateContent(
            symbolName: "magnifyingglass",
            title: "Search in this document",
            subtitle: "Type a word or phrase above to find matches"
        )
        emptyStateView.isHidden = false
    }

    public func performSearch(_ query: String) {
        if searchField.stringValue != query {
            searchField.stringValue = query
        }
        executeSearch(query)
    }

    private func executeSearch(_ rawQuery: String) {
        if let prevDoc = activeSearchDocument, prevDoc.isFinding {
            prevDoc.cancelFindString()
        }
        hoverCoordinator.clear(in: outlineView)
        results.removeAll()

        let trimmed = rawQuery.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, let pdfView, let doc = pdfView.document else {
            pdfView?.highlightedSelections = nil
            outlineView.reloadData()
            statusLabel.stringValue = ""
            statusLabel.isHidden = true
            scrollView.isHidden = true
            emptyStateView.updateContent(
                symbolName: "magnifyingglass",
                title: "Search in this document",
                subtitle: "Type a word or phrase above to find matches"
            )
            emptyStateView.isHidden = false
            return
        }

        activeSearchDocument = doc
        let pageCount = doc.pageCount

        if pageCount <= Self.asyncPageThreshold {
            let found = doc.findString(trimmed, withOptions: [.caseInsensitive, .diacriticInsensitive])
            var highlighted: [PDFSelection] = []

            for sel in found.prefix(Self.maxResultsCap) {
                sel.color = .systemYellow
                highlighted.append(sel)
                let item = Self.buildResultItem(from: sel, in: doc)
                results.append(item)
            }

            pdfView.highlightedSelections = highlighted
            outlineView.reloadData()
            updateVisibilityAfterSearch(isSearchingAsync: false)
        } else {
            doc.delegate = self
            statusLabel.stringValue = "Searching…"
            statusLabel.isHidden = false
            doc.beginFindString(trimmed, withOptions: [.caseInsensitive, .diacriticInsensitive])
        }
    }

    public static func buildResultItem(from selection: PDFSelection, in doc: PDFDocument) -> PDFSearchResultItem {
        var pageIdx = 0
        if let firstPage = selection.pages.first {
            let idx = doc.index(for: firstPage)
            if idx != NSNotFound && idx >= 0 {
                pageIdx = idx
            }
        }

        let matchText = (selection.string ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
        var snippet = matchText
        if let lineSelection = selection.copy() as? PDFSelection {
            lineSelection.extendForLineBoundaries()
            if let lineStr = lineSelection.string?.trimmingCharacters(in: .whitespacesAndNewlines), !lineStr.isEmpty {
                let singleLine = lineStr.components(separatedBy: .whitespacesAndNewlines)
                    .filter { !$0.isEmpty }
                    .joined(separator: " ")
                if !singleLine.isEmpty {
                    snippet = singleLine
                }
            }
        } else if let page = doc.page(at: pageIdx), let pageStr = page.string, !matchText.isEmpty {
            if let range = pageStr.range(of: matchText, options: [.caseInsensitive, .diacriticInsensitive]) {
                snippet = WorkspaceIndex.buildSnippet(in: pageStr, matchRange: range, contextChars: 28)
            }
        }
        if snippet.count > 100 {
            let start = snippet.startIndex
            let end = snippet.index(start, offsetBy: 97)
            snippet = String(snippet[start..<end]) + "…"
        }
        if snippet.isEmpty {
            snippet = "Match on page \(pageIdx + 1)"
        }
        return PDFSearchResultItem(selection: selection, pageIndex: pageIdx, snippet: snippet)
    }

    private func updateVisibilityAfterSearch(isSearchingAsync: Bool) {
        let count = results.count
        if count == 0 {
            scrollView.isHidden = true
            statusLabel.stringValue = isSearchingAsync ? "Searching…" : "0 matches"
            statusLabel.isHidden = false
            emptyStateView.updateContent(
                symbolName: "magnifyingglass",
                title: "No matches found",
                subtitle: "Try a different keyword or check your spelling"
            )
            emptyStateView.isHidden = false
        } else {
            emptyStateView.isHidden = true
            scrollView.isHidden = false
            let suffix = count == 1 ? "match" : "matches"
            statusLabel.stringValue = isSearchingAsync ? "\(count) \(suffix) (searching…)" : "\(count) \(suffix)"
            statusLabel.isHidden = false
        }
    }

    // MARK: - PDFDocumentDelegate (Async Search > 500 pages)

    nonisolated public func didMatchString(_ instance: PDFSelection) {
        nonisolated(unsafe) let matchedSelection = instance
        MainActor.assumeIsolated {
            guard let doc = activeSearchDocument, doc === pdfView?.document else { return }
            guard results.count < Self.maxResultsCap else {
                doc.cancelFindString()
                return
            }
            matchedSelection.color = .systemYellow
            let item = Self.buildResultItem(from: matchedSelection, in: doc)
            results.append(item)
            if results.count % 20 == 0 {
                outlineView.reloadData()
                updateVisibilityAfterSearch(isSearchingAsync: true)
            }
        }
    }

    nonisolated public func documentDidEndDocumentFind(_ notification: Notification) {
        MainActor.assumeIsolated {
            guard let doc = activeSearchDocument, doc === pdfView?.document else { return }
            pdfView?.highlightedSelections = results.map(\.selection)
            outlineView.reloadData()
            updateVisibilityAfterSearch(isSearchingAsync: false)
        }
    }

    // MARK: - NSTextFieldDelegate

    public func controlTextDidChange(_ obj: Notification) {
        executeSearch(searchField.stringValue)
    }

    public func control(_ control: NSControl, textView: NSTextView, doCommandBy commandSelector: Selector) -> Bool {
        if commandSelector == #selector(NSResponder.insertNewline(_:)) {
            guard !results.isEmpty else { return true }
            let current = outlineView.selectedRow
            let next = (current < 0 || current + 1 >= results.count) ? 0 : (current + 1)
            outlineView.selectRowIndexes(IndexSet(integer: next), byExtendingSelection: false)
            outlineView.scrollRowToVisible(next)
            jumpToRow(next)
            return true
        } else if commandSelector == #selector(NSResponder.moveDown(_:)) {
            guard !results.isEmpty else { return true }
            let next = min(outlineView.selectedRow + 1, results.count - 1)
            outlineView.selectRowIndexes(IndexSet(integer: next), byExtendingSelection: false)
            outlineView.scrollRowToVisible(next)
            jumpToRow(next)
            return true
        } else if commandSelector == #selector(NSResponder.moveUp(_:)) {
            guard !results.isEmpty else { return true }
            let prev = max(outlineView.selectedRow - 1, 0)
            outlineView.selectRowIndexes(IndexSet(integer: prev), byExtendingSelection: false)
            outlineView.scrollRowToVisible(prev)
            jumpToRow(prev)
            return true
        } else if commandSelector == #selector(NSResponder.cancelOperation(_:)) {
            cancelAndReset()
            return true
        }
        return false
    }

    // MARK: - NSOutlineViewDataSource & Delegate

    public func outlineView(_ outlineView: NSOutlineView, numberOfChildrenOfItem item: Any?) -> Int {
        item == nil ? results.count : 0
    }

    public func outlineView(_ outlineView: NSOutlineView, isItemExpandable item: Any) -> Bool {
        false
    }

    public func outlineView(_ outlineView: NSOutlineView, child index: Int, ofItem item: Any?) -> Any {
        guard item == nil, index >= 0, index < results.count else { return NSObject() }
        return results[index]
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
        guard let res = item as? PDFSearchResultItem else { return nil }

        let cell = NSView(frame: NSRect(x: 0, y: 0, width: 210, height: 34))

        let snippetLabel = NSTextField(labelWithString: res.snippet)
        snippetLabel.font = .systemFont(ofSize: 11.5, weight: .regular)
        snippetLabel.textColor = .labelColor
        snippetLabel.lineBreakMode = .byTruncatingTail
        snippetLabel.maximumNumberOfLines = 1
        snippetLabel.usesSingleLineMode = true
        snippetLabel.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        snippetLabel.translatesAutoresizingMaskIntoConstraints = false

        let pageBadge = NSTextField(labelWithString: "p.\(res.pageIndex + 1)")
        pageBadge.font = .monospacedDigitSystemFont(ofSize: 10.5, weight: .semibold)
        pageBadge.textColor = .secondaryLabelColor
        pageBadge.alignment = .right
        pageBadge.setContentCompressionResistancePriority(.required, for: .horizontal)
        pageBadge.setContentHuggingPriority(.required, for: .horizontal)
        pageBadge.translatesAutoresizingMaskIntoConstraints = false

        cell.addSubview(snippetLabel)
        cell.addSubview(pageBadge)

        NSLayoutConstraint.activate([
            snippetLabel.leadingAnchor.constraint(equalTo: cell.leadingAnchor, constant: 6.0),
            snippetLabel.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
            snippetLabel.trailingAnchor.constraint(lessThanOrEqualTo: pageBadge.leadingAnchor, constant: -6.0),

            pageBadge.trailingAnchor.constraint(equalTo: cell.trailingAnchor, constant: -12.0),
            pageBadge.centerYAnchor.constraint(equalTo: cell.centerYAnchor)
        ])

        return cell
    }

    @objc private func searchResultClicked(_ sender: Any?) {
        let row = outlineView.clickedRow >= 0 ? outlineView.clickedRow : outlineView.selectedRow
        jumpToRow(row)
    }

    public func outlineViewSelectionDidChange(_ notification: Notification) {
        jumpToRow(outlineView.selectedRow)
    }

    private func jumpToRow(_ row: Int) {
        guard row >= 0, row < results.count, let pdfView else { return }
        let item = results[row]
        pdfView.go(to: item.selection)
        pdfView.setCurrentSelection(item.selection, animate: true)
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
