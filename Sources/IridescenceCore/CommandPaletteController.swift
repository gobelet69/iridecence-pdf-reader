import AppKit
import Foundation

public enum PaletteItemAction: Sendable, Equatable {
    case openFile(String)
    case openFileAtPage(String, Int)
    case goToPage(Int) // 1-based page number
    case searchInPDF(String)
    case executeCommand(String)
}

public struct PaletteItem: Sendable, Equatable {
    public let title: String
    public let subtitle: String
    public let badge: String
    public let iconName: String
    public let action: PaletteItemAction
    public let score: Int

    public init(
        title: String,
        subtitle: String,
        badge: String = "",
        iconName: String = "doc.richtext",
        action: PaletteItemAction,
        score: Int = 0
    ) {
        self.title = title
        self.subtitle = subtitle
        self.badge = badge
        self.iconName = iconName
        self.action = action
        self.score = score
    }
}

/// Spotlight / Raycast-grade borderless floating `NSPanel` for the Command Palette (`Cmd+P` / `Cmd+Shift+P`).
@MainActor
public final class FloatingPalettePanel: NSPanel {
    public override var canBecomeKey: Bool { true }
    public override var canBecomeMain: Bool { false }

    public init() {
        let frame = WindowGeometry.initialFrame(for: .palette)
        super.init(
            contentRect: frame,
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        self.isFloatingPanel = true
        self.level = .floating
        self.isOpaque = false
        self.backgroundColor = .clear
        self.hasShadow = true
        self.hidesOnDeactivate = true
        self.isMovableByWindowBackground = false
    }
}

/// Coordinates the Spotlight/Raycast-style Command Palette (`Cmd+P`, `Cmd+Shift+P`),
/// prefix routing (`>`, `/`, `//`, `@`, `#`, `p:`/`:`), fuzzy scoring, and cursor placement.
@MainActor
public final class CommandPaletteController: NSObject, NSTableViewDataSource, NSTableViewDelegate, NSTextFieldDelegate, NSWindowDelegate {
    public let panel: FloatingPalettePanel
    public let effectView: NSVisualEffectView
    public let searchField: NSTextField
    public let modePillBadge = IridescentPillBadge(text: "RECENT", style: .violet)
    public let separator: NSView
    public let scrollView: NSScrollView
    public let tableView: NSTableView
    public let footerBar: NSView
    public let footerLabel: NSTextField

    public private(set) var items: [PaletteItem] = []

    /// Providers supplied by the main application coordinator
    public var workspaceFilesProvider: (() -> [String])?
    public var recentFilesProvider: (() -> [String])?
    public var outlineEntriesProvider: (() -> [(title: String, pageIndex: Int)])?
    public var currentDocumentBookmarksProvider: (() -> [BookmarkEntry])?
    public var onTriggerAction: ((PaletteItemAction) -> Void)?

    private let hoverCoordinator = HoverCoordinator()

    public override init() {
        self.panel = FloatingPalettePanel()
        self.effectView = NSVisualEffectView(frame: NSRect(x: 0, y: 0, width: UIConstants.paletteWidth, height: UIConstants.paletteCollapsedHeight))
        self.searchField = NSTextField(string: "")
        self.separator = NSView()
        self.scrollView = NSScrollView()
        self.tableView = NSTableView()
        self.footerBar = NSView()
        self.footerLabel = NSTextField(labelWithString: "↑↓ Navigate   ↵ Open / Execute   ⇥ Mode: > Commands  / PDF  // Workspace  @ Outline  # Bookmarks   ⎋ Close")
        super.init()
        setupViews()
    }

