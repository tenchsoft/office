// ---------------------------------------------------------------------------
// Menu types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct MenuState {
    /// Which top-level menu is open (index).
    pub open_menu: Option<usize>,
    /// Which submenu is hovered (menu_idx, submenu_idx).
    pub hovered_submenu: Option<(usize, usize)>,
}

/// A single menu item definition.
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: String,
    pub action: MenuAction,
    pub enabled: bool,
    /// Submenu items (if any).
    pub submenu: Vec<MenuItem>,
}

impl MenuItem {
    pub fn action(label: &str, shortcut: &str, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            shortcut: shortcut.into(),
            action,
            enabled: true,
            submenu: Vec::new(),
        }
    }

    pub fn separator() -> Self {
        Self {
            label: String::new(),
            shortcut: String::new(),
            action: MenuAction::None,
            enabled: false,
            submenu: Vec::new(),
        }
    }

    pub fn submenu(label: &str, items: Vec<MenuItem>) -> Self {
        Self {
            label: label.into(),
            shortcut: String::new(),
            action: MenuAction::None,
            enabled: true,
            submenu: items,
        }
    }

    pub fn is_separator(&self) -> bool {
        self.label.is_empty() && self.shortcut.is_empty()
    }
}

/// Actions that can be triggered from menus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    None,
    // File
    NewWorkbook,
    OpenFile,
    Save,
    SaveAs,
    ImportCsv,
    ImportTsv,
    ImportOds,
    ExportXlsx,
    ExportPdf,
    ExportHtml,
    ExportCsv,
    Print,
    PrintPreview,
    PageSetup,
    Close,
    // Edit
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    PasteSpecial,
    Delete,
    SelectAll,
    Find,
    Replace,
    // View
    ToggleFormulaBar,
    ToggleGridLines,
    ToggleHeaders,
    FreezePanes,
    Zoom75,
    Zoom100,
    Zoom125,
    Zoom150,
    Zoom200,
    ToggleFullScreen,
    // Insert
    InsertRowAbove,
    InsertRowBelow,
    InsertColLeft,
    InsertColRight,
    InsertSheet,
    InsertChart,
    InsertFunction,
    DefineName,
    // Format
    FormatCells,
    RowHeight,
    ColWidth,
    RenameSheet,
    ConditionalFormat,
    // Data
    Sort,
    FilterToggle,
    DataValidation,
    PivotTable,
    // Tools
    Settings,
    // License
    ActivateLicense,
    GeneratePcCode,
    ReleaseDevice,
    // Help
    About,
    Shortcuts,
}

/// Context menu target.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ContextMenuTarget {
    Cell { row: usize, col: usize },
    RowHeader { row: usize },
    ColHeader { col: usize },
    SheetTab { sheet_idx: usize },
}

/// Context menu state.
#[derive(Debug, Clone)]
pub struct ContextMenuState {
    pub x: f64,
    pub y: f64,
    pub target: ContextMenuTarget,
}

