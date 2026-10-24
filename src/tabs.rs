use crate::AppContentMode;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadOnly, msg_send, sel};
use objc2_app_kit::{
    NSButton, NSColor, NSFont, NSImage, NSImageSymbolConfiguration, NSImageSymbolScale,
    NSImageView, NSLineBreakMode, NSStackView, NSTextField, NSUserInterfaceLayoutOrientation,
    NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState,
    NSVisualEffectView,
};
use objc2_foundation::{MainThreadMarker, NSInteger, NSPoint, NSRect, NSSize, NSString};
use objc2_pdf_kit::PDFDocument;

#[derive(Clone)]
pub struct TabSession {
    pub id: usize,
    pub path: String,
    pub title: String,
    pub document: Retained<PDFDocument>,
    pub is_edited: bool,
    pub page_index: usize,
    pub scale_factor: f64,
    pub mode: AppContentMode,
}

#[derive(Default)]
pub struct TabManager {
    pub tabs: Vec<TabSession>,
    pub active_index: usize,
    next_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_index: 0,
            next_id: 1,
        }
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    pub fn active_tab(&self) -> Option<&TabSession> {
        self.tabs.get(self.active_index)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut TabSession> {
        self.tabs.get_mut(self.active_index)
    }

    pub fn find_tab_by_path(&self, path: &str) -> Option<usize> {
        let norm = crate::bookmarks::normalize_bookmark_path(path);
        self.tabs
            .iter()
            .position(|t| crate::bookmarks::normalize_bookmark_path(&t.path) == norm)
    }

    pub fn add_tab(
        &mut self,
        path: String,
        title: String,
        doc: Retained<PDFDocument>,
        mode: AppContentMode,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let session = TabSession {
            id,
            path,
            title,
            document: doc,
            is_edited: false,
            page_index: 0,
            scale_factor: 1.0,
            mode,
        };
        self.tabs.push(session);
        let idx = self.tabs.len() - 1;
        self.active_index = idx;
        idx
    }

    pub fn remove_tab(&mut self, index: usize) -> Option<TabSession> {
        if index >= self.tabs.len() {
            return None;
        }
        let removed = self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active_index = 0;
        } else if self.active_index >= self.tabs.len() {
            self.active_index = self.tabs.len() - 1;
        } else if self.active_index > index {
            self.active_index -= 1;
        }
        Some(removed)
    }

    pub fn next_index(&self) -> usize {
        if self.tabs.is_empty() {
            0
        } else {
            (self.active_index + 1) % self.tabs.len()
        }
    }

    pub fn prev_index(&self) -> usize {
        if self.tabs.is_empty() {
            0
        } else if self.active_index == 0 {
            self.tabs.len() - 1
        } else {
            self.active_index - 1
        }
    }
}

