use super::super::*;

// ---------------------------------------------------------------------------
// Find/Replace dialog
// ---------------------------------------------------------------------------

/// Paint the find/replace dialog.
pub(crate) fn paint_find_replace_dialog(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    state: &SheetsState,
) {
    let w = 380.0;
    let h = 260.0;
    let modal = Rect::new(
        size.width / 2.0 - w / 2.0,
        MENU_H + FORMULA_H + 10.0,
        size.width / 2.0 + w / 2.0,
        MENU_H + FORMULA_H + 10.0 + h,
    );

    // Background
    p.fill_rounded_rect(modal, theme.surface, theme.border_radius);
    p.stroke_rounded_rect(modal, theme.border, 1.0, theme.border_radius);

    let x0 = modal.x0 + 16.0;
    let mut y = modal.y0 + 24.0;

    // Title
    p.draw_text(
        "Find and Replace",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    y += 28.0;

    // Find input
    let find_box = Rect::new(x0, y, modal.x1 - 16.0, y + 22.0);
    p.fill_rounded_rect(find_box, theme.background, 3.0);
    p.stroke_rounded_rect(find_box, theme.border, 0.5, 3.0);
    let find_display = if state.find_replace.find_text.is_empty() {
        "Type to search..."
    } else {
        &state.find_replace.find_text
    };
    p.draw_text(
        find_display,
        x0 + 8.0,
        y + 16.0,
        if state.find_replace.find_text.is_empty() {
            theme.disabled
        } else {
            theme.on_surface
        },
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 28.0;

    // Replace input
    let replace_box = Rect::new(x0, y, modal.x1 - 16.0, y + 22.0);
    p.fill_rounded_rect(replace_box, theme.background, 3.0);
    p.stroke_rounded_rect(replace_box, theme.border, 0.5, 3.0);
    let replace_display = if state.find_replace.replace_text.is_empty() {
        "Replace with..."
    } else {
        &state.find_replace.replace_text
    };
    p.draw_text(
        replace_display,
        x0 + 8.0,
        y + 16.0,
        if state.find_replace.replace_text.is_empty() {
            theme.disabled
        } else {
            theme.on_surface
        },
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 30.0;

    // Options row
    let options_text = format!(
        "{} Case   {} Regex   {} Formulas",
        if state.find_replace.case_sensitive {
            "[x]"
        } else {
            "[ ]"
        },
        if state.find_replace.use_regex {
            "[x]"
        } else {
            "[ ]"
        },
        if state.find_replace.search_in_formulas {
            "[x]"
        } else {
            "[ ]"
        },
    );
    p.draw_text(
        &options_text,
        x0,
        y,
        theme.secondary,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 22.0;

    // Scope
    let scope_text = format!(
        "Scope: {}",
        match state.find_replace.scope {
            state::SearchScope::CurrentSheet => "Current Sheet",
            state::SearchScope::EntireWorkbook => "Entire Workbook",
            state::SearchScope::Selection => "Selection",
        }
    );
    p.draw_text(
        &scope_text,
        x0,
        y,
        theme.secondary,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 24.0;

    // Buttons row
    let buttons = ["Find", "Next", "Prev", "Replace", "Replace All", "Close"];
    let mut bx = x0;
    for btn in &buttons {
        let btn_rect = Rect::new(bx, y, bx + 52.0, y + 22.0);
        p.fill_rounded_rect(btn_rect, theme.primary, 3.0);
        p.draw_text(
            btn,
            bx + 6.0,
            y + 16.0,
            Color::WHITE,
            theme.font_size_small * 0.85,
            tench_ui::parley::FontWeight::BOLD,
            false,
        );
        bx += 58.0;
    }
    y += 28.0;

    // Match count
    let match_info = if state.find_replace.matches.is_empty() {
        "No results".to_string()
    } else {
        let current = state.find_replace.current_match.map(|i| i + 1).unwrap_or(0);
        format!("{} / {}", current, state.find_replace.matches.len())
    };
    p.draw_text(
        &match_info,
        x0,
        y,
        theme.secondary,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
}
