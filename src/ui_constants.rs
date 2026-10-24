/// Centralized UI constants for Iridecence PDF Reader.
/// Enforces Apple Liquid Glass compliance, standard radii, spacing, and geometry constraints.

pub struct UIConstants;

impl UIConstants {
    // Corner radii
    pub const PALETTE_CORNER_RADIUS: f64 = 18.0;
    pub const CARD_CORNER_RADIUS: f64 = 10.0;
    pub const CONTROL_CORNER_RADIUS: f64 = 6.0;
    pub const WINDOW_CORNER_RADIUS: f64 = 14.0;

    // Borders
    pub const SPECULAR_BORDER_WIDTH: f64 = 0.5;
    pub const SEPARATOR_THICKNESS: f64 = 0.5;

    // Sidebar dimensions
    pub const SIDEBAR_MIN_WIDTH: f64 = 180.0;
    pub const SIDEBAR_MAX_WIDTH: f64 = 360.0;
    pub const SIDEBAR_DEFAULT_WIDTH: f64 = 230.0;

    pub const HOME_SIDEBAR_MIN_WIDTH: f64 = 180.0;
    pub const HOME_SIDEBAR_MAX_WIDTH: f64 = 280.0;
    pub const HOME_SIDEBAR_DEFAULT_WIDTH: f64 = 210.0;

    // Control heights
    pub const SEARCH_FIELD_HEIGHT: f64 = 28.0;
    pub const SEGMENTED_CONTROL_HEIGHT: f64 = 24.0;
    pub const TABLE_ROW_COMPACT: f64 = 26.0;
    pub const TABLE_ROW_STANDARD: f64 = 44.0;
    pub const TABLE_ROW_HOME_RECENT: f64 = 58.0;

    // Spacing and paddings
    pub const HEADER_PADDING_HORIZONTAL: f64 = 24.0;
    pub const HEADER_PADDING_VERTICAL: f64 = 16.0;
    pub const CONTENT_PADDING: f64 = 16.0;
    pub const INTER_ITEM_SPACING: f64 = 8.0;
    pub const PDF_CANVAS_MARGIN: f64 = 24.0;

    // Animations (subtle, 120-180ms)
    pub const ANIMATION_DURATION_FAST: f64 = 0.12;
    pub const ANIMATION_DURATION_NORMAL: f64 = 0.18;
}
