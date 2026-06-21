//! Hanja (Korean-Chinese character) lookup table and conversion.
//!
//! Provides a small built-in table of common Korean syllable-to-Hanja mappings
//! for the document editor's Hanja conversion popup.

mod single;
mod table;
mod types;

pub use types::HanjaEntry;

use single::single_char_hanja;
use table::HANJA_TABLE;

/// Returns Hanja candidates for a common Korean word.
/// Returns an empty vector if no mapping exists.
pub fn lookup_hanja(korean: &str) -> Vec<HanjaEntry> {
    let entries: &[(&str, &[HanjaEntry])] = HANJA_TABLE;

    for (key, candidates) in entries {
        if *key == korean {
            return candidates.to_vec();
        }
    }

    // Fallback: try single-character lookup
    if korean.chars().count() == 1 {
        let ch = korean.chars().next().unwrap();
        return single_char_hanja(ch);
    }

    Vec::new()
}
