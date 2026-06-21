//! Basic equation/math input dialog for Kodocs.
//!
//! Provides a simple modal where users can type common mathematical
//! expressions using a text-based notation. The dialog renders a preview
//! of the formula using Unicode math symbols.

use super::state::{c_accent, c_menu_bg, c_separator, c_text_dim, c_text_light};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

/// State for the equation editor dialog.
#[derive(Debug, Clone, Default)]
pub struct EquationEditorState {
    /// The raw equation input text.
    pub input: String,
    /// Cursor position in the input.
    pub cursor_pos: usize,
    /// Whether the equation has been confirmed.
    pub confirmed: bool,
}

/// Render the equation editor dialog.
pub fn paint_equation_dialog(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    size: Size,
    state: &EquationEditorState,
) {
    let modal_w = 480.0;
    let modal_h = 280.0;
    let modal_x = size.width / 2.0 - modal_w / 2.0;
    let modal_y = size.height / 2.0 - modal_h / 2.0;

    // Semi-transparent backdrop
    let backdrop = Rect::new(0.0, 0.0, size.width, size.height);
    p.fill_rect(backdrop, Color::rgba8(0, 0, 0, 100));

    let modal = Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);
    p.fill_rounded_rect(modal, c_menu_bg(), 8.0);
    p.stroke_rounded_rect(modal, c_separator(), 1.0, 8.0);

    // Title
    p.draw_text_cached(
        cache,
        "수식 입력",
        modal_x + 16.0,
        modal_y + 24.0,
        c_text_light(),
        14.0,
        FontWeight::BOLD,
        false,
        false,
    );

    // Close button
    let close_rect = Rect::new(
        modal_x + modal_w - 28.0,
        modal_y + 6.0,
        modal_x + modal_w - 10.0,
        modal_y + 24.0,
    );
    p.draw_text_cached(
        cache,
        "X",
        close_rect.x0 + close_rect.width() / 2.0,
        close_rect.y0 + close_rect.height() / 2.0,
        c_text_dim(),
        11.0,
        FontWeight::BOLD,
        true,
        false,
    );

    // Input label
    p.draw_text_cached(
        cache,
        "수식:",
        modal_x + 16.0,
        modal_y + 52.0,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );

    // Input field
    let input_rect = Rect::new(
        modal_x + 16.0,
        modal_y + 64.0,
        modal_x + modal_w - 16.0,
        modal_y + 92.0,
    );
    p.fill_rounded_rect(input_rect, Color::rgba8(0x0A, 0x0A, 0x0A, 255), 4.0);
    p.stroke_rounded_rect(input_rect, c_separator(), 1.0, 4.0);

    let display_text = if state.input.is_empty() {
        "수식을 입력하세요...".to_string()
    } else {
        state.input.clone()
    };
    let text_color = if state.input.is_empty() {
        c_text_dim()
    } else {
        c_text_light()
    };
    p.draw_text_cached(
        cache,
        &display_text,
        input_rect.x0 + 8.0,
        input_rect.y0 + input_rect.height() / 2.0,
        text_color,
        13.0,
        FontWeight::NORMAL,
        false,
        false,
    );

    // Preview area
    p.draw_text_cached(
        cache,
        "미리보기:",
        modal_x + 16.0,
        modal_y + 112.0,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );

    let preview_rect = Rect::new(
        modal_x + 16.0,
        modal_y + 124.0,
        modal_x + modal_w - 16.0,
        modal_y + 196.0,
    );
    p.fill_rounded_rect(preview_rect, Color::rgba8(0x0A, 0x0A, 0x0A, 255), 4.0);
    p.stroke_rounded_rect(preview_rect, c_separator(), 1.0, 4.0);

    // Render the preview of the equation
    let preview_text = render_equation_preview(&state.input);
    if preview_text.is_empty() {
        p.draw_text_cached(
            cache,
            "수식 미리보기",
            preview_rect.x0 + 8.0,
            preview_rect.y0 + 20.0,
            c_text_dim(),
            12.0,
            FontWeight::NORMAL,
            false,
            false,
        );
    } else {
        // Render each line of the preview
        let mut preview_y = preview_rect.y0 + 20.0;
        for line in preview_text.lines() {
            if preview_y > preview_rect.y1 - 12.0 {
                break;
            }
            p.draw_text_cached(
                cache,
                line,
                preview_rect.x0 + 12.0,
                preview_y,
                c_text_light(),
                16.0,
                FontWeight::NORMAL,
                false,
                false,
            );
            preview_y += 22.0;
        }
    }

    // Symbol palette (common math symbols)
    let symbols = [
        "+", "-", "\u{00D7}", "\u{00F7}", "=", "\u{2260}", "\u{2264}", "\u{2265}", "\u{221A}",
        "\u{03C0}", "\u{2211}", "\u{222B}", "\u{00B2}", "\u{00B3}", "(", ")", "{", "}", "[", "]",
    ];
    let symbol_y = modal_y + 204.0;
    let symbol_w = 28.0;
    let symbol_h = 24.0;
    let symbol_gap = 4.0;
    let mut symbol_x = modal_x + 16.0;

    for (i, sym) in symbols.iter().enumerate() {
        if i > 0 && i % 10 == 0 {
            symbol_x = modal_x + 16.0;
        }
        let rect = Rect::new(
            symbol_x,
            symbol_y + (i / 10) as f64 * (symbol_h + symbol_gap),
            symbol_x + symbol_w,
            symbol_y + (i / 10) as f64 * (symbol_h + symbol_gap) + symbol_h,
        );
        p.fill_rounded_rect(rect, Color::rgba8(0x2A, 0x2A, 0x2A, 255), 3.0);
        p.stroke_rounded_rect(rect, c_separator(), 1.0, 3.0);
        p.draw_text_cached(
            cache,
            sym,
            rect.x0 + rect.width() / 2.0,
            rect.y0 + rect.height() / 2.0,
            c_text_light(),
            12.0,
            FontWeight::NORMAL,
            true,
            false,
        );
        symbol_x += symbol_w + symbol_gap;
    }

    // OK and Cancel buttons
    let ok_rect = Rect::new(
        modal_x + modal_w - 160.0,
        modal_y + modal_h - 40.0,
        modal_x + modal_w - 90.0,
        modal_y + modal_h - 12.0,
    );
    p.fill_rounded_rect(ok_rect, c_accent(), 4.0);
    p.draw_text_cached(
        cache,
        "삽입",
        ok_rect.x0 + ok_rect.width() / 2.0,
        ok_rect.y0 + ok_rect.height() / 2.0,
        Color::WHITE,
        11.0,
        FontWeight::BOLD,
        true,
        false,
    );

    let cancel_rect = Rect::new(
        modal_x + modal_w - 80.0,
        modal_y + modal_h - 40.0,
        modal_x + modal_w - 12.0,
        modal_y + modal_h - 12.0,
    );
    p.stroke_rounded_rect(cancel_rect, c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        "취소",
        cancel_rect.x0 + cancel_rect.width() / 2.0,
        cancel_rect.y0 + cancel_rect.height() / 2.0,
        c_text_light(),
        11.0,
        FontWeight::NORMAL,
        true,
        false,
    );

    // Keyboard hint
    p.draw_text_cached(
        cache,
        "Enter: 삽입 | Esc: 취소",
        modal_x + 16.0,
        modal_y + modal_h - 20.0,
        c_text_dim(),
        9.0,
        FontWeight::NORMAL,
        false,
        false,
    );
}