    private func setupViews() {
        panel.delegate = self

        effectView.material = .popover
        effectView.blendingMode = .behindWindow
        effectView.state = .active
        effectView.wantsLayer = true
        effectView.layer?.cornerRadius = UIConstants.radiusPanel
        effectView.layer?.masksToBounds = true
        effectView.layer?.borderWidth = 1.0
        effectView.layer?.borderColor = NSColor.separatorColor.withAlphaComponent(0.45).cgColor
        effectView.autoresizingMask = [.width, .height]

        let paletteWash = IridescentSurfaceWashView(role: .palette)

        let magIcon = NSImageView()
        magIcon.translatesAutoresizingMaskIntoConstraints = false
        if let img = NSImage(systemSymbolName: "sparkle.magnifyingglass", accessibilityDescription: "Search") {
            let cfg = NSImage.SymbolConfiguration(pointSize: 16.0, weight: .semibold)
            magIcon.image = img.withSymbolConfiguration(cfg) ?? img
            magIcon.contentTintColor = IridescenceDesign.primaryViolet
        }

        searchField.isBordered = false
        searchField.drawsBackground = false
        searchField.focusRingType = .none
        searchField.font = .systemFont(ofSize: 16.5, weight: .regular)
        searchField.textColor = .labelColor
        searchField.placeholderString = "Search files or type > for commands…"
        searchField.delegate = self
        searchField.translatesAutoresizingMaskIntoConstraints = false

        separator.wantsLayer = true
        separator.layer?.backgroundColor = NSColor.separatorColor.cgColor
        separator.translatesAutoresizingMaskIntoConstraints = false
        separator.isHidden = true

        let col = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("PaletteColumn"))
        col.width = UIConstants.paletteWidth - 36.0
        col.resizingMask = .autoresizingMask
        tableView.addTableColumn(col)
        tableView.columnAutoresizingStyle = .uniformColumnAutoresizingStyle
        tableView.headerView = nil
        tableView.backgroundColor = .clear
        tableView.rowHeight = UIConstants.rowHeightPalette
        tableView.style = .inset
        tableView.intercellSpacing = NSSize(width: 0, height: 2)
        tableView.delegate = self
        tableView.dataSource = self
        tableView.target = self
        tableView.action = #selector(rowClicked(_:))
        tableView.doubleAction = #selector(rowDoubleClicked(_:))

        let trackingArea = NSTrackingArea(
            rect: tableView.bounds,
            options: [.mouseEnteredAndExited, .mouseMoved, .activeAlways, .inVisibleRect],
            owner: self,
            userInfo: nil
        )
        tableView.addTrackingArea(trackingArea)

        scrollView.hasVerticalScroller = true
        scrollView.documentView = tableView
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

        footerBar.wantsLayer = true
        footerBar.layer?.backgroundColor = NSColor.labelColor.withAlphaComponent(0.035).cgColor
        footerBar.translatesAutoresizingMaskIntoConstraints = false
        footerBar.isHidden = true

        let footerTopBorder = NSView()
        footerTopBorder.wantsLayer = true
        footerTopBorder.layer?.backgroundColor = NSColor.separatorColor.withAlphaComponent(0.5).cgColor
        footerTopBorder.translatesAutoresizingMaskIntoConstraints = false

        footerLabel.font = .systemFont(ofSize: 10.5, weight: .medium)
        footerLabel.textColor = .tertiaryLabelColor
        footerLabel.lineBreakMode = .byTruncatingTail
        footerLabel.translatesAutoresizingMaskIntoConstraints = false

        footerBar.addSubview(footerTopBorder)
        footerBar.addSubview(footerLabel)

        effectView.addSubview(paletteWash)
        effectView.addSubview(magIcon)
        effectView.addSubview(searchField)
        effectView.addSubview(modePillBadge)
        effectView.addSubview(separator)
        effectView.addSubview(scrollView)
        effectView.addSubview(footerBar)

