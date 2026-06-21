use super::super::*;

// ---------------------------------------------------------------------------
// File dialog (Phase 4) painting
// ---------------------------------------------------------------------------

/// Paint the file dialog (Phase 4).
pub(crate) fn paint_file_dialog(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    state: &SheetsState,
) {
    let w = 420.0;
    let h = 160.0;
    let modal = Rect::new(
        size.width / 2.0 - w / 2.0,
        size.height / 2.0 - h / 2.0,
        size.width / 2.0 + w / 2.0,
        size.height / 2.0 + h / 2.0,
    );

    p.fill_rounded_rect(modal, theme.background, 6.0);
    p.stroke_rounded_rect(modal, theme.border, 1.0, 6.0);

    let title = match state.file_dialog_mode {
        FileDialogMode::Open => "Open File",
        FileDialogMode::SaveAs => "Save As",
        FileDialogMode::ImportCsv => "Import CSV",
        FileDialogMode::ImportTsv => "Import TSV",
        FileDialogMode::ImportOds => "Import ODS",
        FileDialogMode::ExportXlsx => "Export XLSX",
        FileDialogMode::ExportPdf => "Export PDF",
        FileDialogMode::ExportHtml => "Export HTML",
        FileDialogMode::ExportCsv => "Export CSV",
    };
    p.draw_text(
        title,
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
        "File path:",
        x0,
        y0 + 14.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );

    let input_rect = Rect::new(x0, y0 + 20.0, x0 + 370.0, y0 + 44.0);
    p.fill_rounded_rect(input_rect, Color::WHITE, 3.0);
    p.stroke_rounded_rect(input_rect, theme.primary, 1.0, 3.0);

    let display_text = if state.file_dialog_path.is_empty() {
        "Type a file path...".to_string()
    } else {
        state.file_dialog_path.clone()
    };
    let text_color = theme.on_surface;
    p.draw_text(
        &display_text,
        x0 + 6.0,
        y0 + 38.0,
        text_color,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );

    let btn_y = y0 + 70.0;
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