/// Convert a simple text-based equation notation into a display string
/// using Unicode math symbols.
pub fn render_equation_preview(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut result = input.to_string();

    // Simple text replacements for common math notation
    result = result.replace("sqrt", "\u{221A}");
    result = result.replace("pi", "\u{03C0}");
    result = result.replace("theta", "\u{03B8}");
    result = result.replace("alpha", "\u{03B1}");
    result = result.replace("beta", "\u{03B2}");
    result = result.replace("gamma", "\u{03B3}");
    result = result.replace("delta", "\u{03B4}");
    result = result.replace("omega", "\u{03C9}");
    result = result.replace("sigma", "\u{03C3}");
    result = result.replace("lambda", "\u{03BB}");
    result = result.replace("mu", "\u{03BC}");
    result = result.replace("inf", "\u{221E}");
    result = result.replace("!=", "\u{2260}");
    result = result.replace("<=", "\u{2264}");
    result = result.replace(">=", "\u{2265}");
    result = result.replace("sum", "\u{2211}");
    result = result.replace("int", "\u{222B}");
    result = result.replace("*", "\u{00D7}");
    result = result.replace("/", "\u{00F7}");
    result = result.replace("^2", "\u{00B2}");
    result = result.replace("^3", "\u{00B3}");

    // Handle fractions: a/b -> a/b (display as-is for now)
    // Handle subscripts: x_1 -> x₁ etc.
    result = replace_subscripts(&result);

    result
}

/// Replace _N patterns with Unicode subscript digits.
fn replace_subscripts(s: &str) -> String {
    let subscripts = [
        '\u{2080}', '\u{2081}', '\u{2082}', '\u{2083}', '\u{2084}', '\u{2085}', '\u{2086}',
        '\u{2087}', '\u{2088}', '\u{2089}',
    ];

    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '_' && i + 1 < chars.len() {
            if let Some(d) = chars[i + 1].to_digit(10) {
                result.push(subscripts[d as usize]);
                i += 2;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}
