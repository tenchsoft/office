use super::*;

// ----- Toolbar painting -----

/// Paint the formatting toolbar.
pub(crate) fn paint_toolbar(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    width: f64,
    y: f64,
) {
    p.fill_rect(Rect::new(0.0, y, width, y + TOOLBAR_H), theme.surface);
    p.draw_line(
        Point::new(0.0, y + TOOLBAR_H),
        Point::new(width, y + TOOLBAR_H),
        theme.border,
        0.5,
    );

    let btn_w = 28.0;
    let gap = 2.0;
    let mut x = 8.0;
    let btn_h = TOOLBAR_H - 4.0;
    let btn_y = y + 2.0;

    // Bold button
    let bold_active = state.get_selected_cell_format().bold;
    let bold_bg = if bold_active {
        theme.primary
    } else {
        theme.background
    };
    let bold_fg = if bold_active {
        Color::WHITE
    } else {
        theme.on_surface
    };
    p.fill_rounded_rect(Rect::new(x, btn_y, x + btn_w, btn_y + btn_h), bold_bg, 3.0);
    p.stroke_rounded_rect(
        Rect::new(x, btn_y, x + btn_w, btn_y + btn_h),
        theme.border,
        0.5,
        3.0,
    );
    p.draw_text(
        "B",
        x + 9.0,
        btn_y + 21.0,
        bold_fg,
        13.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    x += btn_w + gap;

    // Italic button
    let italic_active = state.get_selected_cell_format().italic;
    let italic_bg = if italic_active {
        theme.primary
    } else {
        theme.background
    };
    let italic_fg = if italic_active {
        Color::WHITE
    } else {
        theme.on_surface
    };
    p.fill_rounded_rect(
        Rect::new(x, btn_y, x + btn_w, btn_y + btn_h),
        italic_bg,
        3.0,
    );
    p.stroke_rounded_rect(
        Rect::new(x, btn_y, x + btn_w, btn_y + btn_h),
        theme.border,
        0.5,
        3.0,
    );
    p.draw_text(
        "I",
        x + 10.0,
        btn_y + 21.0,
        italic_fg,
        13.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    x += btn_w + gap;

    // Underline button
    let ul_active = state.get_selected_cell_format().underline;
    let ul_bg = if ul_active {
        theme.primary
    } else {
        theme.background
    };
    let ul_fg = if ul_active {
        Color::WHITE
    } else {
        theme.on_surface
    };
    p.fill_rounded_rect(Rect::new(x, btn_y, x + btn_w, btn_y + btn_h), ul_bg, 3.0);
    p.stroke_rounded_rect(
        Rect::new(x, btn_y, x + btn_w, btn_y + btn_h),
        theme.border,
        0.5,
        3.0,
    );
    p.draw_text(
        "U",
        x + 8.0,
        btn_y + 21.0,
        ul_fg,
        13.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    x += btn_w + gap;

    // Separator
    // Separator
    p.draw_line(
        Point::new(x + 4.0, btn_y + 4.0),
        Point::new(x + 4.0, btn_y + btn_h - 4.0),
        theme.border,
        0.5,
    );
    x += 8.0;

    // Align Left
    p.fill_rounded_rect(
        Rect::new(x, btn_y, x + btn_w, btn_y + btn_h),
        theme.background,
        3.0,
    );
    p.stroke_rounded_rect(
        Rect::new(x, btn_y, x + btn_w, btn_y + btn_h),
        theme.border,
        0.5,
        3.0,
    );
    p.draw_text(
        "L",
        x + 9.0,
        btn_y + 21.0,
        theme.on_surface,
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    x += btn_w + gap;

    // Align Center
    p.fill_rounded_rect(
        Rect::new(x, btn_y, x + btn_w, btn_y + btn_h),
        theme.background,
        3.0,
    );
    p.stroke_rounded_rect(
        Rect::new(x, btn_y, x + btn_w, btn_y + btn_h),
        theme.border,
        0.5,
        3.0,
    );
    p.draw_text(
        "C",
        x + 9.0,
        btn_y + 21.0,
        theme.on_surface,
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    x += btn_w + gap;

    // Align Right
    p.fill_rounded_rect(
        Rect::new(x, btn_y, x + btn_w, btn_y + btn_h),
        theme.background,
        3.0,
    );
    p.stroke_rounded_rect(
        Rect::new(x, btn_y, x + btn_w, btn_y + btn_h),
        theme.border,
        0.5,
        3.0,
    );
    p.draw_text(
        "R",
        x + 9.0,
        btn_y + 21.0,
        theme.on_surface,
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    x += btn_w + gap;

    // Separator
    p.draw_line(
        Point::new(x + 4.0, btn_y + 4.0),
        Point::new(x + 4.0, btn_y + btn_h - 4.0),
        theme.border,
        0.5,
    );
    x += 8.0;

    // Number format label
    let nf_label = match &state.get_selected_cell_format().number_format {
        NumberFormat::General => "General",
        NumberFormat::Number { .. } => "Number",
        NumberFormat::Currency { .. } => "Currency",
        NumberFormat::Percentage { .. } => "Percent",
        NumberFormat::Date => "Date",
        NumberFormat::Text => "Text",
    };
    let nf_w = 70.0;
    p.fill_rounded_rect(
        Rect::new(x, btn_y, x + nf_w, btn_y + btn_h),
        theme.background,
        3.0,
    );
    p.stroke_rounded_rect(
        Rect::new(x, btn_y, x + nf_w, btn_y + btn_h),
        theme.border,
        0.5,
        3.0,
    );
    p.draw_text(
        nf_label,
        x + 6.0,
        btn_y + 21.0,
        theme.on_surface,
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    x += nf_w + gap;

    // Separator
    p.draw_line(
        Point::new(x + 4.0, btn_y + 4.0),
        Point::new(x + 4.0, btn_y + btn_h - 4.0),
        theme.border,
        0.5,
    );
    x += 8.0;

    // Format Painter
    let fp_active = state.format_painter_active;
    let fp_bg = if fp_active {
        theme.primary
    } else {
        theme.background
    };
    let fp_fg = if fp_active {
        Color::WHITE
    } else {
        theme.on_surface
    };
    let fp_w = btn_w + 20.0;
    p.fill_rounded_rect(Rect::new(x, btn_y, x + fp_w, btn_y + btn_h), fp_bg, 3.0);
    p.stroke_rounded_rect(
        Rect::new(x, btn_y, x + fp_w, btn_y + btn_h),
        theme.border,
        0.5,
        3.0,
    );
    p.draw_text(
        "Painter",
        x + 4.0,
        btn_y + 21.0,
        fp_fg,
        10.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    x += fp_w + gap;

    // Merge cells
    let merge_w = btn_w + 20.0;
    p.fill_rounded_rect(
        Rect::new(x, btn_y, x + merge_w, btn_y + btn_h),
        theme.background,
        3.0,
    );
    p.stroke_rounded_rect(
        Rect::new(x, btn_y, x + merge_w, btn_y + btn_h),
        theme.border,
        0.5,
        3.0,
    );
    p.draw_text(
        "Merge",
        x + 4.0,
        btn_y + 21.0,
        theme.on_surface,
        10.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
}