        NSLayoutConstraint.activate([
            paletteWash.topAnchor.constraint(equalTo: effectView.topAnchor),
            paletteWash.leadingAnchor.constraint(equalTo: effectView.leadingAnchor),
            paletteWash.trailingAnchor.constraint(equalTo: effectView.trailingAnchor),
            paletteWash.bottomAnchor.constraint(equalTo: effectView.bottomAnchor),

            magIcon.leadingAnchor.constraint(equalTo: effectView.leadingAnchor, constant: 16.0),
            magIcon.topAnchor.constraint(equalTo: effectView.topAnchor, constant: 15.0),
            magIcon.widthAnchor.constraint(equalToConstant: 18.0),
            magIcon.heightAnchor.constraint(equalToConstant: 18.0),

            searchField.leadingAnchor.constraint(equalTo: magIcon.trailingAnchor, constant: 10.0),
            searchField.trailingAnchor.constraint(equalTo: modePillBadge.leadingAnchor, constant: -10.0),
            searchField.centerYAnchor.constraint(equalTo: magIcon.centerYAnchor),

            modePillBadge.trailingAnchor.constraint(equalTo: effectView.trailingAnchor, constant: -16.0),
            modePillBadge.centerYAnchor.constraint(equalTo: magIcon.centerYAnchor),

            separator.topAnchor.constraint(equalTo: effectView.topAnchor, constant: UIConstants.paletteCollapsedHeight),
            separator.leadingAnchor.constraint(equalTo: effectView.leadingAnchor),
            separator.trailingAnchor.constraint(equalTo: effectView.trailingAnchor),
            separator.heightAnchor.constraint(equalToConstant: 1.0),

            scrollView.topAnchor.constraint(equalTo: separator.bottomAnchor, constant: 4.0),
            scrollView.leadingAnchor.constraint(equalTo: effectView.leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: effectView.trailingAnchor),
            scrollView.bottomAnchor.constraint(equalTo: footerBar.topAnchor),

            footerBar.leadingAnchor.constraint(equalTo: effectView.leadingAnchor),
            footerBar.trailingAnchor.constraint(equalTo: effectView.trailingAnchor),
            footerBar.bottomAnchor.constraint(equalTo: effectView.bottomAnchor),
            footerBar.heightAnchor.constraint(equalToConstant: UIConstants.paletteFooterHeight),

            footerTopBorder.topAnchor.constraint(equalTo: footerBar.topAnchor),
            footerTopBorder.leadingAnchor.constraint(equalTo: footerBar.leadingAnchor),
            footerTopBorder.trailingAnchor.constraint(equalTo: footerBar.trailingAnchor),
            footerTopBorder.heightAnchor.constraint(equalToConstant: 1.0),

            footerLabel.leadingAnchor.constraint(equalTo: footerBar.leadingAnchor, constant: 14.0),
            footerLabel.trailingAnchor.constraint(equalTo: footerBar.trailingAnchor, constant: -14.0),
            footerLabel.centerYAnchor.constraint(equalTo: footerBar.centerYAnchor)
        ])

        panel.contentView = effectView
    }

    // MARK: - Show / Hide & Cursor Placement (Rule 6.4)

    public func present(prefill: String = "") {
        hoverCoordinator.clear(in: tableView)
        positionPanelRelativeToMainWindow()
        panel.makeKeyAndOrderFront(nil)
        searchField.stringValue = prefill
        panel.makeFirstResponder(searchField)
        placeCursorAtEnd()
        updateSearch(prefill)
    }

    public func dismiss() {
        hoverCoordinator.clear(in: tableView)
        panel.orderOut(nil)
    }

    /// Explicitly places the insertion point at the end of `searchField` with a zero-length selection range
    /// so prefilled prefixes (`>`, `p:`) never display AppKit's full-text blue selection highlight (Rule 6.4).
    public func placeCursorAtEnd() {
        if let editor = searchField.currentEditor() {
            let length = (searchField.stringValue as NSString).length
            editor.selectedRange = NSRange(location: length, length: 0)
        }
    }

    private func positionPanelRelativeToMainWindow() {
        let width = UIConstants.paletteWidth
        let height = panel.frame.height
        if let mainWin = NSApp.mainWindow {
            let mf = mainWin.frame
            let x = mf.origin.x + (mf.width - width) * 0.5
            let topY = mf.origin.y + mf.height - 90.0
            let y = topY - height
            panel.setFrame(NSRect(x: x.rounded(), y: y.rounded(), width: width, height: height), display: true)
        } else {
            let initFrame = WindowGeometry.initialFrame(for: .palette)
            panel.setFrame(initFrame, display: true)
        }
    }

    private func resizePanelForItemCount(_ count: Int) {
        let currentFrame = panel.frame
        let topEdge = currentFrame.origin.y + currentFrame.height

        let newHeight: CGFloat
        if count == 0 {
            newHeight = UIConstants.paletteCollapsedHeight
            separator.isHidden = true
            scrollView.isHidden = true
            footerBar.isHidden = true
        } else {
            let visibleRows = min(count, UIConstants.paletteMaxVisibleRows)
            let listHeight = CGFloat(visibleRows) * (UIConstants.rowHeightPalette + 2.0) + 12.0
            newHeight = UIConstants.paletteCollapsedHeight + 1.0 + listHeight + UIConstants.paletteFooterHeight
            separator.isHidden = false
            scrollView.isHidden = false
            footerBar.isHidden = false
        }

        let newOriginY = topEdge - newHeight
        let targetRect = NSRect(x: currentFrame.origin.x, y: newOriginY.rounded(), width: UIConstants.paletteWidth, height: newHeight.rounded())
        let clamped = WindowGeometry.sanitizeRect(targetRect, for: .palette, in: WindowGeometry.activeVisibleFrame(for: panel))
        panel.setFrame(clamped, display: true)
        effectView.maskImage = Self.roundedMaskImage(size: clamped.size, radius: UIConstants.radiusPanel)
    }

