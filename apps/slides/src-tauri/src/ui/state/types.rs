use tench_ui::prelude::{Color, Point};

use super::SlideElement;

// ── Extended slide model (Phase 0.6) ────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SlideLayoutType {
    #[default]
    Blank,
    Title,
    TitleContent,
    TwoColumn,
    SectionHeader,
}

#[derive(Debug, Clone, Default)]
pub struct SlideBackground {
    pub color: Option<Color>,
    pub gradient_start: Option<Color>,
    pub gradient_end: Option<Color>,
    pub image_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SlideTransition {
    pub name: String,
    pub duration_ms: u32,
}

impl Default for SlideTransition {
    fn default() -> Self {
        Self {
            name: "none".into(),
            duration_ms: 500,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Slide {
    pub title: String,
    pub elements: Vec<SlideElement>,
    pub notes: String,
    // Phase 0.6: extended properties
    pub background: SlideBackground,
    pub layout_type: SlideLayoutType,
    pub transition: SlideTransition,
}

// ── Undo/Redo (Phase 1.8) ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub slides: Vec<Slide>,
    pub current_slide: usize,
    pub selected_element: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct UndoRedoStack {
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
}

impl UndoRedoStack {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, entry: HistoryEntry) {
        self.undo_stack.push(entry);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current: &HistoryEntry) -> Option<HistoryEntry> {
        if let Some(entry) = self.undo_stack.pop() {
            self.redo_stack.push(current.clone());
            return Some(entry);
        }
        None
    }

    pub fn redo(&mut self, current: &HistoryEntry) -> Option<HistoryEntry> {
        if let Some(entry) = self.redo_stack.pop() {
            self.undo_stack.push(current.clone());
            return Some(entry);
        }
        None
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

impl Default for UndoRedoStack {
    fn default() -> Self {
        Self::new()
    }
}

// ── Canvas interaction state (Phase 1) ─────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragMode {
    None,
    Move,
    Resize(ResizeHandle),
    Rotate,
    BoxSelect,
    Pan,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeHandle {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone)]
pub struct CanvasInteraction {
    pub mode: DragMode,
    pub start_pos: Point,
    pub element_origins: Vec<(usize, f64, f64, f64, f64)>,
    pub box_select_origin: Option<Point>,
}

impl Default for CanvasInteraction {
    fn default() -> Self {
        Self {
            mode: DragMode::None,
            start_pos: Point::ZERO,
            element_origins: Vec::new(),
            box_select_origin: None,
        }
    }
}

// ── Clipboard (Phase 8.3) ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClipboardData {
    pub elements: Vec<SlideElement>,
    pub source_slide_index: Option<usize>,
}

// ── Text editing state (Phase 2) ───────────────────────────────────

#[derive(Debug, Clone)]
pub struct TextEditState {
    pub editing: bool,
    pub element_index: usize,
    pub cursor_pos: usize,
    pub selection_start: Option<usize>,
    /// Font size for the text element being edited.
    pub font_size: f64,
    /// Whether bold is active.
    pub bold: bool,
    /// Whether italic is active.
    pub italic: bool,
    /// Text color.
    pub text_color: Color,
}

impl Default for TextEditState {
    fn default() -> Self {
        Self {
            editing: false,
            element_index: 0,
            cursor_pos: 0,
            selection_start: None,
            font_size: 16.0,
            bold: false,
            italic: false,
            text_color: Color::BLACK,
        }
    }
}

// ── Double-click detection (Phase 3) ────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct ClickTracker {
    pub last_click_time: Option<std::time::Instant>,
    pub last_click_pos: Option<Point>,
    pub click_count: u32,
}

impl ClickTracker {
    /// Record a click and return the current click count (1=single, 2=double, 3=triple).
    pub fn record_click(&mut self, pos: Point) -> u32 {
        let now = std::time::Instant::now();
        let is_double = match (self.last_click_time, self.last_click_pos) {
            (Some(last_time), Some(last_pos)) => {
                now.duration_since(last_time).as_millis() < 500
                    && (pos.x - last_pos.x).abs() < 5.0
                    && (pos.y - last_pos.y).abs() < 5.0
            }
            _ => false,
        };
        if is_double {
            self.click_count += 1;
        } else {
            self.click_count = 1;
        }
        self.last_click_time = Some(now);
        self.last_click_pos = Some(pos);
        self.click_count
    }
}

// ── Find/Replace state (Phase 8.1) ─────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct FindReplaceState {
    pub find_text: String,
    pub replace_text: String,
    pub current_match: Option<(usize, usize)>,
    pub matches: Vec<(usize, usize)>,
}

// ── Presentation mode state (Phase 6) ──────────────────────────────

#[derive(Debug, Clone)]
pub struct PresentationState {
    pub active: bool,
    pub current_slide: usize,
    pub start_time: Option<std::time::Instant>,
    pub slide_start_time: Option<std::time::Instant>,
    pub laser_pointer: bool,
    pub laser_pos: Point,
    pub auto_advance_ms: Option<u32>,
}

impl Default for PresentationState {
    fn default() -> Self {
        Self {
            active: false,
            current_slide: 0,
            start_time: None,
            slide_start_time: None,
            laser_pointer: false,
            laser_pos: Point::ZERO,
            auto_advance_ms: None,
        }
    }
}

// ── Auto-save state (Phase 8.2) ────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AutoSaveState {
    pub last_save_time: Option<std::time::Instant>,
    pub interval_secs: u64,
}

impl Default for AutoSaveState {
    fn default() -> Self {
        Self {
            last_save_time: None,
            interval_secs: 30,
        }
    }
}

// ── Print state (Phase 11) ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PrintSettings {
    pub paper_width: f64,
    pub paper_height: f64,
    pub orientation: PrintOrientation,
    pub margin_top: f64,
    pub margin_bottom: f64,
    pub margin_left: f64,
    pub margin_right: f64,
    pub slides_per_page: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrintOrientation {
    Landscape,
    Portrait,
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self {
            paper_width: 11.0,
            paper_height: 8.5,
            orientation: PrintOrientation::Landscape,
            margin_top: 0.5,
            margin_bottom: 0.5,
            margin_left: 0.5,
            margin_right: 0.5,
            slides_per_page: 1,
        }
    }
}

// ── Zoom state (Phase 9.6) ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ZoomState {
    pub level: f64,
    pub pan_x: f64,
    pub pan_y: f64,
}

impl Default for ZoomState {
    fn default() -> Self {
        Self {
            level: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }
}

// ── Alignment enum (Phase 1.5) ─────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    Left,
    Right,
    Top,
    Bottom,
    CenterH,
    CenterV,
    DistributeH,
    DistributeV,
}

// ── License modal state ───────────────────────────────────────────

/// State for the License activation modal (License toolbar button).
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct LicenseModalState {
    /// Current text in the license key input field.
    pub license_key_input: String,
    /// Last status message shown below the input ("", "Activating...",
    /// "Activated", or an error string).
    pub status_message: String,
    /// True while an activation request is in flight.
    pub busy: bool,
}