/// Create the native Liquid Glass tab bar container view.
pub fn create_tab_bar_view(mtm: MainThreadMarker, target: &AnyObject) -> Retained<NSView> {
    let container = unsafe {
        NSVisualEffectView::initWithFrame(
            NSVisualEffectView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(600.0, 32.0)),
        )
    };
    unsafe {
        container.setMaterial(NSVisualEffectMaterial::HeaderView);
        container.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        container.setState(NSVisualEffectState::FollowsWindowActiveState);
    }

    // 1pt bottom separator
    let separator = unsafe {
        NSView::initWithFrame(
            NSView::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(600.0, 1.0)),
        )
    };
    separator.setWantsLayer(true);
    if let Some(layer) = unsafe { separator.layer() } {
        let cg = unsafe { NSColor::separatorColor().CGColor() };
        let ptr = Retained::as_ptr(&cg);
        let _: () = unsafe { msg_send![&layer, setBackgroundColor: ptr] };
    }
    separator.setTranslatesAutoresizingMaskIntoConstraints(false);
    unsafe {
        container.addSubview(&separator);
        separator
            .leadingAnchor()
            .constraintEqualToAnchor(&container.leadingAnchor())
            .setActive(true);
        separator
            .trailingAnchor()
            .constraintEqualToAnchor(&container.trailingAnchor())
            .setActive(true);
        separator
            .bottomAnchor()
            .constraintEqualToAnchor(&container.bottomAnchor())
            .setActive(true);
        separator
            .heightAnchor()
            .constraintEqualToConstant(1.0)
            .setActive(true);
    }

    // Horizontal stack view for tabs
    let tabs_stack = unsafe {
        NSStackView::initWithFrame(
            NSStackView::alloc(mtm),
            NSRect::new(NSPoint::new(8.0, 3.0), NSSize::new(550.0, 26.0)),
        )
    };
    unsafe {
        tabs_stack.setOrientation(NSUserInterfaceLayoutOrientation::Horizontal);
        tabs_stack.setSpacing(4.0);
    }
    tabs_stack.setTranslatesAutoresizingMaskIntoConstraints(false);

    // New Tab (+) button on trailing edge
    let new_tab_btn = unsafe {
        NSButton::initWithFrame(
            NSButton::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(26.0, 24.0)),
        )
    };
    unsafe {
        new_tab_btn.setBordered(false);
        new_tab_btn.setTitle(&NSString::from_str(""));
        if let Some(img) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str("plus"),
            None,
        ) {
            let config =
                NSImageSymbolConfiguration::configurationWithScale(NSImageSymbolScale::Small);
            let symbol_img = img.imageWithSymbolConfiguration(&config);
            new_tab_btn.setImage(symbol_img.as_deref().or(Some(&img)));
        }
        new_tab_btn.setToolTip(Some(&NSString::from_str("New Tab (⌘T)")));
        new_tab_btn.setTarget(Some(target));
        new_tab_btn.setAction(Some(sel!(newTab:)));
    }
    new_tab_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

    unsafe {
        container.addSubview(&tabs_stack);
        container.addSubview(&new_tab_btn);

        tabs_stack
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&container.leadingAnchor(), 8.0)
            .setActive(true);
        tabs_stack
            .centerYAnchor()
            .constraintEqualToAnchor(&container.centerYAnchor())
            .setActive(true);
        tabs_stack
            .trailingAnchor()
            .constraintEqualToAnchor_constant(&new_tab_btn.leadingAnchor(), -6.0)
            .setActive(true);
        tabs_stack
            .heightAnchor()
            .constraintEqualToConstant(26.0)
            .setActive(true);

        new_tab_btn
            .trailingAnchor()
            .constraintEqualToAnchor_constant(&container.trailingAnchor(), -8.0)
            .setActive(true);
        new_tab_btn
            .centerYAnchor()
            .constraintEqualToAnchor(&container.centerYAnchor())
            .setActive(true);
        new_tab_btn
            .widthAnchor()
            .constraintEqualToConstant(26.0)
            .setActive(true);
        new_tab_btn
            .heightAnchor()
            .constraintEqualToConstant(24.0)
            .setActive(true);
    }

    let raw: Retained<NSView> = unsafe { Retained::cast_unchecked(container) };
    raw
}