    private static func roundedMaskImage(size: NSSize, radius: CGFloat) -> NSImage? {
        guard size.width > 0, size.height > 0 else { return nil }
        let img = NSImage(size: size, flipped: false) { rect in
            let path = NSBezierPath(roundedRect: rect, xRadius: radius, yRadius: radius)
            NSColor.black.setFill()
            path.fill()
            return true
        }
        img.capInsets = NSEdgeInsets(top: radius, left: radius, bottom: radius, right: radius)
        return img
    }

    // MARK: - Search & Prefix Routing

    public static func scoreQueryMatch(candidate: String, query: String) -> Int? {
        let q = query.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        if q.isEmpty { return 10 }
        let c = candidate.lowercased()

        if c == q { return 1000 }
        if c.hasPrefix(q) { return 800 - min(c.count, 100) }
        if c.contains(" " + q) || c.contains("_" + q) || c.contains("-" + q) {
            return 650 - min(c.count, 100)
        }
        if c.contains(q) {
            return 500 - min(c.count, 100)
        }

        // Subsequence fuzzy match
        var qIdx = q.startIndex
        var cIdx = c.startIndex
        var matched = 0
        while qIdx < q.endIndex && cIdx < c.endIndex {
            if q[qIdx] == c[cIdx] {
                matched += 1
                qIdx = q.index(after: qIdx)
            }
            cIdx = c.index(after: cIdx)
        }
        if qIdx == q.endIndex && matched >= 2 {
            return 200 + matched * 10 - min(c.count, 100)
        }
        return nil
    }

    public static func formatPathSubtitle(_ fullPath: String) -> String {
        var display = fullPath
        if display.hasPrefix("/private/tmp") {
            display = String(display.dropFirst("/private".count))
        } else if display.hasPrefix("/private/var") {
            display = String(display.dropFirst("/private".count))
        }
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        if display.hasPrefix(home) {
            display = "~" + String(display.dropFirst(home.count))
        }
        let parent = (display as NSString).deletingLastPathComponent
        return parent.isEmpty ? display : parent
    }

