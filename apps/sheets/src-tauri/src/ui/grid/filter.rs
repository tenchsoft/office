use crate::ui::state::{col_letter, SheetsState};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

use super::{GRID_COL_W, GRID_HEADER_H, ROW_HEADER_W};

const FILTER_DROPDOWN_W: f64 = 180.0;
const FILTER_ITEM_H: f64 = 22.0;

/// Paint the filter value dropdown for a given column.
pub(super) fn paint_filter_dropdown(
    p: &mut Painter<'_>,
    theme: &Theme,
    state: &SheetsState,
    x: f64,
    y: f64,
    col: usize,
) {
    let unique_vals = state.unique_values_for_col(col);
    let item_count = unique_vals.len().min(8);
    let dropdown_h = 36.0 + item_count as f64 * FILTER_ITEM_H + 28.0; // header + items + buttons
    let rect = Rect::new(x, y, x + FILTER_DROPDOWN_W, y + dropdown_h);

    // Background
    p.fill_rounded_rect_with_shadow(
        rect,
        4.0,
        Color::WHITE,
        (2.0, 2.0),
        6.0,
        Color::rgba8(0, 0, 0, 40),
    );
    p.stroke_rounded_rect(rect, theme.border, 0.5, 4.0);

    let mut cy = y + 8.0;

    // Title
    p.draw_text(
        &format!("Filter: {}", col_letter(col)),
        x + 10.0,
        cy + 10.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
    cy += 28.0;

    // (Select All) checkbox
    let all_selected = state.filter_values.len() == unique_vals.len()
        || (state.filter_col == Some(col)
            && !state.filter_values.is_empty()
            && state.filter_values.len() >= unique_vals.len());
    let marker = if all_selected { "[x]" } else { "[ ]" };
    p.draw_text(
        &format!("{} (Select All)", marker),
        x + 10.0,
        cy + 10.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
    cy += FILTER_ITEM_H;

    // Individual value checkboxes
    for val in unique_vals.iter().take(8) {
        let checked = if state.filter_col == Some(col) {
            state.filter_values.contains(val)
        } else {
            true // default: all checked
        };
        let marker = if checked { "[x]" } else { "[ ]" };
        let display = if val.is_empty() {
            "(blank)"
        } else {
            val.as_str()
        };
        p.draw_text(
            &format!("{} {}", marker, display),
            x + 10.0,
            cy + 10.0,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        cy += FILTER_ITEM_H;
    }

    // OK / Cancel buttons
    let ok_rect = Rect::new(x + 10.0, cy + 4.0, x + 70.0, cy + 24.0);
    p.fill_rounded_rect(ok_rect, theme.primary, 3.0);
    p.draw_text(
        "OK",
        x + 32.0,
        cy + 17.0,
        Color::WHITE,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );

    let cancel_rect = Rect::new(x + 80.0, cy + 4.0, x + 140.0, cy + 24.0);
    p.fill_rounded_rect(cancel_rect, theme.background, 3.0);
    p.stroke_rounded_rect(cancel_rect, theme.border, 0.5, 3.0);
    p.draw_text(
        "Cancel",
        x + 86.0,
        cy + 17.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
}

/// Hit-test the filter dropdown. Returns the action to take.
pub fn hit_filter_dropdown(
    state: &SheetsState,
    click_x: f64,
    click_y: f64,
) -> Option<FilterDropdownAction> {
    let filter_col = state.filter_dropdown_col?;
    let vis_c = (0..filter_col)
        .filter(|cc| state.is_col_visible(*cc))
        .count();
    let dropdown_x = ROW_HEADER_W + vis_c as f64 * GRID_COL_W;
    let dropdown_y = GRID_HEADER_H; // grid_top relative

    let unique_vals = state.unique_values_for_col(filter_col);
    let item_count = unique_vals.len().min(8);
    let dropdown_h = 36.0 + item_count as f64 * FILTER_ITEM_H + 28.0;

    if click_x < dropdown_x || click_x > dropdown_x + FILTER_DROPDOWN_W {
        return None;
    }
    if click_y < dropdown_y || click_y > dropdown_y + dropdown_h {
        return None;
    }

    let mut cy = dropdown_y + 8.0 + 28.0;

    // (Select All) checkbox
    if click_y >= cy && click_y < cy + FILTER_ITEM_H {
        return Some(FilterDropdownAction::ToggleSelectAll);
    }
    cy += FILTER_ITEM_H;

    // Individual items
    for (idx, _) in unique_vals.iter().enumerate().take(8) {
        if click_y >= cy && click_y < cy + FILTER_ITEM_H {
            return Some(FilterDropdownAction::ToggleItem(idx));
        }
        cy += FILTER_ITEM_H;
    }

    // OK button
    if click_y >= cy + 4.0 && click_y < cy + 24.0 {
        if click_x >= dropdown_x + 10.0 && click_x <= dropdown_x + 70.0 {
            return Some(FilterDropdownAction::Apply);
        }
        if click_x >= dropdown_x + 80.0 && click_x <= dropdown_x + 140.0 {
            return Some(FilterDropdownAction::Cancel);
        }
    }

    None
}

/// Actions from the filter dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterDropdownAction {
    ToggleSelectAll,
    ToggleItem(usize),
    Apply,
    Cancel,
}

/// Hit-test the filter arrow in column headers. Returns the column index if the arrow was clicked.
pub fn hit_filter_arrow(state: &SheetsState, click_x: f64, click_y: f64) -> Option<usize> {
    if !state.filter_active || !state.show_headers {
        return None;
    }
    if click_y > GRID_HEADER_H {
        return None;
    }
    let num_cols = state.grid.first().map(|row| row.len()).unwrap_or(0);
    let mut vis_c: usize = 0;
    for c in 0..num_cols {
        if !state.is_col_visible(c) {
            continue;
        }
        let arrow_x = ROW_HEADER_W + (vis_c + 1) as f64 * GRID_COL_W - 16.0;
        let arrow_rect = Rect::new(arrow_x, 0.0, arrow_x + 16.0, GRID_HEADER_H);
        if arrow_rect.contains((click_x, click_y)) {
            return Some(c);
        }
        vis_c += 1;
    }
    None
}
