use tench_ui::prelude::Color;

// ── Theme system (Phase 7.2) ────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SlideTheme {
    pub name: String,
    pub primary_color: Color,
    pub secondary_color: Color,
    pub accent_color: Color,
    pub background_color: Color,
    pub text_color: Color,
    pub title_font: String,
    pub body_font: String,
    pub heading_size: f64,
    pub body_size: f64,
}

impl Default for SlideTheme {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            primary_color: Color::rgb8(0x33, 0x33, 0x33),
            secondary_color: Color::rgb8(0x66, 0x66, 0x66),
            accent_color: Color::rgb8(0x60, 0xA5, 0xFA),
            background_color: Color::WHITE,
            text_color: Color::rgb8(0x33, 0x33, 0x33),
            title_font: "sans-serif".into(),
            body_font: "sans-serif".into(),
            heading_size: 28.0,
            body_size: 16.0,
        }
    }
}

impl SlideTheme {
    pub fn dark() -> Self {
        Self {
            name: "Dark".into(),
            primary_color: Color::WHITE,
            secondary_color: Color::rgb8(0xAA, 0xAA, 0xAA),
            accent_color: Color::rgb8(0x60, 0xA5, 0xFA),
            background_color: Color::rgb8(0x1E, 0x1E, 0x2E),
            text_color: Color::WHITE,
            title_font: "sans-serif".into(),
            body_font: "sans-serif".into(),
            heading_size: 28.0,
            body_size: 16.0,
        }
    }

    pub fn blue_professional() -> Self {
        Self {
            name: "Blue Professional".into(),
            primary_color: Color::rgb8(0x1A, 0x56, 0xDB),
            secondary_color: Color::rgb8(0x3B, 0x82, 0xF6),
            accent_color: Color::rgb8(0xEF, 0x44, 0x44),
            background_color: Color::WHITE,
            text_color: Color::rgb8(0x1E, 0x29, 0x3B),
            title_font: "sans-serif".into(),
            body_font: "sans-serif".into(),
            heading_size: 28.0,
            body_size: 16.0,
        }
    }

    pub fn warm_earth() -> Self {
        Self {
            name: "Warm Earth".into(),
            primary_color: Color::rgb8(0x92, 0x43, 0x1A),
            secondary_color: Color::rgb8(0xC2, 0x7A, 0x3F),
            accent_color: Color::rgb8(0xD9, 0x77, 0x06),
            background_color: Color::rgb8(0xFE, 0xF3, 0xE2),
            text_color: Color::rgb8(0x45, 0x1A, 0x03),
            title_font: "sans-serif".into(),
            body_font: "sans-serif".into(),
            heading_size: 28.0,
            body_size: 16.0,
        }
    }

    pub fn all_themes() -> Vec<SlideTheme> {
        vec![
            Self::default(),
            Self::dark(),
            Self::blue_professional(),
            Self::warm_earth(),
        ]
    }
}
