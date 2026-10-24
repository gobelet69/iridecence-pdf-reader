use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSAppearance, NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSApplication,
    NSBackingStoreType, NSButton, NSColor, NSFont, NSSegmentedControl, NSTextField, NSView,
    NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
    NSWindow, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString, NSUserDefaults, ns_string};

use crate::window_geometry::{WindowGeometry, WindowKind};

// Preference Keys
pub const PREF_RESTORE_SESSION: &str = "PrefRestoreSession";
pub const PREF_REMEMBER_RECENTS: &str = "PrefRememberRecents";
pub const PREF_REOPEN_CLOSED_TABS: &str = "PrefReopenClosedTabs";
pub const PREF_OPEN_IN_NEW_TAB: &str = "PrefOpenInNewTab";

pub const PREF_DEFAULT_DISPLAY_MODE: &str = "PrefDefaultDisplayMode";
pub const PREF_DEFAULT_ZOOM_MODE: &str = "PrefDefaultZoomMode";
pub const PREF_DEFAULT_HIGHLIGHT_COLOR: &str = "PrefDefaultHighlightColor";
pub const PREF_RESTORE_READING_POS: &str = "PrefRestoreReadingPos";

pub const PREF_AUTO_REVEAL_SIDEBAR: &str = "PrefAutoRevealSidebar";
pub const PREF_SHOW_HIDDEN_FILES: &str = "PrefShowHiddenFiles";
pub const PREF_DEEP_WORKSPACE_SEARCH: &str = "PrefDeepWorkspaceSearch";

pub const PREF_APPEARANCE_MODE: &str = "PrefAppearanceMode";
pub const PREF_LIQUID_GLASS: &str = "PrefLiquidGlass";
pub const PREF_AUTO_COPY_OCR: &str = "PrefAutoCopyOcr";

pub struct SettingsManager;

impl SettingsManager {
    pub fn bool_pref(key: &str, default: bool) -> bool {
        get_bool_pref(key, default)
    }

    pub fn int_pref(key: &str, default: isize) -> isize {
        get_int_pref(key, default)
    }
}

pub fn get_bool_pref(key: &str, default: bool) -> bool {
    unsafe {
        let defaults = NSUserDefaults::standardUserDefaults();
        let ns_key = NSString::from_str(key);
        if defaults.objectForKey(&ns_key).is_some() {
            defaults.boolForKey(&ns_key)
        } else {
            default
        }
    }
}

pub fn set_bool_pref(key: &str, val: bool) {
    unsafe {
        let defaults = NSUserDefaults::standardUserDefaults();
        let ns_key = NSString::from_str(key);
        defaults.setBool_forKey(val, &ns_key);
    }
}

pub fn get_int_pref(key: &str, default: isize) -> isize {
    unsafe {
        let defaults = NSUserDefaults::standardUserDefaults();
        let ns_key = NSString::from_str(key);
        if defaults.objectForKey(&ns_key).is_some() {
            defaults.integerForKey(&ns_key)
        } else {
            default
        }
    }
}

pub fn set_int_pref(key: &str, val: isize) {
    unsafe {
        let defaults = NSUserDefaults::standardUserDefaults();
        let ns_key = NSString::from_str(key);
        defaults.setInteger_forKey(val, &ns_key);
    }
}

#[derive(Default)]
pub struct SettingsIvars {
    pub window: RefCell<Option<Retained<NSWindow>>>,
    pub content_area: RefCell<Option<Retained<NSView>>>,
    pub active_tab: RefCell<isize>,
    pub seg_control: RefCell<Option<Retained<NSSegmentedControl>>>,
    pub tab_pills: RefCell<Vec<Retained<NSView>>>,
    pub tab_icons: RefCell<Vec<Retained<objc2_app_kit::NSImageView>>>,
    pub tab_labels: RefCell<Vec<Retained<NSTextField>>>,
}

