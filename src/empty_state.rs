use objc2::rc::Retained;
use objc2_app_kit::{
    NSColor, NSFont, NSImage, NSImageSymbolConfiguration, NSImageSymbolScale, NSImageView,
    NSLayoutAttribute, NSLineBreakMode, NSStackView, NSTextAlignment, NSTextField,
    NSUserInterfaceLayoutOrientation, NSView,
};
use objc2_foundation::{MainThreadMarker, NSString};

pub struct EmptyStateView {
    pub container: Retained<NSView>,
    pub icon_view: Retained<NSImageView>,
    pub title_label: Retained<NSTextField>,
    pub subtitle_label: Retained<NSTextField>,
}

impl EmptyStateView {
    pub fn new(mtm: MainThreadMarker, symbol: &str, title: &str, subtitle: &str) -> Self {
        let container = unsafe { NSView::new(mtm) };
        container.setTranslatesAutoresizingMaskIntoConstraints(false);

        let stack = unsafe { NSStackView::new(mtm) };
        stack.setTranslatesAutoresizingMaskIntoConstraints(false);
        stack.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
        stack.setAlignment(NSLayoutAttribute::CenterX);
        stack.setSpacing(6.0);

        let icon_view = unsafe { NSImageView::new(mtm) };
        icon_view.setTranslatesAutoresizingMaskIntoConstraints(false);
        icon_view.setContentTintColor(Some(&NSColor::tertiaryLabelColor()));

        let title_label = unsafe { NSTextField::labelWithString(&NSString::from_str(title), mtm) };
        title_label.setTranslatesAutoresizingMaskIntoConstraints(false);
        title_label.setFont(Some(&NSFont::systemFontOfSize_weight(13.0, 0.2)));
        title_label.setTextColor(Some(&NSColor::secondaryLabelColor()));
        title_label.setAlignment(NSTextAlignment::Center);
        title_label.setUsesSingleLineMode(false);
        title_label.setLineBreakMode(NSLineBreakMode::ByWordWrapping);

        let subtitle_label =
            unsafe { NSTextField::labelWithString(&NSString::from_str(subtitle), mtm) };
        subtitle_label.setTranslatesAutoresizingMaskIntoConstraints(false);
        subtitle_label.setFont(Some(&NSFont::systemFontOfSize(11.5)));
        subtitle_label.setTextColor(Some(&NSColor::tertiaryLabelColor()));
        subtitle_label.setAlignment(NSTextAlignment::Center);
        subtitle_label.setUsesSingleLineMode(false);
        subtitle_label.setLineBreakMode(NSLineBreakMode::ByWordWrapping);

        unsafe {
            stack.addArrangedSubview(&icon_view);
            stack.setCustomSpacing_afterView(12.0, &icon_view);
            stack.addArrangedSubview(&title_label);
            stack.setCustomSpacing_afterView(6.0, &title_label);
            stack.addArrangedSubview(&subtitle_label);

            container.addSubview(&stack);

            stack
                .centerXAnchor()
                .constraintEqualToAnchor(&container.centerXAnchor())
                .setActive(true);
            stack
                .centerYAnchor()
                .constraintEqualToAnchor_constant(&container.centerYAnchor(), -20.0)
                .setActive(true);
            stack
                .leadingAnchor()
                .constraintGreaterThanOrEqualToAnchor_constant(&container.leadingAnchor(), 16.0)
                .setActive(true);
            stack
                .trailingAnchor()
                .constraintLessThanOrEqualToAnchor_constant(&container.trailingAnchor(), -16.0)
                .setActive(true);
            title_label
                .widthAnchor()
                .constraintLessThanOrEqualToConstant(200.0)
                .setActive(true);
            subtitle_label
                .widthAnchor()
                .constraintLessThanOrEqualToConstant(200.0)
                .setActive(true);
        }

        let this = Self {
            container,
            icon_view,
            title_label,
            subtitle_label,
        };
        this.set_symbol(symbol);
        this
    }

    pub fn set_symbol(&self, symbol: &str) {
        let icon_cfg = NSImageSymbolConfiguration::configurationWithPointSize_weight_scale(
            28.0,
            0.0,
            NSImageSymbolScale::Medium,
        );
        if let Some(img) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str(symbol),
            None,
        ) {
            let configured = img.imageWithSymbolConfiguration(&icon_cfg);
            unsafe {
                self.icon_view
                    .setImage(configured.as_deref().or(Some(&img)));
            }
        }
    }

    pub fn set_text(&self, title: &str, subtitle: &str) {
        unsafe {
            self.title_label.setStringValue(&NSString::from_str(title));
            self.subtitle_label
                .setStringValue(&NSString::from_str(subtitle));
        }
    }

    pub fn set_content(&self, symbol: &str, title: &str, subtitle: &str) {
        self.set_symbol(symbol);
        self.set_text(title, subtitle);
    }

    pub fn set_hidden(&self, hidden: bool) {
        self.container.setHidden(hidden);
    }
}
