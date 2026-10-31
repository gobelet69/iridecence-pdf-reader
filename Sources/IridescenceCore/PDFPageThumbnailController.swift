import AppKit
import Foundation
import PDFKit

/// Custom thumbnail card cell that draws an unambiguous iridescent frame and pill badge
/// ONLY for the currently active page, while keeping unselected page numbers clean and unhighlighted.
@MainActor
private final class ThumbnailPageCardView: NSView {
    let imageView = NSImageView()
    let pageLabel = NSTextField(labelWithString: "1")
    var isCurrentPage: Bool = false {
        didSet { needsDisplay = true; updateLabelStyle() }
    }
    var isHovered: Bool = false {
        didSet { needsDisplay = true }
    }

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        setupSubviews()
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupSubviews() {
        imageView.imageScaling = .scaleProportionallyUpOrDown
        imageView.translatesAutoresizingMaskIntoConstraints = false

        pageLabel.font = .monospacedDigitSystemFont(ofSize: 11.0, weight: .semibold)
        pageLabel.alignment = .center
        pageLabel.translatesAutoresizingMaskIntoConstraints = false

        addSubview(imageView)
        addSubview(pageLabel)

        NSLayoutConstraint.activate([
            imageView.topAnchor.constraint(equalTo: topAnchor, constant: 10.0),
            imageView.centerXAnchor.constraint(equalTo: centerXAnchor),
            imageView.widthAnchor.constraint(equalToConstant: 106.0),
            imageView.heightAnchor.constraint(equalToConstant: 136.0),

            pageLabel.topAnchor.constraint(equalTo: imageView.bottomAnchor, constant: 6.0),
            pageLabel.centerXAnchor.constraint(equalTo: centerXAnchor),
            pageLabel.widthAnchor.constraint(greaterThanOrEqualToConstant: 36.0)
        ])
        updateLabelStyle()
    }

    private func updateLabelStyle() {
        if isCurrentPage {
            pageLabel.font = .monospacedDigitSystemFont(ofSize: 11.0, weight: .bold)
            pageLabel.textColor = .white
        } else {
            pageLabel.font = .monospacedDigitSystemFont(ofSize: 11.0, weight: .medium)
            pageLabel.textColor = .secondaryLabelColor
        }
    }

    override func draw(_ dirtyRect: NSRect) {
        let b = bounds
        guard b.width > 20, b.height > 20 else { return }

        let thumbRect = imageView.frame
        let frameRect = thumbRect.insetBy(dx: -3.5, dy: -3.5)
        let framePath = NSBezierPath(roundedRect: frameRect, xRadius: 6.0, yRadius: 6.0)

        if isCurrentPage {
            let isDark = effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.22 : 0.12).setFill()
            framePath.fill()

            IridescenceDesign.primaryViolet.setStroke()
            framePath.lineWidth = 2.2
            framePath.stroke()

            // Active page number pill — ONLY drawn when `isCurrentPage == true`
            let labelFrame = pageLabel.frame
            let pillWidth = max(48.0, labelFrame.width + 16.0)
            let pillRect = NSRect(
                x: b.midX - pillWidth * 0.5,
                y: labelFrame.minY - 2.0,
                width: pillWidth,
                height: labelFrame.height + 4.0
            )
            let pillPath = NSBezierPath(roundedRect: pillRect, xRadius: pillRect.height * 0.5, yRadius: pillRect.height * 0.5)
            if let grad = NSGradient(starting: IridescenceDesign.primaryViolet, ending: IridescenceDesign.secondaryIndigo) {
                grad.draw(in: pillPath, angle: 0.0)
            } else {
                IridescenceDesign.primaryViolet.setFill()
                pillPath.fill()
            }
        } else if isHovered {
            IridescenceDesign.primaryViolet.withAlphaComponent(0.06).setFill()
            framePath.fill()
            IridescenceDesign.primaryViolet.withAlphaComponent(0.35).setStroke()
            framePath.lineWidth = 1.0
            framePath.stroke()
        } else {
            NSColor.separatorColor.withAlphaComponent(0.45).setStroke()
            let innerBorder = NSBezierPath(roundedRect: thumbRect.insetBy(dx: -1.0, dy: -1.0), xRadius: 4.0, yRadius: 4.0)
            innerBorder.lineWidth = 0.75
            innerBorder.stroke()
        }
    }
}

