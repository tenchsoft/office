use super::state::{c_accent, c_menu_bg, c_separator, c_text_dim, c_text_light, DocsState};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

pub fn paint_menu_bar(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect, state: &DocsState) {
    p.fill_rect(rect, c_menu_bg());
    let mut x = rect.x0 + 12.0;
    let y = rect.y0 + rect.height() / 2.0;
    for name in ["File", "Edit", "View", "Insert", "Format", "Tools", "Help"] {
        p.draw_text_cached(
            cache,
            name,
            x,
            y,
            c_text_light(),
            12.0,
            FontWeight::NORMAL,
            false,
            false,
        );
        x += match name {
            "Insert" | "Format" => 54.0,
            _ => 42.0,
        };
    }
    // Use real document title instead of hardcoded "Project Proposal"
    p.draw_text_cached(
        cache,
        state.title(),
        rect.x1 - 230.0,
        y,
        c_text_dim(),
        12.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    let pill = Rect::new(rect.x1 - 80.0, rect.y0 + 8.0, rect.x1 - 18.0, rect.y1 - 8.0);
    p.stroke_rounded_rect(pill, c_separator(), 1.0, 999.0);
    // Use real save status instead of hardcoded "Saved"
    let status_label = if state.is_dirty() { "Unsaved" } else { "Saved" };
    p.draw_text_cached(
        cache,
        status_label,
        pill.x0 + 14.0,
        y,
        c_accent(),
        11.0,
        FontWeight::BOLD,
        false,
        false,
    );
    p.draw_line(
        Point::new(rect.x0, rect.y1 - 1.0),
        Point::new(rect.x1, rect.y1 - 1.0),
        c_separator(),
        1.0,
    );
}
