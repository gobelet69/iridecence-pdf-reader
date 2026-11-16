import AppKit
import Foundation

/// Native semantic styling for the reader. Color identifies selection or an
/// actionable status; the document supplies the rest of the color.
public enum ReaderAppearance {
    @MainActor
    public static func drawSelection(in rect: NSRect, radius: CGFloat, emphasized: Bool = true) {
        guard rect.width > 4, rect.height > 4 else { return }
        let path = NSBezierPath(roundedRect: rect, xRadius: radius, yRadius: radius)
        let highContrast = NSWorkspace.shared.accessibilityDisplayShouldIncreaseContrast
        let color: NSColor = emphasized ? .controlAccentColor : .secondaryLabelColor
        color.withAlphaComponent(highContrast ? 0.28 : 0.14).setFill()
        path.fill()
        if highContrast {
            color.setStroke()
            path.lineWidth = 1
            path.stroke()
        }
    }

    @MainActor
    public static func drawHover(in rect: NSRect, radius: CGFloat) {
        guard rect.width > 4, rect.height > 4 else { return }
        NSColor.labelColor.withAlphaComponent(0.06).setFill()
        NSBezierPath(roundedRect: rect, xRadius: radius, yRadius: radius).fill()
    }
}

@MainActor
public final class DocumentEmblemView: NSImageView {
    public init(size: CGFloat = 40) {
        super.init(frame: NSRect(x: 0, y: 0, width: size, height: size))
        translatesAutoresizingMaskIntoConstraints = false
        image = NSImage(systemSymbolName: "doc.richtext", accessibilityDescription: "PDF Reader")
        contentTintColor = .secondaryLabelColor
        symbolConfiguration = NSImage.SymbolConfiguration(pointSize: size * 0.75, weight: .regular)
        imageScaling = .scaleProportionallyUpOrDown
    }
    @available(*, unavailable)
    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }
}

/// Quiet, noninteractive metadata with monospaced digits for stable counters.
@MainActor
public final class MetadataBadge: NSView {
    public enum Style {
        case neutral
        case error
    }

    public let label: NSTextField
    public var style: Style = .neutral {
        didSet { needsDisplay = true; updateTextColor() }
    }

    public init(text: String = "", style: Style = .neutral) {
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
        label.font = .monospacedDigitSystemFont(ofSize: 11, weight: .regular)
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

    private func updateTextColor() {
        label.textColor = style == .error ? .systemRed : .secondaryLabelColor
    }
}
