use std::collections::HashMap;

use crate::ui::state::{self, HorizontalAlignment, NumberFormat};
use tench_ui::prelude::*;

// ---------------------------------------------------------------------------
// 9.1 Cell render cache
// ---------------------------------------------------------------------------

/// Cached rendering data for a single cell.
///
/// Avoids re-computing display text and text width when the cell content
/// and style have not changed since the last frame.
#[derive(Debug, Clone)]
pub(super) struct CachedCell {
    /// The formatted display text (e.g. number formatting applied).
    pub(super) display_text: String,
    /// Measured text width using `TextCache::measure_text_width`.
    #[allow(dead_code)] // used by column auto-fit which will consume this field
    pub(super) text_width: Option<f64>,
    /// Hash combining cell value + style properties.
    pub(super) style_hash: u64,
}

/// Per-frame render cache keyed by `(row, col)`.
///
/// On each paint call the cache is validated against a content hash that
/// captures the entire grid state. If the hash matches, cached display text
/// and text widths are reused instead of being recomputed.
pub struct CellRenderCache {
    entries: HashMap<(usize, usize), CachedCell>,
    last_content_hash: u64,
}

impl CellRenderCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            last_content_hash: 0,
        }
    }

    /// Compute a lightweight hash of the grid content to detect changes.
    ///
    /// Uses a simple FNV-1a-like fold over cell values so that any edit
    /// produces a different hash and invalidates the cache.
    pub fn compute_grid_hash(grid: &[Vec<state::CellData>]) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
        for row in grid {
            for cell in row {
                // Mix cell value
                for byte in cell.value.bytes() {
                    hash ^= byte as u64;
                    hash = hash.wrapping_mul(0x100000001b3); // FNV prime
                }
                // Mix is_formula flag
                hash ^= cell.is_formula as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            // Row separator
            hash ^= 0xFF;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    /// Invalidate the entire cache (e.g. after structural changes like
    /// row/column insertion or deletion).
    #[allow(dead_code)] // public API — used after row/col insertion/deletion
    pub fn invalidate(&mut self) {
        self.entries.clear();
        self.last_content_hash = 0;
    }

    /// Update the content hash and return whether the grid changed.
    pub fn update_hash(&mut self, grid: &[Vec<state::CellData>]) -> bool {
        let new_hash = Self::compute_grid_hash(grid);
        let changed = new_hash != self.last_content_hash;
        self.last_content_hash = new_hash;
        if changed {
            // Clear stale entries — they will be repopulated during paint.
            self.entries.clear();
        }
        changed
    }

    /// Get a cached entry for the given cell, if present and still valid.
    pub(super) fn get(&self, row: usize, col: usize) -> Option<&CachedCell> {
        self.entries.get(&(row, col))
    }

    /// Insert or update a cached entry.
    pub(super) fn insert(&mut self, row: usize, col: usize, cell: CachedCell) {
        self.entries.insert((row, col), cell);
    }
}

impl Default for CellRenderCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a style hash for a cell based on its visual properties.
pub(super) fn cell_style_hash(
    cell: &state::CellData,
    is_header: bool,
    is_selected: bool,
    is_match: bool,
    is_current: bool,
) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in cell.value.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash ^= cell.is_formula as u64;
    hash = hash.wrapping_mul(0x100000001b3);
    hash ^= is_header as u64;
    hash = hash.wrapping_mul(0x100000001b3);
    hash ^= is_selected as u64;
    hash = hash.wrapping_mul(0x100000001b3);
    hash ^= is_match as u64;
    hash = hash.wrapping_mul(0x100000001b3);
    hash ^= is_current as u64;
    hash = hash.wrapping_mul(0x100000001b3);
    // Include format fields in hash
    hash ^= cell.format.bold as u64;
    hash = hash.wrapping_mul(0x100000001b3);
    hash ^= cell.format.italic as u64;
    hash = hash.wrapping_mul(0x100000001b3);
    hash ^= cell.format.underline as u64;
    hash = hash.wrapping_mul(0x100000001b3);
    if let Some(color) = cell.format.text_color {
        hash ^= color.to_u32() as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    if let Some(color) = cell.format.bg_color {
        hash ^= color.to_u32() as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let h_align_bits: u64 = match cell.format.h_align {
        Some(HorizontalAlignment::Left) => 1,
        Some(HorizontalAlignment::Center) => 2,
        Some(HorizontalAlignment::Right) => 3,
        None => 0,
    };
    hash ^= h_align_bits;
    hash = hash.wrapping_mul(0x100000001b3);
    let nf_bits: u64 = match &cell.format.number_format {
        NumberFormat::General => 0,
        NumberFormat::Number {
            decimals,
            thousands_sep,
        } => 1 | (*decimals as u64) << 8 | (*thousands_sep as u64) << 16,
        NumberFormat::Currency { decimals, .. } => 2 | (*decimals as u64) << 8,
        NumberFormat::Percentage { decimals } => 3 | (*decimals as u64) << 8,
        NumberFormat::Date => 4,
        NumberFormat::Text => 5,
    };
    hash ^= nf_bits;
    hash = hash.wrapping_mul(0x100000001b3);
    hash
}

// ---------------------------------------------------------------------------
// Batch background rendering
// ---------------------------------------------------------------------------

/// Groups cell background rectangles by color so that cells sharing the
/// same background can be drawn in a single batch, reducing GPU state
/// changes.
pub(super) struct BatchBackgrounds {
    /// Map from packed RGBA color to list of rects to fill.
    batches: HashMap<u32, Vec<Rect>>,
}

impl BatchBackgrounds {
    pub(super) fn new() -> Self {
        Self {
            batches: HashMap::new(),
        }
    }

    pub(super) fn push(&mut self, color: Color, rect: Rect) {
        let key = color.to_u32();
        self.batches.entry(key).or_default().push(rect);
    }

    /// Flush all batched rectangles to the painter.
    pub(super) fn flush(self, p: &mut Painter<'_>) {
        for (packed, rects) in self.batches {
            let color = Color::from_u32(packed);
            for rect in rects {
                p.fill_rect(rect, color);
            }
        }
    }
}
