import AppKit
import Foundation

/// Centralized design system for **Iridescence PDF Reader**.
/// Defines dynamic spectral colors, surface tints, typography, corner radii,
/// and reusable iridescent UI components that adapt automatically to Light Mode, Dark Mode,
/// Increase Contrast, and Reduce Transparency accessibility settings.
public enum IridescenceDesign {

    // MARK: - Dynamic Spectral Palette (Synced with 111iridescence v2.1 Homepage Matrix)

    /// Primary Iridescence Deep Violet (`#702EE4` in Light, `#9478FF` in Dark)
    public static var primaryViolet: NSColor {
        NSColor(name: nil) { appearance in
            let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            let highContrast = NSWorkspace.shared.accessibilityDisplayShouldIncreaseContrast
            if highContrast {
                return isDark
                    ? NSColor(srgbRed: 0.68, green: 0.58, blue: 1.00, alpha: 1.0)
                    : NSColor(srgbRed: 0.34, green: 0.14, blue: 0.78, alpha: 1.0)
            }
            return isDark
                ? NSColor(srgbRed: 0.580, green: 0.471, blue: 1.000, alpha: 1.0)
                : NSColor(srgbRed: 0.439, green: 0.180, blue: 0.894, alpha: 1.0)
        }
    }

    /// Homepage Prefix Electric Blue (`#2266F2` in Light, `#60A5FA` in Dark)
    public static var secondaryIndigo: NSColor {
        NSColor(name: nil) { appearance in
            let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            return isDark
                ? NSColor(srgbRed: 0.376, green: 0.647, blue: 0.980, alpha: 1.0)
                : NSColor(srgbRed: 0.133, green: 0.400, blue: 0.949, alpha: 1.0)
        }
    }

    /// Luminous Spectral Cyan (`#0CC0E8` in Light, `#38BDF8` in Dark)
    public static var accentCyan: NSColor {
        NSColor(name: nil) { appearance in
            let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            return isDark
                ? NSColor(srgbRed: 0.220, green: 0.741, blue: 0.973, alpha: 1.0)
                : NSColor(srgbRed: 0.047, green: 0.753, blue: 0.910, alpha: 1.0)
        }
    }

    /// Subtle Iridescent Magenta (`#DC3EA0` in Light, `#F472B6` in Dark)
    public static var accentRose: NSColor {
        NSColor(name: nil) { appearance in
            let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            return isDark
                ? NSColor(srgbRed: 0.957, green: 0.447, blue: 0.714, alpha: 1.0)
                : NSColor(srgbRed: 0.863, green: 0.243, blue: 0.627, alpha: 1.0)
        }
    }

    /// Controlled Turquoise (`#12CCB8` in Light, `#2DD4BF` in Dark)
    public static var controlledTurquoise: NSColor {
        NSColor(name: nil) { appearance in
            let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            return isDark
                ? NSColor(srgbRed: 0.176, green: 0.831, blue: 0.749, alpha: 1.0)
                : NSColor(srgbRed: 0.071, green: 0.800, blue: 0.722, alpha: 1.0)
        }
    }

    /// Warm Amber Gold for Favorites / Bookmarks (`#D97706` in Light, `#FBBF24` in Dark)
    public static var accentAmber: NSColor {
        NSColor(name: nil) { appearance in
            let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            return isDark
                ? NSColor(srgbRed: 0.98, green: 0.75, blue: 0.14, alpha: 1.0)
                : NSColor(srgbRed: 0.85, green: 0.47, blue: 0.02, alpha: 1.0)
        }
    }

    /// Subtle elevated card fill color that avoids flat neutral gray
    public static var cardSurfaceFill: NSColor {
        NSColor(name: nil) { appearance in
            let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            let reduceTrans = NSWorkspace.shared.accessibilityDisplayShouldReduceTransparency
            if reduceTrans {
                return isDark
                    ? NSColor(srgbRed: 0.14, green: 0.14, blue: 0.18, alpha: 1.0)
                    : NSColor(srgbRed: 0.96, green: 0.96, blue: 0.99, alpha: 1.0)
            }
            return isDark
                ? NSColor(srgbRed: 0.48, green: 0.42, blue: 0.92, alpha: 0.09)
                : NSColor(srgbRed: 0.44, green: 0.32, blue: 0.95, alpha: 0.045)
        }
    }

