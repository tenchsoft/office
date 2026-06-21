use super::state::SlidesState;
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

pub fn paint_notes(state: &SlidesState, p: &mut Painter<'_>, theme: &Theme, rect: Rect) {
    p.fill_rect(rect, theme.surface);
    p.draw_text(
        "Speaker Notes",
        rect.x0 + 12.0,
        rect.y0 + 18.0,
        theme.on_surface,
        theme.font_size,
        FontWeight::BOLD,
        false,
    );
    let Some(slide) = state.current_slide() else {
        return;
    };
    let mut y = rect.y0 + 36.0;
    for line in slide.notes.lines() {
        p.draw_text(
            line,
            rect.x0 + 12.0,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        y += 16.0;
    }
}
