use super::state::{DragMode, SlidesState};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::painter::GradientDirection;

pub fn slide_page_rect(canvas: Rect, zoom: f64, pan_x: f64, pan_y: f64) -> Rect {
    let margin = 30.0;
    let base_w = (canvas.width() - margin * 2.0).min(640.0);
    let base_h = (canvas.height() - margin * 2.0).min(360.0);
    let slide_w = base_w * zoom;
    let slide_h = base_h * zoom;
    let x = canvas.x0 + (canvas.width() - slide_w) / 2.0 + pan_x;
    let y = canvas.y0 + (canvas.height() - slide_h) / 2.0 + pan_y;
    Rect::new(x, y, x + slide_w, y + slide_h)
}

pub fn paint_slide_canvas(state: &SlidesState, p: &mut Painter<'_>, theme: &Theme, rect: Rect) {
    p.fill_rect(rect, Color::rgb8(0x18, 0x18, 0x28));
    let page = slide_page_rect(rect, state.zoom.level, state.zoom.pan_x, state.zoom.pan_y);

    // Phase 9.3: Grid rendering
    if state.show_grid {
        paint_grid(p, page, state.grid_size);
    }
    if let Some(slide) = state.current_slide() {
        let bg = &slide.background;
        if let (Some(start), Some(end)) = (bg.gradient_start, bg.gradient_end) {
            // Phase 7.3: actual gradient rendering
            p.fill_rect_linear_gradient(page, start, end, GradientDirection::Vertical);
        } else if let Some(bg_color) = bg.color {
            p.fill_rounded_rect(page, bg_color, 4.0);
        } else {
            p.fill_rounded_rect(page, Color::WHITE, 4.0);
        }
    } else {
        p.fill_rounded_rect(page, Color::WHITE, 4.0);
    }

    let Some(slide) = state.current_slide() else {
        return;
    };
    let scale_x = page.width() / 640.0;
    let scale_y = page.height() / 360.0;
    for (idx, elem) in slide.elements.iter().enumerate() {
        let elem_rect = Rect::new(
            page.x0 + elem.x * scale_x,
            page.y0 + elem.y * scale_y,
            page.x0 + (elem.x + elem.w) * scale_x,
            page.y0 + (elem.y + elem.h) * scale_y,
        );

        // Phase 0.5: shadow
        if let Some(shadow) = &elem.shadow {
            p.fill_rounded_rect_with_shadow(
                elem_rect,
                2.0,
                elem.fill.unwrap_or(Color::WHITE),
                (shadow.offset_x, shadow.offset_y),
                shadow.blur,
                shadow.color,
            );
        } else if let Some(fill) = elem.fill {
            p.fill_rounded_rect(elem_rect, fill, 2.0);
        }

        // Phase 0.5: border
        if let Some(border) = &elem.border {
            if border.width > 0.0 {
                p.stroke_rounded_rect(elem_rect, border.color, border.width, 2.0);
            }
        }

        // Render text content
        if let Some(text) = &elem.text {
            match elem.kind.as_str() {
                "table" => paint_table_placeholder(p, elem_rect, text, scale_x, scale_y),
                "chart" => paint_chart_placeholder(p, elem_rect, text),
                "image" => paint_image_placeholder(p, elem_rect),
                _ => {
                    let title = elem.kind == "title";
                    let font_size = if title { 20.0 } else { 14.0 };
                    let text_color = if elem.fill.is_some() {
                        Color::WHITE
                    } else {
                        Color::BLACK
                    };
                    p.draw_text(
                        text,
                        elem_rect.x0 + 8.0,
                        elem_rect.y0 + elem_rect.height() / 2.0 + font_size as f64 / 3.0,
                        text_color,
                        font_size,
                        if title {
                            FontWeight::BOLD
                        } else {
                            FontWeight::NORMAL
                        },
                        false,
                    );
                    // Phase 3: Draw text editing cursor
                    if state.text_edit.editing && state.text_edit.element_index == idx {
                        let cursor_pos = state.text_edit.cursor_pos;
                        let text_before = &text[..cursor_pos.min(text.len())];
                        // Approximate cursor x position based on character count
                        let char_width = font_size as f64 * 0.6;
                        let cursor_x =
                            elem_rect.x0 + 8.0 + text_before.chars().count() as f64 * char_width;
                        let cursor_y0 = elem_rect.y0 + elem_rect.height() * 0.2;
                        let cursor_y1 = elem_rect.y0 + elem_rect.height() * 0.8;
                        p.draw_line(
                            Point::new(cursor_x, cursor_y0),
                            Point::new(cursor_x, cursor_y1),
                            text_color,
                            2.0,
                        );
                    }
                }
            }
        }

        // Selection outline for primary selected element
        if state.selected_element == Some(idx) {
            stroke_rect(p, elem_rect, theme.primary);
            // Phase 1.2: draw resize handles
            paint_resize_handles(p, elem_rect, theme);
            // Phase 1.3: draw rotation handle
            paint_rotation_handle(p, elem_rect, theme);
        }

        // Phase 1.4: multi-select outline
        if state.selected_elements.contains(&idx) && state.selected_element != Some(idx) {
            stroke_rect(p, elem_rect, theme.primary);
        }
    }

    // Phase 1.4: draw box selection rectangle
    if state.interaction.mode == DragMode::BoxSelect {
        if let Some(origin) = state.interaction.box_select_origin {
            // We draw a dashed-looking selection box
            let current = state.interaction.start_pos;
            let sel_rect = Rect::from_points(origin, current);
            p.stroke_rounded_rect(sel_rect, Color::rgb8(0x60, 0xA5, 0xFA), 1.0, 2.0);
            p.fill_rounded_rect(sel_rect, Color::rgba8(0x60, 0xA5, 0xFA, 30), 2.0);
        }
    }
}

