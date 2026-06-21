use tench_document_core::{
    Alignment, Comment, CursorState, DocumentEngine, Margins, OfficeArtifact, Orientation,
    PageSetup, PaperSize, SearchMatch, TenchDocument, TrackedChange,
};

/// Holds all per-document state for a single open tab.
///
/// Each tab owns its own engine, document, cursor, scroll position, and dirty
/// flag so that switching between tabs is an instant swap with no data loss.
#[allow(dead_code)]
pub struct DocumentSession {
    pub engine: DocumentEngine,
    pub document: TenchDocument,
    pub cursor: CursorState,
    pub scroll_y: f64,
    pub dirty: bool,
    pub title: String,
}

impl DocumentSession {
    /// Create a new session from a document with the given title.
    pub fn new(document: TenchDocument, title: String) -> Self {
        let engine = DocumentEngine::new(document.clone());
        Self {
            engine,
            document,
            cursor: CursorState::default(),
            scroll_y: 0.0,
            dirty: false,
            title,
        }
    }
}

/// Cached layout information for the current document.
///
/// Recomputed only when the document content or zoom level changes.
#[derive(Clone)]
pub struct DocumentLayoutCache {
    /// Hash of the document content to detect changes.
    content_hash: u64,
    /// Zoom level used for the cached layout.
    zoom: f64,
    /// Pre-computed page map (which block starts on which page).
    page_map: Vec<PageMapEntry>,
    /// Total estimated content height.
    total_content_h: f64,
    /// Number of pages.
    num_pages: usize,
}

/// Entry describing which blocks belong to a page.
#[derive(Clone)]
pub struct PageMapEntry {
    /// Index of the first block on this page.
    pub start_block: usize,
}

impl DocumentLayoutCache {
    pub fn new() -> Self {
        Self {
            content_hash: 0,
            zoom: 0.0,
            page_map: Vec::new(),
            total_content_h: 0.0,
            num_pages: 1,
        }
    }

    /// Returns true if the cache is valid for the given document and zoom.
    pub fn is_valid(&self, doc: &TenchDocument, zoom: f64) -> bool {
        self.content_hash == Self::hash_document(doc) && (self.zoom - zoom).abs() < 0.001
    }

    /// Invalidate the cache (e.g. when content changes).
    pub fn invalidate(&mut self) {
        self.content_hash = 0;
        self.zoom = 0.0;
    }

    /// Update the cache with new layout data.
    pub fn update(
        &mut self,
        content_hash: u64,
        zoom: f64,
        page_map: Vec<PageMapEntry>,
        total_content_h: f64,
        num_pages: usize,
    ) {
        self.content_hash = content_hash;
        self.zoom = zoom;
        self.page_map = page_map;
        self.total_content_h = total_content_h;
        self.num_pages = num_pages;
    }

    /// Get the cached page map.
    pub fn page_map(&self) -> &[PageMapEntry] {
        &self.page_map
    }

    /// Get the cached page count.
    pub fn num_pages(&self) -> usize {
        self.num_pages
    }

    /// Get the cached total content height.
    pub fn total_content_h(&self) -> f64 {
        self.total_content_h
    }

