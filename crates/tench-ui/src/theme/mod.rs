//! Theme system - colors, typography, spacing constants.

use crate::core::types::Color;

/// Application theme.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Background color for the main surface.
    pub background: Color,
    /// Background for elevated surfaces (cards, modals).
    pub surface: Color,
    /// Primary accent color.
    pub primary: Color,
    /// Secondary accent color.
    pub secondary: Color,
    /// Text color on background.
    pub on_background: Color,
    /// Text color on surface.
    pub on_surface: Color,
    /// Text color on primary.
    pub on_primary: Color,
    /// Error / danger color.
    pub error: Color,
    /// Border color.
    pub border: Color,
    /// Disabled state overlay.
    pub disabled: Color,
    /// Default font size in pixels.
    pub font_size: f32,
    /// Small font size.
    pub font_size_small: f32,
    /// Large font size.
    pub font_size_large: f32,
    /// Default spacing in pixels.
    pub spacing: f64,
    /// Small spacing.
    pub spacing_small: f64,
    /// Large spacing.
    pub spacing_large: f64,
    /// Border radius for rounded elements.
    pub border_radius: f64,
    /// Button height.
    pub button_height: f64,
    /// Input field height.
    pub input_height: f64,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::rgb8(0x0F, 0x0F, 0x0F),
            surface: Color::rgb8(0x1A, 0x1A, 0x1A),
            primary: Color::rgb8(0x60, 0xA5, 0xFA),
            secondary: Color::rgb8(0x8A, 0x8A, 0x8A),
            on_background: Color::rgb8(0xD4, 0xD4, 0xD4),
            on_surface: Color::rgb8(0xD4, 0xD4, 0xD4),
            on_primary: Color::rgb8(0x0F, 0x0F, 0x0F),
            error: Color::rgb8(0xEF, 0x44, 0x44),
            border: Color::rgb8(0x3A, 0x3A, 0x3A),
            disabled: Color::rgb8(0x4A, 0x4A, 0x4A),
            font_size: 13.0,
            font_size_small: 11.0,
            font_size_large: 18.0,
            spacing: 8.0,
            spacing_small: 4.0,
            spacing_large: 16.0,
            border_radius: 6.0,
            button_height: 36.0,
            input_height: 32.0,
        }
    }
}

impl Theme {
    /// Returns a light theme variant.
    pub fn light() -> Self {
        Self {
            background: Color::rgb8(0xEF, 0xF1, 0xF5), // Catppuccin Latte base
            surface: Color::rgb8(0xCC, 0xD0, 0xDA),    // Catppuccin Latte surface0
            primary: Color::rgb8(0x1E, 0x66, 0xF5),    // Catppuccin Latte blue
            secondary: Color::rgb8(0x5C, 0x5F, 0x77),  // Catppuccin Latte subtext1
            on_background: Color::rgb8(0x4C, 0x4F, 0x69), // Catppuccin Latte text
            on_surface: Color::rgb8(0x4C, 0x4F, 0x69),
            on_primary: Color::rgb8(0xFF, 0xFF, 0xFF),
            error: Color::rgb8(0xD2, 0x0F, 0x39), // Catppuccin Latte red
            border: Color::rgb8(0xBC, 0xBF, 0xCE), // Catppuccin Latte surface1
            disabled: Color::rgb8(0x9C, 0xA0, 0xB0), // Catppuccin Latte overlay0
            ..Default::default()
        }
    }
}