    /// Subtle card border with iridescent violet-indigo tint
    public static var cardBorderColor: NSColor {
        NSColor(name: nil) { appearance in
            let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            let highContrast = NSWorkspace.shared.accessibilityDisplayShouldIncreaseContrast
            if highContrast {
                return isDark
                    ? NSColor(srgbRed: 0.65, green: 0.58, blue: 1.00, alpha: 0.65)
                    : NSColor(srgbRed: 0.35, green: 0.22, blue: 0.85, alpha: 0.55)
            }
            return isDark
                ? NSColor(srgbRed: 0.62, green: 0.55, blue: 1.00, alpha: 0.22)
                : NSColor(srgbRed: 0.45, green: 0.34, blue: 0.92, alpha: 0.16)
        }
    }

    /// Calm, high-contrast neutral backdrop for the PDF reading canvas (never tinted with distracting gradients)
    public static var pdfCanvasBackdropColor: NSColor {
        NSColor(name: nil) { appearance in
            let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            return isDark
                ? NSColor(srgbRed: 0.10, green: 0.105, blue: 0.125, alpha: 0.88)
                : NSColor(srgbRed: 0.945, green: 0.948, blue: 0.962, alpha: 0.82)
        }
    }

    // MARK: - Selection & Hover Gradient Stops

    @MainActor
    public static func drawIridescentSelectionCapsule(in rect: NSRect, radius: CGFloat) {
        guard rect.width > 4.0, rect.height > 4.0 else { return }
        let path = NSBezierPath(roundedRect: rect, xRadius: radius, yRadius: radius)

        let isDark = NSApp.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        let highContrast = NSWorkspace.shared.accessibilityDisplayShouldIncreaseContrast

        let startAlpha: CGFloat = highContrast ? 0.42 : (isDark ? 0.30 : 0.18)
        let midAlpha: CGFloat   = highContrast ? 0.38 : (isDark ? 0.25 : 0.15)
        let endAlpha: CGFloat   = highContrast ? 0.36 : (isDark ? 0.24 : 0.14)

        let c1 = primaryViolet.withAlphaComponent(startAlpha)
        let c2 = secondaryIndigo.withAlphaComponent(midAlpha)
        let c3 = accentCyan.withAlphaComponent(endAlpha)

        if let gradient = NSGradient(colors: [c1, c2, c3], atLocations: [0.0, 0.58, 1.0], colorSpace: .sRGB) {
            gradient.draw(in: path, angle: 12.0)
        } else {
            c1.setFill()
            path.fill()
        }

        // Specular iridescent border
        let borderAlpha: CGFloat = highContrast ? 0.85 : (isDark ? 0.52 : 0.38)
        primaryViolet.withAlphaComponent(borderAlpha).setStroke()
        path.lineWidth = highContrast ? 1.5 : 1.0
        path.stroke()

        // Left accent indicator bar inside the selection capsule for unambiguous selection identity
        let barHeight = max(6.0, rect.height - 10.0)
        let barRect = NSRect(
            x: rect.minX + 3.0,
            y: rect.minY + (rect.height - barHeight) * 0.5,
            width: 3.0,
            height: barHeight
        )
        let barPath = NSBezierPath(roundedRect: barRect, xRadius: 1.5, yRadius: 1.5)
        primaryViolet.withAlphaComponent(isDark ? 0.95 : 0.88).setFill()
        barPath.fill()
    }

    @MainActor
    public static func drawIridescentHoverCapsule(in rect: NSRect, radius: CGFloat) {
        guard rect.width > 4.0, rect.height > 4.0 else { return }
        let path = NSBezierPath(roundedRect: rect, xRadius: radius, yRadius: radius)

        let isDark = NSApp.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        let highContrast = NSWorkspace.shared.accessibilityDisplayShouldIncreaseContrast

        let fillAlpha: CGFloat = highContrast ? 0.16 : (isDark ? 0.10 : 0.065)
        let borderAlpha: CGFloat = highContrast ? 0.32 : (isDark ? 0.18 : 0.12)

        primaryViolet.withAlphaComponent(fillAlpha).setFill()
        path.fill()

        primaryViolet.withAlphaComponent(borderAlpha).setStroke()
        path.lineWidth = 0.75
        path.stroke()
    }
}

