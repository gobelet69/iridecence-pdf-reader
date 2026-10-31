import AppKit
import Foundation

public enum UIConstants {
    // MARK: - Corner Radii (Concentric Geometry)
    public static let radiusWindow: CGFloat = 12.0
    public static let radiusPanel: CGFloat = 16.0
    public static let radiusCard: CGFloat = 10.0
    public static let radiusRow: CGFloat = 6.5
    public static let radiusBadge: CGFloat = 4.0

    // MARK: - Sidebar & Split View Widths
    public static let sidebarWidthMin: CGFloat = 180.0
    public static let sidebarWidthDefault: CGFloat = 230.0
    public static let sidebarWidthMax: CGFloat = 360.0

    public static let homeSidebarWidthMin: CGFloat = 180.0
    public static let homeSidebarWidthDefault: CGFloat = 210.0
    public static let homeSidebarWidthMax: CGFloat = 280.0

    public static let readerCenterMinWidth: CGFloat = 420.0
    public static let responsiveCollapseInspectorWidth: CGFloat = 860.0
    public static let responsiveCollapseSidebarWidth: CGFloat = 680.0

    // MARK: - Row Heights
    public static let rowHeightCompact: CGFloat = 28.0
    public static let rowHeightSidebar: CGFloat = 32.0
    public static let rowHeightStandard: CGFloat = 34.0
    public static let rowHeightPalette: CGFloat = 38.0
    public static let rowHeightSpacious: CGFloat = 44.0

    // MARK: - Command Palette Dimensions
    public static let paletteWidth: CGFloat = 620.0
    public static let paletteCollapsedHeight: CGFloat = 48.0
    public static let paletteFooterHeight: CGFloat = 26.0
    public static let paletteMaxVisibleRows: Int = 8

    // MARK: - Path Normalization Helper
    /// Resolves `/private/tmp` and `/private/var` symlink differences on macOS so file identity comparisons never fail.
    public static func canonicalPath(_ rawPath: String) -> String {
        let trimmed = rawPath.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return "" }
        let url = URL(fileURLWithPath: trimmed).standardizedFileURL
        if let resolved = try? FileManager.default.destinationOfSymbolicLink(atPath: url.path) {
            let resolvedURL = URL(fileURLWithPath: resolved, relativeTo: url.deletingLastPathComponent()).standardizedFileURL
            return stripPrivatePrefix(resolvedURL.resolvingSymlinksInPath().path)
        }
        return stripPrivatePrefix(url.resolvingSymlinksInPath().path)
    }

    public static func stripPrivatePrefix(_ path: String) -> String {
        if path == "/private" {
            return "/"
        }
        if path.hasPrefix("/private/tmp/") || path == "/private/tmp" {
            return String(path.dropFirst("/private".count))
        }
        if path.hasPrefix("/private/var/") || path == "/private/var" {
            return String(path.dropFirst("/private".count))
        }
        return path
    }
}