    public func updateSearch(_ rawInput: String) {
        hoverCoordinator.clear(in: tableView)
        if searchField.stringValue != rawInput {
            searchField.stringValue = rawInput
            placeCursorAtEnd()
        }

        let trimmed = rawInput.trimmingCharacters(in: .whitespacesAndNewlines)
        var built: [PaletteItem] = []

        if trimmed.isEmpty {
            modePillBadge.style = .violet
            modePillBadge.stringValue = "RECENT"
        } else if trimmed.hasPrefix(">") {
            modePillBadge.style = .rose
            modePillBadge.stringValue = "COMMANDS"
        } else if trimmed.hasPrefix("//") {
            modePillBadge.style = .cyan
            modePillBadge.stringValue = "WORKSPACE"
        } else if trimmed.hasPrefix("/") {
            modePillBadge.style = .cyan
            modePillBadge.stringValue = "IN PDF"
        } else if trimmed.hasPrefix("@") {
            modePillBadge.style = .violet
            modePillBadge.stringValue = "OUTLINE"
        } else if trimmed.hasPrefix("#") {
            modePillBadge.style = .amber
            modePillBadge.stringValue = "BOOKMARKS"
        } else if trimmed.lowercased().hasPrefix("p:") || trimmed.hasPrefix(":") {
            modePillBadge.style = .violet
            modePillBadge.stringValue = "PAGE JUMP"
        } else {
            modePillBadge.style = .violet
            modePillBadge.stringValue = "SEARCH"
        }

        if trimmed.isEmpty {
            // Empty state: show recent files (or keep collapsed if none)
            let recents = recentFilesProvider?() ?? []
            if !recents.isEmpty {
                for path in recents.prefix(6) {
                    let name = URL(fileURLWithPath: path).deletingPathExtension().lastPathComponent
                    built.append(PaletteItem(
                        title: name.isEmpty ? path : name,
                        subtitle: Self.formatPathSubtitle(path),
                        badge: "Recent",
                        iconName: "clock",
                        action: .openFile(path),
                        score: 100
                    ))
                }
            }
        } else if trimmed.hasPrefix(">") {
            let cmdQuery = String(trimmed.dropFirst()).trimmingCharacters(in: .whitespaces)
            for cmd in Self.allBuiltInCommands() {
                if let s1 = Self.scoreQueryMatch(candidate: cmd.title, query: cmdQuery) {
                    built.append(PaletteItem(
                        title: cmd.title,
                        subtitle: cmd.subtitle,
                        badge: cmd.shortcut,
                        iconName: cmd.icon,
                        action: .executeCommand(cmd.commandId),
                        score: s1
                    ))
                } else if let s2 = Self.scoreQueryMatch(candidate: cmd.subtitle, query: cmdQuery) {
                    built.append(PaletteItem(
                        title: cmd.title,
                        subtitle: cmd.subtitle,
                        badge: cmd.shortcut,
                        iconName: cmd.icon,
                        action: .executeCommand(cmd.commandId),
                        score: s2 - 50
                    ))
                }
            }
            built.sort { $0.score > $1.score }
        } else if trimmed.hasPrefix("//") {
            let wsQuery = String(trimmed.dropFirst(2)).trimmingCharacters(in: .whitespaces)
            if wsQuery.isEmpty {
                built.append(PaletteItem(
                    title: "Type a keyword after // to search across all workspace PDFs",
                    subtitle: "Full-text workspace index search",
                    badge: "Workspace",
                    iconName: "doc.text.magnifyingglass",
                    action: .executeCommand("noop"),
                    score: 100
                ))
            } else {
                let matches = WorkspaceIndex.shared.search(wsQuery, maxResults: 30)
                for m in matches {
                    built.append(PaletteItem(
                        title: m.snippet,
                        subtitle: "\(m.fileName) — Page \(m.pageIndex + 1)",
                        badge: "p.\(m.pageIndex + 1)",
                        iconName: "doc.text.magnifyingglass",
                        action: .openFileAtPage(m.pdfPath, m.pageIndex),
                        score: 500
                    ))
                }
            }
        } else if trimmed.hasPrefix("/") {
            let pdfQuery = String(trimmed.dropFirst()).trimmingCharacters(in: .whitespaces)
            if !pdfQuery.isEmpty {
                built.append(PaletteItem(
                    title: "Search in current PDF for “\(pdfQuery)”",
                    subtitle: "Opens the Search inspector and highlights all matches",
                    badge: "↵ Find",
                    iconName: "magnifyingglass",
                    action: .searchInPDF(pdfQuery),
                    score: 1000
                ))
            }
        } else if trimmed.hasPrefix("@") {
            let outlineQuery = String(trimmed.dropFirst()).trimmingCharacters(in: .whitespaces)
            let entries = outlineEntriesProvider?() ?? []
            for entry in entries {
                if let score = Self.scoreQueryMatch(candidate: entry.title, query: outlineQuery) {
                    built.append(PaletteItem(
                        title: entry.title,
                        subtitle: "Document Outline Section",
                        badge: "p.\(entry.pageIndex + 1)",
                        iconName: "list.bullet.indent",
                        action: .goToPage(entry.pageIndex + 1),
                        score: score
                    ))
                }
            }
            if outlineQuery.isEmpty {
                // Keep natural document order when query is just "@"
            } else {
                built.sort { $0.score > $1.score }
            }
        } else if trimmed.hasPrefix("#") {
            let bmQuery = String(trimmed.dropFirst()).trimmingCharacters(in: .whitespaces)
            let bookmarks = currentDocumentBookmarksProvider?() ?? BookmarksManager.shared.loadAllBookmarks()
            for bm in bookmarks {
                if let score = Self.scoreQueryMatch(candidate: bm.label, query: bmQuery) {
                    let docName = URL(fileURLWithPath: bm.pdfPath).lastPathComponent
                    built.append(PaletteItem(
                        title: bm.label,
                        subtitle: docName,
                        badge: "p.\(bm.pageIndex + 1)",
                        iconName: "bookmark.fill",
                        action: .openFileAtPage(bm.pdfPath, bm.pageIndex),
                        score: score
                    ))
                }
            }
            built.sort { $0.score > $1.score }
        } else if trimmed.lowercased().hasPrefix("p:") || trimmed.hasPrefix(":") {
            let numStr = trimmed.lowercased().hasPrefix("p:")
                ? String(trimmed.dropFirst(2)).trimmingCharacters(in: .whitespaces)
                : String(trimmed.dropFirst(1)).trimmingCharacters(in: .whitespaces)
            if let pageNum = Int(numStr), pageNum >= 1 {
                built.append(PaletteItem(
                    title: "Go to Page \(pageNum)",
                    subtitle: "Jump directly to page \(pageNum) in the active PDF",
                    badge: "p.\(pageNum)",
                    iconName: "arrow.right.doc.on.clipboard",
                    action: .goToPage(pageNum),
                    score: 1000
                ))
            } else {
                built.append(PaletteItem(
                    title: "Type a page number (e.g. p:12)",
                    subtitle: "Jump directly to a page in the active PDF",
                    badge: "Page Jump",
                    iconName: "number",
                    action: .executeCommand("noop"),
                    score: 100
                ))
            }
        } else {
            // Standard file & command search
            if let pageNum = Int(trimmed), pageNum >= 1 {
                built.append(PaletteItem(
                    title: "Go to Page \(pageNum)",
                    subtitle: "Jump directly to page \(pageNum)",
                    badge: "p.\(pageNum)",
                    iconName: "arrow.right.doc.on.clipboard",
                    action: .goToPage(pageNum),
                    score: 950
                ))
            }

            var seenPaths: Set<String> = []
            let allFiles = (workspaceFilesProvider?() ?? []) + (recentFilesProvider?() ?? [])
            for path in allFiles {
                let canon = UIConstants.canonicalPath(path)
                guard !seenPaths.contains(canon) else { continue }
                seenPaths.insert(canon)

                let name = URL(fileURLWithPath: path).deletingPathExtension().lastPathComponent
                if let score = Self.scoreQueryMatch(candidate: name, query: trimmed) {
                    built.append(PaletteItem(
                        title: name,
                        subtitle: Self.formatPathSubtitle(path),
                        badge: "PDF",
                        iconName: "doc.richtext",
                        action: .openFile(path),
                        score: score
                    ))
                }
            }

            for cmd in Self.allBuiltInCommands() {
                if let score = Self.scoreQueryMatch(candidate: cmd.title, query: trimmed) {
                    built.append(PaletteItem(
                        title: cmd.title,
                        subtitle: cmd.subtitle,
                        badge: cmd.shortcut,
                        iconName: cmd.icon,
                        action: .executeCommand(cmd.commandId),
                        score: score - 40
                    ))
                }
            }

            built.sort { $0.score > $1.score }
        }

        items = Array(built.prefix(50))
        tableView.reloadData()
        resizePanelForItemCount(items.count)

        if !items.isEmpty {
            tableView.selectRowIndexes(IndexSet(integer: 0), byExtendingSelection: false)
            tableView.scrollRowToVisible(0)
        }
    }