/// Subtle ambient spectral overlay placed inside navigation/inspector `NSVisualEffectView` containers
/// so sidebars, the welcome screen, and the command palette have a warm, iridescent surface character
/// rather than looking like monotonous flat gray rectangles.
/// Automatically disables itself when `accessibilityDisplayShouldReduceTransparency` is enabled.
@MainActor
public final class IridescentSurfaceWashView: NSView {
    public enum SurfaceRole {
        case sidebar
        case welcomeHero
        case welcomeCanvas
        case inspector
        case palette
    }

    public let role: SurfaceRole

    public init(role: SurfaceRole, frame: NSRect = .zero) {
        self.role = role
        super.init(frame: frame)
        self.translatesAutoresizingMaskIntoConstraints = false
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    public override var isOpaque: Bool { false }

    public override func hitTest(_ point: NSPoint) -> NSView? {
        // Purely decorative background layer; never intercepts mouse events
        nil
    }

    public override func draw(_ dirtyRect: NSRect) {
        let b = bounds
        guard b.width > 1.0, b.height > 1.0 else { return }

        if NSWorkspace.shared.accessibilityDisplayShouldReduceTransparency {
            return
        }

        let isDark = effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua

        switch role {
        case .sidebar:
            let topColor = IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.085 : 0.048)
            let midColor = IridescenceDesign.secondaryIndigo.withAlphaComponent(isDark ? 0.045 : 0.025)
            let botColor = IridescenceDesign.accentCyan.withAlphaComponent(isDark ? 0.055 : 0.030)
            if let grad = NSGradient(colors: [topColor, midColor, botColor], atLocations: [0.0, 0.55, 1.0], colorSpace: .sRGB) {
                grad.draw(in: b, angle: -75.0)
            }

        case .inspector:
            let topColor = IridescenceDesign.secondaryIndigo.withAlphaComponent(isDark ? 0.075 : 0.040)
            let botColor = IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.050 : 0.025)
            if let grad = NSGradient(starting: topColor, ending: botColor) {
                grad.draw(in: b, angle: -90.0)
            }

        case .welcomeCanvas:
            let c1 = IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.09 : 0.045)
            let c2 = IridescenceDesign.accentCyan.withAlphaComponent(isDark ? 0.05 : 0.025)
            let c3 = NSColor.clear
            if let grad = NSGradient(colors: [c1, c2, c3], atLocations: [0.0, 0.45, 1.0], colorSpace: .sRGB) {
                grad.draw(in: b, angle: -45.0)
            }

        case .welcomeHero:
            let path = NSBezierPath(roundedRect: b.insetBy(dx: 0.5, dy: 0.5), xRadius: UIConstants.radiusCard, yRadius: UIConstants.radiusCard)
            let c1 = IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.22 : 0.11)
            let c2 = IridescenceDesign.secondaryIndigo.withAlphaComponent(isDark ? 0.16 : 0.08)
            let c3 = IridescenceDesign.accentCyan.withAlphaComponent(isDark ? 0.18 : 0.09)
            let c4 = IridescenceDesign.accentRose.withAlphaComponent(isDark ? 0.12 : 0.06)
            if let grad = NSGradient(colors: [c1, c2, c3, c4], atLocations: [0.0, 0.4, 0.78, 1.0], colorSpace: .sRGB) {
                grad.draw(in: path, angle: 18.0)
            }
            IridescenceDesign.cardBorderColor.setStroke()
            path.lineWidth = 1.0
            path.stroke()

        case .palette:
            let c1 = IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.12 : 0.06)
            let c2 = IridescenceDesign.accentCyan.withAlphaComponent(isDark ? 0.07 : 0.035)
            let c3 = NSColor.clear
            if let grad = NSGradient(colors: [c1, c2, c3], atLocations: [0.0, 0.35, 1.0], colorSpace: .sRGB) {
                grad.draw(in: b, angle: -65.0)
            }
            let borderRect = b.insetBy(dx: 0.75, dy: 0.75)
            let borderPath = NSBezierPath(roundedRect: borderRect, xRadius: UIConstants.radiusPanel, yRadius: UIConstants.radiusPanel)
            IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.38 : 0.24).setStroke()
            borderPath.lineWidth = 1.2
            borderPath.stroke()
        }
    }
}

