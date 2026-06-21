use super::state::{
    c_accent, c_btn_hover, c_menu_bg, c_separator, c_text_dark, c_text_dim, c_text_light,
    c_toolbar_bg, DocsState, ParagraphStyle, ToolbarDropdown, COLOR_PALETTE, FONT_FAMILIES,
    FONT_SIZES,
};
use super::toolbar_actions::{
    font_family_select_x, font_size_select_x, paragraph_style_select_x, BTN_GAP,
    FONT_FAMILY_SELECT_W, FONT_SIZE_SELECT_W, PARAGRAPH_SELECT_W, TOOLBAR_LAYOUT, TOOLBAR_LEFT_PAD,
};
use tench_document_core::Alignment;
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

pub fn paint_toolbar(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect, state: &DocsState) {
    p.fill_rect(rect, c_toolbar_bg());

    let mut x = rect.x0 + TOOLBAR_LEFT_PAD;
    let y = rect.y0 + 8.0;

    // Paint TOOLBAR_LAYOUT items (groups 0, 4-8)
    let mut prev_group = 0;
    let mut first_item = true;
    for item in TOOLBAR_LAYOUT.iter() {
        // Add separator between groups
        if !first_item && item.group != prev_group {
            x = paint_separator(p, rect, x);
            // If jumping from group 0 to group 4, paint the dropdown selects in between
            if prev_group == 0 && item.group >= 4 {
                x = paint_font_size_select(p, cache, x, y, state);
                x = paint_separator(p, rect, x);
                x = paint_font_family_select(p, cache, x, y, state);
                x = paint_separator(p, rect, x);
                x = paint_paragraph_style_select(p, cache, x, y, state);
                x = paint_separator(p, rect, x);
            }
        }
        first_item = false;

        // Determine if this button is active
        let active = is_item_active(item, state);
        let btn_rect = Rect::new(x, y, x + item.width, y + 32.0);
        paint_toolbar_button(p, cache, btn_rect, item.label, active, active, state);
        x += item.width + BTN_GAP;
        prev_group = item.group;
    }

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
}

pub fn paint_toolbar_tooltip(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &DocsState,
) {
    if let Some(tooltip) = &state.hovered_tooltip {
        let tooltip_w = tooltip.len() as f64 * 7.0 + 12.0;
        let tooltip_h = 20.0;
        let tooltip_x = state.hovered_tooltip_x;
        let tooltip_y = rect.y1 + 4.0;
        p.fill_rounded_rect(
            Rect::new(
                tooltip_x,
                tooltip_y,
                tooltip_x + tooltip_w,
                tooltip_y + tooltip_h,
            ),
            c_menu_bg(),
            4.0,
        );
        p.stroke_rounded_rect(
            Rect::new(
                tooltip_x,
                tooltip_y,
                tooltip_x + tooltip_w,
                tooltip_y + tooltip_h,
            ),
            c_separator(),
            1.0,
            4.0,
        );
        p.draw_text_cached(
            cache,
            tooltip,
            tooltip_x + 6.0,
            tooltip_y + tooltip_h / 2.0,
            c_text_light(),
            10.0,
            FontWeight::NORMAL,
            false,
            false,
        );
    }
}

/// Render dropdown popups that appear below the toolbar.
/// This must be called AFTER all other UI elements to ensure correct Z-order.
pub fn paint_toolbar_dropdowns(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    toolbar_rect: Rect,
    state: &DocsState,
    hovered_dropdown_item: Option<(ToolbarDropdown, usize)>,
) {
    let dropdown_top = toolbar_rect.y0 + 8.0 + 32.0;
    match state.open_dropdown {
        Some(ToolbarDropdown::TableGrid) => {
            paint_table_grid_dropdown(p, cache, toolbar_rect.x1, dropdown_top, state);
        }
        Some(ToolbarDropdown::FontFamily) => {
            let ff_x = font_family_select_x();
            paint_font_family_dropdown_with_hover(
                p,
                cache,
                ff_x,
                dropdown_top,
                state.current_font_family.as_str(),
                hovered_dropdown_item.and_then(|(d, i)| {
                    if d == ToolbarDropdown::FontFamily {
                        Some(i)
                    } else {
                        None
                    }
                }),
            );
        }
        Some(ToolbarDropdown::ColorPicker) => {
            paint_color_picker_dropdown(p, cache, toolbar_rect.x1, dropdown_top, state);
        }
        Some(ToolbarDropdown::MarkPicker) => {
            paint_color_picker_dropdown(p, cache, toolbar_rect.x1, dropdown_top, state);
        }
        Some(ToolbarDropdown::FontSize) => {
            let fs_x = font_size_select_x();
            paint_font_size_dropdown_with_hover(
                p,
                cache,
                fs_x,
                dropdown_top,
                hovered_dropdown_item.and_then(|(d, i)| {
                    if d == ToolbarDropdown::FontSize {
                        Some(i)
                    } else {
                        None
                    }
                }),
            );
        }
        Some(ToolbarDropdown::ParagraphStyle) => {
            let ps_x = paragraph_style_select_x();
            paint_paragraph_style_dropdown_with_hover(
                p,
                cache,
                ps_x,
                dropdown_top,
                state.current_paragraph_style,
                hovered_dropdown_item.and_then(|(d, i)| {
                    if d == ToolbarDropdown::ParagraphStyle {
                        Some(i)
                    } else {
                        None
                    }
                }),
            );
        }
        _ => {}
    }
}

