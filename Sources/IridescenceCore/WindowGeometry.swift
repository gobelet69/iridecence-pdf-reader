import AppKit
import Foundation

public enum WindowKind: Sendable {
    case home
    case reader
    case palette
    case debugConsole
    case utility

    public var defaultSize: NSSize {
        switch self {
        case .home:
            return NSSize(width: 900.0, height: 600.0)
        case .reader:
            return NSSize(width: 1240.0, height: 820.0)
        case .palette:
            return NSSize(width: 620.0, height: 48.0)
        case .debugConsole:
            return NSSize(width: 720.0, height: 480.0)
        case .utility:
            return NSSize(width: 500.0, height: 380.0)
        }
    }

    public var minSize: NSSize {
        switch self {
        case .home:
            return NSSize(width: 740.0, height: 480.0)
        case .reader:
            return NSSize(width: 780.0, height: 520.0)
        case .palette:
            return NSSize(width: 620.0, height: 48.0)
        case .debugConsole:
            return NSSize(width: 520.0, height: 340.0)
        case .utility:
            return NSSize(width: 400.0, height: 280.0)
        }
    }
}

public enum WindowGeometry {
    /// Pure helper to clamp and sanitize any window frame against a screen visibleFrame.
    /// Protects against NaN/Infinite coordinates and offscreen frames on smaller MacBook displays.
    public static func sanitizeRect(
        _ current: NSRect,
        for kind: WindowKind,
        in visible: NSRect
    ) -> NSRect {
        let minSize = kind.minSize
        let defSize = kind.defaultSize

        let maxW = max(minSize.width, visible.width - 24.0)
        let maxH = max(minSize.height, visible.height - 24.0)

        var w = current.width
        var h = current.height

        if !w.isFinite || !h.isFinite || w < minSize.width || h < minSize.height {
            w = defSize.width
            h = defSize.height
        }

        w = min(max(w, minSize.width), maxW)
        h = min(max(h, minSize.height), maxH)

        var x = current.origin.x
        var y = current.origin.y

        if !x.isFinite || !y.isFinite {
            x = visible.origin.x + (visible.width - w) * 0.5
            y = visible.origin.y + (visible.height - h) * 0.5
        } else {
            let maxX = max(visible.origin.x, visible.origin.x + visible.width - w)
            let maxY = max(visible.origin.y, visible.origin.y + visible.height - h)
            x = min(max(x, visible.origin.x), maxX)
            y = min(max(y, visible.origin.y), maxY)
        }

        return NSRect(x: x.rounded(), y: y.rounded(), width: w.rounded(), height: h.rounded())
    }

    @MainActor
    public static func activeVisibleFrame(for window: NSWindow? = nil) -> NSRect {
        if let screen = window?.screen ?? NSScreen.main ?? NSScreen.screens.first {
            let vf = screen.visibleFrame
            if vf.width >= 600.0 && vf.height >= 400.0 {
                return vf
            }
        }
        return NSRect(x: 0.0, y: 0.0, width: 1440.0, height: 900.0)
    }

    @MainActor
    public static func initialFrame(for kind: WindowKind) -> NSRect {
        let visible = activeVisibleFrame()
        let defSize = kind.defaultSize
        let minSize = kind.minSize

        let maxW = max(minSize.width, visible.width - 40.0)
        let maxH = max(minSize.height, visible.height - 40.0)

        let w = min(max(defSize.width, minSize.width), maxW)
        let h = min(max(defSize.height, minSize.height), maxH)

        let x = visible.origin.x + (visible.width - w) * 0.5
        let yRatio: CGFloat = (kind == .palette) ? 0.68 : 0.52
        let y = visible.origin.y + (visible.height - h) * yRatio

        return NSRect(x: x.rounded(), y: y.rounded(), width: w.rounded(), height: h.rounded())
    }

    @MainActor
    public static func sanitizeWindow(_ window: NSWindow, kind: WindowKind) {
        let visible = activeVisibleFrame(for: window)
        let minSize = kind.minSize
        let maxW = max(minSize.width, visible.width - 24.0)
        let maxH = max(minSize.height, visible.height - 24.0)

        window.minSize = NSSize(width: min(minSize.width, maxW), height: min(minSize.height, maxH))
        let sanitized = sanitizeRect(window.frame, for: kind, in: visible)
        if abs(sanitized.origin.x - window.frame.origin.x) > 1.0
            || abs(sanitized.origin.y - window.frame.origin.y) > 1.0
            || abs(sanitized.width - window.frame.width) > 1.0
            || abs(sanitized.height - window.frame.height) > 1.0
        {
            window.setFrame(sanitized, display: true)
        }
    }
}
