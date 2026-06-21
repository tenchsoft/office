//! Modal system for Slides — proper modal enum with separate paint functions.

use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::painter::GradientDirection;

/// All possible modal dialogs in the Slides editor.
#[derive(Debug, Clone)]
pub enum ActiveModal {
    OpenFile,
    InsertImage,
    ShapeSelector,
    FindReplace {
        find_text: String,
        replace_text: String,
    },
    AnimationPanel,
    SlideTransition,
    BackgroundSettings {
        preset_index: usize,
    },
    ThemeSelector {
        selected_index: usize,
    },
    TableWizard {
        rows: usize,
        cols: usize,
    },
    ChartWizard {
        chart_type: String,
    },
    LayoutSelector,
    SaveError(String),
    KeyboardShortcuts,
    Export {
        format_index: usize,
    },
    SaveAs,
}

/// Paint a modal dialog based on its type.
pub fn paint_modal(p: &mut Painter<'_>, theme: &Theme, size: Size, modal: &ActiveModal) {
    let modal_w = 360.0;
    let modal_h = match modal {
        ActiveModal::ShapeSelector => 200.0,
        ActiveModal::FindReplace { .. } => 220.0,
        ActiveModal::TableWizard { .. } => 200.0,
        ActiveModal::ChartWizard { .. } => 220.0,
        ActiveModal::LayoutSelector => 280.0,
        ActiveModal::KeyboardShortcuts => 400.0,
        ActiveModal::BackgroundSettings { .. } => 340.0,
        ActiveModal::ThemeSelector { .. } => 240.0,
        ActiveModal::Export { .. } => 240.0,
        ActiveModal::SaveError(_) => 180.0,
        _ => 180.0,
    };
    let modal_rect = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );

    // Dim background
    let dim = Rect::new(0.0, 0.0, size.width, size.height);
    p.fill_rect(dim, Color::rgba8(0, 0, 0, 120));

    // Modal surface
    p.fill_rounded_rect(modal_rect, theme.surface, theme.border_radius);
    p.stroke_rounded_rect(modal_rect, theme.border, 1.0, theme.border_radius);

    let x = modal_rect.x0 + 16.0;
    let mut y = modal_rect.y0 + 28.0;

    match modal {
        ActiveModal::OpenFile => {
            paint_modal_title(p, "Open Presentation", x, y, theme);
            y += 28.0;
            p.draw_text(
                "Click OK to open file dialog",
                x,
                y,
                theme.secondary,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
        ActiveModal::InsertImage => {
            paint_modal_title(p, "Insert Image", x, y, theme);
            y += 28.0;
            p.draw_text(
                "Click OK to select an image file",
                x,
                y,
                theme.secondary,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
        ActiveModal::ShapeSelector => {
            paint_modal_title(p, "Insert Shape", x, y, theme);
            y += 28.0;
            let shapes = [
                ("Rectangle", "rectangle"),
                ("Ellipse", "ellipse"),
                ("Line", "line"),
                ("Arrow", "arrow"),
            ];
            for (label, _kind) in shapes.iter() {
                let btn = Rect::new(x, y - 4.0, x + 160.0, y + 16.0);
                p.fill_rounded_rect(btn, theme.background, 3.0);
                p.draw_text(
                    label,
                    x + 8.0,
                    y + 10.0,
                    theme.on_surface,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
                y += 28.0;
            }
        }
        ActiveModal::FindReplace {
            find_text,
            replace_text,
        } => {
            paint_modal_title(p, "Find / Replace", x, y, theme);
            y += 28.0;
            p.draw_text(
                "Find:",
                x,
                y,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
            let find_box = Rect::new(x + 50.0, y - 10.0, x + 300.0, y + 6.0);
            p.fill_rounded_rect(find_box, theme.background, 3.0);
            p.stroke_rounded_rect(find_box, theme.border, 1.0, 3.0);
            if !find_text.is_empty() {
                p.draw_text(
                    find_text,
                    x + 54.0,
                    y,
                    theme.on_surface,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
            }
            y += 28.0;
            p.draw_text(
                "Replace:",
                x,
                y,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
            let repl_box = Rect::new(x + 50.0, y - 10.0, x + 300.0, y + 6.0);
            p.fill_rounded_rect(repl_box, theme.background, 3.0);
            p.stroke_rounded_rect(repl_box, theme.border, 1.0, 3.0);
            if !replace_text.is_empty() {
                p.draw_text(
                    replace_text,
                    x + 54.0,
                    y,
                    theme.on_surface,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
            }
            y += 28.0;
            // Action buttons
            let mut btn_x = x;
            for label in ["Find Next", "Replace", "Replace All"] {
                let btn = Rect::new(btn_x, y - 4.0, btn_x + 100.0, y + 16.0);
                p.fill_rounded_rect(btn, theme.background, 3.0);
                p.draw_text(
                    label,
                    btn_x + 8.0,
                    y + 10.0,
                    theme.on_surface,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
                btn_x += 110.0;
            }
        }
        ActiveModal::AnimationPanel => {
            paint_modal_title(p, "Animation Panel", x, y, theme);
            y += 28.0;
            p.draw_text(
                "Element animations will appear here",
                x,
                y,
                theme.secondary,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
        ActiveModal::SlideTransition => {
            paint_modal_title(p, "Slide Transition", x, y, theme);
            y += 28.0;
            for name in &["None", "Fade", "Push", "Wipe"] {
                let btn = Rect::new(x, y - 4.0, x + 160.0, y + 16.0);
                p.fill_rounded_rect(btn, theme.background, 3.0);
                p.draw_text(
                    name,
                    x + 8.0,
                    y + 10.0,
                    theme.on_surface,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
                y += 24.0;
            }
        }
        ActiveModal::BackgroundSettings { preset_index } => {
            paint_modal_title(p, "Slide Background", x, y, theme);
            y += 28.0;
            // Preset color swatches
            let presets = [
                (Color::WHITE, "White"),
                (Color::rgb8(0xF8, 0xF8, 0xF8), "Light Gray"),
                (Color::rgb8(0x1E, 0x1E, 0x2E), "Dark"),
                (Color::rgb8(0x0D, 0x1B, 0x2A), "Navy"),
                (Color::rgb8(0x2D, 0x1B, 0x0E), "Brown"),
            ];
            for (i, (color, label)) in presets.iter().enumerate() {
                let swatch = Rect::new(x, y - 4.0, x + 24.0, y + 16.0);
                p.fill_rounded_rect(swatch, *color, 3.0);
                if i == *preset_index {
                    p.stroke_rounded_rect(swatch, theme.primary, 2.0, 3.0);
                } else {
                    p.stroke_rounded_rect(swatch, theme.border, 1.0, 3.0);
                }
                p.draw_text(
                    label,
                    x + 30.0,
                    y + 10.0,
                    theme.on_surface,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
                y += 24.0;
            }
            y += 4.0;
            p.draw_text(
                "Gradient:",
                x,
                y + 10.0,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::BOLD,
                false,
            );
            y += 20.0;
            let gradients: [(Color, Color, &str); 3] = [
                (
                    Color::rgb8(0x60, 0xA5, 0xFA),
                    Color::rgb8(0x93, 0x52, 0xF6),
                    "Blue-Purple",
                ),
                (
                    Color::rgb8(0xEF, 0x44, 0x44),
                    Color::rgb8(0xF5, 0x9E, 0x0B),
                    "Red-Orange",
                ),
                (
                    Color::rgb8(0x22, 0xC5, 0x5E),
                    Color::rgb8(0x06, 0xB6, 0xD4),
                    "Green-Cyan",
                ),
            ];
            for (start, end, label) in &gradients {
                let swatch = Rect::new(x, y - 4.0, x + 100.0, y + 16.0);
                p.fill_rect_linear_gradient(swatch, *start, *end, GradientDirection::Horizontal);
                p.stroke_rounded_rect(swatch, theme.border, 1.0, 3.0);
                p.draw_text(
                    label,
                    x + 106.0,
                    y + 10.0,
                    theme.on_surface,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
                y += 24.0;
            }
        }
        ActiveModal::ThemeSelector { selected_index } => {
            paint_modal_title(p, "Select Theme", x, y, theme);
            y += 28.0;
            let themes = [
                ("Default", Color::WHITE, Color::rgb8(0x33, 0x33, 0x33)),
                ("Dark", Color::rgb8(0x1E, 0x1E, 0x2E), Color::WHITE),
                (
                    "Blue Professional",
                    Color::WHITE,
                    Color::rgb8(0x1A, 0x56, 0xDB),
                ),
                (
                    "Warm Earth",
                    Color::rgb8(0xFE, 0xF3, 0xE2),
                    Color::rgb8(0x92, 0x43, 0x1A),
                ),
            ];
            for (i, (name, bg, text)) in themes.iter().enumerate() {
                let btn = Rect::new(x, y - 4.0, x + 280.0, y + 20.0);
                if i == *selected_index {
                    p.fill_rounded_rect(btn, theme.primary, 3.0);
                    p.draw_text(
                        name,
                        x + 8.0,
                        y + 12.0,
                        theme.on_primary,
                        theme.font_size_small,
                        FontWeight::BOLD,
                        false,
                    );
                } else {
                    p.fill_rounded_rect(btn, *bg, 3.0);
                    p.stroke_rounded_rect(btn, theme.border, 1.0, 3.0);
                    p.draw_text(
                        name,
                        x + 8.0,
                        y + 12.0,
                        *text,
                        theme.font_size_small,
                        FontWeight::NORMAL,
                        false,
                    );
                }
                y += 28.0;
            }
        }
        ActiveModal::TableWizard { rows, cols } => {
            paint_modal_title(p, "Insert Table", x, y, theme);
            y += 28.0;
            p.draw_text(
                &format!("Rows: {}   Cols: {}", rows, cols),
                x,
                y,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
        ActiveModal::ChartWizard { chart_type } => {
            paint_modal_title(p, "Insert Chart", x, y, theme);
            y += 28.0;
            for ct in &["bar", "line", "pie", "scatter"] {
                let btn = Rect::new(x, y - 4.0, x + 160.0, y + 16.0);
                let is_selected = ct == chart_type;
                p.fill_rounded_rect(
                    btn,
                    if is_selected {
                        theme.primary
                    } else {
                        theme.background
                    },
                    3.0,
                );
                p.draw_text(
                    ct,
                    x + 8.0,
                    y + 10.0,
                    if is_selected {
                        theme.on_primary
                    } else {
                        theme.on_surface
                    },
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
                y += 28.0;
            }
        }
        ActiveModal::LayoutSelector => {
            paint_modal_title(p, "Slide Layout", x, y, theme);
            y += 28.0;
            for name in &[
                "Blank",
                "Title",
                "Title + Content",
                "Two Column",
                "Section Header",
            ] {
                let btn = Rect::new(x, y - 4.0, x + 160.0, y + 16.0);
                p.fill_rounded_rect(btn, theme.background, 3.0);
                p.draw_text(
                    name,
                    x + 8.0,
                    y + 10.0,
                    theme.on_surface,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
                y += 24.0;
            }
        }
        ActiveModal::SaveError(msg) => {
            paint_modal_title(p, "Save Error", x, y, theme);
            y += 28.0;
            p.draw_text(
                msg,
                x,
                y,
                theme.error,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
        ActiveModal::KeyboardShortcuts => {
            paint_modal_title(p, "Keyboard Shortcuts", x, y, theme);
            y += 24.0;
            let shortcuts = [
                ("Ctrl+S", "Save"),
                ("Ctrl+Z", "Undo"),
                ("Ctrl+Shift+Z / Ctrl+Y", "Redo"),
                ("Ctrl+D", "Duplicate element"),
                ("Ctrl+C / X / V", "Copy / Cut / Paste"),
                ("Ctrl+F", "Find / Replace"),
                ("Ctrl+P", "Start presentation"),
                ("Ctrl+= / - / 0", "Zoom in / out / reset"),
                ("Arrow keys", "Nudge element (1px)"),
                ("Shift+Arrow", "Nudge element (10px)"),
                ("Delete / Backspace", "Delete element"),
                ("Escape", "Close modal / exit edit"),
                ("Space+Drag", "Pan canvas"),
                ("Ctrl+G", "Toggle grid"),
                ("Ctrl+Shift+S", "Save As"),
                ("Ctrl+E", "Export"),
            ];
            for (key, desc) in &shortcuts {
                p.draw_text(
                    key,
                    x,
                    y,
                    theme.primary,
                    theme.font_size_small,
                    FontWeight::BOLD,
                    false,
                );
                p.draw_text(
                    desc,
                    x + 180.0,
                    y,
                    theme.on_surface,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
                y += 20.0;
            }
        }
        ActiveModal::Export { format_index } => {
            paint_modal_title(p, "Export Presentation", x, y, theme);
            y += 28.0;
            let formats = ["PDF", "PNG", "PPTX"];
            for (i, fmt) in formats.iter().enumerate() {
                let btn = Rect::new(x, y - 4.0, x + 160.0, y + 16.0);
                p.fill_rounded_rect(
                    btn,
                    if i == *format_index {
                        theme.primary
                    } else {
                        theme.background
                    },
                    3.0,
                );
                p.draw_text(
                    fmt,
                    x + 8.0,
                    y + 10.0,
                    if i == *format_index {
                        theme.on_primary
                    } else {
                        theme.on_surface
                    },
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
                y += 28.0;
            }
        }
        ActiveModal::SaveAs => {
            paint_modal_title(p, "Save As", x, y, theme);
            y += 28.0;
            p.draw_text(
                "Click OK to choose save location",
                x,
                y,
                theme.secondary,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
    }

    // OK / Cancel buttons at bottom of modal
    paint_modal_buttons(p, modal_rect, theme);
}

fn paint_modal_title(p: &mut Painter<'_>, title: &str, x: f64, y: f64, theme: &Theme) {
    p.draw_text(
        title,
        x,
        y,
        theme.on_surface,
        theme.font_size,
        FontWeight::BOLD,
        false,
    );
}

fn paint_modal_buttons(p: &mut Painter<'_>, modal_rect: Rect, theme: &Theme) {
    let btn_w = 70.0;
    let btn_h = 28.0;
    let y = modal_rect.y1 - 40.0;
    // Cancel button
    let cancel_rect = Rect::new(
        modal_rect.x1 - 160.0,
        y,
        modal_rect.x1 - 160.0 + btn_w,
        y + btn_h,
    );
    p.fill_rounded_rect(cancel_rect, theme.background, 4.0);
    p.stroke_rounded_rect(cancel_rect, theme.border, 1.0, 4.0);
    p.draw_text(
        "Cancel",
        cancel_rect.x0 + 16.0,
        cancel_rect.y0 + 18.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
    // OK button
    let ok_rect = Rect::new(
        modal_rect.x1 - 80.0,
        y,
        modal_rect.x1 - 80.0 + btn_w,
        y + btn_h,
    );
    p.fill_rounded_rect(ok_rect, theme.primary, 4.0);
    p.draw_text(
        "OK",
        ok_rect.x0 + 24.0,
        ok_rect.y0 + 18.0,
        theme.on_primary,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
}

/// Hit-test for modal OK/Cancel buttons. Returns true if the click was inside the modal area.
/// Returns (handled, confirmed) where confirmed=true means OK was clicked.
pub fn hit_test_modal_buttons(size: Size, modal: &ActiveModal, pos: Point) -> (bool, bool) {
    let modal_w = 360.0;
    let modal_h = match modal {
        ActiveModal::ShapeSelector => 200.0,
        ActiveModal::FindReplace { .. } => 220.0,
        ActiveModal::TableWizard { .. } => 200.0,
        ActiveModal::ChartWizard { .. } => 220.0,
        ActiveModal::LayoutSelector => 280.0,
        ActiveModal::KeyboardShortcuts => 400.0,
        ActiveModal::BackgroundSettings { .. } => 340.0,
        ActiveModal::ThemeSelector { .. } => 240.0,
        ActiveModal::Export { .. } => 240.0,
        _ => 180.0,
    };
    let modal_rect = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );

    if !modal_rect.contains(pos) {
        // Clicked outside modal — ignore
        return (false, false);
    }

    let btn_w = 70.0;
    let btn_h = 28.0;
    let y = modal_rect.y1 - 40.0;

    // Cancel
    let cancel_rect = Rect::new(
        modal_rect.x1 - 160.0,
        y,
        modal_rect.x1 - 160.0 + btn_w,
        y + btn_h,
    );
    if cancel_rect.contains(pos) {
        return (true, false);
    }

    // OK
    let ok_rect = Rect::new(
        modal_rect.x1 - 80.0,
        y,
        modal_rect.x1 - 80.0 + btn_w,
        y + btn_h,
    );
    if ok_rect.contains(pos) {
        return (true, true);
    }

    // Clicked inside modal but not on a button — don't dismiss
    (true, false)
}

/// Hit-test for shape selector items. Returns the shape kind string if clicked.
pub fn hit_test_shape_selector(size: Size, pos: Point) -> Option<&'static str> {
    let modal_w = 360.0;
    let modal_h = 200.0;
    let modal_rect = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );
    if !modal_rect.contains(pos) {
        return None;
    }
    let x = modal_rect.x0 + 16.0;
    let mut y = modal_rect.y0 + 28.0 + 28.0;
    let shapes = ["rectangle", "ellipse", "line", "arrow"];
    for kind in &shapes {
        let btn = Rect::new(x, y - 4.0, x + 160.0, y + 16.0);
        if btn.contains(pos) {
            return Some(*kind);
        }
        y += 28.0;
    }
    None
}

/// Hit-test for chart wizard items. Returns the chart type string if clicked.
pub fn hit_test_chart_wizard(size: Size, pos: Point) -> Option<&'static str> {
    let modal_w = 360.0;
    let modal_h = 220.0;
    let modal_rect = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );
    if !modal_rect.contains(pos) {
        return None;
    }
    let x = modal_rect.x0 + 16.0;
    let mut y = modal_rect.y0 + 28.0 + 28.0;
    let chart_types = ["bar", "line", "pie", "scatter"];
    for ct in &chart_types {
        let btn = Rect::new(x, y - 4.0, x + 160.0, y + 16.0);
        if btn.contains(pos) {
            return Some(*ct);
        }
        y += 28.0;
    }
    None
}

/// Hit-test for layout selector items. Returns the layout index (0-4) if clicked.
pub fn hit_test_layout_selector(size: Size, pos: Point) -> Option<usize> {
    let modal_w = 360.0;
    let modal_h = 280.0;
    let modal_rect = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );
    if !modal_rect.contains(pos) {
        return None;
    }
    let x = modal_rect.x0 + 16.0;
    let mut y = modal_rect.y0 + 28.0 + 28.0;
    for i in 0..5 {
        let btn = Rect::new(x, y - 4.0, x + 160.0, y + 16.0);
        if btn.contains(pos) {
            return Some(i);
        }
        y += 24.0;
    }
    None
}

/// Hit-test for transition items. Returns the transition name if clicked.
pub fn hit_test_transition_selector(size: Size, pos: Point) -> Option<&'static str> {
    let modal_w = 360.0;
    let modal_h = 180.0;
    let modal_rect = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );
    if !modal_rect.contains(pos) {
        return None;
    }
    let x = modal_rect.x0 + 16.0;
    let mut y = modal_rect.y0 + 28.0 + 28.0;
    let names = ["none", "fade", "push", "wipe"];
    for name in &names {
        let btn = Rect::new(x, y - 4.0, x + 160.0, y + 16.0);
        if btn.contains(pos) {
            return Some(*name);
        }
        y += 24.0;
    }
    None
}

/// Hit-test for background settings color presets. Returns preset index (0-4) if clicked.
pub fn hit_test_background_presets(size: Size, pos: Point) -> Option<usize> {
    let modal_w = 360.0;
    let modal_h = 340.0;
    let modal_rect = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );
    if !modal_rect.contains(pos) {
        return None;
    }
    let x = modal_rect.x0 + 16.0;
    let mut y = modal_rect.y0 + 28.0 + 28.0;
    for i in 0..5 {
        let swatch = Rect::new(x, y - 4.0, x + 24.0, y + 16.0);
        if swatch.contains(pos) {
            return Some(i);
        }
        y += 24.0;
    }
    None
}

/// Hit-test for background gradient presets. Returns gradient index (0-2) if clicked.
pub fn hit_test_background_gradients(size: Size, pos: Point) -> Option<usize> {
    let modal_w = 360.0;
    let modal_h = 340.0;
    let modal_rect = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );
    if !modal_rect.contains(pos) {
        return None;
    }
    let x = modal_rect.x0 + 16.0;
    // Skip past color presets: 5 * 24 + 4 (gap) + 20 (gradient label)
    let mut y = modal_rect.y0 + 28.0 + 28.0 + 5.0 * 24.0 + 4.0 + 20.0;
    for i in 0..3 {
        let swatch = Rect::new(x, y - 4.0, x + 100.0, y + 16.0);
        if swatch.contains(pos) {
            return Some(i);
        }
        y += 24.0;
    }
    None
}

/// Hit-test for theme selector items. Returns theme index (0-3) if clicked.
pub fn hit_test_theme_selector(size: Size, pos: Point) -> Option<usize> {
    let modal_w = 360.0;
    let modal_h = 240.0;
    let modal_rect = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );
    if !modal_rect.contains(pos) {
        return None;
    }
    let x = modal_rect.x0 + 16.0;
    let mut y = modal_rect.y0 + 28.0 + 28.0;
    for i in 0..4 {
        let btn = Rect::new(x, y - 4.0, x + 280.0, y + 20.0);
        if btn.contains(pos) {
            return Some(i);
        }
        y += 28.0;
    }
    None
}

/// Hit-test for export format selection. Returns the format index if clicked.
pub fn hit_test_export_formats(size: Size, pos: Point) -> Option<usize> {
    let modal_w = 360.0;
    let modal_h = 240.0;
    let modal_rect = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );
    if !modal_rect.contains(pos) {
        return None;
    }
    let x = modal_rect.x0 + 16.0;
    let mut y = modal_rect.y0 + 28.0 + 28.0;
    for i in 0..3 {
        let btn = Rect::new(x, y - 4.0, x + 160.0, y + 16.0);
        if btn.contains(pos) {
            return Some(i);
        }
        y += 28.0;
    }
    None
}
