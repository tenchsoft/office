use super::state::{c_accent, c_ruler_bg, c_separator, c_text_dim, KodocsState, RulerDragTarget};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

/// Half-inch in pixels at 96 DPI.
const HALF_INCH_PX: f64 = 48.0;

pub fn paint_ruler(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect, state: &KodocsState) {
    p.fill_rect(rect, c_ruler_bg());
    p.draw_line(
        Point::new(rect.x0, rect.y1 - 1.0),
        Point::new(rect.x1, rect.y1 - 1.0),
        c_separator(),
        1.0,
    );

    let scale = state.zoom / 100.0;
    let doc = state.current_document();
    let setup = &doc.page_setup;
    let (page_w_raw, _page_h_raw) = setup.page_size_px();
    let page_w = page_w_raw * scale;
    let mm_to_px = 96.0 / 25.4;
    let margin_left_px = setup.margins.left as f64 * mm_to_px * scale;
    let margin_right_px = setup.margins.right as f64 * mm_to_px * scale;

    // Track is the page width area, centered in the ruler
    let track_w = page_w.min((rect.width() - 40.0).max(240.0));
    let track_x = rect.x0 + (rect.width() - track_w) / 2.0;

    // Scale factor from track to actual page
    let track_scale = track_w / page_w;

    // Content area within the track
    let content_left = track_x + margin_left_px * track_scale;
    let content_right = track_x + (page_w - margin_right_px) * track_scale;
    let _content_w = content_right - content_left;

    // Draw tick marks
    let tick_spacing = HALF_INCH_PX * track_scale;
    let mut tick_x = track_x;
    let mut half_inch_idx = 0;
    while tick_x <= track_x + track_w {
        let tick_h = if half_inch_idx % 2 == 0 { 16.0 } else { 8.0 };
        p.draw_line(
            Point::new(tick_x, rect.y1 - tick_h),
            Point::new(tick_x, rect.y1),
            c_text_dim(),
            1.0,
        );
        if half_inch_idx % 2 == 0 {
            let label = format!("{}", half_inch_idx / 2);
            p.draw_text_cached(
                cache,
                &label,
                tick_x,
                rect.y0 + 6.0,
                c_text_dim(),
                9.0,
                FontWeight::NORMAL,
                true,
                false,
            );
        }
        tick_x += tick_spacing;
        half_inch_idx += 1;
    }

    // Highlight content area with a subtle background
    let content_rect = Rect::new(content_left, rect.y0 + 2.0, content_right, rect.y1 - 2.0);
    p.fill_rect(content_rect, Color::rgba8(0x30, 0x30, 0x40, 60));

    // Draw margin markers
    draw_margin_marker(
        p,
        content_left,
        rect,
        state.ruler_drag == Some(RulerDragTarget::LeftMargin),
    );
    draw_margin_marker(
        p,
        content_right,
        rect,
        state.ruler_drag == Some(RulerDragTarget::RightMargin),
    );

    // Draw indent markers for the current paragraph
    if let Some(block) = doc.content.get(state.cursor().block_idx) {
        let (indent_left, indent_right, indent_first) = extract_indents(block);

        // Left indent marker
        let indent_left_x = content_left + indent_left as f64 * track_scale;
        if indent_left > 0.0 {
            draw_indent_marker(
                p,
                indent_left_x,
                rect,
                state.ruler_drag == Some(RulerDragTarget::IndentLeft),
                MarkerShape::TriangleDown,
            );
        }

        // Right indent marker
        let indent_right_x = content_right - indent_right as f64 * track_scale;
        if indent_right > 0.0 {
            draw_indent_marker(
                p,
                indent_right_x,
                rect,
                state.ruler_drag == Some(RulerDragTarget::IndentRight),
                MarkerShape::TriangleDown,
            );
        }

        // First-line indent marker
        if indent_first != 0.0 {
            let first_indent_x = indent_left_x + indent_first as f64 * track_scale;
            draw_indent_marker(
                p,
                first_indent_x,
                rect,
                state.ruler_drag == Some(RulerDragTarget::IndentFirstLine),
                MarkerShape::TriangleUp,
            );
        }
    }
}

/// Draw a margin marker (vertical line with a small triangle at the bottom).
fn draw_margin_marker(p: &mut Painter<'_>, x: f64, rect: Rect, active: bool) {
    let color = if active { c_accent() } else { c_text_dim() };
    // Vertical line
    p.draw_line(
        Point::new(x, rect.y0 + 4.0),
        Point::new(x, rect.y1 - 4.0),
        color,
        1.5,
    );
    // Small triangle at bottom
    let tri_size = 5.0;
    let tri_y = rect.y1 - 4.0;
    p.draw_line(
        Point::new(x - tri_size, tri_y - tri_size),
        Point::new(x, tri_y),
        color,
        1.5,
    );
    p.draw_line(
        Point::new(x + tri_size, tri_y - tri_size),
        Point::new(x, tri_y),
        color,
        1.5,
    );
    p.draw_line(
        Point::new(x - tri_size, tri_y - tri_size),
        Point::new(x + tri_size, tri_y - tri_size),
        color,
        1.5,
    );
}

/// Shape of the indent marker.
enum MarkerShape {
    TriangleUp,
    TriangleDown,
}