    public static func allBuiltInCommands() -> [(commandId: String, title: String, subtitle: String, shortcut: String, icon: String)] {
        [
            ("open_pdf", "Open PDF Document…", "Choose a PDF file from disk", "⌘O", "doc.badge.plus"),
            ("open_folder", "Open Workspace Folder…", "Mount a directory tree in the Workspace Explorer", "⇧⌘O", "folder.badge.plus"),
            ("save", "Save Document", "Safely save annotations to disk", "⌘S", "square.and.arrow.down"),
            ("save_as", "Save Document As…", "Save a copy of the current PDF to a new location", "⇧⌘S", "doc.on.doc"),
            ("export_markdown", "Export Notes & Annotations as Markdown…", "Export all bookmarks and annotations to a .md file", "⇧⌘E", "arrow.up.doc"),
            ("highlight", "Highlight Selected Text", "Apply highlight markup to current text selection", "⌃⌘H", "highlighter"),
            ("underline", "Underline Selected Text", "Apply underline markup to current text selection", "⌃⌘U", "underline"),
            ("strikeout", "Strikeout Selected Text", "Apply strikethrough markup to current text selection", "⌃⌘S", "strikethrough"),
            ("add_note", "Add Text Note to Selection", "Attach a sticky note annotation to selected text", "⌃⌘N", "note.text.badge.plus"),
            ("bookmark_toggle", "Toggle Page Bookmark", "Add or remove a bookmark on the current page", "⌘D", "bookmark"),
            ("ocr_page", "OCR Current Page (Apple Vision)", "Extract text from scanned page using Vision framework", "⇧⌘O", "text.viewfinder"),
            ("zoom_in", "Zoom In", "Increase PDF magnification", "⌘+", "plus.magnifyingglass"),
            ("zoom_out", "Zoom Out", "Decrease PDF magnification", "⌘-", "minus.magnifyingglass"),
            ("zoom_actual", "Actual Size (100%)", "Reset PDF zoom to 100%", "⌘0", "1.magnifyingglass"),
            ("zoom_fit", "Zoom to Fit Width", "Automatically scale PDF to viewport width", "⌘9", "arrow.left.and.right.square"),
            ("mode_continuous", "Display Mode: Continuous Scroll", "Scroll seamlessly across pages", "", "rectangle.grid.1x2"),
            ("mode_single", "Display Mode: Single Page", "Show one page at a time", "", "doc"),
            ("mode_twoup", "Display Mode: Two-Up Spread", "Show two pages side by side", "", "book"),
            ("toggle_left_sidebar", "Toggle Left Sidebar", "Show or hide the Workspace Explorer", "⌘B", "sidebar.left"),
            ("toggle_right_sidebar", "Toggle Right Inspector", "Show or hide Pages, Outline, Search & Bookmarks", "⌥⌘B", "sidebar.right"),
            ("focus_mode", "Toggle Focus Mode", "Collapse both sidebars for distraction-free reading", "⇧⌘F", "arrow.up.left.and.arrow.down.right"),
            ("presentation_mode", "Toggle Presentation Mode", "Enter or exit fullscreen presentation mode", "⌃⌘F", "play.rectangle"),
            ("tab_next", "Switch to Next Tab", "Cycle forward through open document tabs", "⌃⇥", "arrow.right.square"),
            ("tab_prev", "Switch to Previous Tab", "Cycle backward through open document tabs", "⌃⇧⇥", "arrow.left.square"),
            ("tab_reopen", "Reopen Closed Tab", "Restore the most recently closed PDF tab", "⇧⌘T", "arrow.uturn.backward.square"),
            ("tab_close", "Close Active Tab", "Close the current PDF tab", "⌘W", "xmark.square"),
            ("doc_info", "Show Document Info", "Inspect PDF metadata, dimensions, and file path", "⌘I", "info.circle"),
            ("debug_console", "Open Live Debug Console", "Inspect diagnostic logs and runtime events", "⌥⌘L", "terminal"),
            ("settings", "Open Settings…", "Configure reading, workspace, and appearance preferences", "⌘,", "gearshape"),
            ("go_home", "Return to Home Screen", "Show Recent, Favorites, and Workspaces hub", "⇧⌘H", "house")
        ]
    }