/// Custom table row view for the thumbnail list that suppresses standard row selection highlights
/// so `ThumbnailPageCardView` controls its own crisp page border and active badge.
@MainActor
private final class ThumbnailTableRowView: NSTableRowView {
    override func drawSelection(in dirtyRect: NSRect) {
        // Handled by ThumbnailPageCardView
    }
    override func layout() {
        super.layout()
        for subview in subviews where subview is NSVisualEffectView {
            subview.isHidden = true
        }
    }
}

/// Replaces `PDFThumbnailView` in the right inspector with a lazy-cached, unambiguous page thumbnail list
/// where only the active page displays an iridescent selection ring and highlighted page number pill.
@MainActor
public final class PDFPageThumbnailController: NSObject, NSTableViewDataSource, NSTableViewDelegate {
    public let scrollView: NSScrollView
    public let tableView: NSTableView
    public weak var pdfView: PDFView?

    private weak var currentDocument: PDFDocument?
    private var documentIdentityKey: String = ""
    private var activePageIndex: Int = 0
    private var hoveredRow: Int = -1
    private var suppressPageChangeFeedback = false

    private let thumbnailCache: NSCache<NSString, NSImage> = {
        let cache = NSCache<NSString, NSImage>()
        cache.countLimit = 120
        return cache
    }()

    public init(pdfView: PDFView) {
        self.pdfView = pdfView
        self.scrollView = NSScrollView(frame: NSRect(x: 0, y: 0, width: 230, height: 640))
        self.tableView = NSTableView(frame: NSRect(x: 0, y: 0, width: 230, height: 640))
        super.init()
        setupViews()
    }

    private func setupViews() {
        let col = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("ThumbnailCol"))
        col.width = 210.0
        col.resizingMask = .autoresizingMask
        tableView.addTableColumn(col)
        tableView.columnAutoresizingStyle = .uniformColumnAutoresizingStyle
        tableView.headerView = nil
        tableView.backgroundColor = .clear
        tableView.rowHeight = 176.0
        tableView.style = .plain
        tableView.selectionHighlightStyle = .none
        tableView.intercellSpacing = NSSize(width: 0, height: 4)
        tableView.delegate = self
        tableView.dataSource = self
        tableView.target = self
        tableView.action = #selector(rowClicked(_:))

        let trackingArea = NSTrackingArea(
            rect: tableView.bounds,
            options: [.mouseEnteredAndExited, .mouseMoved, .activeInKeyWindow, .inVisibleRect],
            owner: self,
            userInfo: nil
        )
        tableView.addTrackingArea(trackingArea)

