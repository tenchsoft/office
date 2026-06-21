use crate::ui::state::{col_letter, HorizontalAlignment, SheetsState};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::painter::TextCache;

use super::cache::{cell_style_hash, BatchBackgrounds, CachedCell, CellRenderCache};
use super::filter;
use super::{GRID_COL_W, GRID_HEADER_H, GRID_ROW_H, ROW_HEADER_W};

const FILL_HANDLE_SIZE: f64 = 6.0;
const CURSOR_BLINK_PERIOD_MS: u64 = 530;

pub fn paint_grid(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    rect: Rect,
    cache: &mut CellRenderCache,
    text_cache: &mut TextCache,
) {
    let col_header_rect = Rect::new(ROW_HEADER_W, rect.y0, rect.x1, rect.y0 + GRID_HEADER_H);
    p.fill_rect(col_header_rect, theme.surface);
    p.fill_rect(
        Rect::new(0.0, rect.y0, ROW_HEADER_W, rect.y0 + GRID_HEADER_H),
        theme.surface,
    );

    let num_cols = state.grid.first().map(|row| row.len()).unwrap_or(0);

    // Column headers
    if state.show_headers {
        let mut vis_c: usize = 0;
        for c in 0..num_cols {
            if !state.is_col_visible(c) {
                continue;
            }
            let x = ROW_HEADER_W + vis_c as f64 * GRID_COL_W + GRID_COL_W / 2.0;
            p.draw_text(
                &col_letter(c),
                x - 4.0,
                rect.y0 + 18.0,
                theme.secondary,
                theme.font_size_small,
                FontWeight::MEDIUM,
                false,
            );
            // Phase 6: Filter dropdown arrow when filter is active
            if state.filter_active {
                let arrow_x = ROW_HEADER_W + (vis_c + 1) as f64 * GRID_COL_W - 16.0;
                let arrow_y = rect.y0 + 12.0;
                let has_filter = state.filter_col == Some(c);
                let arrow_color = if has_filter {
                    theme.primary
                } else {
                    theme.secondary
                };
                // Draw a small downward triangle
                p.draw_text(
                    "\u{25BC}", // ▼
                    arrow_x,
                    arrow_y + 10.0,
                    arrow_color,
                    theme.font_size_small * 0.7,
                    FontWeight::NORMAL,
                    false,
                );
            }
            vis_c += 1;
        }
    }

    let grid_top = rect.y0 + GRID_HEADER_H;
    let zoom = state.zoom_percent as f64 / 100.0;
    let row_h = GRID_ROW_H * zoom;
    let col_w = GRID_COL_W * zoom;

    // Frozen pane divider lines
    if state.freeze_rows > 0 {
        let freeze_y = grid_top + state.freeze_rows as f64 * row_h;
        if freeze_y < rect.y1 {
            p.draw_line(
                Point::new(0.0, freeze_y),
                Point::new(rect.x1, freeze_y),
                theme.primary,
                2.0,
            );
        }
    }
    if state.freeze_cols > 0 {
        let freeze_x = ROW_HEADER_W + state.freeze_cols as f64 * col_w;
        if freeze_x < rect.x1 {
            p.draw_line(
                Point::new(freeze_x, grid_top),
                Point::new(freeze_x, rect.y1),
                theme.primary,
                2.0,
            );
        }
    }

    // Update cache hash — clear stale entries if grid changed.
    cache.update_hash(&state.grid);

    // Phase 1: Collect cell backgrounds into batches.
    let mut backgrounds = BatchBackgrounds::new();

    let mut vis_r: usize = 0;
    for (r, row) in state.grid.iter().enumerate() {
        if !state.is_row_visible(r) {
            continue;
        }
        let y = grid_top + vis_r as f64 * row_h;
        vis_r += 1;
        if y > rect.y1 {
            break;
        }
        if y + row_h < grid_top {
            continue; // Skip rows above the visible area
        }
        if state.show_headers {
            let is_selected = state.is_cell_selected(r, 0);
            let bg = if is_selected {
                theme.primary
            } else if state.is_row_filtered(r) {
                Color::rgb8(0xE0, 0xE0, 0xE0)
            } else {
                theme.surface
            };
            backgrounds.push(bg, Rect::new(0.0, y, ROW_HEADER_W, y + row_h));
        }
        let mut vis_c: usize = 0;
        for (c, cell) in row.iter().enumerate() {
            if !state.is_col_visible(c) {
                continue;
            }
            let x = ROW_HEADER_W + vis_c as f64 * col_w;
            vis_c += 1;
            if x > rect.x1 {
                break;
            }
            if x + col_w < ROW_HEADER_W {
                continue; // Skip columns left of the visible area
            }
            let cell_rect = Rect::new(x, y, x + col_w, y + row_h);
            let is_header = r == 0;
            let is_active = r == state.selected_row && c == state.selected_col;
            let in_range = state.is_cell_selected(r, c) && !is_active;
            let is_match = state.is_search_match(r, c);
            let is_current = state.is_current_match(r, c);

            // Apply conditional formatting colors
            let cond_colors = state.conditional_format_colors(r, c);

            if is_active {
                backgrounds.push(Color::rgb8(0x3B, 0x5B, 0x8A), cell_rect);
            } else if in_range {
                backgrounds.push(Color::rgb8(0xD0, 0xDE, 0xF0), cell_rect);
            } else if is_current {
                backgrounds.push(Color::rgb8(0xFF, 0xA5, 0x00), cell_rect);
            } else if is_match {
                backgrounds.push(Color::rgb8(0xFF, 0xEB, 0x3B), cell_rect);
            } else if is_header {
                backgrounds.push(theme.surface, cell_rect);
            } else if let Some(bg) = cond_colors.1.or(cell.format.bg_color) {
                // Cell format background color or conditional format background
                backgrounds.push(bg, cell_rect);
            }
        }
    }

    // Flush all backgrounds in color-grouped batches.
    backgrounds.flush(p);

    // Phase 2: Grid lines.
    if state.show_grid_lines {
        let mut vis_r: usize = 0;
        for (r, _row) in state.grid.iter().enumerate() {
            if !state.is_row_visible(r) {
                continue;
            }
            let y = grid_top + vis_r as f64 * row_h;
            vis_r += 1;
            if y > rect.y1 {
                break;
            }
            if y + row_h < grid_top {
                continue;
            }
            let mut vis_c: usize = 0;
            for c in 0..num_cols {
                if !state.is_col_visible(c) {
                    continue;
                }
                let x = ROW_HEADER_W + vis_c as f64 * col_w;
                vis_c += 1;
                if x > rect.x1 {
                    break;
                }
                if x + col_w < ROW_HEADER_W {
                    continue;
                }
                p.draw_line(
                    Point::new(x, y),
                    Point::new(x + col_w, y),
                    theme.border,
                    0.5,
                );
            }
        }
    }

    // Phase 3: Text rendering with cache.
    let mut vis_r: usize = 0;
    for (r, row) in state.grid.iter().enumerate() {
        if !state.is_row_visible(r) {
            continue;
        }
        let y = grid_top + vis_r as f64 * row_h;
        vis_r += 1;
        if y > rect.y1 {
            break;
        }
        if y + row_h < grid_top {
            continue;
        }

        // Row header text
        if state.show_headers {
            let is_selected = state.is_cell_selected(r, 0);
            let row_color = if is_selected {
                theme.on_primary
            } else if state.is_row_filtered(r) {
                Color::rgb8(0x99, 0x99, 0x99)
            } else {
                theme.secondary
            };
            p.draw_text(
                &format!("{}", r + 1),
                22.0,
                y + row_h * 0.65,
                row_color,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }

        let mut vis_c: usize = 0;
        for (c, cell) in row.iter().enumerate() {
            if !state.is_col_visible(c) {
                continue;
            }
            let x = ROW_HEADER_W + vis_c as f64 * col_w;
            vis_c += 1;
            if x > rect.x1 {
                break;
            }
            if x + col_w < ROW_HEADER_W {
                continue;
            }

            let is_header = r == 0;
            let is_selected = r == state.selected_row && c == state.selected_col;
            let is_match = state.is_search_match(r, c);
            let is_current = state.is_current_match(r, c);

            let style_hash = cell_style_hash(cell, is_header, is_selected, is_match, is_current);

            // Try to use cached display text
            let display_text = if let Some(cached) = cache.get(r, c) {
                if cached.style_hash == style_hash {
                    // Cache hit — reuse display text
                    &cached.display_text
                } else {
                    // Style changed — recompute
                    let display = cell.formatted_display();
                    let new_cached = CachedCell {
                        display_text: display,
                        text_width: None,
                        style_hash,
                    };
                    cache.insert(r, c, new_cached);
                    cache
                        .get(r, c)
                        .map(|c| c.display_text.as_str())
                        .unwrap_or(&cell.value)
                }
            } else {
                // No cache entry — compute and cache
                let display = cell.formatted_display();
                let new_cached = CachedCell {
                    display_text: display,
                    text_width: None,
                    style_hash,
                };
                cache.insert(r, c, new_cached);
                cache
                    .get(r, c)
                    .map(|c| c.display_text.as_str())
                    .unwrap_or(&cell.value)
            };

            // Apply conditional formatting colors
            let cond_colors = state.conditional_format_colors(r, c);

            // Text color: cell format > conditional > default
            let text_color = if is_selected {
                Color::WHITE
            } else if let Some(tc) = cell.format.text_color.or(cond_colors.0) {
                tc
            } else if cell.is_formula {
                theme.primary
            } else if is_header {
                theme.on_surface
            } else {
                theme.on_background
            };

            // Horizontal alignment
            let text_x = match cell.format.h_align {
                Some(HorizontalAlignment::Center) => x + col_w / 2.0,
                Some(HorizontalAlignment::Right) => x + col_w - 8.0,
                _ => x + 8.0, // Left (default)
            };

            // Font weight: bold if cell format says so, or header
            let font_weight = if cell.format.bold || is_header {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            };

            p.draw_text_cached(
                text_cache,
                display_text,
                text_x,
                y + row_h * 0.65,
                text_color,
                theme.font_size_small,
                font_weight,
                cell.format.italic,
                false,
            );
        }
    }

    // Vertical grid lines
    if state.show_grid_lines {
        let mut vis_c: usize = 0;
        for c in 0..=num_cols {
            // For the last line (num_cols), always draw it if previous cols were visible
            if c < num_cols && !state.is_col_visible(c) {
                continue;
            }
            let x = ROW_HEADER_W + vis_c as f64 * col_w;
            vis_c += 1;
            if x > rect.x1 {
                break;
            }
            if x + col_w < ROW_HEADER_W {
                continue;
            }
            p.draw_line(
                Point::new(x, grid_top),
                Point::new(x, rect.y1),
                theme.border,
                0.5,
            );
        }
    }

    // Fill handle (small square at bottom-right of selected cell)
    {
        let vis_sel_col = (0..=state.selected_col)
            .filter(|c| state.is_col_visible(*c))
            .count()
            .saturating_sub(1);
        let vis_sel_row = (0..=state.selected_row)
            .filter(|r| state.is_row_visible(*r))
            .count()
            .saturating_sub(1);
        let sel_x = ROW_HEADER_W + (vis_sel_col + 1) as f64 * col_w;
        let sel_y = grid_top + (vis_sel_row + 1) as f64 * row_h;
        p.fill_rect(
            Rect::new(
                sel_x - FILL_HANDLE_SIZE,
                sel_y - FILL_HANDLE_SIZE,
                sel_x,
                sel_y,
            ),
            Color::rgb8(0x3B, 0x5B, 0x8A),
        );
    }

    // Phase 1: Formula reference color highlighting during edit
    if state.editing_cell.is_some() && !state.formula_refs.is_empty() {
        for fref in &state.formula_refs {
            let color = state
                .formula_ref_color_for_cell(fref.start_row, fref.start_col)
                .unwrap_or(Color::rgb8(0x42, 0xA5, 0xF5));
            // Highlight the range with a colored border
            for r in fref.start_row..=fref.end_row {
                if !state.is_row_visible(r) {
                    continue;
                }
                let vis_r = (0..r).filter(|rr| state.is_row_visible(*rr)).count();
                for c in fref.start_col..=fref.end_col {
                    if !state.is_col_visible(c) {
                        continue;
                    }
                    let vis_c = (0..c).filter(|cc| state.is_col_visible(*cc)).count();
                    let x = ROW_HEADER_W + vis_c as f64 * col_w;
                    let y = grid_top + vis_r as f64 * row_h;
                    let cell_rect = Rect::new(x, y, x + col_w, y + row_h);
                    p.stroke_rounded_rect(cell_rect, color, 2.5, 0.0);
                }
            }
        }
    }

    // Phase 1: Editing cell cursor and draft text
    if let Some(ref edit) = state.editing_cell {
        if edit.row < state.grid.len() && state.is_row_visible(edit.row) {
            let vis_ec = (0..edit.col).filter(|cc| state.is_col_visible(*cc)).count();
            let vis_er = (0..edit.row).filter(|rr| state.is_row_visible(*rr)).count();
            let edit_x = ROW_HEADER_W + vis_ec as f64 * col_w;
            let edit_y = grid_top + vis_er as f64 * row_h;
            let edit_rect = Rect::new(edit_x, edit_y, edit_x + col_w, edit_y + row_h);

            // Clear the cell background and redraw with edit style
            p.fill_rect(edit_rect, Color::WHITE);
            p.stroke_rounded_rect(edit_rect, Color::rgb8(0x3B, 0x5B, 0x8A), 2.0, 0.0);

            // Draw the draft text (what the user has typed so far)
            let display_text = if edit.draft.is_empty() {
                ""
            } else {
                &edit.draft
            };
            p.draw_text(
                display_text,
                edit_x + 8.0,
                edit_y + row_h * 0.65,
                Color::BLACK,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );

            // Draw cursor (blinking)
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let show_cursor = (now_ms / CURSOR_BLINK_PERIOD_MS).is_multiple_of(2);
            if show_cursor {
                // Approximate cursor x position based on characters before cursor
                let chars_before: String = edit.draft[..edit.cursor_pos].to_string();
                let cursor_x = edit_x + 8.0 + chars_before.len() as f64 * 7.0; // ~7px per char
                p.draw_line(
                    Point::new(cursor_x, edit_y + 4.0),
                    Point::new(cursor_x, edit_y + row_h - 4.0),
                    Color::BLACK,
                    1.5,
                );
            }
        }
    }

    // Phase 1: Autocomplete popup
    if let Some(ref edit) = state.editing_cell {
        if let Some(ref ac) = edit.autocomplete {
            if !ac.candidates.is_empty() {
                let vis_ec = (0..edit.col).filter(|cc| state.is_col_visible(*cc)).count();
                let vis_er = (0..=edit.row)
                    .filter(|rr| state.is_row_visible(*rr))
                    .count();
                let edit_x = ROW_HEADER_W + vis_ec as f64 * col_w;
                let edit_y = grid_top + vis_er as f64 * row_h;
                let popup_w = 180.0;
                let item_h = 22.0;
                let popup_h = 4.0 + ac.candidates.len().min(6) as f64 * item_h;
                let popup_rect = Rect::new(edit_x, edit_y, edit_x + popup_w, edit_y + popup_h);

                p.fill_rounded_rect_with_shadow(
                    popup_rect,
                    4.0,
                    Color::WHITE,
                    (2.0, 2.0),
                    6.0,
                    Color::rgba8(0, 0, 0, 40),
                );
                p.stroke_rounded_rect(popup_rect, Color::rgb8(0xCC, 0xCC, 0xCC), 0.5, 4.0);

                let mut item_y = edit_y + 4.0;
                for (idx, name) in ac.candidates.iter().enumerate().take(6) {
                    if idx == ac.selected_idx {
                        p.fill_rounded_rect(
                            Rect::new(
                                edit_x + 2.0,
                                item_y,
                                edit_x + popup_w - 2.0,
                                item_y + item_h,
                            ),
                            Color::rgb8(0x3B, 0x5B, 0x8A),
                            2.0,
                        );
                        p.draw_text(
                            name,
                            edit_x + 10.0,
                            item_y + item_h * 0.75,
                            Color::WHITE,
                            theme.font_size_small,
                            FontWeight::NORMAL,
                            false,
                        );
                    } else {
                        p.draw_text(
                            name,
                            edit_x + 10.0,
                            item_y + item_h * 0.75,
                            Color::BLACK,
                            theme.font_size_small,
                            FontWeight::NORMAL,
                            false,
                        );
                    }
                    item_y += item_h;
                }
            }
        }
    }

    // Phase 1: Function argument hint bar
    if let Some(hint) = state.current_function_hint() {
        let edit = state.editing_cell.as_ref().unwrap();
        let vis_ec = (0..edit.col).filter(|cc| state.is_col_visible(*cc)).count();
        let vis_er = (0..edit.row).filter(|rr| state.is_row_visible(*rr)).count();
        let edit_x = ROW_HEADER_W + vis_ec as f64 * col_w;
        let edit_y = grid_top + vis_er as f64 * row_h - 20.0;
        let hint_w = hint.len() as f64 * 6.5 + 16.0;
        let hint_rect = Rect::new(edit_x, edit_y, edit_x + hint_w, edit_y + 18.0);
        p.fill_rounded_rect(hint_rect, Color::rgb8(0x33, 0x33, 0x33), 3.0);
        p.draw_text(
            hint,
            edit_x + 8.0,
            edit_y + 13.0,
            Color::WHITE,
            theme.font_size_small * 0.85,
            FontWeight::NORMAL,
            false,
        );
    }

    // Phase 6: Filter dropdown
    if state.show_filter_dropdown {
        if let Some(filter_col) = state.filter_dropdown_col {
            let vis_c = (0..filter_col)
                .filter(|cc| state.is_col_visible(*cc))
                .count();
            let dropdown_x = ROW_HEADER_W + vis_c as f64 * col_w;
            let dropdown_y = grid_top;
            filter::paint_filter_dropdown(p, theme, state, dropdown_x, dropdown_y, filter_col);
        }
    }
}

/// Measure the width of text for column auto-fit using the shared TextCache.
///
/// Returns the advance width of `text` at the given font size and weight.
#[allow(dead_code)] // public API — used by column auto-fit logic
pub fn measure_column_text_width(
    text_cache: &mut TextCache,
    text: &str,
    font_size: f32,
    weight: FontWeight,
) -> f64 {
    text_cache.measure_text_width(text, font_size, weight)
}
