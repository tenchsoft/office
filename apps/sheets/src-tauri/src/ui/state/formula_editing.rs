// ---------------------------------------------------------------------------
// Formula editing helpers
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;

use super::{
    col_letter, function_catalog, parse_col_letters, AutocompleteState, EditingCell, FormulaRef,
    SheetsState, FORMULA_REF_COLORS,
};

impl SheetsState {
    /// Accept the current autocomplete suggestion.
    pub fn accept_autocomplete(&mut self) -> bool {
        let (draft, is_formula) = {
            let Some(edit) = self.editing_cell.as_mut() else {
                return false;
            };
            let Some(ref ac) = edit.autocomplete else {
                return false;
            };
            if ac.candidates.is_empty() {
                return false;
            }
            let name = ac.candidates[ac.selected_idx.min(ac.candidates.len() - 1)];
            let prefix_len = ac.prefix.len();
            let replace_start = edit.cursor_pos.saturating_sub(prefix_len);
            edit.draft = format!(
                "{}{}{}",
                &edit.draft[..replace_start],
                name,
                &edit.draft[edit.cursor_pos..]
            );
            edit.cursor_pos = replace_start + name.len();
            edit.autocomplete = None;
            self.formula_draft = edit.draft.clone();
            (edit.draft.clone(), edit.is_formula_edit)
        };
        if is_formula {
            self.parse_formula_refs(&draft);
        }
        true
    }

    /// Navigate autocomplete selection up/down.
    pub fn autocomplete_navigate(&mut self, direction: isize) -> bool {
        let Some(edit) = self.editing_cell.as_mut() else {
            return false;
        };
        let Some(ref mut ac) = edit.autocomplete else {
            return false;
        };
        if ac.candidates.is_empty() {
            return false;
        }
        let len = ac.candidates.len();
        if direction > 0 {
            ac.selected_idx = (ac.selected_idx + 1).min(len - 1);
        } else {
            ac.selected_idx = ac.selected_idx.saturating_sub(1);
        }
        true
    }

    /// Dismiss autocomplete popup.
    pub fn dismiss_autocomplete(&mut self) {
        if let Some(edit) = self.editing_cell.as_mut() {
            edit.autocomplete = None;
        }
    }

    /// Insert a cell reference into the formula draft during formula editing.
    pub(super) fn insert_formula_reference(&mut self, row: usize, col: usize) {
        let draft = {
            let Some(edit) = self.editing_cell.as_mut() else {
                return;
            };
            let ref_str = format!("{}{}", col_letter(col), row + 1);
            edit.draft.insert_str(edit.cursor_pos, &ref_str);
            edit.cursor_pos += ref_str.len();
            self.formula_draft = edit.draft.clone();
            edit.draft.clone()
        };
        self.parse_formula_refs(&draft);
    }

    /// Update autocomplete candidates based on current cursor position.
    pub(super) fn update_autocomplete(&self, edit: &mut EditingCell) {
        if !edit.is_formula_edit {
            edit.autocomplete = None;
            return;
        }
        let before_cursor = &edit.draft[..edit.cursor_pos];
        let word_start = before_cursor
            .rfind(|c: char| !c.is_ascii_alphabetic())
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &before_cursor[word_start..];
        if prefix.is_empty() {
            edit.autocomplete = None;
            return;
        }
        let prefix_upper = prefix.to_uppercase();
        let catalog = function_catalog();
        let candidates: Vec<&'static str> = catalog
            .iter()
            .filter(|f| f.name.starts_with(&prefix_upper))
            .map(|f| f.name)
            .collect();
        if candidates.is_empty() {
            edit.autocomplete = None;
        } else {
            edit.autocomplete = Some(AutocompleteState {
                prefix: prefix.to_string(),
                candidates,
                selected_idx: 0,
                popup_offset: (0.0, 0.0),
            });
        }
    }

    /// Rebuild autocomplete for the current editing_cell (after releasing borrows).
    pub(super) fn rebuild_autocomplete(&mut self) {
        let Some(edit) = self.editing_cell.as_mut() else {
            return;
        };
        if !edit.is_formula_edit {
            edit.autocomplete = None;
            return;
        }
        let before_cursor = &edit.draft[..edit.cursor_pos];
        let word_start = before_cursor
            .rfind(|c: char| !c.is_ascii_alphabetic())
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &before_cursor[word_start..];
        if prefix.is_empty() {
            edit.autocomplete = None;
            return;
        }
        let prefix_upper = prefix.to_uppercase();
        let catalog = function_catalog();
        let candidates: Vec<&'static str> = catalog
            .iter()
            .filter(|f| f.name.starts_with(&prefix_upper))
            .map(|f| f.name)
            .collect();
        if candidates.is_empty() {
            edit.autocomplete = None;
        } else {
            edit.autocomplete = Some(AutocompleteState {
                prefix: prefix.to_string(),
                candidates,
                selected_idx: 0,
                popup_offset: (0.0, 0.0),
            });
        }
    }