/// Custom vector prismatic emblem representing **Iridescence PDF Reader**.
@MainActor
public final class IridescentEmblemView: NSView {
    public init(size: CGFloat = 40.0) {
        super.init(frame: NSRect(x: 0, y: 0, width: size, height: size))
        self.translatesAutoresizingMaskIntoConstraints = false
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    public override func draw(_ dirtyRect: NSRect) {
        let b = bounds.insetBy(dx: 1.0, dy: 1.0)
        guard b.width > 8.0, b.height > 8.0 else { return }

        let isDark = effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        let radius = min(b.width, b.height) * 0.24
        let outerPath = NSBezierPath(roundedRect: b, xRadius: radius, yRadius: radius)

        // Luminous Pearl squircle base (or Deep Obsidian in dark mode)
        let baseStart = isDark
            ? NSColor(srgbRed: 0.14, green: 0.11, blue: 0.20, alpha: 1.0)
            : NSColor(srgbRed: 1.00, green: 1.00, blue: 1.00, alpha: 1.0)
        let baseEnd = isDark
            ? NSColor(srgbRed: 0.09, green: 0.07, blue: 0.13, alpha: 1.0)
            : NSColor(srgbRed: 0.94, green: 0.92, blue: 0.97, alpha: 1.0)
        if let grad = NSGradient(starting: baseStart, ending: baseEnd) {
            grad.draw(in: outerPath, angle: -90.0)
        } else {
            baseStart.setFill()
            outerPath.fill()
        }

        IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.45 : 0.22).setStroke()
        outerPath.lineWidth = 1.0
        outerPath.stroke()

        // Inner folio document card
        let docRect = b.insetBy(dx: b.width * 0.14, dy: b.height * 0.12)
        let docRadius = max(3.0, docRect.width * 0.12)
        let docPath = NSBezierPath(roundedRect: docRect, xRadius: docRadius, yRadius: docRadius)
        NSColor.white.withAlphaComponent(isDark ? 0.94 : 1.0).setFill()
        docPath.fill()

        let obsidian = NSColor(srgbRed: 0.086, green: 0.063, blue: 0.114, alpha: 1.0)
        obsidian.withAlphaComponent(0.85).setStroke()
        docPath.lineWidth = 1.0
        docPath.stroke()

        // Left Obsidian pixel-grid reading spine
        let spineWidth = max(6.0, floor(docRect.width * 0.26))
        let spineRect = NSRect(x: docRect.minX, y: docRect.minY, width: spineWidth, height: docRect.height)
        NSGraphicsContext.saveGraphicsState()
        docPath.addClip()
        obsidian.setFill()
        spineRect.fill()

        // Grid-locked spectral pixel cells inside spine (5 rows)
        let rows = 5
        let cellW = max(2.0, floor((spineWidth - 3.0) * 0.55))
        let cellH = max(2.0, floor((docRect.height - 6.0) / CGFloat(rows) - 1.5))
        let spineColors = [
            IridescenceDesign.accentCyan,
            IridescenceDesign.secondaryIndigo,
            IridescenceDesign.primaryViolet,
            IridescenceDesign.primaryViolet,
            IridescenceDesign.controlledTurquoise
        ]
        for r in 0..<rows {
            let cy = docRect.maxY - 3.0 - CGFloat(r + 1) * (cellH + 1.2)
            let cx = spineRect.minX + floor((spineWidth - cellW) * 0.5)
            spineColors[r].setFill()
            NSBezierPath(roundedRect: NSRect(x: cx, y: cy, width: cellW, height: cellH), xRadius: 0.8, yRadius: 0.8).fill()
        }
        NSGraphicsContext.restoreGraphicsState()

        // Right page content: structured reading bars + top-right 5-cell pixel star (`cross-5`)
        let contentMinX = spineRect.maxX + max(2.5, docRect.width * 0.08)
        let contentMaxW = max(4.0, docRect.maxX - contentMinX - 3.0)

        // Highlight callout bar
        let calloutY = docRect.minY + docRect.height * 0.50
        let calloutRect = NSRect(x: contentMinX, y: calloutY, width: contentMaxW * 0.88, height: max(3.0, docRect.height * 0.16))
        IridescenceDesign.primaryViolet.withAlphaComponent(0.18).setFill()
        NSBezierPath(roundedRect: calloutRect, xRadius: 1.5, yRadius: 1.5).fill()

        // Reading lines
        let lineYs: [CGFloat] = [0.74, 0.34, 0.20]
        let lineWs: [CGFloat] = [0.52, 0.82, 0.64]
        for i in 0..<lineYs.count {
            let ly = docRect.minY + docRect.height * lineYs[i]
            let lw = contentMaxW * lineWs[i]
            let lRect = NSRect(x: contentMinX, y: ly, width: lw, height: max(1.6, docRect.height * 0.065))
            (i == 0 ? obsidian : obsidian.withAlphaComponent(0.65)).setFill()
            NSBezierPath(roundedRect: lRect, xRadius: 0.8, yRadius: 0.8).fill()
        }

        // Signature 5-cell pixel cross star (`cross-5`) in upper right of page
        let px = max(1.5, floor(docRect.width * 0.055))
        let starCx = docRect.maxX - px * 2.6
        let starCy = docRect.maxY - px * 2.6
        let starOffsets: [(CGFloat, CGFloat, NSColor)] = [
            (0, 0, IridescenceDesign.accentCyan),
            (0, 1, IridescenceDesign.secondaryIndigo),
            (-1, 0, IridescenceDesign.primaryViolet),
            (1, 0, IridescenceDesign.accentRose),
            (0, -1, IridescenceDesign.primaryViolet)
        ]
        for (dx, dy, color) in starOffsets {
            color.setFill()
            NSRect(x: starCx + dx * px, y: starCy + dy * px, width: px * 0.88, height: px * 0.88).fill()
        }
    }
}

