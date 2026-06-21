use super::super::*;

// ---------------------------------------------------------------------------
// Data Validation dialog (Phase 6) painting
// ---------------------------------------------------------------------------

pub(crate) fn paint_data_validation_dialog(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    state: &SheetsState,
) {
    let w = 400.0;
    let h = 340.0;
    let modal = Rect::new(
        size.width / 2.0 - w / 2.0,
        size.height / 2.0 - h / 2.0,
        size.width / 2.0 + w / 2.0,
        size.height / 2.0 + h / 2.0,
    );

    p.fill_rounded_rect(modal, theme.surface, theme.border_radius);
    p.stroke_rounded_rect(modal, theme.border, 1.0, theme.border_radius);

    let x0 = modal.x0 + 16.0;
    let mut y = modal.y0 + 24.0;

    // Title
    p.draw_text(
        "Data Validation",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    y += 28.0;

    // Validation type
    let vtype = state.data_validation_dialog.draft.validation_type;
    p.draw_text(
        "Type:",
        x0,
        y + 10.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    let type_rect = Rect::new(x0 + 60.0, y, modal.x1 - 16.0, y + 22.0);
    p.fill_rounded_rect(type_rect, theme.background, 3.0);
    p.stroke_rounded_rect(type_rect, theme.border, 0.5, 3.0);
    p.draw_text(
        vtype.label(),
        x0 + 68.0,
        y + 10.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    p.draw_text(
        "\u{25BC}",
        modal.x1 - 30.0,
        y + 10.0,
        theme.secondary,
        theme.font_size_small * 0.7,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 30.0;

    // Operator
    let op = state.data_validation_dialog.draft.operator;
    p.draw_text(
        "Operator:",
        x0,
        y + 10.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    let op_rect = Rect::new(x0 + 60.0, y, modal.x1 - 16.0, y + 22.0);
    p.fill_rounded_rect(op_rect, theme.background, 3.0);
    p.stroke_rounded_rect(op_rect, theme.border, 0.5, 3.0);
    p.draw_text(
        op.label(),
        x0 + 68.0,
        y + 10.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    p.draw_text(
        "\u{25BC}",
        modal.x1 - 30.0,
        y + 10.0,
        theme.secondary,
        theme.font_size_small * 0.7,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 30.0;

    // Value1
    p.draw_text(
        "Value 1:",
        x0,
        y + 10.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    let v1_rect = Rect::new(x0 + 60.0, y, modal.x1 - 16.0, y + 22.0);
    p.fill_rounded_rect(v1_rect, Color::WHITE, 3.0);
    p.stroke_rounded_rect(v1_rect, theme.border, 0.5, 3.0);
    if !state.data_validation_dialog.draft.value1.is_empty() {
        p.draw_text(
            &state.data_validation_dialog.draft.value1,
            x0 + 68.0,
            y + 10.0,
            theme.on_surface,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
    } else {
        p.draw_text(
            "Enter value...",
            x0 + 68.0,
            y + 10.0,
            theme.secondary,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
    }
    y += 30.0;

    // Value2
    p.draw_text(
        "Value 2:",
        x0,
        y + 10.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    let v2_rect = Rect::new(x0 + 60.0, y, modal.x1 - 16.0, y + 22.0);
    p.fill_rounded_rect(v2_rect, Color::WHITE, 3.0);
    p.stroke_rounded_rect(v2_rect, theme.border, 0.5, 3.0);
    if !state.data_validation_dialog.draft.value2.is_empty() {
        p.draw_text(
            &state.data_validation_dialog.draft.value2,
            x0 + 68.0,
            y + 10.0,
            theme.on_surface,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
    } else {
        p.draw_text(
            "(optional)",
            x0 + 68.0,
            y + 10.0,
            theme.secondary,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
    }
    y += 30.0;

    // Error message
    p.draw_text(
        "Error msg:",
        x0,
        y + 10.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    let err_rect = Rect::new(x0 + 68.0, y, modal.x1 - 16.0, y + 22.0);
    p.fill_rounded_rect(err_rect, Color::WHITE, 3.0);
    p.stroke_rounded_rect(err_rect, theme.border, 0.5, 3.0);
    if !state.data_validation_dialog.draft.error_message.is_empty() {
        p.draw_text(
            &state.data_validation_dialog.draft.error_message,
            x0 + 76.0,
            y + 10.0,
            theme.on_surface,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
    } else {
        p.draw_text(
            "Invalid input",
            x0 + 76.0,
            y + 10.0,
            theme.secondary,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
    }
    y += 40.0;

    // OK / Cancel buttons
    let ok_rect = Rect::new(x0, y, x0 + 60.0, y + 24.0);
    p.fill_rounded_rect(ok_rect, theme.primary, 3.0);
    p.draw_text(
        "OK",
        x0 + 22.0,
        y + 17.0,
        Color::WHITE,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );

    let cancel_rect = Rect::new(x0 + 72.0, y, x0 + 132.0, y + 24.0);
    p.fill_rounded_rect(cancel_rect, theme.background, 3.0);
    p.stroke_rounded_rect(cancel_rect, theme.border, 0.5, 3.0);
    p.draw_text(
        "Cancel",
        x0 + 78.0,
        y + 17.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
}
