use tench_ui::prelude::Color;

// ── Extended element model (Phase 0.5) ─────────────────────────────

#[derive(Debug, Clone)]
pub struct ElementBorder {
    pub color: Color,
    pub width: f64,
    pub style: BorderStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BorderStyle {
    Solid,
    Dashed,
    Dotted,
    #[default]
    None,
}

#[derive(Debug, Clone)]
pub struct ElementShadow {
    pub color: Color,
    pub offset_x: f64,
    pub offset_y: f64,
    pub blur: f64,
}

#[derive(Debug, Clone)]
pub struct SlideElement {
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub text: Option<String>,
    pub fill: Option<Color>,
    pub rotation: f64,
    pub opacity: f64,
    pub border: Option<ElementBorder>,
    pub shadow: Option<ElementShadow>,
    pub z_index: i32,
    pub group_id: Option<String>,
}

impl Default for SlideElement {
    fn default() -> Self {
        Self {
            kind: "text".into(),
            x: 0.0,
            y: 0.0,
            w: 200.0,
            h: 60.0,
            text: None,
            fill: None,
            rotation: 0.0,
            opacity: 1.0,
            border: None,
            shadow: None,
            z_index: 0,
            group_id: None,
        }
    }
}

impl SlideElement {
    pub fn new_text(text: impl Into<String>, x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            kind: "text".into(),
            x,
            y,
            w,
            h,
            text: Some(text.into()),
            ..Default::default()
        }
    }

    pub fn new_title(text: impl Into<String>, x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            kind: "title".into(),
            x,
            y,
            w,
            h,
            text: Some(text.into()),
            ..Default::default()
        }
    }

    pub fn new_subtitle(text: impl Into<String>, x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            kind: "subtitle".into(),
            x,
            y,
            w,
            h,
            text: Some(text.into()),
            ..Default::default()
        }
    }

    pub fn new_rect(x: f64, y: f64, w: f64, h: f64, fill: Color) -> Self {
        Self {
            kind: "rectangle".into(),
            x,
            y,
            w,
            h,
            fill: Some(fill),
            ..Default::default()
        }
    }
}