define_class!(
    #[unsafe(super(objc2_foundation::NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "SettingsDelegate"]
    #[ivars = SettingsIvars]
    pub struct SettingsDelegate;

    impl SettingsDelegate {
        #[unsafe(method(tabChanged:))]
        fn tab_changed(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(seg) = sender.downcast_ref::<NSSegmentedControl>() {
                    let sel_tab = seg.selectedSegment();
                    self.switch_to_tab(sel_tab);
                } else if let Some(btn) = sender.downcast_ref::<NSButton>() {
                    let sel_tab = btn.tag();
                    self.switch_to_tab(sel_tab);
                }
            }
        }

        #[unsafe(method(toggleCheckbox:))]
        fn toggle_checkbox(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(btn) = sender.downcast_ref::<NSButton>() {
                    let tag = btn.tag();
                    let is_on = btn.state() == objc2_app_kit::NSControlStateValueOn;
                    match tag {
                        101 => set_bool_pref(PREF_RESTORE_SESSION, is_on),
                        102 => set_bool_pref(PREF_REMEMBER_RECENTS, is_on),
                        103 => set_bool_pref(PREF_REOPEN_CLOSED_TABS, is_on),
                        104 => set_bool_pref(PREF_OPEN_IN_NEW_TAB, is_on),
                        201 => set_bool_pref(PREF_RESTORE_READING_POS, is_on),
                        301 => set_bool_pref(PREF_AUTO_REVEAL_SIDEBAR, is_on),
                        302 => {
                            set_bool_pref(PREF_SHOW_HIDDEN_FILES, is_on);
                            self.notify_app_runtime_settings();
                        }
                        303 => set_bool_pref(PREF_DEEP_WORKSPACE_SEARCH, is_on),
                        401 => set_bool_pref(PREF_LIQUID_GLASS, is_on),
                        501 => set_bool_pref(PREF_AUTO_COPY_OCR, is_on),
                        _ => {}
                    }
                }
            }
        }

        #[unsafe(method(segmentChanged:))]
        fn segment_changed(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(seg) = sender.downcast_ref::<NSSegmentedControl>() {
                    let tag = seg.tag();
                    let sel_idx = seg.selectedSegment();
                    match tag {
                        210 => {
                            set_int_pref(PREF_DEFAULT_DISPLAY_MODE, sel_idx);
                            self.notify_app_runtime_settings();
                        }
                        220 => {
                            set_int_pref(PREF_DEFAULT_ZOOM_MODE, sel_idx);
                            self.notify_app_runtime_settings();
                        }
                        230 => set_int_pref(PREF_DEFAULT_HIGHLIGHT_COLOR, sel_idx),
                        410 => {
                            set_int_pref(PREF_APPEARANCE_MODE, sel_idx);
                            self.apply_appearance(sel_idx);
                        }
                        _ => {}
                    }
                }
            }
        }

        #[unsafe(method(resetDefaults:))]
        fn reset_defaults(&self, _sender: Option<&AnyObject>) {
            set_bool_pref(PREF_RESTORE_SESSION, true);
            set_bool_pref(PREF_REMEMBER_RECENTS, true);
            set_bool_pref(PREF_REOPEN_CLOSED_TABS, true);
            set_bool_pref(PREF_OPEN_IN_NEW_TAB, true);
            set_int_pref(PREF_DEFAULT_DISPLAY_MODE, 1);
            set_int_pref(PREF_DEFAULT_ZOOM_MODE, 0);
            set_int_pref(PREF_DEFAULT_HIGHLIGHT_COLOR, 0);
            set_bool_pref(PREF_RESTORE_READING_POS, true);
            set_bool_pref(PREF_AUTO_REVEAL_SIDEBAR, true);
            set_bool_pref(PREF_SHOW_HIDDEN_FILES, false);
            set_bool_pref(PREF_DEEP_WORKSPACE_SEARCH, true);
            set_int_pref(PREF_APPEARANCE_MODE, 0);
            set_bool_pref(PREF_LIQUID_GLASS, true);
            set_bool_pref(PREF_AUTO_COPY_OCR, true);
            self.apply_appearance(0);
            self.notify_app_runtime_settings();

            let cur_tab = *self.ivars().active_tab.borrow();
            self.switch_to_tab(cur_tab);
        }
    }
);

