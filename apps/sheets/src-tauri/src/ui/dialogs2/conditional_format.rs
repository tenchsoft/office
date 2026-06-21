use super::super::*;

// ---------------------------------------------------------------------------
// Conditional Format dialog (Phase 5) painting
// ---------------------------------------------------------------------------

/// Paint the Conditional Format dialog (Phase 5).
pub(crate) fn paint_cond_format_dialog(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    state: &SheetsState,
) {
    let w = 380.0;
    let h = 300.0;
    let modal = Rect::new(
        size.width / 2.0 - w / 2.0,
        size.height / 2.0 - h / 2.0,
        size.width / 2.0 + w / 2.0,
        size.height / 2.0 + h / 2.0,
    );

    p.fill_rounded_rect(modal, theme.background, 6.0);
    p.stroke_rounded_rect(modal, theme.border, 1.0, 6.0);

    p.draw_text(
        "Conditional Formatting",
        modal.x0 + 16.0,
        modal.y0 + 22.0,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );

    let x0 = modal.x0 + 16.0;
    let y0 = modal.y0 + 40.0;

    p.draw_text(
        "Condition:",
        x0,
        y0 + 14.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );

    let ops = ["Greater Than", "Less Than", "Equal To", "Between"];
    let current_op = match state.conditional_format_dialog.condition {
        ConditionOp::GreaterThan => 0,
        ConditionOp::LessThan => 1,
        ConditionOp::EqualTo => 2,
        ConditionOp::Between => 3,
    };
    for (i, label) in ops.iter().enumerate() {
        let row_y = y0 + i as f64 * 26.0;
        let item_rect = Rect::new(x0, row_y, x0 + 140.0, row_y + 22.0);
        let bg = if i == current_op {
            theme.primary
        } else {
            theme.background
        };
        let fg = if i == current_op {
            Color::WHITE
        } else {
            theme.on_surface
        };
        p.fill_rounded_rect(item_rect, bg, 3.0);
        p.stroke_rounded_rect(item_rect, theme.border, 0.5, 3.0);
        p.draw_text(
            label,
            x0 + 8.0,
            row_y + 16.0,
            fg,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
    }

    let val_y = y0 + 110.0;
    p.draw_text(
        "Value:",
        x0,
        val_y + 14.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    let val_rect = Rect::new(x0 + 50.0, val_y, x0 + 200.0, val_y + 22.0);
    p.fill_rounded_rect(val_rect, Color::WHITE, 3.0);
    p.stroke_rounded_rect(val_rect, theme.border, 0.5, 3.0);
    let val_display = if state.conditional_format_dialog.value_text.is_empty() {
        "0"
    } else {
        &state.conditional_format_dialog.value_text
    };
    p.draw_text(
        val_display,
        x0 + 56.0,
        val_y + 16.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );

    let color_y = val_y + 30.0;
    p.draw_text(
        "Highlight:",
        x0,
        color_y + 14.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );

    let colors = [
        ("Red BG", Color::rgb8(255, 200, 200)),
        ("Green BG", Color::rgb8(200, 255, 200)),
        ("Blue BG", Color::rgb8(200, 200, 255)),
        ("Yellow BG", Color::rgb8(255, 255, 200)),
    ];
    for (i, (label, color)) in colors.iter().enumerate() {
        let row_y = color_y + 20.0 + i as f64 * 26.0;
        let item_rect = Rect::new(x0, row_y, x0 + 100.0, row_y + 22.0);
        p.fill_rounded_rect(item_rect, theme.background, 3.0);
        p.stroke_rounded_rect(item_rect, theme.border, 0.5, 3.0);
        p.fill_rounded_rect(
            Rect::new(x0 + 4.0, row_y + 4.0, x0 + 20.0, row_y + 18.0),
            *color,
            2.0,
        );
        p.draw_text(
            label,
            x0 + 26.0,
            row_y + 16.0,
            theme.on_surface,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
    }

    let btn_y = modal.y1 - 44.0;
    let ok_rect = Rect::new(x0, btn_y, x0 + 60.0, btn_y + 24.0);
    p.fill_rounded_rect(ok_rect, theme.primary, 3.0);
    p.draw_text(
        "OK",
        x0 + 22.0,
        btn_y + 17.0,
        Color::WHITE,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );

    let cancel_rect = Rect::new(x0 + 72.0, btn_y, x0 + 132.0, btn_y + 24.0);
    p.fill_rounded_rect(cancel_rect, theme.background, 3.0);
    p.stroke_rounded_rect(cancel_rect, theme.border, 0.5, 3.0);
    p.draw_text(
        "Cancel",
        x0 + 78.0,
        btn_y + 17.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
}