/// Reusable capsule pill badge (`p.1`, `Workspace`, `iCloud`, `Downloading…`, `COMMANDS`)
/// with semantic iridescent tinting and crisp monospaced-digit typography.
@MainActor
public final class IridescentPillBadge: NSView {
    public enum Style {
        case violet
        case cyan
        case amber
        case rose
        case neutral
        case error
    }

    public let label: NSTextField
    public var style: Style = .violet {
        didSet { needsDisplay = true; updateTextColor() }
    }

    public init(text: String = "", style: Style = .violet) {
        self.label = NSTextField(labelWithString: text)
        self.style = style
        super.init(frame: .zero)
        self.translatesAutoresizingMaskIntoConstraints = false
        setupLabel()
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupLabel() {
        label.font = .monospacedDigitSystemFont(ofSize: 10.5, weight: .semibold)
        label.alignment = .center
        label.lineBreakMode = .byTruncatingTail
        label.setContentCompressionResistancePriority(.required, for: .horizontal)
        label.setContentHuggingPriority(.required, for: .horizontal)
        label.translatesAutoresizingMaskIntoConstraints = false
        addSubview(label)

        NSLayoutConstraint.activate([
            label.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 7.0),
            label.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -7.0),
            label.topAnchor.constraint(equalTo: topAnchor, constant: 2.0),
            label.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -2.0)
        ])
        updateTextColor()
    }

    public var stringValue: String {
        get { label.stringValue }
        set {
            label.stringValue = newValue
            isHidden = newValue.isEmpty
            invalidateIntrinsicContentSize()
            needsDisplay = true
        }
    }

    private func baseAccentColor() -> NSColor {
        switch style {
        case .violet: return IridescenceDesign.primaryViolet
        case .cyan: return IridescenceDesign.accentCyan
        case .amber: return IridescenceDesign.accentAmber
        case .rose: return IridescenceDesign.accentRose
        case .neutral: return .secondaryLabelColor
        case .error: return .systemRed
        }
    }

    private func updateTextColor() {
        switch style {
        case .neutral:
            label.textColor = .secondaryLabelColor
        default:
            label.textColor = baseAccentColor()
        }
    }

    public override func draw(_ dirtyRect: NSRect) {
        guard !label.stringValue.isEmpty else { return }
        let b = bounds.insetBy(dx: 0.5, dy: 0.5)
        guard b.width > 4.0, b.height > 4.0 else { return }

        let radius = b.height * 0.5
        let path = NSBezierPath(roundedRect: b, xRadius: radius, yRadius: radius)
        let isDark = effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        let accent = baseAccentColor()

        accent.withAlphaComponent(isDark ? 0.18 : 0.11).setFill()
        path.fill()

        accent.withAlphaComponent(isDark ? 0.36 : 0.24).setStroke()
        path.lineWidth = 0.75
        path.stroke()
    }
}
