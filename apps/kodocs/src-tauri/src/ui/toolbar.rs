use super::state::{
    c_accent, c_btn_hover, c_separator, c_text_dark, c_text_dim, c_text_light, c_toolbar_bg,
    KodocsState, ParagraphStyle, ToolbarDropdown, COLOR_PALETTE, FONT_FAMILIES, FONT_SIZES,
};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

/// Render the toolbar buttons and return the last x position for dropdown placement.
pub fn paint_toolbar(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &KodocsState,
) -> f64 {
    p.fill_rect(rect, c_toolbar_bg());

    let mut x = rect.x0 + 12.0;
    let y = rect.y0 + 8.0;

    x = paint_button_group(
        p,
        cache,
        x,
        y,
        state,
        &[("실행취소", 48.0, false), ("다시실행", 48.0, false)],
    );
    x = paint_separator(p, rect, x);
    x = paint_font_family_select(p, cache, x, y, state);
    x = paint_separator(p, rect, x);
    x = paint_font_size_select(p, cache, x, y, state);
    x = paint_separator(p, rect, x);
    x = paint_paragraph_style_select(p, cache, x, y, state);
    x = paint_separator(p, rect, x);

    x = paint_button_group(
        p,
        cache,
        x,
        y,
        state,
        &[
            ("B", 32.0, state.bold),
            ("I", 32.0, state.italic),
            ("U", 32.0, state.underline),
            ("S", 32.0, state.strikethrough),
            ("<>", 32.0, state.code),
            ("x²", 32.0, state.superscript),
            ("x_", 32.0, state.subscript),
            ("강조", 36.0, false),
        ],
    );
    x = paint_separator(p, rect, x);

    x = paint_button_group(
        p,
        cache,
        x,
        y,
        state,
        &[
            ("•", 28.0, false),
            ("1.", 28.0, false),
            ("☑", 28.0, false),
            ("◀", 32.0, false),
            ("▶", 32.0, false),
        ],
    );
    x = paint_separator(p, rect, x);

    x = paint_button_group(
        p,
        cache,
        x,
        y,
        state,
        &[
            ("좌", 28.0, false),
            ("중", 28.0, false),
            ("우", 28.0, false),
            ("양", 28.0, false),
        ],
    );
    x = paint_separator(p, rect, x);

    x = paint_button_group(
        p,
        cache,
        x,
        y,
        state,
        &[
            ("링크", 42.0, false),
            ("그림", 38.0, false),
            ("표", 28.0, false),
            ("줄", 28.0, false),
            ("인용", 36.0, false),
        ],
    );
    x = paint_separator(p, rect, x);
    x = paint_button_group(
        p,
        cache,
        x,
        y,
        state,
        &[("글자색", 50.0, false), ("배경색", 50.0, false)],
    );

    if x < rect.x1 - 36.0 {
        let more = Rect::new(rect.x1 - 40.0, y + 2.0, rect.x1 - 12.0, y + 30.0);
        paint_toolbar_button(p, cache, more, "...", false, false, state);
    }

    p.draw_line(
        Point::new(rect.x0, rect.y1 - 1.0),
        Point::new(rect.x1, rect.y1 - 1.0),
        c_separator(),
        1.0,
    );

    x
}

/// Render toolbar dropdown popups. Called after all other elements for correct Z-order.
pub fn paint_toolbar_dropdowns(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    toolbar_rect: Rect,
    dropdown_x: f64,
    state: &KodocsState,
) {
    let dropdown_top = toolbar_rect.y0 + 8.0 + 32.0;
    match state.open_dropdown {
        Some(ToolbarDropdown::TableGrid) => {
            paint_table_grid_dropdown(p, cache, dropdown_x, dropdown_top, state);
        }
        Some(ToolbarDropdown::ColorPicker) => {
            paint_color_picker_dropdown(p, cache, dropdown_x, dropdown_top);
        }
        Some(ToolbarDropdown::MarkPicker) => {
            paint_color_picker_dropdown(p, cache, dropdown_x, dropdown_top);
        }
        Some(ToolbarDropdown::FontFamily) => {
            paint_font_family_dropdown(
                p,
                cache,
                dropdown_x,
                dropdown_top,
                &state.current_font_family,
            );
        }
        Some(ToolbarDropdown::FontSize) => {
            paint_font_size_dropdown(p, cache, dropdown_x, dropdown_top);
        }
        Some(ToolbarDropdown::ParagraphStyle) => {
            paint_paragraph_style_dropdown(
                p,
                cache,
                dropdown_x,
                dropdown_top,
                state.current_paragraph_style,
            );
        }
        _ => {}
    }
}

