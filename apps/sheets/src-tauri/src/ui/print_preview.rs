//! 10.2 Print preview modal rendering.
//!
//! Paints a full-screen overlay showing the grid split into printable pages
//! with navigation controls and zoom.

use super::grid::{GRID_COL_W, GRID_ROW_H};
use super::state::{col_letter, PrintPage, PrintPreviewState, Scaling, SheetsState};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

const PREVIEW_MARGIN: f64 = 40.0;
const NAV_BAR_H: f64 = 40.0;
const PAGE_SHADOW_OFFSET: f64 = 4.0;

/// Resolve print token placeholders in header/footer text.
fn resolve_print_tokens(text: &str, page_number: usize, total_pages: usize) -> String {
    let date_str = "today"; // stub
    let time_str = "now"; // stub
    text.replace("&P", &page_number.to_string())
        .replace("&N", &total_pages.to_string())
        .replace("&D", date_str)
        .replace("&T", time_str)
}

/// Paint the print preview modal overlay.
pub fn paint_print_preview(state: &SheetsState, p: &mut Painter<'_>, theme: &Theme, size: Size) {
    // Dim background
    p.fill_rect(
        Rect::new(0.0, 0.0, size.width, size.height),
        Color::rgba8(0, 0, 0, 180),
    );

    let preview = &state.print_preview;
    if preview.pages.is_empty() {
        // No pages to show
        p.draw_text(
            "No pages to preview",
            size.width / 2.0 - 80.0,
            size.height / 2.0,
            Color::WHITE,
            theme.font_size,
            FontWeight::BOLD,
            false,
        );
        paint_close_button(p, theme, size);
        return;
    }

    // Navigation bar at top
    let nav_rect = Rect::new(0.0, 0.0, size.width, NAV_BAR_H);
    p.fill_rect(nav_rect, theme.surface);
    paint_nav_bar(p, theme, nav_rect, preview);

    // Page display area
    let area_top = NAV_BAR_H + PREVIEW_MARGIN;
    let area_bottom = size.height - PREVIEW_MARGIN;
    let area_left = PREVIEW_MARGIN;
    let area_right = size.width - PREVIEW_MARGIN;

    let page = &preview.pages[preview.current_page];
    let zoom = preview.zoom;

    // Calculate page dimensions based on content
    let page_w = ((page.cols.1 - page.cols.0 + 1) as f64 * GRID_COL_W * zoom).max(100.0);
    let page_h = ((page.rows.1 - page.rows.0 + 1) as f64 * GRID_ROW_H * zoom).max(100.0);

    // Center the page in the available area
    let area_w = area_right - area_left;
    let area_h = area_bottom - area_top;
    let page_x = area_left + (area_w - page_w) / 2.0;
    let page_y = area_top + (area_h - page_h) / 2.0;

    let page_rect = Rect::new(page_x, page_y, page_x + page_w, page_y + page_h);

    // Page shadow
    let shadow_rect = Rect::new(
        page_rect.x0 + PAGE_SHADOW_OFFSET,
        page_rect.y0 + PAGE_SHADOW_OFFSET,
        page_rect.x1 + PAGE_SHADOW_OFFSET,
        page_rect.y1 + PAGE_SHADOW_OFFSET,
    );
    p.fill_rounded_rect(shadow_rect, Color::rgba8(0, 0, 0, 60), 2.0);

    // Page background (white)
    p.fill_rounded_rect(page_rect, Color::WHITE, 2.0);
    p.stroke_rounded_rect(page_rect, theme.border, 1.0, 2.0);

    // Render cells on the page
    paint_page_cells(state, p, theme, page_rect, page, zoom);

    // Render header/footer
    let setup = &state.page_setup;
    let has_header = !setup.header_left.is_empty()
        || !setup.header_center.is_empty()
        || !setup.header_right.is_empty();
    let has_footer = !setup.footer_left.is_empty()
        || !setup.footer_center.is_empty()
        || !setup.footer_right.is_empty();

    if has_header {
        let header_y = page_rect.y0 + 4.0;
        if !setup.header_left.is_empty() {
            p.draw_text(
                &resolve_print_tokens(&setup.header_left, page.page_number, preview.pages.len()),
                page_rect.x0 + 4.0,
                header_y + 10.0,
                Color::rgb8(0x99, 0x99, 0x99),
                theme.font_size_small * 0.8,
                FontWeight::NORMAL,
                false,
            );
        }
        if !setup.header_center.is_empty() {
            let text =
                resolve_print_tokens(&setup.header_center, page.page_number, preview.pages.len());
            p.draw_text(
                &text,
                page_rect.x0 + page_w / 2.0 - text.len() as f64 * 3.0,
                header_y + 10.0,
                Color::rgb8(0x99, 0x99, 0x99),
                theme.font_size_small * 0.8,
                FontWeight::NORMAL,
                false,
            );
        }
        if !setup.header_right.is_empty() {
            let text =
                resolve_print_tokens(&setup.header_right, page.page_number, preview.pages.len());
            p.draw_text(
                &text,
                page_rect.x1 - text.len() as f64 * 6.0 - 4.0,
                header_y + 10.0,
                Color::rgb8(0x99, 0x99, 0x99),
                theme.font_size_small * 0.8,
                FontWeight::NORMAL,
                false,
            );
        }
    }

    if has_footer {
        let footer_y = page_rect.y1 - 14.0;
        if !setup.footer_left.is_empty() {
            p.draw_text(
                &resolve_print_tokens(&setup.footer_left, page.page_number, preview.pages.len()),
                page_rect.x0 + 4.0,
                footer_y,
                Color::rgb8(0x99, 0x99, 0x99),
                theme.font_size_small * 0.8,
                FontWeight::NORMAL,
                false,
            );
        }
        if !setup.footer_center.is_empty() {
            let text =
                resolve_print_tokens(&setup.footer_center, page.page_number, preview.pages.len());
            p.draw_text(
                &text,
                page_rect.x0 + page_w / 2.0 - text.len() as f64 * 3.0,
                footer_y,
                Color::rgb8(0x99, 0x99, 0x99),
                theme.font_size_small * 0.8,
                FontWeight::NORMAL,
                false,
            );
        }
        if !setup.footer_right.is_empty() {
            let text =
                resolve_print_tokens(&setup.footer_right, page.page_number, preview.pages.len());
            p.draw_text(
                &text,
                page_rect.x1 - text.len() as f64 * 6.0 - 4.0,
                footer_y,
                Color::rgb8(0x99, 0x99, 0x99),
                theme.font_size_small * 0.8,
                FontWeight::NORMAL,
                false,
            );
        }
    }

    // Page number at bottom center
    let page_info = format!("Page {} of {}", page.page_number, preview.pages.len());
    p.draw_text(
        &page_info,
        size.width / 2.0 - 40.0,
        size.height - PREVIEW_MARGIN / 2.0,
        Color::WHITE,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );

    paint_close_button(p, theme, size);
}