    // MARK: - NSTableViewDataSource & Delegate

    public func numberOfRows(in tableView: NSTableView) -> Int {
        items.count
    }

    public func tableView(_ tableView: NSTableView, rowViewForRow row: Int) -> NSTableRowView? {
        let rowView = GlassTableRowView()
        if row == hoverCoordinator.currentRow {
            rowView.isHovered = true
        }
        return rowView
    }

    public func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        guard row >= 0, row < items.count else { return nil }
        let item = items[row]

        let cell = NSView(frame: NSRect(x: 0, y: 0, width: UIConstants.paletteWidth - 20.0, height: UIConstants.rowHeightPalette))

        let iconView = NSImageView()
        iconView.translatesAutoresizingMaskIntoConstraints = false
        if let img = NSImage(systemSymbolName: item.iconName, accessibilityDescription: nil) {
            img.isTemplate = true
            iconView.image = img
            switch item.action {
            case .executeCommand:
                iconView.contentTintColor = IridescenceDesign.accentRose
            case .openFileAtPage, .searchInPDF:
                iconView.contentTintColor = IridescenceDesign.accentCyan
            default:
                iconView.contentTintColor = IridescenceDesign.primaryViolet
            }
        }

        let titleField = NSTextField(labelWithString: item.title)
        titleField.font = .systemFont(ofSize: 13.0, weight: .medium)
        titleField.textColor = .labelColor
        titleField.lineBreakMode = .byTruncatingTail
        titleField.maximumNumberOfLines = 1
        titleField.usesSingleLineMode = true
        titleField.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        titleField.translatesAutoresizingMaskIntoConstraints = false

        let subtitleField = NSTextField(labelWithString: item.subtitle)
        subtitleField.font = .systemFont(ofSize: 11.0, weight: .regular)
        subtitleField.textColor = .secondaryLabelColor
        subtitleField.lineBreakMode = .byTruncatingMiddle
        subtitleField.maximumNumberOfLines = 1
        subtitleField.usesSingleLineMode = true
        subtitleField.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        subtitleField.translatesAutoresizingMaskIntoConstraints = false

