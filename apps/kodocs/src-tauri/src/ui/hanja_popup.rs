// ---------------------------------------------------------------------------
// Hanja conversion popup
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;
use tench_ui::render::TextCache;

use super::state::{self, c_menu_bg, c_separator, c_text_dim, c_text_light};

/// Extract the Korean word at the given byte offset in the text.
pub(super) fn extract_korean_word_at(text: &str, byte_offset: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let char_offset = text[..byte_offset.min(text.len())].chars().count();

    let start = chars[..char_offset]
        .iter()
        .rposition(|c| !is_korean_syllable(*c))
        .map(|p| p + 1)
        .unwrap_or(0);

    let end = chars[char_offset..]
        .iter()
        .position(|c| !is_korean_syllable(*c))
        .map(|p| char_offset + p)
        .unwrap_or(chars.len());

    chars[start..end].iter().collect()
}

fn is_korean_syllable(c: char) -> bool {
    matches!(c, '\u{AC00}'..='\u{D7A3}')
}

/// Paint the Hanja conversion popup.
pub(super) fn paint_hanja_popup(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    size: Size,
    state: &state::HanjaPopupState,
) {
    let modal_w = 300.0;
    let row_h = 28.0;
    let visible = state.candidates.len().min(8);
    let modal_h = 68.0 + visible as f64 * row_h;
    let modal_x = size.width / 2.0 - modal_w / 2.0;
    let modal_y = size.height / 2.0 - modal_h / 2.0;

    let backdrop = Rect::new(0.0, 0.0, size.width, size.height);
    p.fill_rect(backdrop, Color::rgba8(0, 0, 0, 100));

    let modal = Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);
    p.fill_rounded_rect(modal, c_menu_bg(), 8.0);
    p.stroke_rounded_rect(modal, c_separator(), 1.0, 8.0);

    p.draw_text_cached(
        cache,
        "한자 변환",
        modal_x + 16.0,
        modal_y + 22.0,
        c_text_light(),
        14.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );

    p.draw_text_cached(
        cache,
        &format!("원본: {}", state.source_word),
        modal_x + 100.0,
        modal_y + 22.0,
        c_text_dim(),
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    p.draw_line(
        Point::new(modal_x + 12.0, modal_y + 36.0),
        Point::new(modal_x + modal_w - 12.0, modal_y + 36.0),
        c_separator(),
        1.0,
    );

    for (i, candidate) in state.candidates.iter().take(visible).enumerate() {
        let row_y = modal_y + 40.0 + i as f64 * row_h;
        let row_rect = Rect::new(modal_x + 8.0, row_y, modal_x + modal_w - 8.0, row_y + row_h);

        if i == state.selected_idx {
            p.fill_rounded_rect(row_rect, Color::rgba8(0x1A, 0x3A, 0x5C, 255), 4.0);
        }

        p.draw_text_cached(
            cache,
            candidate,
            row_rect.x0 + 12.0,
            row_y + row_h / 2.0,
            c_text_light(),
            13.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );
    }

    p.draw_text_cached(
        cache,
        "Enter: 선택 | Esc: 취소 | ↑↓: 이동",
        modal_x + 16.0,
        modal_y + modal_h - 14.0,
        c_text_dim(),
        9.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );
}