fn paint_button_group(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    mut x: f64,
    y: f64,
    state: &KodocsState,
    buttons: &[(&str, f64, bool)],
) -> f64 {
    for (idx, (label, width, active)) in buttons.iter().enumerate() {
        let rect = Rect::new(x, y, x + *width, y + 32.0);
        paint_toolbar_button(
            p,
            cache,
            rect,
            label,
            *active || state.hovered_btn == Some(idx),
            *active,
            state,
        );
        x += *width + 2.0;
    }
    x
}

/// Font family dropdown width.
const FONT_FAMILY_SELECT_W: f64 = 120.0;
/// Font size dropdown position constants.
const FONT_SIZE_SELECT_W: f64 = 62.0;
const PARAGRAPH_SELECT_W: f64 = 112.0;
const DROPDOWN_ITEM_H: f64 = 26.0;

fn paint_font_family_select(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    state: &KodocsState,
) -> f64 {
    paint_select_box(
        p,
        cache,
        x,
        y,
        &state.current_font_family,
        FONT_FAMILY_SELECT_W,
        state.open_dropdown == Some(ToolbarDropdown::FontFamily),
    );

    // Dropdown rendering is handled by paint_toolbar_dropdowns() for correct Z-order

    x + FONT_FAMILY_SELECT_W + 2.0
}

pub(super) fn paint_font_family_dropdown(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    current: &str,
) {
    let count = FONT_FAMILIES.len();
    let dropdown_h = count as f64 * DROPDOWN_ITEM_H + 4.0;
    let bg = Rect::new(x, y, x + FONT_FAMILY_SELECT_W, y + dropdown_h);
    p.fill_rounded_rect(bg, c_toolbar_bg(), 6.0);
    p.stroke_rounded_rect(bg, c_separator(), 1.0, 6.0);

    for (i, &family) in FONT_FAMILIES.iter().enumerate() {
        let item_y = y + 2.0 + i as f64 * DROPDOWN_ITEM_H;
        let is_current = family == current;
        let color = if is_current {
            c_accent()
        } else {
            c_text_light()
        };
        let weight = if is_current {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        };

        if is_current {
            let highlight = Rect::new(
                x + 2.0,
                item_y,
                x + FONT_FAMILY_SELECT_W - 2.0,
                item_y + DROPDOWN_ITEM_H,
            );
            p.fill_rounded_rect(highlight, c_btn_hover(), 3.0);
        }

        p.draw_text_cached(
            cache,
            family,
            x + 10.0,
            item_y + DROPDOWN_ITEM_H / 2.0,
            color,
            11.0,
            weight,
            false,
            false,
        );
    }
}

fn paint_font_size_select(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    state: &KodocsState,
) -> f64 {
    let label = format!("{:.0}px", state.current_font_size);
    paint_select_box(
        p,
        cache,
        x,
        y,
        &label,
        FONT_SIZE_SELECT_W,
        state.open_dropdown == Some(ToolbarDropdown::FontSize),
    );

    // Dropdown rendering is handled by paint_toolbar_dropdowns() for correct Z-order

    x + FONT_SIZE_SELECT_W + 2.0
}

fn paint_paragraph_style_select(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    state: &KodocsState,
) -> f64 {
    let label = state.current_paragraph_style.label();
    paint_select_box(
        p,
        cache,
        x,
        y,
        label,
        PARAGRAPH_SELECT_W,
        state.open_dropdown == Some(ToolbarDropdown::ParagraphStyle),
    );

    // Dropdown rendering is handled by paint_toolbar_dropdowns() for correct Z-order

    x + PARAGRAPH_SELECT_W + 2.0
}

