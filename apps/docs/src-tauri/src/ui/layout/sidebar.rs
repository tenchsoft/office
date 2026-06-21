use tench_ui::kurbo::Rect;

use super::super::state::STYLE_PANEL_W;

/// Number of sidebar tabs (Style, Navigate, AI).
pub const TAB_COUNT: usize = 3;
/// Height of each sidebar tab header row.
pub const TAB_H: f64 = 36.0;

/// Computed geometry for the docs sidebar.
pub struct SidebarLayout {
    /// Full sidebar bounding rectangle.
    pub area: Rect,
    /// Bounding rectangles for each tab header.
    pub tabs: [Rect; TAB_COUNT],
    /// "Save snapshot" button rectangle (Style tab).
    pub save_snapshot: Rect,
}

/// Compute sidebar geometry from the current window dimensions.
///
/// `content_w` is the width of the main content area (window width minus sidebar).
/// `main_y` is the top of the main area (below menu bar + toolbar).
/// `status_y` is the top of the status bar.
pub fn compute_sidebar(content_w: f64, main_y: f64, status_y: f64) -> SidebarLayout {
    let sidebar_w = STYLE_PANEL_W;
    let sidebar_x = content_w;
    let area = Rect::new(sidebar_x, main_y, sidebar_x + sidebar_w, status_y);

    let tab_w = sidebar_w / TAB_COUNT as f64;
    let tabs = std::array::from_fn(|i| {
        let x = sidebar_x + i as f64 * tab_w;
        Rect::new(x, main_y, x + tab_w, main_y + TAB_H)
    });

    // Save snapshot button — matches the formula used in automation nodes.
    // The paint code computes this incrementally; both must stay in sync.
    let btn_y = main_y + 56.0 + 56.0 * 3.0 + 8.0 + 24.0 + 32.0 + 44.0 + 24.0;
    let save_snapshot = Rect::new(
        sidebar_x + 16.0,
        btn_y,
        sidebar_x + sidebar_w - 16.0,
        btn_y + 32.0,
    );

    SidebarLayout {
        area,
        tabs,
        save_snapshot,
    }
}