    /// Simple hash of document content for change detection.
    pub fn hash_document(doc: &TenchDocument) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        doc.content.len().hash(&mut hasher);
        // Hash block types and text lengths for fast change detection
        for block in &doc.content {
            std::mem::discriminant(block).hash(&mut hasher);
            match block {
                tench_document_core::BlockNode::Paragraph { content, .. }
                | tench_document_core::BlockNode::Heading { content, .. } => {
                    for node in content {
                        match node {
                            tench_document_core::InlineNode::Text { text, .. } => {
                                text.len().hash(&mut hasher);
                            }
                            tench_document_core::InlineNode::Link { text, .. } => {
                                text.len().hash(&mut hasher);
                            }
                            tench_document_core::InlineNode::HardBreak => {}
                            tench_document_core::InlineNode::InlineImage { alt, .. } => {
                                alt.as_ref().map(|s| s.len()).hash(&mut hasher);
                            }
                        }
                    }
                }
                tench_document_core::BlockNode::CodeBlock { code, .. } => {
                    code.len().hash(&mut hasher);
                }
                tench_document_core::BlockNode::BulletList { items }
                | tench_document_core::BlockNode::OrderedList { items, .. } => {
                    items.len().hash(&mut hasher);
                }
                tench_document_core::BlockNode::TaskList { items } => {
                    items.len().hash(&mut hasher);
                }
                tench_document_core::BlockNode::BlockQuote { content } => {
                    content.len().hash(&mut hasher);
                }
                tench_document_core::BlockNode::Table { rows } => {
                    rows.len().hash(&mut hasher);
                }
                tench_document_core::BlockNode::Image { alt, .. } => {
                    alt.as_ref().map(|s| s.len()).hash(&mut hasher);
                }
                tench_document_core::BlockNode::HorizontalRule
                | tench_document_core::BlockNode::PageBreak => {}
                tench_document_core::BlockNode::Footnote { content, .. } => {
                    content.len().hash(&mut hasher);
                }
            }
        }
        hasher.finish()
    }
}

impl Default for DocumentLayoutCache {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DocumentLayoutCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentLayoutCache")
            .field("content_hash", &self.content_hash)
            .field("zoom", &self.zoom)
            .field("num_pages", &self.num_pages)
            .field("page_map_len", &self.page_map.len())
            .field("total_content_h", &self.total_content_h)
            .finish()
    }
}

pub const MENU_BAR_H: f64 = 36.0;
pub const TOOLBAR_H: f64 = 48.0;
pub const TITLE_ROW_H: f64 = 48.0;
pub const RULER_H: f64 = 24.0;
pub const STATUS_BAR_H: f64 = 28.0;
pub const THUMB_PANEL_W: f64 = 140.0;
pub const STYLE_PANEL_W: f64 = 304.0;
pub const PAGE_MARGIN_X: f64 = 24.0;
pub const PAGE_MARGIN_Y: f64 = 24.0;
pub const PAGE_GAP: f64 = 24.0;
pub const HEADER_H: f64 = 30.0;
pub const FOOTER_H: f64 = 30.0;

/// Legacy constants kept for backwards compatibility with rendering code
/// that hasn't been migrated to PageSetup-aware dimensions yet.
pub const PAGE_W: f64 = 794.0;
pub const PAGE_H: f64 = 1123.0;
pub const PAGE_PAD_X: f64 = 96.0;

/// Which toolbar dropdown is currently open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarDropdown {
    ParagraphStyle,
    FontSize,
    FontFamily,
    TableGrid,
    ColorPicker,
    MarkPicker,
}

/// Paragraph style options for the dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParagraphStyle {
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    BlockQuote,
    CodeBlock,
}

impl ParagraphStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::Paragraph => "Paragraph",
            Self::Heading1 => "Heading 1",
            Self::Heading2 => "Heading 2",
            Self::Heading3 => "Heading 3",
            Self::Heading4 => "Heading 4",
            Self::Heading5 => "Heading 5",
            Self::Heading6 => "Heading 6",
            Self::BlockQuote => "Block Quote",
            Self::CodeBlock => "Code Block",
        }
    }

    pub fn all() -> &'static [ParagraphStyle] {
        &[
            Self::Paragraph,
            Self::Heading1,
            Self::Heading2,
            Self::Heading3,
            Self::Heading4,
            Self::Heading5,
            Self::Heading6,
            Self::BlockQuote,
            Self::CodeBlock,
        ]
    }
}

/// Available font sizes for the dropdown.
pub const FONT_SIZES: &[f32] = &[
    8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 32.0, 36.0, 48.0, 72.0,
];