pub fn build_menus() -> Vec<Vec<MenuItem>> {
    vec![
        vec![
            MenuItem::action("New Workbook", "Ctrl+N", MenuAction::NewWorkbook),
            MenuItem::action("Open", "Ctrl+O", MenuAction::OpenFile),
            MenuItem::action("Save", "Ctrl+S", MenuAction::Save),
            MenuItem::action("Save As", "Ctrl+Shift+S", MenuAction::SaveAs),
            MenuItem::separator(),
            MenuItem::submenu(
                "Import",
                vec![
                    MenuItem::action("CSV", "", MenuAction::ImportCsv),
                    MenuItem::action("TSV", "", MenuAction::ImportTsv),
                    MenuItem::action("ODS", "", MenuAction::ImportOds),
                ],
            ),
            MenuItem::submenu(
                "Export",
                vec![
                    MenuItem::action("XLSX", "", MenuAction::ExportXlsx),
                    MenuItem::action("PDF", "", MenuAction::ExportPdf),
                    MenuItem::action("HTML", "", MenuAction::ExportHtml),
                    MenuItem::action("CSV", "", MenuAction::ExportCsv),
                ],
            ),
            MenuItem::separator(),
            MenuItem::action("Page Setup", "", MenuAction::PageSetup),
            MenuItem::action("Print Preview", "Ctrl+P", MenuAction::PrintPreview),
            MenuItem::action("Print", "", MenuAction::Print),
            MenuItem::separator(),
            MenuItem::action("Close", "", MenuAction::Close),
        ],
        vec![
            MenuItem::action("Undo", "Ctrl+Z", MenuAction::Undo),
            MenuItem::action("Redo", "Ctrl+Y", MenuAction::Redo),
            MenuItem::separator(),
            MenuItem::action("Cut", "Ctrl+X", MenuAction::Cut),
            MenuItem::action("Copy", "Ctrl+C", MenuAction::Copy),
            MenuItem::action("Paste", "Ctrl+V", MenuAction::Paste),
            MenuItem::action("Paste Special", "Ctrl+Shift+V", MenuAction::PasteSpecial),
            MenuItem::separator(),
            MenuItem::action("Delete", "Delete", MenuAction::Delete),
            MenuItem::action("Select All", "Ctrl+A", MenuAction::SelectAll),
            MenuItem::separator(),
            MenuItem::action("Find", "Ctrl+F", MenuAction::Find),
            MenuItem::action("Replace", "Ctrl+H", MenuAction::Replace),
        ],
        vec![
            MenuItem::action("Formula Bar", "", MenuAction::ToggleFormulaBar),
            MenuItem::action("Grid Lines", "", MenuAction::ToggleGridLines),
            MenuItem::action("Headers", "", MenuAction::ToggleHeaders),
            MenuItem::action("Freeze Panes", "", MenuAction::FreezePanes),
            MenuItem::separator(),
            MenuItem::submenu(
                "Zoom",
                vec![
                    MenuItem::action("75%", "", MenuAction::Zoom75),
                    MenuItem::action("100%", "", MenuAction::Zoom100),
                    MenuItem::action("125%", "", MenuAction::Zoom125),
                    MenuItem::action("150%", "", MenuAction::Zoom150),
                    MenuItem::action("200%", "", MenuAction::Zoom200),
                ],
            ),
            MenuItem::separator(),
            MenuItem::action("Full Screen", "F11", MenuAction::ToggleFullScreen),
        ],
        vec![
            MenuItem::action("Insert Row Above", "", MenuAction::InsertRowAbove),
            MenuItem::action("Insert Row Below", "", MenuAction::InsertRowBelow),
            MenuItem::action("Insert Column Left", "", MenuAction::InsertColLeft),
            MenuItem::action("Insert Column Right", "", MenuAction::InsertColRight),
            MenuItem::separator(),
            MenuItem::action("Insert Sheet", "", MenuAction::InsertSheet),
            MenuItem::action("Chart", "", MenuAction::InsertChart),
            MenuItem::action("Function", "", MenuAction::InsertFunction),
            MenuItem::action("Define Name", "", MenuAction::DefineName),
        ],
        vec![
            MenuItem::action("Format Cells", "Ctrl+1", MenuAction::FormatCells),
            MenuItem::action("Row Height", "", MenuAction::RowHeight),
            MenuItem::action("Column Width", "", MenuAction::ColWidth),
            MenuItem::action("Rename Sheet", "", MenuAction::RenameSheet),
            MenuItem::separator(),
            MenuItem::action("Conditional Format", "", MenuAction::ConditionalFormat),
        ],
        vec![
            MenuItem::action("Sort", "", MenuAction::Sort),
            MenuItem::action("Filter", "", MenuAction::FilterToggle),
            MenuItem::action("Data Validation", "", MenuAction::DataValidation),
            MenuItem::action("Pivot Table", "", MenuAction::PivotTable),
        ],
        vec![MenuItem::action("Settings", "", MenuAction::Settings)],
        vec![
            MenuItem::action("Activate License", "", MenuAction::ActivateLicense),
            MenuItem::action("Generate PC Code", "", MenuAction::GeneratePcCode),
            MenuItem::action("Release Device", "", MenuAction::ReleaseDevice),
        ],
        vec![
            MenuItem::action("About", "", MenuAction::About),
            MenuItem::action("Shortcuts", "", MenuAction::Shortcuts),
        ],
    ]
}