/// Phase 1.2: Paint 8-direction resize handles around a selected element.
fn paint_resize_handles(p: &mut Painter<'_>, elem_rect: Rect, theme: &Theme) {
    let handle_size = 4.0;
    let handles = [
        (elem_rect.x0, elem_rect.y0),
        (elem_rect.x0 + elem_rect.width() / 2.0, elem_rect.y0),
        (elem_rect.x1, elem_rect.y0),
        (elem_rect.x0, elem_rect.y0 + elem_rect.height() / 2.0),
        (elem_rect.x1, elem_rect.y0 + elem_rect.height() / 2.0),
        (elem_rect.x0, elem_rect.y1),
        (elem_rect.x0 + elem_rect.width() / 2.0, elem_rect.y1),
        (elem_rect.x1, elem_rect.y1),
    ];
    for (hx, hy) in &handles {
        let hr = Rect::new(
            hx - handle_size,
            hy - handle_size,
            hx + handle_size,
            hy + handle_size,
        );
        p.fill_rounded_rect(hr, Color::WHITE, 1.0);
        p.stroke_rounded_rect(hr, theme.primary, 1.5, 1.0);
    }
}

/// Phase 1.3: Paint rotation handle above the selected element.
fn paint_rotation_handle(p: &mut Painter<'_>, elem_rect: Rect, theme: &Theme) {
    let cx = elem_rect.x0 + elem_rect.width() / 2.0;
    let top_y = elem_rect.y0 - 24.0;
    // Line from top of element to handle
    p.draw_line(
        Point::new(cx, elem_rect.y0),
        Point::new(cx, top_y),
        theme.primary,
        1.0,
    );
    // Handle circle
    p.fill_circle(Point::new(cx, top_y), 5.0, theme.primary);
    p.stroke_circle(Point::new(cx, top_y), 5.0, Color::WHITE, 1.5);
}

/// Phase 4: Table placeholder rendering.
fn paint_table_placeholder(
    p: &mut Painter<'_>,
    rect: Rect,
    data: &str,
    scale_x: f64,
    scale_y: f64,
) {
    let table_info: serde_json::Value = serde_json::from_str(data).unwrap_or_default();
    let rows = table_info.get("rows").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
    let cols = table_info.get("cols").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
    if rows == 0 || cols == 0 {
        return;
    }
    let cell_w = rect.width() / cols as f64;
    let cell_h = rect.height() / rows as f64;
    // Draw grid lines
    for r in 0..=rows {
        let y = rect.y0 + r as f64 * cell_h;
        p.draw_line(
            Point::new(rect.x0, y),
            Point::new(rect.x1, y),
            Color::rgb8(0xCC, 0xCC, 0xCC),
            1.0,
        );
    }
    for c in 0..=cols {
        let x = rect.x0 + c as f64 * cell_w;
        p.draw_line(
            Point::new(x, rect.y0),
            Point::new(x, rect.y1),
            Color::rgb8(0xCC, 0xCC, 0xCC),
            1.0,
        );
    }
    // Draw cell text
    if let Some(cells) = table_info.get("cells").and_then(|v| v.as_array()) {
        for cell in cells {
            let r = cell.get("row").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let c = cell.get("col").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let text = cell.get("text").and_then(|v| v.as_str()).unwrap_or("");
            if !text.is_empty() && r < rows && c < cols {
                let cx = rect.x0 + c as f64 * cell_w + 4.0;
                let cy = rect.y0 + r as f64 * cell_h + cell_h / 2.0 + 4.0;
                let _ = (scale_x, scale_y); // used for coordinate mapping
                p.draw_text(text, cx, cy, Color::BLACK, 10.0, FontWeight::NORMAL, false);
            }
        }
    }
}