/// Determine if a toolbar item should be rendered as active (pressed/toggled).
fn is_item_active(item: &super::toolbar_actions::ToolbarItem, state: &DocsState) -> bool {
    use super::toolbar_actions::ToolbarAction;
    match &item.action {
        ToolbarAction::FormatButton(idx) => match idx {
            0 => state.bold,
            1 => state.italic,
            2 => state.underline,
            3 => state.strikethrough,
            4 => state.code,
            5 => state.superscript,
            6 => state.subscript,
            _ => false,
        },
        ToolbarAction::AlignLeft => state.current_alignment == Alignment::Left,
        ToolbarAction::AlignCenter => state.current_alignment == Alignment::Center,
        ToolbarAction::AlignRight => state.current_alignment == Alignment::Right,
        ToolbarAction::AlignJustify => state.current_alignment == Alignment::Justify,
        _ => false,
    }
}

const DROPDOWN_ITEM_H: f64 = 26.0;

fn paint_separator(p: &mut Painter<'_>, rect: Rect, x: f64) -> f64 {
    p.draw_line(
        Point::new(x + 6.0, rect.y0 + 12.0),
        Point::new(x + 6.0, rect.y1 - 12.0),
        c_separator(),
        1.0,
    );
    x + 14.0
}

fn paint_font_size_select(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    state: &DocsState,
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

    // Render dropdown popup if open
    if state.open_dropdown == Some(ToolbarDropdown::FontSize) {
        paint_font_size_dropdown(p, cache, x, y + 32.0);
    }

    x + FONT_SIZE_SELECT_W + BTN_GAP
}

fn paint_font_family_select(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    state: &DocsState,
) -> f64 {
    let label = if state.current_font_family == "Default" {
        "Font"
    } else {
        &state.current_font_family
    };
    paint_select_box(
        p,
        cache,
        x,
        y,
        label,
        FONT_FAMILY_SELECT_W,
        state.open_dropdown == Some(ToolbarDropdown::FontFamily),
    );

    // Render dropdown popup if open
    if state.open_dropdown == Some(ToolbarDropdown::FontFamily) {
        paint_font_family_dropdown(p, cache, x, y + 32.0, &state.current_font_family);
    }

    x + FONT_FAMILY_SELECT_W + BTN_GAP
}

