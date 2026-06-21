use super::state::SlidesState;
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::painter::GradientDirection;

pub const THUMB_H: f64 = 80.0;
pub const THUMB_W: f64 = 140.0;
pub const THUMB_GAP: f64 = 6.0;

pub fn paint_filmstrip(state: &SlidesState, p: &mut Painter<'_>, theme: &Theme, rect: Rect) {
    p.fill_rect(rect, theme.surface);
    p.draw_text(
        "Slides",
        rect.x0 + 12.0,
        rect.y0 + 18.0,
        theme.on_surface,
        theme.font_size,
        FontWeight::BOLD,
        false,
    );

    let gap = THUMB_GAP;
    let margin = (rect.width() - THUMB_W) / 2.0;
    for (idx, slide) in state.slides.iter().enumerate() {
        let y = rect.y0 + 36.0 + idx as f64 * (THUMB_H + gap);
        if y + THUMB_H > rect.y1 {
            break;
        }
        let x0 = rect.x0 + margin;
        let thumb = Rect::new(x0, y, x0 + THUMB_W, y + THUMB_H);

        // Phase 8.1: draw slide background with gradient support
        let bg = &slide.background;
        if let (Some(start), Some(end)) = (bg.gradient_start, bg.gradient_end) {
            p.fill_rect_linear_gradient(thumb, start, end, GradientDirection::Vertical);
        } else if let Some(bg_color) = bg.color {
            p.fill_rounded_rect(thumb, bg_color, 3.0);
        } else {
            p.fill_rounded_rect(thumb, Color::WHITE, 3.0);
        }

        // draw mini elements
        let sx = THUMB_W / 640.0;
        let sy = THUMB_H / 360.0;
        for el in &slide.elements {
            let ex = x0 + el.x * sx;
            let ey = y + el.y * sy;
            let ew = el.w * sx;
            let eh = el.h * sy;
            if let Some(fill) = el.fill {
                p.fill_rect(Rect::new(ex, ey, ex + ew, ey + eh), fill);
            }
            if let Some(text) = &el.text {
                if !text.is_empty()
                    && el.kind != "table"
                    && el.kind != "chart"
                    && el.kind != "image"
                {
                    p.draw_text(
                        text,
                        ex + 1.0,
                        ey + eh * 0.6,
                        theme.on_surface,
                        5.0,
                        FontWeight::NORMAL,
                        false,
                    );
                }
            }
        }

        // Phase 8.4: multi-select highlight
        let is_multi_selected = state.selected_slides.contains(&idx);
        let current = idx == state.current_slide;
        if current {
            p.stroke_rounded_rect(thumb, theme.primary, 2.0, 3.0);
        } else if is_multi_selected {
            p.stroke_rounded_rect(thumb, theme.secondary, 2.0, 3.0);
        } else {
            p.stroke_rounded_rect(thumb, theme.border, 1.0, 3.0);
        }

        // slide number
        p.draw_text(
            &format!("{}", idx + 1),
            x0 + 3.0,
            y + THUMB_H - 5.0,
            if current || is_multi_selected {
                theme.on_primary
            } else {
                theme.secondary
            },
            7.0,
            FontWeight::NORMAL,
            false,
        );
    }

    // Phase 8.2: drag indicator
    if let Some(drag_idx) = state.filmstrip_drag {
        let y = rect.y0 + 36.0 + drag_idx as f64 * (THUMB_H + gap);
        p.draw_line(
            Point::new(rect.x0 + margin, y),
            Point::new(rect.x0 + margin + THUMB_W, y),
            theme.primary,
            2.0,
        );
    }
}

/// Hit-test: which slide thumbnail is at the given position?
pub fn hit_test_slide(rect: Rect, pos: Point, slide_count: usize) -> Option<usize> {
    let margin = (rect.width() - THUMB_W) / 2.0;
    for idx in 0..slide_count {
        let y = rect.y0 + 36.0 + idx as f64 * (THUMB_H + THUMB_GAP);
        let x0 = rect.x0 + margin;
        let thumb = Rect::new(x0, y, x0 + THUMB_W, y + THUMB_H);
        if thumb.contains(pos) {
            return Some(idx);
        }
    }
    None
}
