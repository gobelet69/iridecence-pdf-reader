import AppKit
import Foundation

/// Circular iridescent halo badge behind empty-state SF Symbol icons.
@MainActor
private final class IridescentIconMedallionView: NSView {
    override func draw(_ dirtyRect: NSRect) {
        let b = bounds.insetBy(dx: 1.0, dy: 1.0)
        guard b.width > 8.0, b.height > 8.0 else { return }
        let path = NSBezierPath(ovalIn: b)
        let isDark = effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua

        let c1 = IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.22 : 0.13)
        let c2 = IridescenceDesign.accentCyan.withAlphaComponent(isDark ? 0.16 : 0.09)
        if let grad = NSGradient(starting: c1, ending: c2) {
            grad.draw(in: path, angle: 45.0)
        } else {
            c1.setFill()
            path.fill()
        }

        IridescenceDesign.primaryViolet.withAlphaComponent(isDark ? 0.38 : 0.24).setStroke()
        path.lineWidth = 1.0
        path.stroke()
    }
}

/// Reusable centered empty state view with an iridescent medallion backplate,
/// SF Symbol icon, primary title, and multi-line guidance subtitle.
@MainActor
public final class EmptyStateView: NSView {
    public let iconView: NSImageView
    public let titleLabel: NSTextField
    public let subtitleLabel: NSTextField
    private let medallionView = IridescentIconMedallionView()

    public init(symbolName: String, title: String, subtitle: String) {
        self.iconView = NSImageView()
        self.titleLabel = NSTextField(labelWithString: title)
        self.subtitleLabel = NSTextField(wrappingLabelWithString: subtitle)
        super.init(frame: NSRect(x: 0, y: 0, width: 230, height: 220))
        setupHierarchy(symbolName: symbolName, title: title, subtitle: subtitle)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupHierarchy(symbolName: String, title: String, subtitle: String) {
        translatesAutoresizingMaskIntoConstraints = false

        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .centerX
        stack.spacing = 6.0
        stack.translatesAutoresizingMaskIntoConstraints = false

        medallionView.translatesAutoresizingMaskIntoConstraints = false
        iconView.translatesAutoresizingMaskIntoConstraints = false
        iconView.contentTintColor = IridescenceDesign.primaryViolet
        configureSymbol(symbolName)
        medallionView.addSubview(iconView)

        titleLabel.stringValue = title
        titleLabel.font = .systemFont(ofSize: 13.5, weight: .semibold)
        titleLabel.textColor = .labelColor
        titleLabel.alignment = .center
        titleLabel.maximumNumberOfLines = 2
        titleLabel.translatesAutoresizingMaskIntoConstraints = false

        subtitleLabel.stringValue = subtitle
        subtitleLabel.font = .systemFont(ofSize: 11.5, weight: .regular)
        subtitleLabel.textColor = .secondaryLabelColor
        subtitleLabel.alignment = .center
        subtitleLabel.maximumNumberOfLines = 0
        subtitleLabel.lineBreakMode = .byWordWrapping
        subtitleLabel.preferredMaxLayoutWidth = 195.0
        subtitleLabel.translatesAutoresizingMaskIntoConstraints = false

        stack.addArrangedSubview(medallionView)
        stack.addArrangedSubview(titleLabel)
        stack.addArrangedSubview(subtitleLabel)
        stack.setCustomSpacing(12.0, after: medallionView)

        addSubview(stack)

        NSLayoutConstraint.activate([
            medallionView.widthAnchor.constraint(equalToConstant: 52.0),
            medallionView.heightAnchor.constraint(equalToConstant: 52.0),
            iconView.centerXAnchor.constraint(equalTo: medallionView.centerXAnchor),
            iconView.centerYAnchor.constraint(equalTo: medallionView.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: 26.0),
            iconView.heightAnchor.constraint(equalToConstant: 26.0),
            stack.centerXAnchor.constraint(equalTo: centerXAnchor),
            stack.centerYAnchor.constraint(equalTo: centerYAnchor, constant: -14.0),
            stack.leadingAnchor.constraint(greaterThanOrEqualTo: leadingAnchor, constant: 16.0),
            stack.trailingAnchor.constraint(lessThanOrEqualTo: trailingAnchor, constant: -16.0)
        ])
    }

    private func configureSymbol(_ symbolName: String) {
        if let image = NSImage(systemSymbolName: symbolName, accessibilityDescription: nil) {
            let config = NSImage.SymbolConfiguration(pointSize: 22.0, weight: .medium)
            let configured = image.withSymbolConfiguration(config) ?? image
            configured.isTemplate = true
            iconView.image = configured
        }
    }

    public func updateContent(symbolName: String, title: String, subtitle: String) {
        configureSymbol(symbolName)
        titleLabel.stringValue = title
        subtitleLabel.stringValue = subtitle
    }
}