/// Available font families for the dropdown.
pub const FONT_FAMILIES: &[&str] = &[
    "Default",
    "Arial",
    "Times New Roman",
    "Courier New",
    "Georgia",
    "Verdana",
    "Helvetica",
    "Tahoma",
    "Trebuchet MS",
    "Palatino",
    "Garamond",
    "Comic Sans MS",
    "Impact",
];

/// Available colors for the color picker.
pub const COLOR_PALETTE: &[&str] = &[
    "#000000", "#434343", "#666666", "#999999", "#B7B7B7", "#CCCCCC", "#D9D9D9", "#EFEFEF",
    "#F3F3F3", "#FFFFFF", "#980000", "#FF0000", "#FF9900", "#FFFF00", "#00FF00", "#00FFFF",
    "#4A86E8", "#0000FF", "#9900FF", "#FF00FF", "#E6B8AF", "#F4CCCC", "#FCE5CD", "#FFF2CC",
    "#D9EAD3", "#D0E0E3", "#C9DAF8", "#CFE2F3", "#D9D2E9", "#EAD1DC",
];

/// State for the link insertion modal.
#[derive(Debug, Clone, Default)]
pub struct LinkModalState {
    pub url: String,
    pub cursor_pos: usize,
}

/// State for the table grid picker.
#[derive(Debug, Clone, Copy, Default)]
pub struct TableGridState {
    pub hover_row: usize,
    pub hover_col: usize,
}

/// State for the find/replace modal.
#[derive(Debug, Clone, Default)]
pub struct FindReplaceState {
    pub query: String,
    pub replacement: String,
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub cursor_pos: usize,
    pub matches: Vec<SearchMatch>,
    pub current_match_idx: Option<usize>,
    pub show_replace: bool,
}

/// State for the page setup dialog.
#[derive(Debug, Clone)]
pub struct PageSetupDialogState {
    pub paper_size: PaperSize,
    pub orientation: Orientation,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    /// Which margin field is currently being edited (0=top, 1=bottom, 2=left, 3=right).
    pub editing_margin_field: Option<usize>,
    /// Temporary text buffer for the margin field being edited.
    pub margin_edit_buffer: String,
}

impl Default for PageSetupDialogState {
    fn default() -> Self {
        let setup = PageSetup::default();
        Self {
            paper_size: setup.paper_size,
            orientation: setup.orientation,
            margin_top: setup.margins.top,
            margin_right: setup.margins.right,
            margin_bottom: setup.margins.bottom,
            margin_left: setup.margins.left,
            editing_margin_field: None,
            margin_edit_buffer: String::new(),
        }
    }
}

impl PageSetupDialogState {
    pub fn from_page_setup(setup: &PageSetup) -> Self {
        Self {
            paper_size: setup.paper_size,
            orientation: setup.orientation,
            margin_top: setup.margins.top,
            margin_right: setup.margins.right,
            margin_bottom: setup.margins.bottom,
            margin_left: setup.margins.left,
            editing_margin_field: None,
            margin_edit_buffer: String::new(),
        }
    }

    pub fn to_page_setup(&self) -> PageSetup {
        PageSetup {
            paper_size: self.paper_size,
            orientation: self.orientation,
            margins: Margins {
                top: self.margin_top,
                right: self.margin_right,
                bottom: self.margin_bottom,
                left: self.margin_left,
            },
        }
    }
}

/// Which ruler marker is being dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerDragTarget {
    LeftMargin,
    RightMargin,
    IndentLeft,
    IndentRight,
    IndentFirstLine,
}

/// Available paper sizes for the page setup dialog.
pub const PAPER_SIZES: &[PaperSize] = &[
    PaperSize::A4,
    PaperSize::Letter,
    PaperSize::Legal,
    PaperSize::A3,
    PaperSize::B5,
];