        scrollView.hasVerticalScroller = true
        scrollView.documentView = tableView
        scrollView.drawsBackground = false
        scrollView.translatesAutoresizingMaskIntoConstraints = false
    }

    public func reload(for document: PDFDocument?, documentPath: String = "") {
        currentDocument = document
        documentIdentityKey = documentPath.isEmpty ? "\(ObjectIdentifier(document as AnyObject).hashValue)" : documentPath
        hoveredRow = -1

        if let doc = document, let curPage = pdfView?.currentPage {
            let idx = doc.index(for: curPage)
            activePageIndex = (idx != NSNotFound && idx >= 0) ? idx : 0
        } else {
            activePageIndex = 0
        }

        tableView.reloadData()
        if let doc = document, doc.pageCount > 0, activePageIndex < doc.pageCount {
            tableView.scrollRowToVisible(activePageIndex)
        }
    }

    public func updateActivePageIndex(_ newIndex: Int) {
        guard !suppressPageChangeFeedback else { return }
        guard let doc = currentDocument, newIndex >= 0, newIndex < doc.pageCount else { return }
        let oldIndex = activePageIndex
        guard oldIndex != newIndex else { return }
        activePageIndex = newIndex

        if let oldCell = tableView.view(atColumn: 0, row: oldIndex, makeIfNecessary: false) as? ThumbnailPageCardView {
            oldCell.isCurrentPage = false
        }
        if let newCell = tableView.view(atColumn: 0, row: newIndex, makeIfNecessary: false) as? ThumbnailPageCardView {
            newCell.isCurrentPage = true
        }
        tableView.scrollRowToVisible(newIndex)
    }

    // MARK: - NSTableViewDataSource & Delegate

    public func numberOfRows(in tableView: NSTableView) -> Int {
        min(currentDocument?.pageCount ?? 0, 25_000)
    }

    public func tableView(_ tableView: NSTableView, rowViewForRow row: Int) -> NSTableRowView? {
        ThumbnailTableRowView()
    }

    public func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        guard let doc = currentDocument, row >= 0, row < doc.pageCount else { return nil }

        let id = NSUserInterfaceItemIdentifier("ThumbCardCell")
        let card: ThumbnailPageCardView
        if let reused = tableView.makeView(withIdentifier: id, owner: self) as? ThumbnailPageCardView {
            card = reused
        } else {
            card = ThumbnailPageCardView(frame: NSRect(x: 0, y: 0, width: 210, height: 176))
            card.identifier = id
        }

        card.pageLabel.stringValue = "Page \(row + 1)"
        card.isCurrentPage = (row == activePageIndex)
        card.isHovered = (row == hoveredRow)

        let cacheKey = "\(documentIdentityKey)#\(row)" as NSString
        if let cached = thumbnailCache.object(forKey: cacheKey) {
            card.imageView.image = cached
        } else if let page = doc.page(at: row) {
            let thumb = page.thumbnail(of: NSSize(width: 212, height: 272), for: .cropBox)
            thumbnailCache.setObject(thumb, forKey: cacheKey)
            card.imageView.image = thumb
        }

        return card
    }

    @objc private func rowClicked(_ sender: Any?) {
        let row = tableView.clickedRow >= 0 ? tableView.clickedRow : tableView.selectedRow
        guard let doc = currentDocument, row >= 0, row < doc.pageCount, let page = doc.page(at: row) else { return }

        let oldIndex = activePageIndex
        activePageIndex = row
        if let oldCell = tableView.view(atColumn: 0, row: oldIndex, makeIfNecessary: false) as? ThumbnailPageCardView {
            oldCell.isCurrentPage = false
        }
        if let newCell = tableView.view(atColumn: 0, row: row, makeIfNecessary: false) as? ThumbnailPageCardView {
            newCell.isCurrentPage = true
        }

        suppressPageChangeFeedback = true
        pdfView?.go(to: page)
        suppressPageChangeFeedback = false
    }

    @objc public func mouseMoved(with event: NSEvent) {
        let pt = tableView.convert(event.locationInWindow, from: nil)
        let row = tableView.visibleRect.contains(pt) ? tableView.row(at: pt) : -1
        guard row != hoveredRow else { return }
        let old = hoveredRow
        hoveredRow = row
        if old >= 0, let oldCell = tableView.view(atColumn: 0, row: old, makeIfNecessary: false) as? ThumbnailPageCardView {
            oldCell.isHovered = false
        }
        if row >= 0, let newCell = tableView.view(atColumn: 0, row: row, makeIfNecessary: false) as? ThumbnailPageCardView {
            newCell.isHovered = true
        }
    }

    @objc public func mouseExited(with event: NSEvent) {
        let old = hoveredRow
        hoveredRow = -1
        if old >= 0, let oldCell = tableView.view(atColumn: 0, row: old, makeIfNecessary: false) as? ThumbnailPageCardView {
            oldCell.isHovered = false
        }
    }
}
