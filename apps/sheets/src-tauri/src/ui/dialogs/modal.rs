use super::super::*;

// ---------------------------------------------------------------------------
// Modal / toast painting
// ---------------------------------------------------------------------------

pub(crate) fn paint_sheets_modal(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    modal_type: &state::ModalType,
) {
    let (w, h) = match modal_type {
        state::ModalType::About => (320.0, 200.0),
        state::ModalType::Welcome => (380.0, 260.0),
        state::ModalType::Shortcuts => (440.0, 380.0),
        state::ModalType::Error(_) => (400.0, 160.0),
    };
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
    match modal_type {
        state::ModalType::About => {
            p.draw_text(
                "About Tench Sheets",
                x0,
                y,
                theme.on_surface,
                theme.font_size,
                tench_ui::parley::FontWeight::BOLD,
                false,
            );
            y += 28.0;
            p.draw_text(
                "Tench Sheets v1.0.0",
                x0,
                y,
                theme.on_surface,
                theme.font_size_small,
                tench_ui::parley::FontWeight::NORMAL,
                false,
            );
            y += 20.0;
            p.draw_text(
                "Copyright (c) 2024-2026 Tench",
                x0,
                y,
                theme.secondary,
                theme.font_size_small,
                tench_ui::parley::FontWeight::NORMAL,
                false,
            );
            y += 20.0;
            p.draw_text(
                "A high-performance native spreadsheet editor.",
                x0,
                y,
                theme.secondary,
                theme.font_size_small,
                tench_ui::parley::FontWeight::NORMAL,
                false,
            );
        }
        state::ModalType::Welcome => {
            p.draw_text(
                "Welcome to Tench Sheets!",
                x0,
                y,
                theme.on_surface,
                theme.font_size,
                tench_ui::parley::FontWeight::BOLD,
                false,
            );
            y += 28.0;
            p.draw_text(
                "Quick Start Tips:",
                x0,
                y,
                theme.on_surface,
                theme.font_size_small,
                tench_ui::parley::FontWeight::BOLD,
                false,
            );
            y += 20.0;
            for tip in [
                "Click a cell and start typing to enter data",
                "Press F2 to edit an existing cell",
                "Use Ctrl+Arrow to jump to data edges",
                "Right-click for context menu options",
                "Use Insert > Chart to visualize data",
            ] {
                p.draw_text(
                    &format!("  - {}", tip),
                    x0,
                    y,
                    theme.secondary,
                    theme.font_size_small,
                    tench_ui::parley::FontWeight::NORMAL,
                    false,
                );
                y += 18.0;
            }
        }
        state::ModalType::Shortcuts => {
            p.draw_text(
                "Keyboard Shortcuts",
                x0,
                y,
                theme.on_surface,
                theme.font_size,
                tench_ui::parley::FontWeight::BOLD,
                false,
            );
            y += 28.0;
            let shortcuts = [
                ("Ctrl+S", "Save"),
                ("Ctrl+Z", "Undo"),
                ("Ctrl+Y", "Redo"),
                ("Ctrl+C", "Copy"),
                ("Ctrl+X", "Cut"),
                ("Ctrl+V", "Paste"),
                ("Ctrl+F", "Find"),
                ("Ctrl+H", "Replace"),
                ("Ctrl+A", "Select All"),
                ("Ctrl+1", "Format Cells"),
                ("F2", "Edit Cell"),
                ("Delete", "Clear Cell"),
                ("Ctrl+Arrow", "Jump to Data Edge"),
                ("Page Up/Down", "Scroll Page"),
                ("Ctrl+Home", "Go to Start"),
                ("Ctrl+End", "Go to Last Data"),
                ("Ctrl+P", "Print Preview"),
                ("F11", "Toggle Full Screen"),
            ];
            for (key, desc) in &shortcuts {
                p.draw_text(
                    key,
                    x0,
                    y,
                    theme.primary,
                    theme.font_size_small,
                    tench_ui::parley::FontWeight::BOLD,
                    false,
                );
                p.draw_text(
                    desc,
                    x0 + 120.0,
                    y,
                    theme.on_surface,
                    theme.font_size_small,
                    tench_ui::parley::FontWeight::NORMAL,
                    false,
                );
                y += 18.0;
            }
        }
        state::ModalType::Error(msg) => {
            p.draw_text(
                "Error",
                x0,
                y,
                theme.on_surface,
                theme.font_size,
                tench_ui::parley::FontWeight::BOLD,
                false,
            );
            y += 28.0;
            p.draw_text(
                msg,
                x0,
                y,
                theme.on_surface,
                theme.font_size_small,
                tench_ui::parley::FontWeight::NORMAL,
                false,
            );
        }
    }
}

pub(crate) fn paint_toast(p: &mut Painter<'_>, theme: &Theme, size: Size, message: &str) {
    let rect = Rect::new(size.width - 250.0, 42.0, size.width - 16.0, 76.0);
    p.fill_rounded_rect(rect, theme.surface, theme.border_radius);
    p.stroke_rounded_rect(rect, theme.border, 1.0, theme.border_radius);
    p.draw_text(
        message,
        rect.x0 + 12.0,
        rect.y0 + 22.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
}
