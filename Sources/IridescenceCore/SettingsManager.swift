import AppKit
import Foundation

public enum SettingsKeys {
    public static let restoreSession = "PrefRestoreSession"
    public static let rememberRecents = "PrefRememberRecents"
    public static let reopenClosedTabs = "PrefReopenClosedTabs"
    public static let openInNewTab = "PrefOpenInNewTab"

    public static let defaultDisplayMode = "PrefDefaultDisplayMode" // 0=Single, 1=Continuous, 2=TwoUp
    public static let defaultZoomMode = "PrefDefaultZoomMode"       // 0=FitWidth, 1=ActualSize
    public static let defaultHighlightColor = "PrefDefaultHighlightColor" // 0=Yellow, 1=Green, 2=Blue, 3=Pink
    public static let restoreReadingPos = "PrefRestoreReadingPos"

    public static let autoRevealSidebar = "PrefAutoRevealSidebar"
    public static let showHiddenFiles = "PrefShowHiddenFiles"
    public static let deepWorkspaceSearch = "PrefDeepWorkspaceSearch"

    public static let appearanceMode = "PrefAppearanceMode" // 0=System, 1=Light, 2=Dark
    public static let liquidGlass = "PrefLiquidGlass"
    public static let autoCopyOcr = "PrefAutoCopyOcr"

    // Non-preference state keys preserved from Rust baseline
    public static let recentFiles = "RecentFiles"
    public static let recentItems = "RecentItems"
    public static let recentWorkspaces = "RecentWorkspaces"
    public static let favoriteFiles = "FavoriteFiles"
    public static let userBookmarksV2 = "UserBookmarks_v2"
    public static let rightSidebarMode = "RightSidebarMode"
}

public final class SettingsManager: @unchecked Sendable {
    public static let shared = SettingsManager()