impl SettingsDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(SettingsIvars::default());
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };
        this
    }

    pub fn notify_app_runtime_settings(&self) {
        let mtm = MainThreadMarker::from(self);
        let app = NSApplication::sharedApplication(mtm);
        if let Some(delegate) = unsafe { app.delegate() } {
            let _: () = unsafe { msg_send![&*delegate, applyRuntimeSettings] };
        }
    }

    pub fn apply_appearance(&self, mode: isize) {
        let mtm = MainThreadMarker::from(self);
        let app = NSApplication::sharedApplication(mtm);
        unsafe {
            match mode {
                1 => {
                    let aqua = NSAppearance::appearanceNamed(NSAppearanceNameAqua);
                    app.setAppearance(aqua.as_deref());
                }
                2 => {
                    let dark = NSAppearance::appearanceNamed(NSAppearanceNameDarkAqua);
                    app.setAppearance(dark.as_deref());
                }
                _ => {
                    app.setAppearance(None);
                }
            }
        }
    }

    pub fn switch_to_tab(&self, index: isize) {
        *self.ivars().active_tab.borrow_mut() = index;
        let mtm = MainThreadMarker::from(self);
        if let Some(seg) = self.ivars().seg_control.borrow().as_ref() {
            seg.setSelected_forSegment(true, index);
        }

        let pills = self.ivars().tab_pills.borrow();
        let icons = self.ivars().tab_icons.borrow();
        let labels = self.ivars().tab_labels.borrow();
        for i in 0..pills.len() {
            let is_sel = (i as isize) == index;
            unsafe {
                if let Some(layer) = pills[i].layer() {
                    let bg = if is_sel {
                        NSColor::controlAccentColor().colorWithAlphaComponent(0.16)
                    } else {
                        NSColor::clearColor()
                    };
                    let cg = bg.CGColor();
                    let ptr = Retained::as_ptr(&cg);
                    let _: () = msg_send![&layer, setBackgroundColor: ptr];
                }
                let accent = NSColor::controlAccentColor();
                let sec = NSColor::secondaryLabelColor();
                let tint = if is_sel { &accent } else { &sec };
                if i < icons.len() {
                    icons[i].setContentTintColor(Some(tint));
                }
                if i < labels.len() {
                    labels[i].setTextColor(Some(tint));
                    labels[i].setFont(Some(&NSFont::systemFontOfSize_weight(
                        10.5,
                        if is_sel { 0.3 } else { 0.0 },
                    )));
                }
            }
        }

        if let Some(content_area) = self.ivars().content_area.borrow().as_ref() {
            // Remove existing subviews
            let subviews = content_area.subviews();
            for i in (0..subviews.count()).rev() {
                let sv = subviews.objectAtIndex(i);
                sv.removeFromSuperview();
            }

            let pane = match index {
                0 => self.create_general_pane(mtm),
                1 => self.create_reading_pane(mtm),
                2 => self.create_workspace_pane(mtm),
                3 => self.create_appearance_pane(mtm),
                _ => self.create_advanced_pane(mtm),
            };

            pane.setTranslatesAutoresizingMaskIntoConstraints(false);
            content_area.addSubview(&pane);

            pane.topAnchor()
                .constraintEqualToAnchor(&content_area.topAnchor())
                .setActive(true);
            pane.leadingAnchor()
                .constraintEqualToAnchor(&content_area.leadingAnchor())
                .setActive(true);
            pane.trailingAnchor()
                .constraintEqualToAnchor(&content_area.trailingAnchor())
                .setActive(true);
            pane.bottomAnchor()
                .constraintEqualToAnchor(&content_area.bottomAnchor())
                .setActive(true);
        }
    }

    fn make_checkbox(
        &self,
        title: &str,
        tag: isize,
        checked: bool,
        mtm: MainThreadMarker,
    ) -> Retained<NSButton> {
        let zero_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
        let btn = unsafe {
            let b = NSButton::initWithFrame(NSButton::alloc(mtm), zero_rect);
            b.setTitle(&NSString::from_str(title));
            let _: () = msg_send![&b, setButtonType: 3usize]; // NSSwitchButton (Checkbox)
            b.setState(if checked {
                objc2_app_kit::NSControlStateValueOn
            } else {
                objc2_app_kit::NSControlStateValueOff
            });
            b.setTag(tag);
            b.setTarget(Some(&*self));
            b.setAction(Some(sel!(toggleCheckbox:)));
            b.setFont(Some(&NSFont::systemFontOfSize(13.0)));
            b.setTranslatesAutoresizingMaskIntoConstraints(false);
            b
        };
        btn
    }

    fn make_header_label(&self, text: &str, mtm: MainThreadMarker) -> Retained<NSTextField> {
        let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
        label.setFont(Some(&NSFont::boldSystemFontOfSize(13.0)));
        label.setTextColor(Some(&NSColor::labelColor()));
        label.setTranslatesAutoresizingMaskIntoConstraints(false);
        label
    }

    fn make_secondary_label(&self, text: &str, mtm: MainThreadMarker) -> Retained<NSTextField> {
        let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
        label.setFont(Some(&NSFont::systemFontOfSize(11.0)));
        label.setTextColor(Some(&NSColor::secondaryLabelColor()));
        label.setTranslatesAutoresizingMaskIntoConstraints(false);
        label
    }

    fn create_general_pane(&self, mtm: MainThreadMarker) -> Retained<NSView> {
        let zero_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
        let view = NSView::initWithFrame(NSView::alloc(mtm), zero_rect);

        let title = self.make_header_label("Session & Startup", mtm);
        let c1 = self.make_checkbox(
            "Restore previous session on launch",
            101,
            get_bool_pref(PREF_RESTORE_SESSION, true),
            mtm,
        );
        let s1 = self.make_secondary_label(
            "Automatically reopens the documents that were active when you quit.",
            mtm,
        );

        let c2 = self.make_checkbox(
            "Remember recent documents and workspaces",
            102,
            get_bool_pref(PREF_REMEMBER_RECENTS, true),
            mtm,
        );
        let s2 = self.make_secondary_label(
            "Displays recent PDFs in Home screen and Command Palette.",
            mtm,
        );

        let c3 = self.make_checkbox(
            "Reopen previously closed tabs",
            103,
            get_bool_pref(PREF_REOPEN_CLOSED_TABS, true),
            mtm,
        );
        let c4 = self.make_checkbox(
            "Open PDFs in a new tab by default",
            104,
            get_bool_pref(PREF_OPEN_IN_NEW_TAB, true),
            mtm,
        );

        view.addSubview(&title);
        view.addSubview(&c1);
        view.addSubview(&s1);
        view.addSubview(&c2);
        view.addSubview(&s2);
        view.addSubview(&c3);
        view.addSubview(&c4);

        title
            .topAnchor()
            .constraintEqualToAnchor_constant(&view.topAnchor(), 16.0)
            .setActive(true);
        title
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);

        c1.topAnchor()
            .constraintEqualToAnchor_constant(&title.bottomAnchor(), 12.0)
            .setActive(true);
        c1.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        s1.topAnchor()
            .constraintEqualToAnchor_constant(&c1.bottomAnchor(), 3.0)
            .setActive(true);
        s1.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 44.0)
            .setActive(true);

        c2.topAnchor()
            .constraintEqualToAnchor_constant(&s1.bottomAnchor(), 12.0)
            .setActive(true);
        c2.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        s2.topAnchor()
            .constraintEqualToAnchor_constant(&c2.bottomAnchor(), 3.0)
            .setActive(true);
        s2.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 44.0)
            .setActive(true);

        c3.topAnchor()
            .constraintEqualToAnchor_constant(&s2.bottomAnchor(), 12.0)
            .setActive(true);
        c3.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);

        c4.topAnchor()
            .constraintEqualToAnchor_constant(&c3.bottomAnchor(), 10.0)
            .setActive(true);
        c4.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);

        view
    }

    fn create_reading_pane(&self, mtm: MainThreadMarker) -> Retained<NSView> {
        let zero_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
        let view = NSView::initWithFrame(NSView::alloc(mtm), zero_rect);

        let l_mode = self.make_header_label("Default Display Mode:", mtm);
        let seg_mode = unsafe {
            let seg = NSSegmentedControl::initWithFrame(NSSegmentedControl::alloc(mtm), zero_rect);
            seg.setSegmentCount(3);
            seg.setLabel_forSegment(ns_string!("Single Page"), 0);
            seg.setLabel_forSegment(ns_string!("Continuous"), 1);
            seg.setLabel_forSegment(ns_string!("Two Pages"), 2);
            seg.setSelectedSegment(get_int_pref(PREF_DEFAULT_DISPLAY_MODE, 1));
            seg.setTag(210);
            seg.setTarget(Some(&*self));
            seg.setAction(Some(sel!(segmentChanged:)));
            seg.setTranslatesAutoresizingMaskIntoConstraints(false);
            seg
        };

        let l_zoom = self.make_header_label("Default Zoom Mode:", mtm);
        let seg_zoom = unsafe {
            let seg = NSSegmentedControl::initWithFrame(NSSegmentedControl::alloc(mtm), zero_rect);
            seg.setSegmentCount(3);
            seg.setLabel_forSegment(ns_string!("Fit Width"), 0);
            seg.setLabel_forSegment(ns_string!("Fit Page"), 1);
            seg.setLabel_forSegment(ns_string!("Actual Size"), 2);
            seg.setSelectedSegment(get_int_pref(PREF_DEFAULT_ZOOM_MODE, 0));
            seg.setTag(220);
            seg.setTarget(Some(&*self));
            seg.setAction(Some(sel!(segmentChanged:)));
            seg.setTranslatesAutoresizingMaskIntoConstraints(false);
            seg
        };

        let l_color = self.make_header_label("Default Annotation Color:", mtm);
        let seg_color = unsafe {
            let seg = NSSegmentedControl::initWithFrame(NSSegmentedControl::alloc(mtm), zero_rect);
            seg.setSegmentCount(4);
            seg.setLabel_forSegment(ns_string!("Yellow"), 0);
            seg.setLabel_forSegment(ns_string!("Green"), 1);
            seg.setLabel_forSegment(ns_string!("Blue"), 2);
            seg.setLabel_forSegment(ns_string!("Pink"), 3);
            seg.setSelectedSegment(get_int_pref(PREF_DEFAULT_HIGHLIGHT_COLOR, 0));
            seg.setTag(230);
            seg.setTarget(Some(&*self));
            seg.setAction(Some(sel!(segmentChanged:)));
            seg.setTranslatesAutoresizingMaskIntoConstraints(false);
            seg
        };

        let c_pos = self.make_checkbox(
            "Restore reading position & zoom factor per PDF",
            201,
            get_bool_pref(PREF_RESTORE_READING_POS, true),
            mtm,
        );
        let s_pos =
            self.make_secondary_label("Saves page number and scale in local preferences.", mtm);

        view.addSubview(&l_mode);
        view.addSubview(&seg_mode);
        view.addSubview(&l_zoom);
        view.addSubview(&seg_zoom);
        view.addSubview(&l_color);
        view.addSubview(&seg_color);
        view.addSubview(&c_pos);
        view.addSubview(&s_pos);

        l_mode
            .topAnchor()
            .constraintEqualToAnchor_constant(&view.topAnchor(), 16.0)
            .setActive(true);
        l_mode
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        seg_mode
            .centerYAnchor()
            .constraintEqualToAnchor(&l_mode.centerYAnchor())
            .setActive(true);
        seg_mode
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 220.0)
            .setActive(true);

        l_zoom
            .topAnchor()
            .constraintEqualToAnchor_constant(&l_mode.bottomAnchor(), 20.0)
            .setActive(true);
        l_zoom
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        seg_zoom
            .centerYAnchor()
            .constraintEqualToAnchor(&l_zoom.centerYAnchor())
            .setActive(true);
        seg_zoom
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 220.0)
            .setActive(true);

        l_color
            .topAnchor()
            .constraintEqualToAnchor_constant(&l_zoom.bottomAnchor(), 20.0)
            .setActive(true);
        l_color
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        seg_color
            .centerYAnchor()
            .constraintEqualToAnchor(&l_color.centerYAnchor())
            .setActive(true);
        seg_color
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 220.0)
            .setActive(true);

        c_pos
            .topAnchor()
            .constraintEqualToAnchor_constant(&l_color.bottomAnchor(), 22.0)
            .setActive(true);
        c_pos
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        s_pos
            .topAnchor()
            .constraintEqualToAnchor_constant(&c_pos.bottomAnchor(), 3.0)
            .setActive(true);
        s_pos
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 44.0)
            .setActive(true);

        view
    }

    fn create_workspace_pane(&self, mtm: MainThreadMarker) -> Retained<NSView> {
        let zero_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
        let view = NSView::initWithFrame(NSView::alloc(mtm), zero_rect);

        let title = self.make_header_label("Workspace Navigation & Indexing", mtm);
        let c1 = self.make_checkbox(
            "Auto-reveal active PDF in sidebar explorer",
            301,
            get_bool_pref(PREF_AUTO_REVEAL_SIDEBAR, true),
            mtm,
        );
        let s1 = self.make_secondary_label(
            "Scrolls the file explorer tree to select the active document.",
            mtm,
        );

        let c2 = self.make_checkbox(
            "Show hidden files and dotfiles",
            302,
            get_bool_pref(PREF_SHOW_HIDDEN_FILES, false),
            mtm,
        );
        let s2 = self.make_secondary_label(
            "Includes hidden files (.git, .config, etc.) in file tree.",
            mtm,
        );

        let c3 = self.make_checkbox(
            "Deep workspace search indexing",
            303,
            get_bool_pref(PREF_DEEP_WORKSPACE_SEARCH, true),
            mtm,
        );
        let s3 = self.make_secondary_label(
            "Indexes titles and outline nodes for instantaneous Cmd+P lookup.",
            mtm,
        );

        view.addSubview(&title);
        view.addSubview(&c1);
        view.addSubview(&s1);
        view.addSubview(&c2);
        view.addSubview(&s2);
        view.addSubview(&c3);
        view.addSubview(&s3);

        title
            .topAnchor()
            .constraintEqualToAnchor_constant(&view.topAnchor(), 16.0)
            .setActive(true);
        title
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);

        c1.topAnchor()
            .constraintEqualToAnchor_constant(&title.bottomAnchor(), 12.0)
            .setActive(true);
        c1.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        s1.topAnchor()
            .constraintEqualToAnchor_constant(&c1.bottomAnchor(), 3.0)
            .setActive(true);
        s1.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 44.0)
            .setActive(true);

        c2.topAnchor()
            .constraintEqualToAnchor_constant(&s1.bottomAnchor(), 12.0)
            .setActive(true);
        c2.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        s2.topAnchor()
            .constraintEqualToAnchor_constant(&c2.bottomAnchor(), 3.0)
            .setActive(true);
        s2.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 44.0)
            .setActive(true);

        c3.topAnchor()
            .constraintEqualToAnchor_constant(&s2.bottomAnchor(), 12.0)
            .setActive(true);
        c3.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        s3.topAnchor()
            .constraintEqualToAnchor_constant(&c3.bottomAnchor(), 3.0)
            .setActive(true);
        s3.leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 44.0)
            .setActive(true);

        view
    }

    fn create_appearance_pane(&self, mtm: MainThreadMarker) -> Retained<NSView> {
        let zero_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
        let view = NSView::initWithFrame(NSView::alloc(mtm), zero_rect);

        let l_theme = self.make_header_label("Interface Theme:", mtm);
        let seg_theme = unsafe {
            let seg = NSSegmentedControl::initWithFrame(NSSegmentedControl::alloc(mtm), zero_rect);
            seg.setSegmentCount(3);
            seg.setLabel_forSegment(ns_string!("System"), 0);
            seg.setLabel_forSegment(ns_string!("Light"), 1);
            seg.setLabel_forSegment(ns_string!("Dark"), 2);
            seg.setSelectedSegment(get_int_pref(PREF_APPEARANCE_MODE, 0));
            seg.setTag(410);
            seg.setTarget(Some(&*self));
            seg.setAction(Some(sel!(segmentChanged:)));
            seg.setTranslatesAutoresizingMaskIntoConstraints(false);
            seg
        };

        let c_glass = self.make_checkbox(
            "Enable Liquid Glass native materials",
            401,
            get_bool_pref(PREF_LIQUID_GLASS, true),
            mtm,
        );
        let s_glass = self.make_secondary_label(
            "Uses Apple AppKit NSVisualEffectView sidebar, header, and HUD materials.",
            mtm,
        );

        view.addSubview(&l_theme);
        view.addSubview(&seg_theme);
        view.addSubview(&c_glass);
        view.addSubview(&s_glass);

        l_theme
            .topAnchor()
            .constraintEqualToAnchor_constant(&view.topAnchor(), 20.0)
            .setActive(true);
        l_theme
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        seg_theme
            .centerYAnchor()
            .constraintEqualToAnchor(&l_theme.centerYAnchor())
            .setActive(true);
        seg_theme
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 180.0)
            .setActive(true);

        c_glass
            .topAnchor()
            .constraintEqualToAnchor_constant(&l_theme.bottomAnchor(), 22.0)
            .setActive(true);
        c_glass
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        s_glass
            .topAnchor()
            .constraintEqualToAnchor_constant(&c_glass.bottomAnchor(), 3.0)
            .setActive(true);
        s_glass
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 44.0)
            .setActive(true);

        view
    }

    fn create_advanced_pane(&self, mtm: MainThreadMarker) -> Retained<NSView> {
        let zero_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
        let view = NSView::initWithFrame(NSView::alloc(mtm), zero_rect);

        let title_shortcuts = self.make_header_label("Keyboard Shortcuts", mtm);
        let s_sc1 = self.make_secondary_label(
            "⌘P Quick Open   •   ⌘⇧P Command Palette   •   ⌘F Search Document",
            mtm,
        );
        let s_sc2 = self.make_secondary_label(
            "⌘D Bookmark Page   •   ⌘I Document Info   •   ⌘⇧R OCR Current Page",
            mtm,
        );

        let title_ocr = self.make_header_label("Optical Character Recognition (OCR)", mtm);
        let ocr_info = self.make_secondary_label(
            "Engine: Apple Vision Framework (Accurate, Neural Engine accelerated)",
            mtm,
        );

        let c_ocr = self.make_checkbox(
            "Automatically copy recognized text to clipboard",
            501,
            get_bool_pref(PREF_AUTO_COPY_OCR, true),
            mtm,
        );

        let title_reset = self.make_header_label("Maintenance & Reset", mtm);
        let btn_reset = unsafe {
            let b = NSButton::initWithFrame(NSButton::alloc(mtm), zero_rect);
            b.setTitle(ns_string!("Reset All Preferences to Defaults"));
            let _: () = msg_send![&b, setBezelStyle: 1usize];
            b.setTarget(Some(&*self));
            b.setAction(Some(sel!(resetDefaults:)));
            b.setFont(Some(&NSFont::systemFontOfSize(12.0)));
            b.setTranslatesAutoresizingMaskIntoConstraints(false);
            b
        };

        view.addSubview(&title_shortcuts);
        view.addSubview(&s_sc1);
        view.addSubview(&s_sc2);
        view.addSubview(&title_ocr);
        view.addSubview(&ocr_info);
        view.addSubview(&c_ocr);
        view.addSubview(&title_reset);
        view.addSubview(&btn_reset);

        title_shortcuts
            .topAnchor()
            .constraintEqualToAnchor_constant(&view.topAnchor(), 14.0)
            .setActive(true);
        title_shortcuts
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);

        s_sc1
            .topAnchor()
            .constraintEqualToAnchor_constant(&title_shortcuts.bottomAnchor(), 4.0)
            .setActive(true);
        s_sc1
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);

        s_sc2
            .topAnchor()
            .constraintEqualToAnchor_constant(&s_sc1.bottomAnchor(), 3.0)
            .setActive(true);
        s_sc2
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);

        title_ocr
            .topAnchor()
            .constraintEqualToAnchor_constant(&s_sc2.bottomAnchor(), 14.0)
            .setActive(true);
        title_ocr
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);
        ocr_info
            .topAnchor()
            .constraintEqualToAnchor_constant(&title_ocr.bottomAnchor(), 4.0)
            .setActive(true);
        ocr_info
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);

        c_ocr
            .topAnchor()
            .constraintEqualToAnchor_constant(&ocr_info.bottomAnchor(), 8.0)
            .setActive(true);
        c_ocr
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);

        title_reset
            .topAnchor()
            .constraintEqualToAnchor_constant(&c_ocr.bottomAnchor(), 14.0)
            .setActive(true);
        title_reset
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);

        btn_reset
            .topAnchor()
            .constraintEqualToAnchor_constant(&title_reset.bottomAnchor(), 6.0)
            .setActive(true);
        btn_reset
            .leadingAnchor()
            .constraintEqualToAnchor_constant(&view.leadingAnchor(), 24.0)
            .setActive(true);

        view
    }
}