/// Paint the grid cells for a single print page.
fn paint_page_cells(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    page_rect: Rect,
    page: &PrintPage,
    zoom: f64,
) {
    let cell_w = GRID_COL_W * zoom;
    let cell_h = GRID_ROW_H * zoom;

    for r in page.rows.0..=page.rows.1 {
        let rel_row = r - page.rows.0;
        let y = page_rect.y0 + rel_row as f64 * cell_h;
        if y > page_rect.y1 {
            break;
        }

        for c in page.cols.0..=page.cols.1 {
            let rel_col = c - page.cols.0;
            let x = page_rect.x0 + rel_col as f64 * cell_w;
            if x > page_rect.x1 {
                break;
            }

            let cell_rect = Rect::new(x, y, x + cell_w, y + cell_h);

            // Grid lines
            if state.page_setup.gridlines_print {
                p.stroke_rounded_rect(cell_rect, Color::rgb8(0xDD, 0xDD, 0xDD), 0.5, 0.0);
            }

            // Cell content
            if let Some(row) = state.grid.get(r) {
                if let Some(cell) = row.get(c) {
                    if !cell.value.is_empty() {
                        let text_color = if cell.is_formula {
                            Color::rgb8(0x00, 0x00, 0xCC)
                        } else {
                            Color::BLACK
                        };
                        p.draw_text(
                            &cell.value,
                            x + 4.0,
                            y + cell_h * 0.65,
                            text_color,
                            theme.font_size_small * zoom as f32,
                            FontWeight::NORMAL,
                            false,
                        );
                    }
                }
            }
        }
    }

    // Row headers on the left edge
    if state.page_setup.row_col_headers_print {
        for r in page.rows.0..=page.rows.1 {
            let rel_row = r - page.rows.0;
            let y = page_rect.y0 + rel_row as f64 * cell_h;
            p.draw_text(
                &format!("{}", r + 1),
                page_rect.x0 - 20.0,
                y + cell_h * 0.65,
                Color::rgb8(0x99, 0x99, 0x99),
                theme.font_size_small * zoom as f32 * 0.8,
                FontWeight::NORMAL,
                false,
            );
        }
        // Column headers at top
        for c in page.cols.0..=page.cols.1 {
            let rel_col = c - page.cols.0;
            let x = page_rect.x0 + rel_col as f64 * cell_w;
            p.draw_text(
                &col_letter(c),
                x + cell_w / 2.0 - 4.0,
                page_rect.y0 - 12.0,
                Color::rgb8(0x99, 0x99, 0x99),
                theme.font_size_small * zoom as f32 * 0.8,
                FontWeight::NORMAL,
                false,
            );
        }
    }
}

