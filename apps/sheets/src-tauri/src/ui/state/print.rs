// ---------------------------------------------------------------------------
// 10.1 Page setup types
// ---------------------------------------------------------------------------

use super::cell::CellRange;

/// Standard paper sizes for printing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PaperSize {
    #[default]
    A4,
    Letter,
    Legal,
    Tabloid,
    A3,
    A5,
    B5,
    Envelope10,
    EnvelopeDL,
    /// Custom size in millimetres (width, height).
    Custom(f64, f64),
}

impl PaperSize {
    /// Return (width_mm, height_mm) in portrait orientation.
    pub fn dimensions_mm(&self) -> (f64, f64) {
        match self {
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::Letter => (215.9, 279.4),
            PaperSize::Legal => (215.9, 355.6),
            PaperSize::Tabloid => (279.4, 431.8),
            PaperSize::A3 => (297.0, 420.0),
            PaperSize::A5 => (148.0, 210.0),
            PaperSize::B5 => (176.0, 250.0),
            PaperSize::Envelope10 => (104.8, 241.3),
            PaperSize::EnvelopeDL => (110.0, 220.0),
            PaperSize::Custom(w, h) => (*w, *h),
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &str {
        match self {
            PaperSize::A4 => "A4",
            PaperSize::Letter => "Letter",
            PaperSize::Legal => "Legal",
            PaperSize::Tabloid => "Tabloid",
            PaperSize::A3 => "A3",
            PaperSize::A5 => "A5",
            PaperSize::B5 => "B5",
            PaperSize::Envelope10 => "Envelope #10",
            PaperSize::EnvelopeDL => "Envelope DL",
            PaperSize::Custom(_, _) => "Custom",
        }
    }
}

/// Page orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

impl Orientation {
    pub fn label(&self) -> &str {
        match self {
            Orientation::Portrait => "Portrait",
            Orientation::Landscape => "Landscape",
        }
    }
}

/// Page margins in millimetres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
    pub header: f64,
    pub footer: f64,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: 19.05, // 0.75 in
            bottom: 19.05,
            left: 17.78, // 0.7 in
            right: 17.78,
            header: 12.7, // 0.5 in
            footer: 12.7,
        }
    }
}

/// Print scaling mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scaling {
    /// Scale by a fixed percentage (e.g. 100 = 100%).
    Percentage(f64),
    /// Fit to a number of pages. `None` means "auto".
    FitToPages {
        width: Option<usize>,
        height: Option<usize>,
    },
}

impl Default for Scaling {
    fn default() -> Self {
        Scaling::Percentage(100.0)
    }
}

/// Complete page setup for a sheet.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PageSetup {
    pub paper_size: PaperSize,
    pub orientation: Orientation,
    pub margins: Margins,
    pub scaling: Scaling,
    pub print_area: Option<CellRange>,
    /// Repeat title rows (start, end) on every printed page.
    pub print_titles_rows: Option<(usize, usize)>,
    /// Repeat title columns (start, end) on every printed page.
    pub print_titles_cols: Option<(usize, usize)>,
    pub repeat_header: bool,
    pub gridlines_print: bool,
    pub row_col_headers_print: bool,
    pub center_horizontally: bool,
    pub center_vertically: bool,
    pub header_left: String,
    pub header_center: String,
    pub header_right: String,
    pub footer_left: String,
    pub footer_center: String,
    pub footer_right: String,
}

impl PageSetup {
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// 10.2 Print preview types
// ---------------------------------------------------------------------------

/// A single page computed for print preview.
#[derive(Debug, Clone)]
pub struct PrintPage {
    /// Row range on this page (start, end inclusive).
    pub rows: (usize, usize),
    /// Column range on this page (start, end inclusive).
    pub cols: (usize, usize),
    /// 1-based page number.
    pub page_number: usize,
}

/// State for the print preview modal.
#[derive(Debug, Clone)]
pub struct PrintPreviewState {
    pub visible: bool,
    pub pages: Vec<PrintPage>,
    pub current_page: usize,
    pub zoom: f64,
}

impl Default for PrintPreviewState {
    fn default() -> Self {
        Self {
            visible: false,
            pages: Vec::new(),
            current_page: 0,
            zoom: 1.0,
        }
    }
}

impl PrintPreviewState {
    /// Navigate to the next page.
    pub fn next_page(&mut self) -> bool {
        if self.current_page + 1 < self.pages.len() {
            self.current_page += 1;
            true
        } else {
            false
        }
    }

    /// Navigate to the previous page.
    pub fn prev_page(&mut self) -> bool {
        if self.current_page > 0 {
            self.current_page -= 1;
            true
        } else {
            false
        }
    }

    /// Zoom in by 25%.
    pub fn zoom_in(&mut self) -> bool {
        let new_zoom = (self.zoom + 0.25).min(3.0);
        if (new_zoom - self.zoom).abs() > f64::EPSILON {
            self.zoom = new_zoom;
            true
        } else {
            false
        }
    }

    /// Zoom out by 25%.
    pub fn zoom_out(&mut self) -> bool {
        let new_zoom = (self.zoom - 0.25).max(0.25);
        if (new_zoom - self.zoom).abs() > f64::EPSILON {
            self.zoom = new_zoom;
            true
        } else {
            false
        }
    }
}
