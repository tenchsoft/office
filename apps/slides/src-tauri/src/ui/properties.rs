//! Properties panel — interactive element properties editor.

use super::state::SlidesState;
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

// ── Layout constants ───────────────────────────────────────────────
const LABEL_W: f64 = 50.0;
const ROW_H: f64 = 22.0;
const BUTTON_H: f64 = 24.0;
const COLOR_SWATCH_SIZE: f64 = 20.0;

// ── Interactive regions ────────────────────────────────────────────

/// Identifies a clickable/draggable region in the properties panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropertyAction {
    /// Action button index: 0=Bring Forward, 1=Send Backward, 2=Duplicate, 3=Delete
    ActionButton(usize),
    /// Slider for a numeric property.
    Slider(SliderKind),
    /// Color swatch click to cycle fill color.
    FillColorSwatch,
    /// Color swatch click to cycle border color.
    BorderColorSwatch,
    /// Open background settings modal.
    BackgroundButton,
    /// Open theme selector modal.
    ThemeButton,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SliderKind {
    PositionX,
    PositionY,
    Width,
    Height,
    Rotation,
    Opacity,
}

/// A single interactive region in the properties panel.
pub struct PropertyRegion {
    pub action: PropertyAction,
    pub rect: Rect,
}

/// Computes all interactive regions for the current properties panel state.
pub fn properties_layout(state: &SlidesState, panel_rect: Rect) -> Vec<PropertyRegion> {
    let mut regions = Vec::new();
    let x = panel_rect.x0 + 12.0;
    let panel_w = panel_rect.width() - 24.0;

    let Some(slide) = state.current_slide() else {
        return regions;
    };
    let Some(idx) = state.selected_element else {
        return regions;
    };
    let Some(elem) = slide.elements.get(idx) else {
        return regions;
    };

    let mut y = panel_rect.y0 + 20.0 + 24.0;

    // Type line (not interactive)
    y += ROW_H;

    // Sliders: X, Y, W, H, Rotation, Opacity
    let slider_kinds = [
        (SliderKind::PositionX, elem.x, 0.0, 640.0),
        (SliderKind::PositionY, elem.y, 0.0, 360.0),
        (SliderKind::Width, elem.w, 10.0, 640.0),
        (SliderKind::Height, elem.h, 10.0, 360.0),
        (SliderKind::Rotation, elem.rotation, 0.0, 360.0),
        (SliderKind::Opacity, elem.opacity, 0.0, 1.0),
    ];
    for (kind, _val, _min, _max) in &slider_kinds {
        let slider_x = x + LABEL_W;
        let slider_w = (panel_w - LABEL_W).max(60.0);
        let track_rect = Rect::new(slider_x, y - 8.0, slider_x + slider_w, y + 8.0);
        regions.push(PropertyRegion {
            action: PropertyAction::Slider(*kind),
            rect: track_rect,
        });
        y += ROW_H;
    }

    // Z-index (not interactive)
    y += ROW_H;

    // Has Text (not interactive)
    if elem.text.is_some() {
        y += ROW_H;
    }

    // Fill color swatch
    if elem.fill.is_some() {
        let swatch_x = x + LABEL_W;
        let swatch = Rect::new(
            swatch_x,
            y - COLOR_SWATCH_SIZE / 2.0,
            swatch_x + COLOR_SWATCH_SIZE,
            y + COLOR_SWATCH_SIZE / 2.0,
        );
        regions.push(PropertyRegion {
            action: PropertyAction::FillColorSwatch,
            rect: swatch,
        });
        y += ROW_H;
    }

    // Border color swatch
    if elem.border.is_some() {
        let swatch_x = x + LABEL_W + 60.0;
        let swatch = Rect::new(
            swatch_x,
            y - COLOR_SWATCH_SIZE / 2.0,
            swatch_x + COLOR_SWATCH_SIZE,
            y + COLOR_SWATCH_SIZE / 2.0,
        );
        regions.push(PropertyRegion {
            action: PropertyAction::BorderColorSwatch,
            rect: swatch,
        });
        y += ROW_H;
    }

    // Shadow (not interactive)
    if elem.shadow.is_some() {
        y += ROW_H;
    }

    // Actions header
    y += 8.0 + 20.0;

    // Action buttons
    for i in 0..4usize {
        let btn = Rect::new(x, y - 4.0, x + panel_w, y + 16.0);
        regions.push(PropertyRegion {
            action: PropertyAction::ActionButton(i),
            rect: btn,
        });
        y += BUTTON_H;
    }

    // Slide-level actions (always visible)
    y += 8.0;
    let bg_btn = Rect::new(x, y - 4.0, x + panel_w, y + 16.0);
    regions.push(PropertyRegion {
        action: PropertyAction::BackgroundButton,
        rect: bg_btn,
    });
    y += BUTTON_H;
    let theme_btn = Rect::new(x, y - 4.0, x + panel_w, y + 16.0);
    regions.push(PropertyRegion {
        action: PropertyAction::ThemeButton,
        rect: theme_btn,
    });

    regions
}