/// Paint the navigation bar with page controls.
fn paint_nav_bar(p: &mut Painter<'_>, theme: &Theme, rect: Rect, preview: &PrintPreviewState) {
    let y_mid = rect.y0 + rect.height() / 2.0;

    // Title
    p.draw_text(
        "Print Preview",
        rect.x0 + 16.0,
        y_mid + 4.0,
        theme.on_surface,
        theme.font_size,
        FontWeight::BOLD,
        false,
    );

    // Page navigation (center)
    let nav_x = rect.x1 / 2.0 - 100.0;

    // Previous button
    let prev_rect = Rect::new(nav_x, rect.y0 + 8.0, nav_x + 28.0, rect.y1 - 8.0);
    p.fill_rounded_rect(prev_rect, theme.background, 3.0);
    p.draw_text(
        "<",
        nav_x + 10.0,
        y_mid + 4.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );

    // Page indicator
    let page_text = format!("{} / {}", preview.current_page + 1, preview.pages.len());
    p.draw_text(
        &page_text,
        nav_x + 40.0,
        y_mid + 4.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );

    // Next button
    let next_x = nav_x + 100.0;
    let next_rect = Rect::new(next_x, rect.y0 + 8.0, next_x + 28.0, rect.y1 - 8.0);
    p.fill_rounded_rect(next_rect, theme.background, 3.0);
    p.draw_text(
        ">",
        next_x + 10.0,
        y_mid + 4.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );

    // Zoom controls (right side)
    let zoom_x = rect.x1 - 200.0;

    // Zoom out
    let zout_rect = Rect::new(zoom_x, rect.y0 + 8.0, zoom_x + 24.0, rect.y1 - 8.0);
    p.fill_rounded_rect(zout_rect, theme.background, 3.0);
    p.draw_text(
        "-",
        zoom_x + 8.0,
        y_mid + 4.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );

    // Zoom percentage
    p.draw_text(
        &format!("{:.0}%", preview.zoom * 100.0),
        zoom_x + 32.0,
        y_mid + 4.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );

    // Zoom in
    let zin_x = zoom_x + 80.0;
    let zin_rect = Rect::new(zin_x, rect.y0 + 8.0, zin_x + 24.0, rect.y1 - 8.0);
    p.fill_rounded_rect(zin_rect, theme.background, 3.0);
    p.draw_text(
        "+",
        zin_x + 8.0,
        y_mid + 4.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );

    // Print button
    let print_x = zoom_x + 120.0;
    let print_rect = Rect::new(print_x, rect.y0 + 8.0, print_x + 48.0, rect.y1 - 8.0);
    p.fill_rounded_rect(print_rect, theme.primary, 3.0);
    p.draw_text(
        "Print",
        print_x + 6.0,
        y_mid + 4.0,
        Color::WHITE,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
}

/// Paint a close button in the top-right corner.
fn paint_close_button(p: &mut Painter<'_>, theme: &Theme, size: Size) {
    let btn_w = 60.0;
    let btn_h = 24.0;
    let btn_rect = Rect::new(
        size.width - btn_w - 12.0,
        size.height - PREVIEW_MARGIN / 2.0 - btn_h / 2.0,
        size.width - 12.0,
        size.height - PREVIEW_MARGIN / 2.0 + btn_h / 2.0,
    );
    p.fill_rounded_rect(btn_rect, theme.primary, 3.0);
    p.draw_text(
        "Close",
        btn_rect.x0 + 12.0,
        btn_rect.y0 + 17.0,
        Color::WHITE,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
}

/// Paint the page setup dialog.
pub fn paint_page_setup_dialog(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
) {
    let w = 400.0;
    let h = 600.0;
    let modal = Rect::new(
        size.width / 2.0 - w / 2.0,
        size.height / 2.0 - h / 2.0,
        size.width / 2.0 + w / 2.0,
        size.height / 2.0 + h / 2.0,
    );

    // Background overlay
    p.fill_rect(
        Rect::new(0.0, 0.0, size.width, size.height),
        Color::rgba8(0, 0, 0, 120),
    );

    // Modal
    p.fill_rounded_rect(modal, theme.surface, theme.border_radius);
    p.stroke_rounded_rect(modal, theme.border, 1.0, theme.border_radius);

    let x0 = modal.x0 + 20.0;
    let mut y = modal.y0 + 28.0;

    // Title
    p.draw_text(
        "Page Setup",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        FontWeight::BOLD,
        false,
    );
    y += 32.0;

    let setup = &state.page_setup;

    // Paper size
    p.draw_text(
        &format!("Paper Size: {}", setup.paper_size.label()),
        x0,
        y,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
    y += 24.0;

    // Orientation
    p.draw_text(
        &format!("Orientation: {}", setup.orientation.label()),
        x0,
        y,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
    y += 24.0;

    // Margins
    p.draw_text(
        "Margins (mm):",
        x0,
        y,
        theme.secondary,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
    y += 20.0;
    let margins = &setup.margins;
    let margin_items = [
        ("Top", margins.top),
        ("Bottom", margins.bottom),
        ("Left", margins.left),
        ("Right", margins.right),
        ("Header", margins.header),
        ("Footer", margins.footer),
    ];
    for (label, value) in &margin_items {
        p.draw_text(
            &format!("  {}: {:.2}", label, value),
            x0,
            y,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        y += 18.0;
    }
    y += 8.0;

    // Scaling
    match setup.scaling {
        Scaling::Percentage(pct) => {
            p.draw_text(
                &format!("Scaling: {:.0}%", pct),
                x0,
                y,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
        Scaling::FitToPages { width, height } => {
            let w_str = width.map(|v| v.to_string()).unwrap_or("auto".into());
            let h_str = height.map(|v| v.to_string()).unwrap_or("auto".into());
            p.draw_text(
                &format!("Fit to: {} x {} pages", w_str, h_str),
                x0,
                y,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
    }
    y += 24.0;

    // Options
    let options = [
        ("Print Gridlines", setup.gridlines_print),
        ("Print Row/Col Headers", setup.row_col_headers_print),
        ("Center Horizontally", setup.center_horizontally),
        ("Center Vertically", setup.center_vertically),
        ("Repeat Header", setup.repeat_header),
    ];
    p.draw_text(
        "Options:",
        x0,
        y,
        theme.secondary,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
    y += 20.0;
    for (label, checked) in &options {
        let check = if *checked { "[x]" } else { "[ ]" };
        p.draw_text(
            &format!("  {} {}", check, label),
            x0,
            y,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        y += 20.0;
    }
    y += 8.0;

    // Print area
    let area_text = if let Some(ref area) = setup.print_area {
        format!("Print Area: {}", area.to_address())
    } else {
        "Print Area: Entire Sheet".to_string()
    };
    p.draw_text(
        &area_text,
        x0,
        y,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
    y += 32.0;

    // Headers/Footers
    y += 8.0;
    p.draw_text(
        "Header/Footer:",
        x0,
        y,
        theme.secondary,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
    y += 20.0;
    let hf_items = [
        ("Header Left", &setup.header_left),
        ("Header Center", &setup.header_center),
        ("Header Right", &setup.header_right),
        ("Footer Left", &setup.footer_left),
        ("Footer Center", &setup.footer_center),
        ("Footer Right", &setup.footer_right),
    ];
    for (label, value) in &hf_items {
        let display = if value.is_empty() {
            "(empty)"
        } else {
            value.as_str()
        };
        p.draw_text(
            &format!("  {}: {}", label, display),
            x0,
            y,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        y += 18.0;
    }

    // Buttons
    let btn_labels = ["OK", "Cancel"];
    let mut bx = x0;
    for label in &btn_labels {
        let btn_rect = Rect::new(bx, y, bx + 64.0, y + 24.0);
        p.fill_rounded_rect(btn_rect, theme.primary, 3.0);
        p.draw_text(
            label,
            bx + 16.0,
            y + 17.0,
            Color::WHITE,
            theme.font_size_small,
            FontWeight::BOLD,
            false,
        );
        bx += 72.0;
    }
}

// ---------------------------------------------------------------------------
// Hit testing for print preview controls
// ---------------------------------------------------------------------------

/// Navigation action for the print preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewNavAction {
    PrevPage,
    NextPage,
    ZoomIn,
    ZoomOut,
    Print,
}

/// Hit-test the print preview navigation bar.
pub fn hit_preview_nav(
    x: f64,
    y: f64,
    width: f64,
    preview: &PrintPreviewState,
) -> Option<PreviewNavAction> {
    if !(0.0..=NAV_BAR_H).contains(&y) {
        return None;
    }

    let nav_x = width / 2.0 - 100.0;

    // Previous button
    if x >= nav_x && x < nav_x + 28.0 && preview.current_page > 0 {
        return Some(PreviewNavAction::PrevPage);
    }

    // Next button
    let next_x = nav_x + 100.0;
    if x >= next_x && x < next_x + 28.0 && preview.current_page + 1 < preview.pages.len() {
        return Some(PreviewNavAction::NextPage);
    }

    // Zoom controls
    let zoom_x = width - 200.0;
    if x >= zoom_x && x < zoom_x + 24.0 {
        return Some(PreviewNavAction::ZoomOut);
    }
    let zin_x = zoom_x + 80.0;
    if x >= zin_x && x < zin_x + 24.0 {
        return Some(PreviewNavAction::ZoomIn);
    }

    // Print button
    let print_x = zoom_x + 120.0;
    if x >= print_x && x < print_x + 48.0 {
        return Some(PreviewNavAction::Print);
    }

    None
}

/// Hit-test the close button in the print preview.
pub fn hit_preview_close(x: f64, y: f64, size: Size) -> bool {
    let btn_w = 60.0;
    let btn_h = 24.0;
    let btn_x0 = size.width - btn_w - 12.0;
    let btn_y0 = size.height - PREVIEW_MARGIN / 2.0 - btn_h / 2.0;
    x >= btn_x0 && x <= btn_x0 + btn_w && y >= btn_y0 && y <= btn_y0 + btn_h
}