fn paint_select_box(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    label: &str,
    width: f64,
    is_open: bool,
) {
    let rect = Rect::new(x, y, x + width, y + 32.0);
    let bg = if is_open {
        c_btn_hover()
    } else {
        c_toolbar_bg()
    };
    p.fill_rounded_rect(rect, bg, 6.0);
    p.stroke_rounded_rect(rect, c_separator(), 1.0, 6.0);
    p.draw_text_cached(
        cache,
        label,
        rect.x0 + 8.0,
        rect.y0 + rect.height() / 2.0,
        c_text_light(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    let arrow = if is_open { "^" } else { "v" };
    p.draw_text_cached(
        cache,
        arrow,
        rect.x1 - 14.0,
        rect.y0 + rect.height() / 2.0,
        c_text_dim(),
        10.0,
        FontWeight::NORMAL,
        true,
        false,
    );
}

pub(super) fn paint_font_size_dropdown(p: &mut Painter<'_>, cache: &mut TextCache, x: f64, y: f64) {
    let count = FONT_SIZES.len();
    let dropdown_h = count as f64 * DROPDOWN_ITEM_H + 4.0;
    let bg = Rect::new(x, y, x + FONT_SIZE_SELECT_W, y + dropdown_h);
    p.fill_rounded_rect(bg, c_toolbar_bg(), 6.0);
    p.stroke_rounded_rect(bg, c_separator(), 1.0, 6.0);

    for (i, &size) in FONT_SIZES.iter().enumerate() {
        let item_y = y + 2.0 + i as f64 * DROPDOWN_ITEM_H;
        let label = format!("{:.0}", size);
        p.draw_text_cached(
            cache,
            &label,
            x + 10.0,
            item_y + DROPDOWN_ITEM_H / 2.0,
            c_text_light(),
            11.0,
            FontWeight::NORMAL,
            false,
            false,
        );
    }
}

pub(super) fn paint_paragraph_style_dropdown(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    current: ParagraphStyle,
) {
    let styles = ParagraphStyle::all();
    let count = styles.len();
    let dropdown_h = count as f64 * DROPDOWN_ITEM_H + 4.0;
    let bg = Rect::new(x, y, x + PARAGRAPH_SELECT_W, y + dropdown_h);
    p.fill_rounded_rect(bg, c_toolbar_bg(), 6.0);
    p.stroke_rounded_rect(bg, c_separator(), 1.0, 6.0);

    for (i, &style) in styles.iter().enumerate() {
        let item_y = y + 2.0 + i as f64 * DROPDOWN_ITEM_H;
        let is_current = style == current;
        let color = if is_current {
            c_accent()
        } else {
            c_text_light()
        };
        let weight = if is_current {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        };

        if is_current {
            let highlight = Rect::new(
                x + 2.0,
                item_y,
                x + PARAGRAPH_SELECT_W - 2.0,
                item_y + DROPDOWN_ITEM_H,
            );
            p.fill_rounded_rect(highlight, c_btn_hover(), 3.0);
        }

        p.draw_text_cached(
            cache,
            style.label(),
            x + 10.0,
            item_y + DROPDOWN_ITEM_H / 2.0,
            color,
            11.0,
            weight,
            false,
            false,
        );
    }
}

fn paint_separator(p: &mut Painter<'_>, rect: Rect, x: f64) -> f64 {
    p.draw_line(
        Point::new(x + 6.0, rect.y0 + 12.0),
        Point::new(x + 6.0, rect.y1 - 12.0),
        c_separator(),
        1.0,
    );
    x + 14.0
}

fn paint_toolbar_button(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    label: &str,
    highlighted: bool,
    active: bool,
    state: &KodocsState,
) {
    if highlighted || active {
        p.fill_rounded_rect(rect, c_btn_hover(), 6.0);
    }
    if active {
        p.stroke_rounded_rect(rect, c_accent(), 1.0, 6.0);
    }

    p.draw_text_cached(
        cache,
        label,
        rect.x0 + rect.width() / 2.0,
        rect.y0 + rect.height() / 2.0,
        if active { c_accent() } else { c_text_light() },
        if label.len() > 3 { 10.0 } else { 11.0 },
        FontWeight::BOLD,
        true,
        false,
    );

    if label == "글자색" {
        // Show current text color indicator bar
        let bar_color = state
            .selected_text_color
            .as_deref()
            .and_then(parse_color)
            .unwrap_or(c_text_dark());
        p.fill_rect(
            Rect::new(rect.x0 + 11.0, rect.y1 - 7.0, rect.x1 - 11.0, rect.y1 - 4.0),
            bar_color,
        );
    }
    if label == "배경색" {
        // Show current highlight color indicator bar
        let bar_color = state
            .selected_bg_color
            .as_deref()
            .and_then(parse_color)
            .unwrap_or(c_accent());
        p.fill_rect(
            Rect::new(rect.x0 + 10.0, rect.y1 - 7.0, rect.x1 - 10.0, rect.y1 - 4.0),
            bar_color,
        );
    }
}

// ---------------------------------------------------------------------------
// Table grid picker dropdown
// ---------------------------------------------------------------------------

const TABLE_GRID_SIZE: usize = 5;
const TABLE_GRID_CELL: f64 = 22.0;
const TABLE_GRID_PAD: f64 = 4.0;

pub(super) fn paint_table_grid_dropdown(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    state: &KodocsState,
) {
    let grid_w = TABLE_GRID_SIZE as f64 * TABLE_GRID_CELL + TABLE_GRID_PAD * 2.0;
    let grid_h = grid_w;
    let bg = Rect::new(x, y, x + grid_w, y + grid_h);
    p.fill_rounded_rect(bg, c_toolbar_bg(), 6.0);
    p.stroke_rounded_rect(bg, c_separator(), 1.0, 6.0);

    // Label at top
    let label = format!(
        "{}x{}",
        state.table_grid.hover_row.max(1),
        state.table_grid.hover_col.max(1)
    );
    p.draw_text_cached(
        cache,
        &label,
        x + grid_w / 2.0,
        y + grid_h + 14.0,
        c_text_light(),
        11.0,
        FontWeight::NORMAL,
        true,
        false,
    );

    for row in 0..TABLE_GRID_SIZE {
        for col in 0..TABLE_GRID_SIZE {
            let cell_x = x + TABLE_GRID_PAD + col as f64 * TABLE_GRID_CELL;
            let cell_y = y + TABLE_GRID_PAD + row as f64 * TABLE_GRID_CELL;
            let cell = Rect::new(
                cell_x + 1.0,
                cell_y + 1.0,
                cell_x + TABLE_GRID_CELL - 1.0,
                cell_y + TABLE_GRID_CELL - 1.0,
            );

            let is_hovered = row < state.table_grid.hover_row && col < state.table_grid.hover_col;
            if is_hovered {
                p.fill_rounded_rect(cell, c_accent(), 2.0);
            } else {
                p.fill_rounded_rect(cell, c_btn_hover(), 2.0);
                p.stroke_rounded_rect(cell, c_separator(), 1.0, 2.0);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Color picker dropdown
// ---------------------------------------------------------------------------

const COLOR_CELL_SIZE: f64 = 20.0;
const COLOR_COLS: usize = 10;

pub(super) fn paint_color_picker_dropdown(
    p: &mut Painter<'_>,
    _cache: &mut TextCache,
    x: f64,
    y: f64,
) {
    let rows = COLOR_PALETTE.len().div_ceil(COLOR_COLS);
    let grid_w = COLOR_COLS as f64 * COLOR_CELL_SIZE + TABLE_GRID_PAD * 2.0;
    let grid_h = rows as f64 * COLOR_CELL_SIZE + TABLE_GRID_PAD * 2.0;
    let bg = Rect::new(x, y, x + grid_w, y + grid_h);
    p.fill_rounded_rect(bg, c_toolbar_bg(), 6.0);
    p.stroke_rounded_rect(bg, c_separator(), 1.0, 6.0);

    for (i, color_str) in COLOR_PALETTE.iter().enumerate() {
        let row = i / COLOR_COLS;
        let col = i % COLOR_COLS;
        let cell_x = x + TABLE_GRID_PAD + col as f64 * COLOR_CELL_SIZE;
        let cell_y = y + TABLE_GRID_PAD + row as f64 * COLOR_CELL_SIZE;
        let cell = Rect::new(
            cell_x + 1.0,
            cell_y + 1.0,
            cell_x + COLOR_CELL_SIZE - 1.0,
            cell_y + COLOR_CELL_SIZE - 1.0,
        );

        let color = parse_color(color_str).unwrap_or(c_text_dark());
        p.fill_rounded_rect(cell, color, 2.0);
        p.stroke_rounded_rect(cell, c_separator(), 1.0, 2.0);
    }
}

/// Parse a hex color string like "#FF0000" into a Color.
fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    let hex = s.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::rgb8(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color::rgba8(r, g, b, a))
        }
        _ => None,
    }
}