    /// Parse cell references from a formula string for color highlighting.
    pub(super) fn parse_formula_refs(&mut self, formula: &str) {
        self.formula_refs.clear();
        let mut color_idx = 0;
        let rest = formula.strip_prefix('=').unwrap_or(formula);
        let bytes = rest.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i] as char;
            if c.is_ascii_alphabetic() {
                let start = i;
                let mut col_end = i;
                while col_end < bytes.len() && (bytes[col_end] as char).is_ascii_alphabetic() {
                    col_end += 1;
                }
                let mut num_end = col_end;
                while num_end < bytes.len() && (bytes[num_end] as char).is_ascii_digit() {
                    num_end += 1;
                }
                if num_end > col_end && col_end > start {
                    let col_str = &rest[start..col_end];
                    let num_str = &rest[col_end..num_end];
                    if let Ok(col_num) = parse_col_letters(col_str) {
                        if let Ok(row_num) = num_str.parse::<usize>() {
                            if row_num > 0 && col_num < 26 * 27 {
                                let r = row_num - 1;
                                let c = col_num;
                                let mut end_r = r;
                                let mut end_c = c;
                                let mut j = num_end;
                                if j < bytes.len() && bytes[j] == b':' {
                                    j += 1;
                                    let range_start = j;
                                    let mut rc_end = j;
                                    while rc_end < bytes.len()
                                        && (bytes[rc_end] as char).is_ascii_alphabetic()
                                    {
                                        rc_end += 1;
                                    }
                                    let mut rn_end = rc_end;
                                    while rn_end < bytes.len()
                                        && (bytes[rn_end] as char).is_ascii_digit()
                                    {
                                        rn_end += 1;
                                    }
                                    if rn_end > rc_end && rc_end > range_start {
                                        if let Ok(ec) =
                                            parse_col_letters(&rest[range_start..rc_end])
                                        {
                                            if let Ok(er) = rest[rc_end..rn_end].parse::<usize>() {
                                                if er > 0 {
                                                    end_r = er - 1;
                                                    end_c = ec;
                                                    j = rn_end;
                                                }
                                            }
                                        }
                                    }
                                }
                                let ci = color_idx % FORMULA_REF_COLORS.len();
                                color_idx += 1;
                                self.formula_refs.push(FormulaRef {
                                    start_row: r,
                                    start_col: c,
                                    end_row: end_r,
                                    end_col: end_c,
                                    color_idx: ci,
                                });
                                i = j;
                                continue;
                            }
                        }
                    }
                }
                i = col_end.max(i + 1);
            } else {
                i += 1;
            }
        }
    }

    /// Get the formula reference color for a cell, if any.
    pub fn formula_ref_color_for_cell(&self, row: usize, col: usize) -> Option<Color> {
        self.formula_refs
            .iter()
            .find(|r| {
                row >= r.start_row && row <= r.end_row && col >= r.start_col && col <= r.end_col
            })
            .map(|r| FORMULA_REF_COLORS[r.color_idx % FORMULA_REF_COLORS.len()])
    }

    /// Get the current function hint (signature) based on cursor position in formula.
    pub fn current_function_hint(&self) -> Option<&'static str> {
        let edit = self.editing_cell.as_ref()?;
        if !edit.is_formula_edit {
            return None;
        }
        let before_cursor = &edit.draft[..edit.cursor_pos];
        let mut depth = 0i32;
        let mut func_start = None;
        for (i, c) in before_cursor.char_indices().rev() {
            match c {
                ')' => depth += 1,
                '(' => {
                    if depth == 0 {
                        func_start = Some(i);
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
        let start = func_start?;
        let before_paren = &before_cursor[..start];
        let name_end = before_paren.len();
        let name_start = before_paren
            .rfind(|c: char| !c.is_ascii_alphabetic() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let func_name = &before_paren[name_start..name_end];
        let func_upper = func_name.to_uppercase();
        function_catalog()
            .iter()
            .find(|f| f.name == func_upper)
            .map(|f| f.signature)
    }
}
