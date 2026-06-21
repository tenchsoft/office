// ---------------------------------------------------------------------------
// Search, find-replace, autosave types
// ---------------------------------------------------------------------------

use std::time::Instant;

/// A single search match within the grid.
#[derive(Debug, Clone)]
pub struct GridSearchMatch {
    pub row: usize,
    pub col: usize,
}

/// Search scope for find/replace.
///
/// All variants are part of the public API consumed by the find/replace dialog.
/// `EntireWorkbook` and `Selection` will be wired to event handlers once
/// multi-sheet and range-selection features land.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // variants are public API — consumed by dialog rendering and future event handlers
pub enum SearchScope {
    CurrentSheet,
    EntireWorkbook,
    Selection,
}

/// State for the find/replace dialog.
#[derive(Debug, Clone)]
pub struct FindReplaceState {
    pub find_text: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub search_in_formulas: bool,
    pub scope: SearchScope,
    pub matches: Vec<GridSearchMatch>,
    pub current_match: Option<usize>,
}

impl Default for FindReplaceState {
    fn default() -> Self {
        Self {
            find_text: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            use_regex: false,
            search_in_formulas: false,
            scope: SearchScope::CurrentSheet,
            matches: Vec::new(),
            current_match: None,
        }
    }
}

/// Auto-save state.
#[derive(Debug, Clone)]
pub struct AutoSaveState {
    pub last_save_time: Instant,
    pub interval_secs: u64,
    pub enabled: bool,
}

impl AutoSaveState {
    pub fn new(interval_secs: u64) -> Self {
        Self {
            last_save_time: Instant::now(),
            interval_secs,
            enabled: true,
        }
    }

    pub fn should_save(&self, dirty: bool) -> bool {
        self.enabled && dirty && self.last_save_time.elapsed().as_secs() >= self.interval_secs
    }

    pub fn mark_saved(&mut self) {
        self.last_save_time = Instant::now();
    }
}
