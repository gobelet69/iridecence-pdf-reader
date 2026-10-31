import AppKit
import Foundation

/// Custom `NSTableRowView` that suppresses private internal `NSVisualEffectView` selection layers
/// (which render as solid pitch-black rectangles in offscreen `cacheDisplayInRect:` captures)
/// and draws a translucent rounded capsule (`6.5pt` corner radius) with specular top edge
/// and full support for Increase Contrast accessibility mode.
@MainActor
public final class GlassTableRowView: NSTableRowView {
    public var isHovered: Bool = false {
        didSet {
            if oldValue != isHovered {
                needsDisplay = true
            }
        }
    }

    public override var isEmphasized: Bool {
        get { true }
        set { super.isEmphasized = newValue }
    }

    public override var interiorBackgroundStyle: NSView.BackgroundStyle {
        .normal
    }

    public override func layout() {
        super.layout()
        for subview in subviews {
            if subview is NSVisualEffectView {
                subview.isHidden = true
            }
        }
    }

    private func capsuleRect() -> NSRect {
        let b = bounds
        if let table = superview {
            let tableRectInRow = convert(table.bounds, from: table)
            let left = max(b.minX + 4.0, tableRectInRow.minX + 8.0)
            let right = min(b.maxX - 4.0, tableRectInRow.maxX - 8.0)
            if right > left {
                return NSRect(x: left, y: b.origin.y + 1.5, width: right - left, height: b.height - 3.0)
            }
        }
        return NSRect(x: b.origin.x + 6.0, y: b.origin.y + 1.5, width: b.width - 12.0, height: b.height - 3.0)
    }

    public override func drawSelection(in dirtyRect: NSRect) {
        guard isSelected else { return }
        let b = bounds
        guard b.width > 12.0, b.height > 4.0 else { return }

        let inset = capsuleRect()
        IridescenceDesign.drawIridescentSelectionCapsule(in: inset, radius: UIConstants.radiusRow)
    }

    public override func drawBackground(in dirtyRect: NSRect) {
        guard isHovered && !isSelected else { return }
        let b = bounds
        guard b.width > 12.0, b.height > 4.0 else { return }

        let inset = capsuleRect()
        IridescenceDesign.drawIridescentHoverCapsule(in: inset, radius: UIConstants.radiusRow)
    }
}

/// Enforces the single-row hover invariant across table and outline views.
/// Automatically clears stale hover highlights when the cursor exits or the view scrolls.
@MainActor
public final class HoverCoordinator {
    public private(set) var currentRow: Int = -1

    public init() {}

    public func update(row newRow: Int, in tableView: NSTableView?) {
        let oldRow = currentRow
        guard oldRow != newRow else { return }
        currentRow = newRow

        guard let tableView else { return }
        let rowCount = tableView.numberOfRows

        if oldRow >= 0 && oldRow < rowCount {
            if let oldView = tableView.rowView(atRow: oldRow, makeIfNecessary: false) as? GlassTableRowView {
                oldView.isHovered = false
            }
        }

        if newRow >= 0 && newRow < rowCount {
            if let newView = tableView.rowView(atRow: newRow, makeIfNecessary: false) as? GlassTableRowView {
                newView.isHovered = true
            }
        }
    }

    public func clear(in tableView: NSTableView?) {
        update(row: -1, in: tableView)
    }

    public func handleMouseMoved(with event: NSEvent, in tableView: NSTableView?) {
        guard let tableView else { return }
        let localPoint = tableView.convert(event.locationInWindow, from: nil)
        let visibleRect = tableView.visibleRect
        guard visibleRect.contains(localPoint) else {
            clear(in: tableView)
            return
        }
        let row = tableView.row(at: localPoint)
        update(row: row, in: tableView)
    }
}
