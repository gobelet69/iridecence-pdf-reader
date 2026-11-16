import AppKit
import Foundation

/// Centered empty state with a plain system symbol and useful guidance.
@MainActor
public final class EmptyStateView: NSView {
    public let iconView: NSImageView
    public let titleLabel: NSTextField
    public let subtitleLabel: NSTextField
    private let medallionView = NSView()

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
        stack.spacing = 8.0
        stack.translatesAutoresizingMaskIntoConstraints = false

        medallionView.translatesAutoresizingMaskIntoConstraints = false
        iconView.translatesAutoresizingMaskIntoConstraints = false
        iconView.contentTintColor = NSColor.secondaryLabelColor
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
