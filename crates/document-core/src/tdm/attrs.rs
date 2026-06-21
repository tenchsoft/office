use serde::{Deserialize, Serialize};

/// Text alignment within a paragraph.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Page orientation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

/// Standard paper sizes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaperSize {
    #[default]
    A4,
    A3,
    Letter,
    Legal,
    Tabloid,
    B5,
    Custom {
        width_mm: f32,
        height_mm: f32,
    },
}

impl PaperSize {
    /// Return the physical dimensions of this paper size in millimeters,
    /// ignoring orientation.
    pub fn dimensions_mm(self) -> (f32, f32) {
        match self {
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::A3 => (297.0, 420.0),
            PaperSize::Letter => (215.9, 279.4),
            PaperSize::Legal => (215.9, 355.6),
            PaperSize::Tabloid => (279.4, 431.8),
            PaperSize::B5 => (176.0, 250.0),
            PaperSize::Custom {
                width_mm,
                height_mm,
            } => (width_mm, height_mm),
        }
    }
}

/// Page margins in millimeters.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Margins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for Margins {
    fn default() -> Self {
        Margins {
            top: 25.4,
            right: 25.4,
            bottom: 25.4,
            left: 25.4,
        }
    }
}

/// Page layout configuration.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PageSetup {
    #[serde(default)]
    pub paper_size: PaperSize,
    #[serde(default)]
    pub orientation: Orientation,
    #[serde(default)]
    pub margins: Margins,
}

impl PageSetup {
    /// Return the page width and height in pixels at 96 DPI.
    /// Orientation is applied so Landscape swaps width/height.
    pub fn page_size_px(&self) -> (f64, f64) {
        let (w_mm, h_mm) = self.paper_size.dimensions_mm();
        let mm_to_px = 96.0 / 25.4;
        let (w, h) = match self.orientation {
            Orientation::Portrait => (w_mm as f64 * mm_to_px, h_mm as f64 * mm_to_px),
            Orientation::Landscape => (h_mm as f64 * mm_to_px, w_mm as f64 * mm_to_px),
        };
        (w, h)
    }

    /// Return the content area width in pixels after subtracting left+right margins.
    pub fn content_width_px(&self) -> f64 {
        let (page_w, _) = self.page_size_px();
        let mm_to_px = 96.0 / 25.4;
        page_w - (self.margins.left + self.margins.right) as f64 * mm_to_px
    }

    /// Return the content area height in pixels after subtracting top+bottom margins.
    pub fn content_height_px(&self) -> f64 {
        let (_, page_h) = self.page_size_px();
        let mm_to_px = 96.0 / 25.4;
        page_h - (self.margins.top + self.margins.bottom) as f64 * mm_to_px
    }
}

/// Paragraph-level formatting attributes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParagraphAttrs {
    #[serde(default)]
    pub alignment: Alignment,
    #[serde(default)]
    pub indent_left: f32,
    #[serde(default)]
    pub indent_right: f32,
    #[serde(default)]
    pub indent_first_line: f32,
    #[serde(default)]
    pub line_height: Option<f32>,
    #[serde(default)]
    pub space_before: f32,
    #[serde(default)]
    pub space_after: f32,
    #[serde(default)]
    pub style_id: Option<String>,
}

impl Default for ParagraphAttrs {
    fn default() -> Self {
        ParagraphAttrs {
            alignment: Alignment::Left,
            indent_left: 0.0,
            indent_right: 0.0,
            indent_first_line: 0.0,
            line_height: None,
            space_before: 0.0,
            space_after: 0.0,
            style_id: None,
        }
    }
}