/// Phase 4.4: Chart rendering — supports bar, line, pie, scatter.
fn paint_chart_placeholder(p: &mut Painter<'_>, rect: Rect, data: &str) {
    let chart_info: serde_json::Value = serde_json::from_str(data).unwrap_or_default();
    let chart_type = chart_info
        .get("chart_type")
        .and_then(|v| v.as_str())
        .unwrap_or("bar");

    let title = chart_info
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Chart");
    p.draw_text(
        title,
        rect.x0 + 8.0,
        rect.y0 + 16.0,
        Color::BLACK,
        12.0,
        FontWeight::BOLD,
        false,
    );

    let chart_area = Rect::new(rect.x0 + 8.0, rect.y0 + 28.0, rect.x1 - 8.0, rect.y1 - 24.0);
    let series = chart_info.get("series").and_then(|v| v.as_array());
    let Some(series) = series else {
        return;
    };

    match chart_type {
        "pie" => paint_pie_chart(p, chart_area, series),
        "line" => paint_line_chart(p, chart_area, series),
        "scatter" => paint_scatter_chart(p, chart_area, series),
        _ => paint_bar_chart(p, chart_area, series),
    }
}

fn paint_bar_chart(p: &mut Painter<'_>, chart_area: Rect, series: &[serde_json::Value]) {
    let bar_count = series
        .first()
        .and_then(|s| s.get("values"))
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(4);
    if bar_count == 0 {
        return;
    }
    let bar_w = chart_area.width() / (bar_count * 2) as f64;
    let max_val = series
        .iter()
        .flat_map(|s| {
            s.get("values")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_f64()).collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .fold(1.0_f64, f64::max);

    for (si, s) in series.iter().enumerate() {
        let values = s
            .get("values")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_f64()).collect::<Vec<_>>())
            .unwrap_or_default();
        let color = if si == 0 {
            Color::rgb8(0x60, 0xA5, 0xFA)
        } else {
            Color::rgb8(0x22, 0xC5, 0x5E)
        };
        for (vi, val) in values.iter().enumerate() {
            let bar_h = (*val / max_val) * chart_area.height() * 0.8;
            let x = chart_area.x0 + (vi * 2 + si) as f64 * bar_w + 2.0;
            let y = chart_area.y1 - bar_h;
            let bar_rect = Rect::new(x, y, x + bar_w - 2.0, chart_area.y1);
            p.fill_rounded_rect(bar_rect, color, 1.0);
        }
    }
}

fn paint_line_chart(p: &mut Painter<'_>, chart_area: Rect, series: &[serde_json::Value]) {
    let colors = [
        Color::rgb8(0x60, 0xA5, 0xFA),
        Color::rgb8(0x22, 0xC5, 0x5E),
        Color::rgb8(0xEF, 0x44, 0x44),
    ];
    let max_val = series
        .iter()
        .flat_map(|s| {
            s.get("values")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_f64()).collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .fold(1.0_f64, f64::max);

    for (si, s) in series.iter().enumerate() {
        let values: Vec<f64> = s
            .get("values")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
            .unwrap_or_default();
        if values.len() < 2 {
            continue;
        }
        let color = colors.get(si).copied().unwrap_or(colors[0]);
        let step_x = chart_area.width() / (values.len() - 1).max(1) as f64;
        let points: Vec<Point> = values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                Point::new(
                    chart_area.x0 + i as f64 * step_x,
                    chart_area.y1 - (v / max_val) * chart_area.height() * 0.8,
                )
            })
            .collect();
        // Draw line segments
        for window in points.windows(2) {
            p.draw_line(window[0], window[1], color, 2.0);
        }
        // Draw dots
        for pt in &points {
            p.fill_circle(*pt, 3.0, color);
        }
    }
}