/// Given a slider kind and a click position within the slider track, compute the new value.
pub fn slider_value_from_click(kind: SliderKind, track_rect: Rect, click_x: f64) -> f64 {
    let (min_val, max_val) = slider_range(kind);
    let normalized = if track_rect.width() > 0.0 {
        ((click_x - track_rect.x0) / track_rect.width()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    min_val + normalized * (max_val - min_val)
}

fn slider_range(kind: SliderKind) -> (f64, f64) {
    match kind {
        SliderKind::PositionX => (0.0, 640.0),
        SliderKind::PositionY => (0.0, 360.0),
        SliderKind::Width => (10.0, 640.0),
        SliderKind::Height => (10.0, 360.0),
        SliderKind::Rotation => (0.0, 360.0),
        SliderKind::Opacity => (0.0, 1.0),
    }
}

// ── Paint ──────────────────────────────────────────────────────────

pub fn paint_properties(state: &SlidesState, p: &mut Painter<'_>, theme: &Theme, rect: Rect) {
    p.fill_rect(rect, theme.surface);
    let x = rect.x0 + 12.0;
    let panel_w = rect.width() - 24.0;
    let mut y = rect.y0 + 20.0;
    p.draw_text(
        "Properties",
        x,
        y,
        theme.on_surface,
        theme.font_size,
        FontWeight::BOLD,
        false,
    );
    y += 24.0;

    let Some(slide) = state.current_slide() else {
        return;
    };
    let Some(idx) = state.selected_element else {
        p.draw_text(
            "No element selected",
            x,
            y,
            theme.disabled,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        return;
    };
    let Some(elem) = slide.elements.get(idx) else {
        return;
    };

    // Type
    p.draw_text(
        &format!("Type: {}", elem.kind),
        x,
        y,
        theme.secondary,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
    y += ROW_H;

    // Sliders
    let slider_data = [
        ("X:", elem.x, 0.0, 640.0),
        ("Y:", elem.y, 0.0, 360.0),
        ("W:", elem.w, 10.0, 640.0),
        ("H:", elem.h, 10.0, 360.0),
        ("Rot:", elem.rotation, 0.0, 360.0),
        ("Opa:", elem.opacity * 100.0, 0.0, 100.0),
    ];
    for (label, value, min_val, max_val) in &slider_data {
        paint_slider_row(
            p,
            theme,
            SliderRow {
                label,
                value: *value,
                min_val: *min_val,
                max_val: *max_val,
                x,
                y,
                panel_w,
            },
        );
        y += ROW_H;
    }

    // Z-index
    p.draw_text(
        &format!("Z-index: {}", elem.z_index),
        x,
        y,
        theme.secondary,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
    y += ROW_H;

    // Has Text
    if elem.text.is_some() {
        p.draw_text(
            "Has Text: Yes",
            x,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        y += ROW_H;
    }

    // Fill color
    if let Some(fill) = elem.fill {
        p.draw_text(
            "Fill:",
            x,
            y,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        let swatch_x = x + LABEL_W;
        let swatch = Rect::new(
            swatch_x,
            y - COLOR_SWATCH_SIZE / 2.0,
            swatch_x + COLOR_SWATCH_SIZE,
            y + COLOR_SWATCH_SIZE / 2.0,
        );
        p.fill_rounded_rect(swatch, fill, 2.0);
        p.stroke_rounded_rect(swatch, theme.border, 1.0, 2.0);
        let hex = format!("{:02x}{:02x}{:02x}", fill.r(), fill.g(), fill.b());
        p.draw_text(
            &hex,
            swatch_x + COLOR_SWATCH_SIZE + 6.0,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        y += ROW_H;
    }

    // Border
    if let Some(border) = &elem.border {
        p.draw_text(
            &format!("Border: {:.0}px", border.width),
            x,
            y,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        let swatch_x = x + LABEL_W + 60.0;
        let swatch = Rect::new(
            swatch_x,
            y - COLOR_SWATCH_SIZE / 2.0,
            swatch_x + COLOR_SWATCH_SIZE,
            y + COLOR_SWATCH_SIZE / 2.0,
        );
        p.fill_rounded_rect(swatch, border.color, 2.0);
        p.stroke_rounded_rect(swatch, theme.border, 1.0, 2.0);
        y += ROW_H;
    }

    // Shadow
    if elem.shadow.is_some() {
        p.draw_text(
            "Shadow: Yes",
            x,
            y,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        y += ROW_H;
    }

    // Actions section
    y += 8.0;
    p.draw_text(
        "Actions",
        x,
        y,
        theme.on_surface,
        theme.font_size,
        FontWeight::BOLD,
        false,
    );
    y += 20.0;

    for action in ["Bring Forward", "Send Backward", "Duplicate", "Delete"] {
        let button = Rect::new(x, y - 4.0, x + panel_w, y + 16.0);
        p.fill_rounded_rect(button, theme.background, 3.0);
        p.stroke_rounded_rect(button, theme.border, 1.0, 3.0);
        p.draw_text(
            action,
            x + 8.0,
            y + 10.0,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        y += BUTTON_H;
    }

    // Slide-level actions
    y += 8.0;
    let bg_button = Rect::new(x, y - 4.0, x + panel_w, y + 16.0);
    p.fill_rounded_rect(bg_button, theme.primary, 3.0);
    p.draw_text(
        "Background...",
        x + 8.0,
        y + 10.0,
        theme.on_primary,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
    y += BUTTON_H;
    let theme_button = Rect::new(x, y - 4.0, x + panel_w, y + 16.0);
    p.fill_rounded_rect(theme_button, theme.background, 3.0);
    p.stroke_rounded_rect(theme_button, theme.border, 1.0, 3.0);
    p.draw_text(
        "Theme...",
        x + 8.0,
        y + 10.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
}

struct SliderRow<'a> {
    label: &'a str,
    value: f64,
    min_val: f64,
    max_val: f64,
    x: f64,
    y: f64,
    panel_w: f64,
}

fn paint_slider_row(p: &mut Painter<'_>, theme: &Theme, row: SliderRow<'_>) {
    let SliderRow {
        label,
        value,
        min_val,
        max_val,
        x,
        y,
        panel_w,
    } = row;
    // Label
    p.draw_text(
        label,
        x,
        y,
        theme.secondary,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );

    let slider_x = x + LABEL_W;
    let slider_w = (panel_w - LABEL_W - 40.0).max(60.0);

    // Track background
    let track = Rect::new(slider_x, y - 3.0, slider_x + slider_w, y + 3.0);
    p.fill_rounded_rect(track, theme.background, 2.0);

    // Filled portion
    let normalized = if max_val > min_val {
        ((value - min_val) / (max_val - min_val)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let filled = Rect::new(slider_x, y - 3.0, slider_x + slider_w * normalized, y + 3.0);
    p.fill_rounded_rect(filled, theme.primary, 2.0);

    // Thumb
    let thumb_x = slider_x + slider_w * normalized;
    let thumb = Rect::new(thumb_x - 5.0, y - 6.0, thumb_x + 5.0, y + 6.0);
    p.fill_rounded_rect(thumb, Color::WHITE, 3.0);
    p.stroke_rounded_rect(thumb, theme.primary, 1.5, 3.0);

    // Value text
    let val_text = if max_val <= 1.0 {
        format!("{:.0}%", value)
    } else {
        format!("{:.0}", value)
    };
    p.draw_text(
        &val_text,
        slider_x + slider_w + 6.0,
        y,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
}