/// Paper size display names for the dialog.
pub fn paper_size_label(size: &PaperSize) -> &'static str {
    match size {
        PaperSize::A4 => "A4 (210 x 297 mm)",
        PaperSize::A3 => "A3 (297 x 420 mm)",
        PaperSize::Letter => "Letter (8.5 x 11 in)",
        PaperSize::Legal => "Legal (8.5 x 14 in)",
        PaperSize::Tabloid => "Tabloid (11 x 17 in)",
        PaperSize::B5 => "B5 (176 x 250 mm)",
        PaperSize::Custom { .. } => "Custom",
    }
}

/// An entry in the version history.
#[derive(Debug, Clone)]
pub struct VersionEntry {
    /// Human-readable timestamp for display.
    pub timestamp_label: String,
    pub path: String,
    pub size_bytes: u64,
    pub label: String,
}

/// An open tab in the tab bar.
#[derive(Debug, Clone)]
pub struct TabInfo {
    pub title: String,
    pub dirty: bool,
}

/// Type of context menu to display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ContextMenuType {
    Text,
    Image,
    TableCell,
    Tab,
}

/// State for the right-click context menu.
#[derive(Debug, Clone)]
pub struct ContextMenuState {
    /// Position where the menu was opened (window coordinates).
    pub x: f64,
    pub y: f64,
    /// Type of context menu to show.
    pub menu_type: ContextMenuType,
    /// Index of the hovered menu item.
    pub hovered_item: Option<usize>,
}

impl ContextMenuState {
    /// Return the menu items for this context menu type.
    pub fn items(&self) -> &[&str] {
        match self.menu_type {
            ContextMenuType::Text => &[
                "Cut",
                "Copy",
                "Paste",
                "Add Comment",
                "Insert Link",
                "Clear Formatting",
            ],
            ContextMenuType::Image => &["Replace Image", "Remove"],
            ContextMenuType::TableCell => &[
                "Insert Row Above",
                "Insert Row Below",
                "Insert Column Left",
                "Insert Column Right",
                "Delete Row",
                "Delete Column",
                "Delete Table",
            ],
            ContextMenuType::Tab => &["Close", "Close Others", "Close All"],
        }
    }
}

/// State for the comment input modal.
#[derive(Debug, Clone, Default)]
pub struct CommentModalState {
    pub text: String,
    pub cursor_pos: usize,
}

/// Which sidebar tab is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum SidebarTab {
    #[default]
    Style,
    Navigate,
    Ai,
}

/// State for the print preview modal.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PrintPreviewState {
    pub page_index: usize,
    pub page_count: usize,
    pub zoom: f64,
}

impl Default for PrintPreviewState {
    fn default() -> Self {
        Self {
            page_index: 0,
            page_count: 1,
            zoom: 60.0,
        }
    }
}

/// State for the goto page/line modal.
#[derive(Debug, Clone)]
pub struct GotoModalState {
    pub input: String,
    pub cursor_pos: usize,
    /// Whether the user is going to a page or line.
    pub mode: GotoMode,
}

impl Default for GotoModalState {
    fn default() -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            mode: GotoMode::Page,
        }
    }
}

/// Mode for the goto modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GotoMode {
    Page,
    Line,
}

/// State for the special character modal.
#[derive(Debug, Clone, Default)]
pub struct SpecialCharModalState {
    /// Currently selected category index.
    pub category_idx: usize,
}