/// Draw an indent marker (small triangle).
fn draw_indent_marker(p: &mut Painter<'_>, x: f64, rect: Rect, active: bool, shape: MarkerShape) {
    let color = if active {
        c_accent()
    } else {
        Color::rgb8(0x70, 0x70, 0x80)
    };
    let tri_size = 4.0;
    match shape {
        MarkerShape::TriangleUp => {
            let base_y = rect.y1 - 6.0;
            let tip_y = base_y - tri_size * 2.0;
            draw_triangle_outline(p, x, base_y, tri_size, tip_y, color);
        }
        MarkerShape::TriangleDown => {
            let base_y = rect.y1 - 6.0;
            let tip_y = base_y + tri_size * 2.0;
            draw_triangle_outline(p, x, base_y, tri_size, tip_y, color);
        }
    }
}

/// Draw a small filled triangle using lines.
fn draw_triangle_outline(
    p: &mut Painter<'_>,
    cx: f64,
    base_y: f64,
    half_w: f64,
    tip_y: f64,
    color: Color,
) {
    let left = Point::new(cx - half_w, base_y);
    let right = Point::new(cx + half_w, base_y);
    let tip = Point::new(cx, tip_y);
    p.draw_line(left, tip, color, 1.5);
    p.draw_line(tip, right, color, 1.5);
    p.draw_line(left, right, color, 1.5);
}

/// Extract indent values from a block node.
fn extract_indents(block: &tench_document_core::BlockNode) -> (f32, f32, f32) {
    match block {
        tench_document_core::BlockNode::Paragraph { attrs, .. }
        | tench_document_core::BlockNode::Heading { attrs, .. } => (
            attrs.indent_left,
            attrs.indent_right,
            attrs.indent_first_line,
        ),
        _ => (0.0, 0.0, 0.0),
    }
}

/// Convert a mouse x position on the ruler to a drag target, if any.
/// Returns `(drag_target, value_px)` where value_px is the new value in page pixels.
pub fn ruler_hit_test(x: f64, rect: Rect, state: &KodocsState) -> Option<(RulerDragTarget, f64)> {
    let scale = state.zoom / 100.0;
    let doc = state.current_document();
    let setup = &doc.page_setup;
    let (page_w_raw, _) = setup.page_size_px();
    let page_w = page_w_raw * scale;
    let mm_to_px = 96.0 / 25.4;
    let margin_left_px = setup.margins.left as f64 * mm_to_px * scale;
    let margin_right_px = setup.margins.right as f64 * mm_to_px * scale;

    let track_w = page_w.min((rect.width() - 40.0).max(240.0));
    let track_x = rect.x0 + (rect.width() - track_w) / 2.0;
    let track_scale = track_w / page_w;

    let content_left = track_x + margin_left_px * track_scale;
    let content_right = track_x + (page_w - margin_right_px) * track_scale;

    let hit_radius = 6.0;

    // Check left margin
    if (x - content_left).abs() < hit_radius {
        return Some((RulerDragTarget::LeftMargin, margin_left_px));
    }
    // Check right margin
    if (x - content_right).abs() < hit_radius {
        return Some((RulerDragTarget::RightMargin, margin_right_px));
    }

    // Check indent markers
    if let Some(block) = doc.content.get(state.cursor().block_idx) {
        let (indent_left, indent_right, indent_first) = extract_indents(block);
        let indent_left_x = content_left + indent_left as f64 * track_scale;
        let indent_right_x = content_right - indent_right as f64 * track_scale;

        if indent_left > 0.0 && (x - indent_left_x).abs() < hit_radius {
            return Some((RulerDragTarget::IndentLeft, indent_left as f64));
        }
        if indent_right > 0.0 && (x - indent_right_x).abs() < hit_radius {
            return Some((RulerDragTarget::IndentRight, indent_right as f64));
        }
        if indent_first != 0.0 {
            let first_indent_x = indent_left_x + indent_first as f64 * track_scale;
            if (x - first_indent_x).abs() < hit_radius {
                return Some((RulerDragTarget::IndentFirstLine, indent_first as f64));
            }
        }
    }

    None
}

/// Convert a mouse x position to a margin value in mm during drag.
pub fn ruler_drag_to_margin(x: f64, rect: Rect, state: &KodocsState) -> f32 {
    let scale = state.zoom / 100.0;
    let doc = state.current_document();
    let setup = &doc.page_setup;
    let (page_w_raw, _) = setup.page_size_px();
    let page_w = page_w_raw * scale;

    let track_w = page_w.min((rect.width() - 40.0).max(240.0));
    let track_x = rect.x0 + (rect.width() - track_w) / 2.0;
    let track_scale = track_w / page_w;

    // Convert x position to page pixels, then to mm
    let page_px = (x - track_x) / track_scale;
    let px_to_mm = 25.4 / 96.0;
    (page_px * px_to_mm).max(0.0) as f32
}

/// Convert a mouse x position to an indent value in px during drag.
pub fn ruler_drag_to_indent(x: f64, rect: Rect, state: &KodocsState) -> f32 {
    let scale = state.zoom / 100.0;
    let doc = state.current_document();
    let setup = &doc.page_setup;
    let (page_w_raw, _) = setup.page_size_px();
    let page_w = page_w_raw * scale;
    let mm_to_px = 96.0 / 25.4;
    let margin_left_px = setup.margins.left as f64 * mm_to_px * scale;

    let track_w = page_w.min((rect.width() - 40.0).max(240.0));
    let track_x = rect.x0 + (rect.width() - track_w) / 2.0;
    let track_scale = track_w / page_w;

    let content_left = track_x + margin_left_px * track_scale;

    // Convert x position to indent px (offset from content left)
    let indent_px = (x - content_left) / track_scale;
    (indent_px).max(0.0) as f32
}