    private let defaults: UserDefaults

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        registerDefaults()
    }

    public func registerDefaults() {
        let initial: [String: Any] = [
            SettingsKeys.restoreSession: true,
            SettingsKeys.rememberRecents: true,
            SettingsKeys.reopenClosedTabs: true,
            SettingsKeys.openInNewTab: true,
            SettingsKeys.defaultDisplayMode: 1,
            SettingsKeys.defaultZoomMode: 0,
            SettingsKeys.defaultHighlightColor: 0,
            SettingsKeys.restoreReadingPos: true,
            SettingsKeys.autoRevealSidebar: true,
            SettingsKeys.showHiddenFiles: false,
            SettingsKeys.deepWorkspaceSearch: true,
            SettingsKeys.appearanceMode: 0,
            SettingsKeys.liquidGlass: true,
            SettingsKeys.autoCopyOcr: true
        ]
        defaults.register(defaults: initial)
    }

    public func bool(forKey key: String, default fallback: Bool = true) -> Bool {
        if defaults.object(forKey: key) == nil {
            return fallback
        }
        return defaults.bool(forKey: key)
    }

    public func int(forKey key: String, default fallback: Int = 0) -> Int {
        if defaults.object(forKey: key) == nil {
            return fallback
        }
        return defaults.integer(forKey: key)
    }

    public func set(_ value: Bool, forKey key: String) {
        defaults.set(value, forKey: key)
    }

    public func set(_ value: Int, forKey key: String) {
        defaults.set(value, forKey: key)
    }

    // MARK: - Typed Properties
    public var restoreSession: Bool {
        get { bool(forKey: SettingsKeys.restoreSession, default: true) }
        set { set(newValue, forKey: SettingsKeys.restoreSession) }
    }

    public var rememberRecents: Bool {
        get { bool(forKey: SettingsKeys.rememberRecents, default: true) }
        set { set(newValue, forKey: SettingsKeys.rememberRecents) }
    }

    public var reopenClosedTabs: Bool {
        get { bool(forKey: SettingsKeys.reopenClosedTabs, default: true) }
        set { set(newValue, forKey: SettingsKeys.reopenClosedTabs) }
    }

    public var openInNewTab: Bool {
        get { bool(forKey: SettingsKeys.openInNewTab, default: true) }
        set { set(newValue, forKey: SettingsKeys.openInNewTab) }
    }

    public var defaultDisplayMode: Int {
        get { min(max(int(forKey: SettingsKeys.defaultDisplayMode, default: 1), 0), 2) }
        set { set(min(max(newValue, 0), 2), forKey: SettingsKeys.defaultDisplayMode) }
    }

    public var defaultZoomMode: Int {
        get { min(max(int(forKey: SettingsKeys.defaultZoomMode, default: 0), 0), 1) }
        set { set(min(max(newValue, 0), 1), forKey: SettingsKeys.defaultZoomMode) }
    }

    public var defaultHighlightColorIndex: Int {
        get { min(max(int(forKey: SettingsKeys.defaultHighlightColor, default: 0), 0), 3) }
        set { set(min(max(newValue, 0), 3), forKey: SettingsKeys.defaultHighlightColor) }
    }

    public var restoreReadingPos: Bool {
        get { bool(forKey: SettingsKeys.restoreReadingPos, default: true) }
        set { set(newValue, forKey: SettingsKeys.restoreReadingPos) }
    }

    public var autoRevealSidebar: Bool {
        get { bool(forKey: SettingsKeys.autoRevealSidebar, default: true) }
        set { set(newValue, forKey: SettingsKeys.autoRevealSidebar) }
    }

    public var showHiddenFiles: Bool {
        get { bool(forKey: SettingsKeys.showHiddenFiles, default: false) }
        set { set(newValue, forKey: SettingsKeys.showHiddenFiles) }
    }

    public var deepWorkspaceSearch: Bool {
        get { bool(forKey: SettingsKeys.deepWorkspaceSearch, default: true) }
        set { set(newValue, forKey: SettingsKeys.deepWorkspaceSearch) }
    }

    public var appearanceMode: Int {
        get { min(max(int(forKey: SettingsKeys.appearanceMode, default: 0), 0), 2) }
        set { set(min(max(newValue, 0), 2), forKey: SettingsKeys.appearanceMode) }
    }

    public var liquidGlass: Bool {
        get { bool(forKey: SettingsKeys.liquidGlass, default: true) }
        set { set(newValue, forKey: SettingsKeys.liquidGlass) }
    }

    public var autoCopyOcr: Bool {
        get { bool(forKey: SettingsKeys.autoCopyOcr, default: true) }
        set { set(newValue, forKey: SettingsKeys.autoCopyOcr) }
    }

    public func restoreAllDefaults() {
        defaults.set(true, forKey: SettingsKeys.restoreSession)
        defaults.set(true, forKey: SettingsKeys.rememberRecents)
        defaults.set(true, forKey: SettingsKeys.reopenClosedTabs)
        defaults.set(true, forKey: SettingsKeys.openInNewTab)
        defaults.set(1, forKey: SettingsKeys.defaultDisplayMode)
        defaults.set(0, forKey: SettingsKeys.defaultZoomMode)
        defaults.set(0, forKey: SettingsKeys.defaultHighlightColor)
        defaults.set(true, forKey: SettingsKeys.restoreReadingPos)
        defaults.set(true, forKey: SettingsKeys.autoRevealSidebar)
        defaults.set(false, forKey: SettingsKeys.showHiddenFiles)
        defaults.set(true, forKey: SettingsKeys.deepWorkspaceSearch)
        defaults.set(0, forKey: SettingsKeys.appearanceMode)
        defaults.set(true, forKey: SettingsKeys.liquidGlass)
        defaults.set(true, forKey: SettingsKeys.autoCopyOcr)
    }

    @MainActor
    public func applySavedAppearance() {
        switch appearanceMode {
        case 1:
            NSApp?.appearance = NSAppearance(named: .aqua)
        case 2:
            NSApp?.appearance = NSAppearance(named: .darkAqua)
        default:
            NSApp?.appearance = nil
        }
    }

    @MainActor
    public func configuredHighlightColor() -> NSColor {
        switch defaultHighlightColorIndex {
        case 1:
            return NSColor(srgbRed: 0.45, green: 0.90, blue: 0.50, alpha: 0.45) // Green
        case 2:
            return NSColor(srgbRed: 0.45, green: 0.75, blue: 0.98, alpha: 0.45) // Blue
        case 3:
            return NSColor(srgbRed: 0.98, green: 0.50, blue: 0.75, alpha: 0.45) // Pink
        default:
            return NSColor(srgbRed: 1.0, green: 0.92, blue: 0.23, alpha: 0.45)  // Yellow
        }
    }
}