/// Common Unicode characters organized by category for the special character picker.
pub const SPECIAL_CHAR_CATEGORIES: &[(&str, &[char])] = &[
    (
        "Common Symbols",
        &[
            '©', '®', '™', '°', '±', '×', '÷', '≠', '≤', '≥', '∞', '√', '∑', '∏', '∫', '∂', '∆',
            '∇', '♥', '♦', '♣', '♠', '★', '☆', '✓', '✗', '•', '…', '—', '–',
        ],
    ),
    (
        "Arrows",
        &[
            '←', '→', '↑', '↓', '↔', '↕', '⇐', '⇒', '⇑', '⇓', '⇔', '⇕', '➜', '➤', '➡', '⬅', '⬆',
            '⬇', '↩', '↪', '↰', '↱', '↲', '↳', '↴', '↵', '↶', '↷', '↺', '↻',
        ],
    ),
    (
        "Math",
        &[
            '∀', '∃', '∅', '∈', '∉', '∋', '∏', '∑', '−', '∗', '∘', '√', '∝', '∞', '∠', '∧', '∨',
            '∩', '∪', '∫', '≈', '≠', '≡', '≤', '≥', '⊂', '⊃', '⊄', '⊆', '⊇',
        ],
    ),
    (
        "Currency",
        &[
            '$', '€', '£', '¥', '₩', '₹', '₽', '₿', '¢', '₱', '₦', '₫', '₭', '₮', '₯', '₰', '₱',
            '₲', '₳', '₴', '₵', '₶', '₷', '₸', '₹', '₺', '₻', '₼', '₽', '₾',
        ],
    ),
    (
        "Latin Extended",
        &[
            'À', 'Á', 'Â', 'Ã', 'Ä', 'Å', 'Æ', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï', 'Ð',
            'Ñ', 'Ò', 'Ó', 'Ô', 'Õ', 'Ö', 'Ø', 'Ù', 'Ú', 'Û', 'Ü', 'Ý', 'Þ',
        ],
    ),
    (
        "Punctuation",
        &[
            '¡', '¿', '«', '»', '‹', '›', '„', '‟', '‚', '‛', '「', '」', '『', '』', '【', '】',
            '〈', '〉', '《', '》', '〔', '〕', '〖', '〗', '〘', '〙', '〚', '〛', '§', '¶',
        ],
    ),
];

/// State for the image resize drag operation.
#[derive(Debug, Clone, Copy)]
pub struct ImageResizeDrag {
    /// Block index of the image being resized.
    pub block_idx: usize,
    /// Which handle is being dragged (0-3: corners TL, TR, BL, BR).
    pub handle: usize,
    /// Starting width of the image.
    pub start_width: f64,
    /// Starting height of the image.
    pub start_height: f64,
    /// Pointer position at drag start.
    pub start_x: f64,
    pub start_y: f64,
    /// Current preview width during drag.
    pub current_width: f64,
    /// Current preview height during drag.
    pub current_height: f64,
}

