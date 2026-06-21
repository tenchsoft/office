use super::state::{col_letter, SheetsState};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

const DOC_TAB_H: f64 = 28.0;
const DOC_TAB_W: f64 = 140.0;
const CLOSE_BTN_W: f64 = 16.0;

/// Calculate dynamic tab width based on sheet name length.
/// Min width 60px, max width 200px.
pub fn sheet_tab_width(name: &str) -> f64 {
    // Approximate: ~8px per character + 24px padding
    let char_width = 8.0;
    let padding = 24.0;
    let estimated = name.len() as f64 * char_width + padding;
    estimated.clamp(60.0, 200.0)
}

pub fn paint_sheet_tabs(state: &SheetsState, p: &mut Painter<'_>, theme: &Theme, rect: Rect) {
    p.fill_rect(rect, theme.surface);

    // Sheet navigation buttons (left side)
    let nav_y = rect.y0 + 4.0;
    let nav_h = rect.height() - 8.0;
    let nav_buttons = ["|<", "<", ">", ">|"];
    let mut nav_x = rect.x0 + 8.0;
    for label in &nav_buttons {
        let btn = Rect::new(nav_x, nav_y, nav_x + 20.0, nav_y + nav_h);
        p.fill_rounded_rect(btn, theme.background, 3.0);
        p.draw_text(
            label,
            nav_x + 4.0,
            nav_y + nav_h * 0.75,
            theme.secondary,
            theme.font_size_small * 0.85,
            FontWeight::NORMAL,
            false,
        );
        nav_x += 24.0;
    }

    // Sheet tabs with dynamic widths
    let tab_start_x = nav_x + 8.0;
    let mut tab_x = tab_start_x;
    for (idx, name) in state.sheet_names.iter().enumerate() {
        let w = sheet_tab_width(name);
        let active = idx == state.active_sheet;
        let item = Rect::new(tab_x, rect.y0 + 4.0, tab_x + w, rect.y1 - 4.0);

        // Tab color indicator (thin bar at the bottom of the tab)
        let tab_color = state.sheet_tab_colors.get(&idx);

        if active {
            p.fill_rounded_rect(item, theme.primary, 4.0);
        } else {
            p.fill_rounded_rect(item, theme.background, 4.0);
        }

        // Tab color bar at the bottom
        if let Some(&color) = tab_color {
            let bar_h = 3.0;
            let bar = Rect::new(
                tab_x + 2.0,
                rect.y1 - 4.0 - bar_h,
                tab_x + w - 2.0,
                rect.y1 - 4.0,
            );
            p.fill_rounded_rect(bar, color, 1.5);
        }

        // Inline rename field or normal text
        if state.renaming_sheet == Some(idx) {
            // Show rename draft with a highlight border
            let edit_rect = Rect::new(tab_x + 2.0, rect.y0 + 6.0, tab_x + w - 2.0, rect.y1 - 6.0);
            p.fill_rounded_rect(edit_rect, Color::WHITE, 2.0);
            p.stroke_rounded_rect(edit_rect, theme.primary, 1.0, 2.0);
            let display = if state.rename_draft.is_empty() {
                name
            } else {
                &state.rename_draft
            };
            p.draw_text(
                display,
                tab_x + 6.0,
                rect.y0 + 20.0,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
            // Cursor indicator
            let cursor_x = tab_x + 6.0 + display.len() as f64 * 7.0;
            p.draw_line(
                Point::new(cursor_x, rect.y0 + 8.0),
                Point::new(cursor_x, rect.y0 + 22.0),
                theme.primary,
                1.0,
            );
        } else {
            // Dirty indicator: show a dot after the name if dirty
            let dirty = state.is_dirty();
            let label = if dirty && active {
                format!("{} *", name)
            } else {
                name.clone()
            };
            p.draw_text(
                &label,
                tab_x + 8.0,
                rect.y0 + 20.0,
                if active {
                    theme.on_primary
                } else {
                    theme.on_surface
                },
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }

        tab_x += w;
    }

    // '+' button to add a new sheet
    let plus_rect = Rect::new(tab_x + 4.0, rect.y0 + 4.0, tab_x + 28.0, rect.y1 - 4.0);
    p.fill_rounded_rect(plus_rect, theme.background, 4.0);
    p.stroke_rounded_rect(plus_rect, theme.border, 0.5, 4.0);
    p.draw_text(
        "+",
        tab_x + 12.0,
        rect.y0 + 20.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
}

pub fn paint_status_bar(state: &SheetsState, p: &mut Painter<'_>, theme: &Theme, rect: Rect) {
    p.fill_rect(rect, theme.background);
    let y = rect.y0 + rect.height() / 2.0 - 2.0;

    // Cell coordinate indicator
    let cell_ref = format!(
        "{}{}",
        col_letter(state.selected_col),
        state.selected_row + 1
    );
    p.draw_text(
        &cell_ref,
        rect.x0 + 8.0,
        y,
        theme.primary,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
    let mut stat_x = rect.x0 + 60.0;

    // Selection stats (left side)
    if !state.status_sum.is_empty() {
        p.draw_text(
            &format!("Sum: {}", state.status_sum),
            stat_x,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        stat_x += 120.0;
    }
    if state.status_count > 0 {
        p.draw_text(
            &format!("Count: {}", state.status_count),
            stat_x,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        stat_x += 110.0;
    }
    if !state.status_average.is_empty() {
        p.draw_text(
            &format!("Avg: {}", state.status_average),
            stat_x,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        stat_x += 110.0;
    }
    if !state.status_min.is_empty() {
        p.draw_text(
            &format!("Min: {}", state.status_min),
            stat_x,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        stat_x += 100.0;
    }
    if !state.status_max.is_empty() {
        p.draw_text(
            &format!("Max: {}", state.status_max),
            stat_x,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        stat_x += 100.0;
    }

    // Status / dirty indicator
    p.draw_text(
        state.status_line(),
        stat_x + 20.0,
        y,
        if state.is_dirty() {
            theme.primary
        } else {
            theme.secondary
        },
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );

    // Undo count
    if state.undo_count() > 0 {
        p.draw_text(
            &format!("Undo: {}", state.undo_count()),
            rect.x1 - 380.0,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
    }

    // Zoom indicator
    p.draw_text(
        &format!("Zoom: {}%", state.zoom_percent),
        rect.x1 - 450.0,
        y,
        theme.secondary,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );

    // Auto-save indicator
    if state.auto_save.enabled {
        let elapsed = state.auto_save.last_save_time.elapsed().as_secs();
        p.draw_text(
            &format!("Saved {}s ago", elapsed),
            rect.x1 - 560.0,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
    }

    // 10.4 Page info in status bar (only when pages have been computed)
    if !state.print_preview.pages.is_empty() {
        p.draw_text(
            &format!("Pages: {}", state.print_preview.pages.len()),
            rect.x1 - 260.0,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
    }

    // Zoom controls (right side)
    let zoom_x = rect.x1 - 160.0;
    let zoom_y = rect.y0 + 4.0;
    let zoom_h = rect.height() - 8.0;

    // Zoom out button
    let zoom_out_rect = Rect::new(zoom_x, zoom_y, zoom_x + 20.0, zoom_y + zoom_h);
    p.fill_rounded_rect(zoom_out_rect, theme.surface, 3.0);
    p.draw_text(
        "-",
        zoom_x + 6.0,
        zoom_y + zoom_h * 0.78,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );

    // Zoom percentage (clickable to reset to 100%)
    let zoom_pct_rect = Rect::new(zoom_x + 24.0, zoom_y, zoom_x + 76.0, zoom_y + zoom_h);
    p.fill_rounded_rect(zoom_pct_rect, theme.surface, 3.0);
    p.draw_text(
        &format!("{}%", state.zoom_percent),
        zoom_x + 32.0,
        zoom_y + zoom_h * 0.78,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );

    // Zoom in button
    let zoom_in_rect = Rect::new(zoom_x + 80.0, zoom_y, zoom_x + 100.0, zoom_y + zoom_h);
    p.fill_rounded_rect(zoom_in_rect, theme.surface, 3.0);
    p.draw_text(
        "+",
        zoom_x + 87.0,
        zoom_y + zoom_h * 0.78,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );

    // Zoom slider track
    let slider_x = zoom_x + 106.0;
    let slider_w = 44.0;
    let slider_y_mid = zoom_y + zoom_h / 2.0;
    p.draw_line(
        Point::new(slider_x, slider_y_mid),
        Point::new(slider_x + slider_w, slider_y_mid),
        theme.border,
        2.0,
    );
    // Slider thumb position based on zoom (25-400 mapped to slider width)
    let thumb_pos = slider_x + (state.zoom_percent - 25) as f64 / (400 - 25) as f64 * slider_w;
    p.fill_circle(Point::new(thumb_pos, slider_y_mid), 5.0, theme.primary);
}

/// Paint the document tabs (above the menu bar).
pub fn paint_doc_tabs(state: &SheetsState, p: &mut Painter<'_>, theme: &Theme, width: f64) {
    let tab_top = 0.0;
    p.fill_rect(
        Rect::new(0.0, tab_top, width, tab_top + DOC_TAB_H),
        theme.background,
    );

    for (idx, tab) in state.doc_tabs.iter().enumerate() {
        let x = 4.0 + idx as f64 * DOC_TAB_W;
        let is_active = idx == state.active_tab_idx;
        let tab_rect = Rect::new(
            x,
            tab_top + 2.0,
            x + DOC_TAB_W - 4.0,
            tab_top + DOC_TAB_H - 2.0,
        );

        p.fill_rounded_rect(
            tab_rect,
            if is_active {
                theme.surface
            } else {
                theme.background
            },
            4.0,
        );
        if is_active {
            p.stroke_rounded_rect(tab_rect, theme.primary, 1.0, 4.0);
        }

        // Title (with dirty indicator)
        let title = if tab.dirty {
            format!("{} *", tab.title)
        } else {
            tab.title.clone()
        };
        p.draw_text(
            &title,
            x + 8.0,
            tab_top + DOC_TAB_H * 0.7,
            if is_active {
                theme.on_surface
            } else {
                theme.secondary
            },
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );

        // Close button (X)
        let close_x = x + DOC_TAB_W - CLOSE_BTN_W - 8.0;
        let close_y = tab_top + DOC_TAB_H / 2.0 - 4.0;
        p.draw_text(
            "x",
            close_x,
            close_y + 10.0,
            theme.disabled,
            theme.font_size_small * 0.85,
            FontWeight::NORMAL,
            false,
        );
    }
}

/// Hit-test document tabs: returns Some(tab_idx) if a tab was clicked.
pub fn hit_doc_tab(x: f64, _y: f64, tab_count: usize) -> Option<usize> {
    for idx in 0..tab_count {
        let tab_x = 4.0 + idx as f64 * DOC_TAB_W;
        if x >= tab_x && x < tab_x + DOC_TAB_W - 4.0 {
            return Some(idx);
        }
    }
    None
}

/// Hit-test the close button on a document tab.
pub fn hit_doc_tab_close(x: f64, _y: f64, tab_idx: usize) -> bool {
    let tab_x = 4.0 + tab_idx as f64 * DOC_TAB_W;
    let close_x = tab_x + DOC_TAB_W - CLOSE_BTN_W - 8.0;
    x >= close_x && x < close_x + CLOSE_BTN_W
}

/// Hit-test zoom controls in the status bar.
pub fn hit_zoom_controls(x: f64, y: f64, rect: Rect) -> Option<ZoomAction> {
    let zoom_x = rect.x1 - 160.0;
    let zoom_y = rect.y0 + 4.0;
    let zoom_h = rect.height() - 8.0;

    if y < zoom_y || y > zoom_y + zoom_h {
        return None;
    }

    // Zoom out
    if x >= zoom_x && x < zoom_x + 20.0 {
        return Some(ZoomAction::ZoomOut);
    }
    // Zoom percentage (reset)
    if x >= zoom_x + 24.0 && x < zoom_x + 76.0 {
        return Some(ZoomAction::ResetZoom);
    }
    // Zoom in
    if x >= zoom_x + 80.0 && x < zoom_x + 100.0 {
        return Some(ZoomAction::ZoomIn);
    }
    None
}

/// Hit-test sheet navigation buttons.
pub fn hit_sheet_nav(x: f64, y: f64, rect: Rect) -> Option<NavAction> {
    if y < rect.y0 + 4.0 || y > rect.y1 - 4.0 {
        return None;
    }
    let nav_x = rect.x0 + 8.0;
    if x >= nav_x && x < nav_x + 20.0 {
        return Some(NavAction::First);
    }
    if x >= nav_x + 24.0 && x < nav_x + 44.0 {
        return Some(NavAction::Prev);
    }
    if x >= nav_x + 48.0 && x < nav_x + 68.0 {
        return Some(NavAction::Next);
    }
    if x >= nav_x + 72.0 && x < nav_x + 92.0 {
        return Some(NavAction::Last);
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomAction {
    ZoomIn,
    ZoomOut,
    ResetZoom,
    SliderDrag,
}

/// Hit-test the zoom slider thumb. Returns `Some(ZoomAction::SliderDrag)` if the
/// thumb area (within 10px of the thumb centre) was clicked.
pub fn hit_zoom_slider(x: f64, y: f64, rect: Rect, zoom_percent: u32) -> Option<ZoomAction> {
    let zoom_x = rect.x1 - 160.0;
    let zoom_y = rect.y0 + 4.0;
    let zoom_h = rect.height() - 8.0;

    let slider_x = zoom_x + 106.0;
    let slider_w = 44.0;
    let slider_y_mid = zoom_y + zoom_h / 2.0;
    let thumb_pos = slider_x + (zoom_percent - 25) as f64 / (400 - 25) as f64 * slider_w;

    // Hit area: within 10px of the thumb centre vertically, and within slider track horizontally
    if (y - slider_y_mid).abs() <= 10.0 && x >= slider_x - 5.0 && x <= slider_x + slider_w + 5.0 {
        // Check if close to the thumb (within 10px)
        if (x - thumb_pos).abs() <= 10.0 {
            return Some(ZoomAction::SliderDrag);
        }
    }
    None
}

/// Compute zoom percent from an x-position on the slider track.
pub fn zoom_from_slider_x(x: f64, rect: Rect) -> u32 {
    let zoom_x = rect.x1 - 160.0;
    let slider_x = zoom_x + 106.0;
    let slider_w = 44.0;
    let ratio = ((x - slider_x) / slider_w).clamp(0.0, 1.0);
    (25.0 + ratio * (400.0 - 25.0)).round() as u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavAction {
    First,
    Prev,
    Next,
    Last,
}