        let pillStyle: IridescentPillBadge.Style = {
            switch item.action {
            case .executeCommand: return .rose
            case .openFileAtPage, .searchInPDF: return .cyan
            default: return .violet
            }
        }()
        let badgeField = IridescentPillBadge(text: item.badge, style: pillStyle)
        badgeField.isHidden = item.badge.isEmpty

        cell.addSubview(iconView)
        cell.addSubview(titleField)
        cell.addSubview(subtitleField)
        cell.addSubview(badgeField)

        NSLayoutConstraint.activate([
            iconView.leadingAnchor.constraint(equalTo: cell.leadingAnchor, constant: 12.0),
            iconView.centerYAnchor.constraint(equalTo: cell.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: 18.0),
            iconView.heightAnchor.constraint(equalToConstant: 18.0),

            titleField.leadingAnchor.constraint(equalTo: iconView.trailingAnchor, constant: 10.0),
            titleField.topAnchor.constraint(equalTo: cell.topAnchor, constant: 3.0),
            titleField.trailingAnchor.constraint(lessThanOrEqualTo: badgeField.leadingAnchor, constant: -10.0),

            subtitleField.leadingAnchor.constraint(equalTo: titleField.leadingAnchor),
            subtitleField.topAnchor.constraint(equalTo: titleField.bottomAnchor, constant: 1.0),
            subtitleField.trailingAnchor.constraint(lessThanOrEqualTo: badgeField.leadingAnchor, constant: -10.0),

            badgeField.trailingAnchor.constraint(equalTo: cell.trailingAnchor, constant: -14.0),
            badgeField.centerYAnchor.constraint(equalTo: cell.centerYAnchor)
        ])

        return cell
    }

    // MARK: - NSTextFieldDelegate & Execution

    public func controlTextDidChange(_ obj: Notification) {
        updateSearch(searchField.stringValue)
    }

    public func control(_ control: NSControl, textView: NSTextView, doCommandBy commandSelector: Selector) -> Bool {
        if commandSelector == #selector(NSResponder.moveDown(_:)) {
            guard !items.isEmpty else { return true }
            let next = min(tableView.selectedRow + 1, items.count - 1)
            tableView.selectRowIndexes(IndexSet(integer: next), byExtendingSelection: false)
            tableView.scrollRowToVisible(next)
            return true
        } else if commandSelector == #selector(NSResponder.moveUp(_:)) {
            guard !items.isEmpty else { return true }
            let prev = max(tableView.selectedRow - 1, 0)
            tableView.selectRowIndexes(IndexSet(integer: prev), byExtendingSelection: false)
            tableView.scrollRowToVisible(prev)
            return true
        } else if commandSelector == #selector(NSResponder.insertNewline(_:)) {
            executeSelectedRow()
            return true
        } else if commandSelector == #selector(NSResponder.cancelOperation(_:)) {
            dismiss()
            return true
        } else if commandSelector == #selector(NSResponder.insertTab(_:)) {
            // Cycle prefixes smoothly on Tab when input is empty or a prefix
            let cur = searchField.stringValue
            let prefixes = [">", "/", "//", "@", "#", "p:", ""]
            if let idx = prefixes.firstIndex(of: cur) {
                let nextPrefix = prefixes[(idx + 1) % prefixes.count]
                updateSearch(nextPrefix)
                return true
            }
        }
        return false
    }

    @objc private func rowClicked(_ sender: Any?) {
        let row = tableView.clickedRow >= 0 ? tableView.clickedRow : tableView.selectedRow
        guard row >= 0, row < items.count else { return }
        tableView.selectRowIndexes(IndexSet(integer: row), byExtendingSelection: false)
        executeSelectedRow()
    }

    @objc private func rowDoubleClicked(_ sender: Any?) {
        executeSelectedRow()
    }

    public func executeSelectedRow() {
        let row = tableView.selectedRow
        guard row >= 0, row < items.count else { return }
        let action = items[row].action
        if action == .executeCommand("noop") {
            return
        }
        dismiss()
        onTriggerAction?(action)
    }

    public func windowDidResignKey(_ notification: Notification) {
        dismiss()
    }

    // MARK: - Hover Tracking

    @objc private func scrollBoundsChanged(_ notification: Notification) {
        hoverCoordinator.clear(in: tableView)
    }

    @objc public func mouseMoved(with event: NSEvent) {
        hoverCoordinator.handleMouseMoved(with: event, in: tableView)
    }

    @objc public func mouseExited(with event: NSEvent) {
        hoverCoordinator.clear(in: tableView)
    }
}