/// Re-render all tab pills into the tab bar.
pub fn update_tab_bar(
    tab_bar: &NSView,
    manager: &TabManager,
    target: &AnyObject,
    mtm: MainThreadMarker,
) {
    let subviews = unsafe { tab_bar.subviews() };
    let tabs_stack: Option<Retained<NSStackView>> = subviews
        .into_iter()
        .find_map(|v| v.downcast::<NSStackView>().ok());

    let stack = match tabs_stack {
        Some(s) => s,
        None => return,
    };

    // Remove existing tab pills from stack
    let old_views = unsafe { stack.views() };
    for view in old_views.iter() {
        unsafe {
            stack.removeView(&view);
            view.removeFromSuperview();
        }
    }

    let count = manager.tab_count();
    let max_visible = 5usize;
    let visible_count = count.min(max_visible);
    let max_tab_width = if visible_count > 0 {
        (560.0 / visible_count as f64).clamp(108.0, 200.0)
    } else {
        180.0
    };

    // Determine window of visible tabs so active_index is always visible
    let start_idx = if count > max_visible && manager.active_index >= max_visible {
        manager.active_index + 1 - max_visible
    } else {
        0
    };
    let end_idx = (start_idx + max_visible).min(count);

    for i in start_idx..end_idx {
        let tab = &manager.tabs[i];
        let is_active = i == manager.active_index;

        let tab_item = unsafe {
            NSView::initWithFrame(
                NSView::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(max_tab_width, 24.0)),
            )
        };
        tab_item.setWantsLayer(true);
        tab_item.setTranslatesAutoresizingMaskIntoConstraints(false);
        let tooltip_str = if tab.is_edited {
            format!("{} — Edited ({})", tab.title, tab.path)
        } else {
            format!("{} ({})", tab.title, tab.path)
        };
        unsafe { tab_item.setToolTip(Some(&NSString::from_str(&tooltip_str))) };

        if let Some(layer) = unsafe { tab_item.layer() } {
            let _: () = unsafe { msg_send![&layer, setCornerRadius: 6.0f64] };
            let _: () = unsafe { msg_send![&layer, setMasksToBounds: true] };

            if is_active {
                let bg_color: Retained<NSColor> =
                    unsafe { msg_send![&NSColor::labelColor(), colorWithAlphaComponent: 0.11_f64] };
                let cg = unsafe { bg_color.CGColor() };
                let ptr = Retained::as_ptr(&cg);
                let _: () = unsafe { msg_send![&layer, setBackgroundColor: ptr] };

                let border_color: Retained<NSColor> = unsafe {
                    msg_send![&NSColor::separatorColor(), colorWithAlphaComponent: 0.75_f64]
                };
                let border_cg = unsafe { border_color.CGColor() };
                let border_ptr = Retained::as_ptr(&border_cg);
                let _: () = unsafe { msg_send![&layer, setBorderColor: border_ptr] };
                let _: () = unsafe { msg_send![&layer, setBorderWidth: 0.5f64] };
            } else {
                let bg_color: Retained<NSColor> = unsafe {
                    msg_send![&NSColor::labelColor(), colorWithAlphaComponent: 0.035_f64]
                };
                let cg = unsafe { bg_color.CGColor() };
                let ptr = Retained::as_ptr(&cg);
                let _: () = unsafe { msg_send![&layer, setBackgroundColor: ptr] };
                let _: () = unsafe { msg_send![&layer, setBorderWidth: 0.0f64] };
            }
        }

        // Invisible selection button covering the tab item
        let select_btn = unsafe {
            NSButton::initWithFrame(
                NSButton::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(max_tab_width, 24.0)),
            )
        };
        unsafe {
            select_btn.setBordered(false);
            select_btn.setTitle(&NSString::from_str(""));
            select_btn.setTag(i as NSInteger);
            select_btn.setTarget(Some(target));
            select_btn.setAction(Some(sel!(tabClicked:)));
            select_btn.setToolTip(Some(&NSString::from_str(&tooltip_str)));
        }
        select_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

        // Document SF Symbol icon
        let doc_icon = unsafe {
            NSImageView::initWithFrame(
                NSImageView::alloc(mtm),
                NSRect::new(NSPoint::new(6.0, 5.0), NSSize::new(13.0, 13.0)),
            )
        };
        if let Some(img) = unsafe {
            NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str("doc.richtext"),
                None,
            )
        } {
            let config = unsafe {
                NSImageSymbolConfiguration::configurationWithScale(NSImageSymbolScale::Small)
            };
            let symbol_img = unsafe { img.imageWithSymbolConfiguration(&config) };
            unsafe {
                doc_icon.setImage(symbol_img.as_deref().or(Some(&img)));
                let tint = if is_active {
                    NSColor::controlAccentColor()
                } else {
                    NSColor::secondaryLabelColor()
                };
                doc_icon.setContentTintColor(Some(&tint));
            };
        }
        doc_icon.setTranslatesAutoresizingMaskIntoConstraints(false);

        // Document title label
        let title_label = unsafe {
            NSTextField::initWithFrame(
                NSTextField::alloc(mtm),
                NSRect::new(
                    NSPoint::new(22.0, 4.0),
                    NSSize::new(max_tab_width - 48.0, 16.0),
                ),
            )
        };
        unsafe {
            title_label.setEditable(false);
            title_label.setSelectable(false);
            title_label.setBordered(false);
            title_label.setDrawsBackground(false);
            title_label.setStringValue(&NSString::from_str(&tab.title));
            let font = if is_active {
                NSFont::systemFontOfSize_weight(11.5, 0.25)
            } else {
                NSFont::systemFontOfSize(11.5)
            };
            title_label.setFont(Some(&font));
            let text_color = if is_active {
                NSColor::labelColor()
            } else {
                NSColor::secondaryLabelColor()
            };
            title_label.setTextColor(Some(&text_color));
            title_label.setLineBreakMode(NSLineBreakMode::ByTruncatingMiddle);
            title_label.setToolTip(Some(&NSString::from_str(&tooltip_str)));
        }
        title_label.setTranslatesAutoresizingMaskIntoConstraints(false);

        // Dirty indicator dot
        let dirty_dot = if tab.is_edited {
            let dot = unsafe {
                NSTextField::initWithFrame(
                    NSTextField::alloc(mtm),
                    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(10.0, 14.0)),
                )
            };
            unsafe {
                dot.setEditable(false);
                dot.setSelectable(false);
                dot.setBordered(false);
                dot.setDrawsBackground(false);
                dot.setStringValue(&NSString::from_str("●"));
                dot.setFont(Some(&NSFont::systemFontOfSize(8.0)));
                dot.setTextColor(Some(&NSColor::systemOrangeColor()));
            }
            dot.setTranslatesAutoresizingMaskIntoConstraints(false);
            Some(dot)
        } else {
            None
        };

        // Close button (x)
        let close_btn = unsafe {
            NSButton::initWithFrame(
                NSButton::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(16.0, 16.0)),
            )
        };
        unsafe {
            close_btn.setBordered(false);
            close_btn.setTitle(&NSString::from_str(""));
            if let Some(img) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str("xmark"),
                Some(&NSString::from_str("Close Tab")),
            ) {
                let config = NSImageSymbolConfiguration::configurationWithPointSize_weight_scale(
                    9.5,
                    0.3,
                    NSImageSymbolScale::Small,
                );
                let symbol_img = img.imageWithSymbolConfiguration(&config);
                close_btn.setImage(symbol_img.as_deref().or(Some(&img)));
            }
            close_btn.setTag(i as NSInteger);
            close_btn.setTarget(Some(target));
            close_btn.setAction(Some(sel!(closeTabAtIndex:)));
            close_btn.setToolTip(Some(&NSString::from_str("Close Tab (⌘W)")));
        }
        close_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

        unsafe {
            tab_item.addSubview(&doc_icon);
            tab_item.addSubview(&title_label);
            if let Some(ref dot) = dirty_dot {
                tab_item.addSubview(dot);
            }
            tab_item.addSubview(&select_btn);
            tab_item.addSubview(&close_btn);

            select_btn
                .leadingAnchor()
                .constraintEqualToAnchor(&tab_item.leadingAnchor())
                .setActive(true);
            select_btn
                .trailingAnchor()
                .constraintEqualToAnchor(&tab_item.trailingAnchor())
                .setActive(true);
            select_btn
                .topAnchor()
                .constraintEqualToAnchor(&tab_item.topAnchor())
                .setActive(true);
            select_btn
                .bottomAnchor()
                .constraintEqualToAnchor(&tab_item.bottomAnchor())
                .setActive(true);

            doc_icon
                .leadingAnchor()
                .constraintEqualToAnchor_constant(&tab_item.leadingAnchor(), 7.0)
                .setActive(true);
            doc_icon
                .centerYAnchor()
                .constraintEqualToAnchor(&tab_item.centerYAnchor())
                .setActive(true);
            doc_icon
                .widthAnchor()
                .constraintEqualToConstant(13.0)
                .setActive(true);
            doc_icon
                .heightAnchor()
                .constraintEqualToConstant(13.0)
                .setActive(true);

            title_label
                .leadingAnchor()
                .constraintEqualToAnchor_constant(&doc_icon.trailingAnchor(), 5.0)
                .setActive(true);
            title_label
                .centerYAnchor()
                .constraintEqualToAnchor(&tab_item.centerYAnchor())
                .setActive(true);

            if let Some(ref dot) = dirty_dot {
                dot.leadingAnchor()
                    .constraintEqualToAnchor_constant(&title_label.trailingAnchor(), 2.0)
                    .setActive(true);
                dot.centerYAnchor()
                    .constraintEqualToAnchor(&tab_item.centerYAnchor())
                    .setActive(true);
                dot.widthAnchor()
                    .constraintEqualToConstant(10.0)
                    .setActive(true);

                close_btn
                    .leadingAnchor()
                    .constraintEqualToAnchor_constant(&dot.trailingAnchor(), 2.0)
                    .setActive(true);
            } else {
                close_btn
                    .leadingAnchor()
                    .constraintEqualToAnchor_constant(&title_label.trailingAnchor(), 4.0)
                    .setActive(true);
            }

            close_btn
                .trailingAnchor()
                .constraintEqualToAnchor_constant(&tab_item.trailingAnchor(), -5.0)
                .setActive(true);
            close_btn
                .centerYAnchor()
                .constraintEqualToAnchor(&tab_item.centerYAnchor())
                .setActive(true);
            close_btn
                .widthAnchor()
                .constraintEqualToConstant(16.0)
                .setActive(true);
            close_btn
                .heightAnchor()
                .constraintEqualToConstant(16.0)
                .setActive(true);

            tab_item
                .widthAnchor()
                .constraintEqualToConstant(max_tab_width)
                .setActive(true);
            tab_item
                .heightAnchor()
                .constraintEqualToConstant(24.0)
                .setActive(true);

            stack.addArrangedSubview(&tab_item);
        }
    }

    // If there are more tabs than max_visible, append an overflow badge button ("+N more")
    if count > max_visible {
        let extra = count - max_visible;
        let overflow_btn = unsafe {
            NSButton::buttonWithTitle_target_action(
                &NSString::from_str(&format!("+{} ▾", extra)),
                Some(target),
                Some(sel!(nextTab:)),
                mtm,
            )
        };
        unsafe {
            overflow_btn.setBordered(false);
            overflow_btn.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
                11.0, 0.2,
            )));
            overflow_btn.setToolTip(Some(&NSString::from_str(&format!(
                "{} open tabs — click to cycle tabs",
                count
            ))));
            overflow_btn.setTranslatesAutoresizingMaskIntoConstraints(false);
            overflow_btn
                .heightAnchor()
                .constraintEqualToConstant(22.0)
                .setActive(true);
            stack.addArrangedSubview(&overflow_btn);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_manager_initial_state() {
        let mgr = TabManager::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.tab_count(), 0);
        assert_eq!(mgr.next_index(), 0);
        assert_eq!(mgr.prev_index(), 0);
        assert!(mgr.active_tab().is_none());
    }

    #[test]
    fn test_tab_manager_out_of_bounds_remove() {
        let mut mgr = TabManager::new();
        assert!(mgr.remove_tab(0).is_none());
        assert!(mgr.remove_tab(10).is_none());
    }

    #[test]
    fn test_tab_manager_lifecycle_and_navigation() {
        let mut mgr = TabManager::new();
        let doc1 = unsafe { PDFDocument::new() };
        let doc2 = unsafe { PDFDocument::new() };
        let doc3 = unsafe { PDFDocument::new() };

        let idx1 = mgr.add_tab(
            "/path/1.pdf".to_string(),
            "Doc 1".to_string(),
            doc1,
            AppContentMode::StandaloneDocument,
        );
        assert_eq!(idx1, 0);
        assert_eq!(mgr.active_index, 0);
        assert_eq!(mgr.tab_count(), 1);

        let idx2 = mgr.add_tab(
            "/path/2.pdf".to_string(),
            "Doc 2".to_string(),
            doc2,
            AppContentMode::StandaloneDocument,
        );
        assert_eq!(idx2, 1);
        assert_eq!(mgr.active_index, 1);
        assert_eq!(mgr.tab_count(), 2);

        let idx3 = mgr.add_tab(
            "/path/3.pdf".to_string(),
            "Doc 3".to_string(),
            doc3,
            AppContentMode::StandaloneDocument,
        );
        assert_eq!(idx3, 2);
        assert_eq!(mgr.active_index, 2);
        assert_eq!(mgr.tab_count(), 3);

        // Test navigation
        assert_eq!(mgr.next_index(), 0); // Wrap around to 0
        assert_eq!(mgr.prev_index(), 1);

        mgr.active_index = 0;
        assert_eq!(mgr.prev_index(), 2); // Wrap around backwards to 2

        // Test find by path
        assert_eq!(mgr.find_tab_by_path("/path/2.pdf"), Some(1));
        assert_eq!(mgr.find_tab_by_path("/private/path/2.pdf"), Some(1));
        assert_eq!(mgr.find_tab_by_path("/path/unknown.pdf"), None);

        // Test removing active tab
        mgr.active_index = 2;
        let removed = mgr.remove_tab(2);
        assert!(removed.is_some());
        assert_eq!(mgr.tab_count(), 2);
        assert_eq!(mgr.active_index, 1); // Shifted down to last remaining
    }

    #[test]
    fn test_tab_switching_keycodes_and_invariants() {
        // Physical keycodes mapping table verification for AZERTY/QWERTY top row
        let keycodes: [(u16, usize); 9] = [
            (18, 0), // '1'
            (19, 1), // '2'
            (20, 2), // '3'
            (21, 3), // '4'
            (23, 4), // '5'
            (22, 5), // '6'
            (26, 6), // '7'
            (28, 7), // '8'
            (25, 8), // '9' -> last tab
        ];

        let mut mgr = TabManager::new();
        // Add 6 tabs for stress test
        for i in 0..6 {
            let doc = unsafe { PDFDocument::new() };
            mgr.add_tab(
                format!("/tmp/doc_{}.pdf", i),
                format!("Doc {}", i),
                doc,
                AppContentMode::StandaloneDocument,
            );
        }

        assert_eq!(mgr.tab_count(), 6);

        // Test 100 switching iterations
        for step in 0..100 {
            let target_kc = keycodes[step % keycodes.len()];
            let target_tab = if target_kc.1 == 8 {
                mgr.tab_count() - 1
            } else if target_kc.1 < mgr.tab_count() {
                target_kc.1
            } else {
                mgr.active_index
            };
            mgr.active_index = target_tab;

            // Assert tab invariants
            assert!(mgr.tab_count() >= 1);
            assert!(mgr.active_index < mgr.tab_count());
            assert!(mgr.active_tab().is_some());
            assert_eq!(mgr.active_tab().unwrap().title, format!("Doc {}", target_tab));
        }
    }
}