thread_local! {
    pub static SETTINGS_CONTROLLER: RefCell<Option<Retained<SettingsDelegate>>> = RefCell::new(None);
}

pub fn get_settings_controller() -> Option<Retained<SettingsDelegate>> {
    SETTINGS_CONTROLLER.with(|c| c.borrow().clone())
}

pub fn show_settings_window(mtm: MainThreadMarker) {
    let delegate = SETTINGS_CONTROLLER.with(|c| {
        let mut guard = c.borrow_mut();
        if let Some(ref del) = *guard {
            del.clone()
        } else {
            let del = SettingsDelegate::new(mtm);

            let vis = WindowGeometry::screen_visible_frame(None, mtm);
            let default_frame = WindowGeometry::default_frame(vis, WindowKind::Utility);
            let rect = NSRect::new(default_frame.origin, NSSize::new(520.0, 340.0));
            let style = NSWindowStyleMask::Titled
                | NSWindowStyleMask::Closable
                | NSWindowStyleMask::Miniaturizable
                | NSWindowStyleMask::FullSizeContentView;

            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(mtm),
                    rect,
                    style,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };

            unsafe {
                window.setReleasedWhenClosed(false);
                window.setTitlebarAppearsTransparent(true);
                window.setOpaque(false);
                window.setBackgroundColor(Some(&NSColor::clearColor()));
                window.setHasShadow(true);
                window.setTitle(ns_string!("Settings"));
                let del_ptr: *const SettingsDelegate = &*del;
                let _: () = msg_send![&window, setDelegate: del_ptr];
            }

            // Top Header with Native Icon-Above-Label Category Bar
            let header_view = unsafe {
                let zero_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
                let effect =
                    NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), zero_rect);
                effect.setMaterial(NSVisualEffectMaterial::HeaderView);
                effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
                effect.setState(NSVisualEffectState::Active);
                effect.setTranslatesAutoresizingMaskIntoConstraints(false);
                effect
            };

            let categories = [
                ("General", "gearshape"),
                ("Reading", "book"),
                ("Workspace", "folder"),
                ("Appearance", "circle.lefthalf.filled"),
                ("Advanced", "slider.horizontal.3"),
            ];

            let tab_bar_container = unsafe {
                let v = NSView::initWithFrame(
                    NSView::alloc(mtm),
                    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(400.0, 46.0)),
                );
                v.setTranslatesAutoresizingMaskIntoConstraints(false);
                v
            };

            let mut pills = Vec::new();
            let mut icons = Vec::new();
            let mut labels = Vec::new();

            for (idx, (label_text, symbol_name)) in categories.iter().enumerate() {
                unsafe {
                    let pill = NSView::initWithFrame(
                        NSView::alloc(mtm),
                        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(74.0, 44.0)),
                    );
                    pill.setWantsLayer(true);
                    if let Some(layer) = pill.layer() {
                        let _: () = msg_send![&layer, setCornerRadius: 7.0f64];
                    }
                    pill.setTranslatesAutoresizingMaskIntoConstraints(false);

                    let icon_cfg = objc2_app_kit::NSImageSymbolConfiguration::configurationWithPointSize_weight_scale(
                        15.0,
                        0.0,
                        objc2_app_kit::NSImageSymbolScale::Medium,
                    );
                    let img = objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(
                        &NSString::from_str(symbol_name),
                        Some(&NSString::from_str(label_text)),
                    )
                    .and_then(|i| i.imageWithSymbolConfiguration(&icon_cfg))
                    .unwrap_or_else(|| objc2_app_kit::NSImage::new());
                    let iv = objc2_app_kit::NSImageView::imageViewWithImage(&img, mtm);
                    iv.setTranslatesAutoresizingMaskIntoConstraints(false);

                    let lbl = NSTextField::labelWithString(&NSString::from_str(label_text), mtm);
                    lbl.setFont(Some(&NSFont::systemFontOfSize(10.5)));
                    lbl.setAlignment(objc2_app_kit::NSTextAlignment::Center);
                    lbl.setTranslatesAutoresizingMaskIntoConstraints(false);

                    let hit_btn = NSButton::initWithFrame(
                        NSButton::alloc(mtm),
                        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(74.0, 44.0)),
                    );
                    hit_btn.setTitle(ns_string!(""));
                    hit_btn.setBordered(false);
                    let _: () = msg_send![&hit_btn, setTransparent: true];
                    hit_btn.setTag(idx as isize);
                    hit_btn.setTarget(Some(&*del));
                    hit_btn.setAction(Some(sel!(tabChanged:)));
                    hit_btn.setTranslatesAutoresizingMaskIntoConstraints(false);

                    pill.addSubview(&iv);
                    pill.addSubview(&lbl);
                    pill.addSubview(&hit_btn);
                    tab_bar_container.addSubview(&pill);

                    let x_offset = (idx as f64) * 78.0;
                    pill.leadingAnchor()
                        .constraintEqualToAnchor_constant(&tab_bar_container.leadingAnchor(), x_offset)
                        .setActive(true);
                    pill.centerYAnchor()
                        .constraintEqualToAnchor(&tab_bar_container.centerYAnchor())
                        .setActive(true);
                    pill.widthAnchor().constraintEqualToConstant(74.0).setActive(true);
                    pill.heightAnchor().constraintEqualToConstant(44.0).setActive(true);

                    iv.centerXAnchor()
                        .constraintEqualToAnchor(&pill.centerXAnchor())
                        .setActive(true);
                    iv.topAnchor()
                        .constraintEqualToAnchor_constant(&pill.topAnchor(), 5.0)
                        .setActive(true);
                    iv.widthAnchor().constraintEqualToConstant(18.0).setActive(true);
                    iv.heightAnchor().constraintEqualToConstant(18.0).setActive(true);

                    lbl.centerXAnchor()
                        .constraintEqualToAnchor(&pill.centerXAnchor())
                        .setActive(true);
                    lbl.bottomAnchor()
                        .constraintEqualToAnchor_constant(&pill.bottomAnchor(), -4.0)
                        .setActive(true);

                    hit_btn
                        .leadingAnchor()
                        .constraintEqualToAnchor(&pill.leadingAnchor())
                        .setActive(true);
                    hit_btn
                        .trailingAnchor()
                        .constraintEqualToAnchor(&pill.trailingAnchor())
                        .setActive(true);
                    hit_btn
                        .topAnchor()
                        .constraintEqualToAnchor(&pill.topAnchor())
                        .setActive(true);
                    hit_btn
                        .bottomAnchor()
                        .constraintEqualToAnchor(&pill.bottomAnchor())
                        .setActive(true);

                    pills.push(pill);
                    icons.push(iv);
                    labels.push(lbl);
                }
            }

            let sep = unsafe {
                let zero_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
                let v = NSView::initWithFrame(NSView::alloc(mtm), zero_rect);
                v.setWantsLayer(true);
                let layer: Retained<objc2::runtime::AnyObject> = msg_send![&v, layer];
                let cg = NSColor::separatorColor().CGColor();
                let ptr = Retained::as_ptr(&cg);
                let _: () = msg_send![&layer, setBackgroundColor: ptr];
                v.setTranslatesAutoresizingMaskIntoConstraints(false);
                v
            };

            let content_area = unsafe {
                let zero_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
                let v = NSView::initWithFrame(NSView::alloc(mtm), zero_rect);
                v.setTranslatesAutoresizingMaskIntoConstraints(false);
                v
            };

            let _root_view = unsafe {
                let v = NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), rect);
                v.setMaterial(NSVisualEffectMaterial::UnderWindowBackground);
                v.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
                v.setState(NSVisualEffectState::FollowsWindowActiveState);
                v.setAutoresizingMask(
                    objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable
                        | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable,
                );
                v.addSubview(&header_view);
                v.addSubview(&content_area);

                header_view.addSubview(&tab_bar_container);
                header_view.addSubview(&sep);

                header_view
                    .topAnchor()
                    .constraintEqualToAnchor(&v.topAnchor())
                    .setActive(true);
                header_view
                    .leadingAnchor()
                    .constraintEqualToAnchor(&v.leadingAnchor())
                    .setActive(true);
                header_view
                    .trailingAnchor()
                    .constraintEqualToAnchor(&v.trailingAnchor())
                    .setActive(true);
                header_view
                    .heightAnchor()
                    .constraintEqualToConstant(80.0)
                    .setActive(true);

                tab_bar_container
                    .centerXAnchor()
                    .constraintEqualToAnchor(&header_view.centerXAnchor())
                    .setActive(true);
                tab_bar_container
                    .bottomAnchor()
                    .constraintEqualToAnchor_constant(&header_view.bottomAnchor(), -4.0)
                    .setActive(true);
                tab_bar_container
                    .widthAnchor()
                    .constraintEqualToConstant(386.0)
                    .setActive(true);
                tab_bar_container
                    .heightAnchor()
                    .constraintEqualToConstant(46.0)
                    .setActive(true);

                sep.bottomAnchor()
                    .constraintEqualToAnchor(&header_view.bottomAnchor())
                    .setActive(true);
                sep.leadingAnchor()
                    .constraintEqualToAnchor(&header_view.leadingAnchor())
                    .setActive(true);
                sep.trailingAnchor()
                    .constraintEqualToAnchor(&header_view.trailingAnchor())
                    .setActive(true);
                sep.heightAnchor()
                    .constraintEqualToConstant(1.0)
                    .setActive(true);

                content_area
                    .topAnchor()
                    .constraintEqualToAnchor(&header_view.bottomAnchor())
                    .setActive(true);
                content_area
                    .leadingAnchor()
                    .constraintEqualToAnchor(&v.leadingAnchor())
                    .setActive(true);
                content_area
                    .trailingAnchor()
                    .constraintEqualToAnchor(&v.trailingAnchor())
                    .setActive(true);
                content_area
                    .bottomAnchor()
                    .constraintEqualToAnchor(&v.bottomAnchor())
                    .setActive(true);

                window.setContentView(Some(&v));
                v
            };

            del.ivars().window.borrow_mut().replace(window.clone());
            del.ivars().content_area.borrow_mut().replace(content_area);
            *del.ivars().tab_pills.borrow_mut() = pills;
            *del.ivars().tab_icons.borrow_mut() = icons;
            *del.ivars().tab_labels.borrow_mut() = labels;

            del.switch_to_tab(0);

            *guard = Some(del.clone());
            del
        }
    });

    if let Some(win) = delegate.ivars().window.borrow().as_ref() {
        unsafe {
            win.center();
            win.makeKeyAndOrderFront(None);
            let app = NSApplication::sharedApplication(mtm);
            let _: () = msg_send![&app, activateIgnoringOtherApps: true];
        }
    }
}

pub fn apply_appearance_mode(mode: isize, mtm: MainThreadMarker) {
    let app = NSApplication::sharedApplication(mtm);
    unsafe {
        match mode {
            1 => {
                let aqua = NSAppearance::appearanceNamed(NSAppearanceNameAqua);
                app.setAppearance(aqua.as_deref());
            }
            2 => {
                let dark = NSAppearance::appearanceNamed(NSAppearanceNameDarkAqua);
                app.setAppearance(dark.as_deref());
            }
            _ => {
                app.setAppearance(None);
            }
        }
    }
}