#[derive(Debug, Clone)]
pub struct DocsState {
    pub(super) artifact: OfficeArtifact,
    pub(super) document: TenchDocument,
    pub(super) cursor: CursorState,
    pub(super) dirty: bool,
    pub(super) last_saved_text: String,
    pub(super) status: String,
    pub show_thumbnails: bool,
    pub show_style_panel: bool,
    pub show_comments: bool,
    pub track_changes: bool,
    pub active_modal: Option<String>,
    pub toast: Option<(String, f64)>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub superscript: bool,
    pub subscript: bool,
    pub current_alignment: Alignment,
    pub selection: Option<tench_document_core::SelectionRange>,
    /// Anchor position where the current shift-arrow selection started.
    /// Used to correctly extend the selection from the original anchor
    /// rather than from the previous cursor position.
    pub selection_anchor: Option<CursorState>,
    pub zoom: f64,
    pub hovered_btn: Option<usize>,
    pub hovered_menu_item: Option<usize>,
    pub cursor_visible: bool,
    pub page_count: usize,
    pub current_page: usize,
    pub word_count: usize,
    pub language: String,
    pub open_dropdown: Option<ToolbarDropdown>,
    pub current_font_size: f32,
    pub current_paragraph_style: ParagraphStyle,
    pub link_modal: Option<LinkModalState>,
    pub table_grid: TableGridState,
    pub selected_text_color: Option<String>,
    pub selected_bg_color: Option<String>,
    pub scroll_y: f64,
    pub page_setup_dialog: Option<PageSetupDialogState>,
    pub editing_header: bool,
    pub editing_footer: bool,
    pub ruler_drag: Option<RulerDragTarget>,
    // Search state
    pub find_replace: Option<FindReplaceState>,
    // Comments (mirrored from engine)
    pub comments: Vec<Comment>,
    // Autosave
    pub last_autosave_ts: f64,
    pub autosave_interval_ms: f64,
    // Keyboard state tracking
    pub ctrl_pressed: bool,
    // Version history
    pub version_history: Vec<VersionEntry>,
    // Tracked changes (mirrored from engine)
    pub tracked_changes: Vec<TrackedChange>,
    // Open tabs (multi-document)
    pub open_tabs: Vec<TabInfo>,
    pub active_tab_idx: usize,
    /// Cached document layout to avoid recomputation every frame.
    pub layout_cache: DocumentLayoutCache,
    /// Last known window size, updated during paint.
    pub last_window_size: (f64, f64),
    /// Hovered dropdown item for hover highlight (dropdown type, item index).
    pub hovered_dropdown_item: Option<(ToolbarDropdown, usize)>,
    /// Pending async file operation (e.g. "open", "save_as", "export_as", "insert_image").
    #[allow(dead_code)]
    pub pending_file_action: Option<String>,
    /// Right-click context menu state.
    pub context_menu: Option<ContextMenuState>,
    /// Comment modal state.
    pub comment_modal: Option<CommentModalState>,
    /// Header text buffer for header editing.
    pub header_text: String,
    /// Footer text buffer for footer editing.
    pub footer_text: String,
    /// Image resize drag state.
    pub image_resize_drag: Option<ImageResizeDrag>,
    /// Index of the currently targeted/selected image block.
    pub targeted_image_block: Option<usize>,
    /// Print preview modal state.
    pub print_preview: Option<PrintPreviewState>,
    /// Current font family.
    pub current_font_family: String,
    /// Custom color hex input for the color picker.
    pub custom_color_input: String,
    /// Active sidebar tab.
    pub sidebar_tab: SidebarTab,
    /// Goto modal state.
    pub goto_modal: Option<GotoModalState>,
    /// Word count modal visibility.
    pub word_count_modal: bool,
    /// Special character modal state.
    pub special_char_modal: Option<SpecialCharModalState>,
    /// Whether the Comments section in the sidebar is collapsed.
    pub comments_collapsed: bool,
    /// Whether the Version History section in the sidebar is collapsed.
    pub version_history_collapsed: bool,
    /// Tooltip text for the currently hovered toolbar button.
    pub hovered_tooltip: Option<String>,
    /// X position of the hovered toolbar button for tooltip placement.
    pub hovered_tooltip_x: f64,
    /// Whether undo is available (engine has undo history).
    pub undo_available: bool,
    /// Whether redo is available (engine has redo history).
    pub redo_available: bool,
    /// Plain text content of the clipboard (mirrored from engine clipboard).
    pub clipboard_text: String,
    /// Number of TDM nodes in the clipboard content.
    pub clipboard_node_count: usize,
    /// Whether the platform window is currently maximized (caption glyph).
    pub window_maximized: bool,
    /// Caption button currently under the pointer, if any (hover feedback).
    pub window_control_hovered: Option<tench_ui::WindowControl>,
    /// License: true when the local store has a valid, non-expired
    /// device_token. The menu bar's notification label hides when this is
    /// true. Synced from LicenseStore on each frame.
    pub license_active: bool,
    /// License: true when the update scheduler has seen a newer manifest
    /// version than the running binary. The notification label cycles two
    /// messages when both `license_active == false` and this is true.
    pub update_available: bool,
    /// License modal state — opens from the License menu, hosts the
    /// license_key input field and shows status text.
    pub license_modal: Option<LicenseModalState>,
}

/// State for the License activation modal (License menu → Activate License).
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