fn paint_pie_chart(p: &mut Painter<'_>, chart_area: Rect, series: &[serde_json::Value]) {
    let values: Vec<f64> = series
        .first()
        .and_then(|s| s.get("values"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default();
    if values.is_empty() {
        return;
    }
    let total: f64 = values.iter().sum();
    if total <= 0.0 {
        return;
    }
    let colors = [
        Color::rgb8(0x60, 0xA5, 0xFA),
        Color::rgb8(0x22, 0xC5, 0x5E),
        Color::rgb8(0xEF, 0x44, 0x44),
        Color::rgb8(0xF5, 0x9E, 0x0B),
        Color::rgb8(0x8B, 0x5C, 0xF6),
        Color::rgb8(0xEC, 0x48, 0x99),
    ];
    let cx = chart_area.x0 + chart_area.width() / 2.0;
    let cy = chart_area.y0 + chart_area.height() / 2.0;
    let radius = chart_area.width().min(chart_area.height()) / 2.0 - 4.0;

    // Draw pie slices as filled triangles from center
    let mut start_angle = -std::f64::consts::FRAC_PI_2;
    for (i, val) in values.iter().enumerate() {
        let slice_angle = (val / total) * 2.0 * std::f64::consts::PI;
        let end_angle = start_angle + slice_angle;
        let color = colors.get(i % colors.len()).copied().unwrap_or(colors[0]);

        // Draw slice as a filled triangle (approximation for pie wedge)
        let steps = 8;
        for s in 0..steps {
            let a1 = start_angle + (slice_angle * s as f64 / steps as f64);
            let a2 = start_angle + (slice_angle * (s + 1) as f64 / steps as f64);
            let p1 = Point::new(cx + radius * a1.cos(), cy + radius * a1.sin());
            let p2 = Point::new(cx + radius * a2.cos(), cy + radius * a2.sin());
            p.fill_triangle(Point::new(cx, cy), p1, p2, color);
        }
        start_angle = end_angle;
    }
    // Draw outline circle
    p.stroke_circle(Point::new(cx, cy), radius, Color::BLACK, 1.0);
}

fn paint_scatter_chart(p: &mut Painter<'_>, chart_area: Rect, series: &[serde_json::Value]) {
    let colors = [Color::rgb8(0x60, 0xA5, 0xFA), Color::rgb8(0x22, 0xC5, 0x5E)];
    let max_val = series
        .iter()
        .flat_map(|s| {
            s.get("values")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_f64()).collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .fold(1.0_f64, f64::max);

    for (si, s) in series.iter().enumerate() {
        let values: Vec<f64> = s
            .get("values")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
            .unwrap_or_default();
        let color = colors.get(si).copied().unwrap_or(colors[0]);
        let count = values.len().max(1);
        for (vi, val) in values.iter().enumerate() {
            let x = chart_area.x0 + (vi as f64 / count as f64) * chart_area.width();
            let y = chart_area.y1 - (val / max_val) * chart_area.height() * 0.8;
            p.fill_circle(Point::new(x, y), 4.0, color);
        }
    }
}

/// Phase 3.4: Image placeholder rendering.
fn paint_image_placeholder(p: &mut Painter<'_>, rect: Rect) {
    p.stroke_rounded_rect(rect, Color::rgb8(0xCC, 0xCC, 0xCC), 1.0, 2.0);
    // Draw a simple image icon
    let cx = rect.x0 + rect.width() / 2.0;
    let cy = rect.y0 + rect.height() / 2.0;
    p.draw_text(
        "[Image]",
        cx,
        cy + 4.0,
        Color::rgb8(0x99, 0x99, 0x99),
        12.0,
        FontWeight::NORMAL,
        true,
    );
}

/// Phase 9.3: Paint grid lines on the slide.
fn paint_grid(p: &mut Painter<'_>, page: Rect, grid_size: f64) {
    let grid_color = Color::rgba8(0x80, 0x80, 0x80, 40);
    let step_x = page.width() / 640.0 * grid_size;
    let step_y = page.height() / 360.0 * grid_size;
    let mut x = page.x0 + step_x;
    while x < page.x1 {
        p.draw_line(
            Point::new(x, page.y0),
            Point::new(x, page.y1),
            grid_color,
            0.5,
        );
        x += step_x;
    }
    let mut y = page.y0 + step_y;
    while y < page.y1 {
        p.draw_line(
            Point::new(page.x0, y),
            Point::new(page.x1, y),
            grid_color,
            0.5,
        );
        y += step_y;
    }
}

fn stroke_rect(p: &mut Painter<'_>, rect: Rect, color: Color) {
    p.draw_line(
        Point::new(rect.x0, rect.y0),
        Point::new(rect.x1, rect.y0),
        color,
        2.0,
    );
    p.draw_line(
        Point::new(rect.x1, rect.y0),
        Point::new(rect.x1, rect.y1),
        color,
        2.0,
    );
    p.draw_line(
        Point::new(rect.x1, rect.y1),
        Point::new(rect.x0, rect.y1),
        color,
        2.0,
    );
    p.draw_line(
        Point::new(rect.x0, rect.y1),
        Point::new(rect.x0, rect.y0),
        color,
        2.0,
    );
}