fn paint_paragraph_style_select(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    state: &DocsState,
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

    // Render dropdown popup if open
    if state.open_dropdown == Some(ToolbarDropdown::ParagraphStyle) {
        paint_paragraph_style_dropdown(p, cache, x, y + 32.0, state.current_paragraph_style);
    }

    x + PARAGRAPH_SELECT_W + BTN_GAP
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

fn paint_font_family_dropdown(
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

fn paint_font_size_dropdown(p: &mut Painter<'_>, cache: &mut TextCache, x: f64, y: f64) {
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

fn paint_paragraph_style_dropdown(
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

fn paint_toolbar_button(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    label: &str,
    highlighted: bool,
    active: bool,
    state: &DocsState,
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

    if label == "Color" {
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
    if label == "Mark" {
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

fn paint_table_grid_dropdown(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    state: &DocsState,
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

fn paint_color_picker_dropdown(
    p: &mut Painter<'_>,
    _cache: &mut TextCache,
    x: f64,
    y: f64,
    state: &DocsState,
) {
    let rows = COLOR_PALETTE.len().div_ceil(COLOR_COLS);
    let grid_w = COLOR_COLS as f64 * COLOR_CELL_SIZE + TABLE_GRID_PAD * 2.0;
    let grid_h = rows as f64 * COLOR_CELL_SIZE + TABLE_GRID_PAD * 2.0;
    // Extra space for "More colors" button
    let more_h = 28.0;
    let total_h = grid_h + more_h;
    let bg = Rect::new(x, y, x + grid_w, y + total_h);
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

    // "More colors..." button
    let more_btn_y = y + grid_h;
    let more_btn = Rect::new(
        x + 4.0,
        more_btn_y + 4.0,
        x + grid_w - 4.0,
        more_btn_y + 4.0 + 20.0,
    );
    p.fill_rounded_rect(more_btn, c_btn_hover(), 4.0);
    p.stroke_rounded_rect(more_btn, c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        _cache,
        "More colors...",
        more_btn.x0 + 8.0,
        more_btn.y0 + more_btn.height() / 2.0,
        c_text_dim(),
        10.0,
        FontWeight::NORMAL,
        false,
        false,
    );

    // Custom hex input field if active
    if !state.custom_color_input.is_empty() {
        let input_y = more_btn_y + 28.0;
        let input_rect = Rect::new(x + 4.0, input_y, x + grid_w - 4.0, input_y + 22.0);
        p.fill_rounded_rect(input_rect, Color::rgb8(0x0A, 0x0A, 0x0A), 4.0);
        p.stroke_rounded_rect(input_rect, c_accent(), 1.0, 4.0);
        p.draw_text_cached(
            _cache,
            &format!("#{}", state.custom_color_input),
            input_rect.x0 + 6.0,
            input_rect.y0 + 12.0,
            c_text_light(),
            11.0,
            FontWeight::NORMAL,
            false,
            false,
        );
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

// ---------------------------------------------------------------------------
// Dropdown with hover highlight
// ---------------------------------------------------------------------------

fn paint_font_size_dropdown_with_hover(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    hovered_idx: Option<usize>,
) {
    let count = FONT_SIZES.len();
    let dropdown_h = count as f64 * DROPDOWN_ITEM_H + 4.0;
    let bg = Rect::new(x, y, x + FONT_SIZE_SELECT_W, y + dropdown_h);
    p.fill_rounded_rect(bg, c_toolbar_bg(), 6.0);
    p.stroke_rounded_rect(bg, c_separator(), 1.0, 6.0);

    for (i, &size) in FONT_SIZES.iter().enumerate() {
        let item_y = y + 2.0 + i as f64 * DROPDOWN_ITEM_H;
        let is_hovered = hovered_idx == Some(i);
        if is_hovered {
            let highlight = Rect::new(
                x + 2.0,
                item_y,
                x + FONT_SIZE_SELECT_W - 2.0,
                item_y + DROPDOWN_ITEM_H,
            );
            p.fill_rounded_rect(highlight, c_btn_hover(), 3.0);
        }
        let label = format!("{:.0}", size);
        p.draw_text_cached(
            cache,
            &label,
            x + 10.0,
            item_y + DROPDOWN_ITEM_H / 2.0,
            if is_hovered {
                c_text_dark()
            } else {
                c_text_light()
            },
            11.0,
            FontWeight::NORMAL,
            false,
            false,
        );
    }
}

fn paint_font_family_dropdown_with_hover(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    current: &str,
    hovered_idx: Option<usize>,
) {
    let count = FONT_FAMILIES.len();
    let dropdown_h = count as f64 * DROPDOWN_ITEM_H + 4.0;
    let bg = Rect::new(x, y, x + FONT_FAMILY_SELECT_W, y + dropdown_h);
    p.fill_rounded_rect(bg, c_toolbar_bg(), 6.0);
    p.stroke_rounded_rect(bg, c_separator(), 1.0, 6.0);

    for (i, &family) in FONT_FAMILIES.iter().enumerate() {
        let item_y = y + 2.0 + i as f64 * DROPDOWN_ITEM_H;
        let is_current = family == current;
        let is_hovered = hovered_idx == Some(i);
        let color = if is_current {
            c_accent()
        } else if is_hovered {
            c_text_dark()
        } else {
            c_text_light()
        };
        let weight = if is_current {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        };

        if is_current || is_hovered {
            let highlight = Rect::new(
                x + 2.0,
                item_y,
                x + FONT_FAMILY_SELECT_W - 2.0,
                item_y + DROPDOWN_ITEM_H,
            );
            p.fill_rounded_rect(
                highlight,
                if is_current {
                    Color::rgba8(0xA7, 0x8B, 0xFA, 40)
                } else {
                    c_btn_hover()
                },
                3.0,
            );
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

fn paint_paragraph_style_dropdown_with_hover(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    current: ParagraphStyle,
    hovered_idx: Option<usize>,
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
        let is_hovered = hovered_idx == Some(i);
        let color = if is_current {
            c_accent()
        } else if is_hovered {
            c_text_dark()
        } else {
            c_text_light()
        };
        let weight = if is_current {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        };

        if is_current || is_hovered {
            let highlight = Rect::new(
                x + 2.0,
                item_y,
                x + PARAGRAPH_SELECT_W - 2.0,
                item_y + DROPDOWN_ITEM_H,
            );
            p.fill_rounded_rect(
                highlight,
                if is_current {
                    Color::rgba8(0xA7, 0x8B, 0xFA, 40)
                } else {
                    c_btn_hover()
                },
                3.0,
            );
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
