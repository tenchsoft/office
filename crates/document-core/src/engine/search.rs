use super::helpers::block_text_of;
use super::*;

// ---------------------------------------------------------------------------
// Search / Replace
// ---------------------------------------------------------------------------

impl DocumentEngine {
    /// Search the document and return all matches.
    pub fn find(&mut self, query: &str, case_sensitive: bool, regex: bool) -> Vec<SearchMatch> {
        let mut matches = Vec::new();
        if query.is_empty() {
            self.search_matches = matches.clone();
            self.current_match_idx = None;
            return matches;
        }

        for (block_idx, _) in self.document.content.iter().enumerate() {
            let text = block_text_of(&self.document.content, block_idx);
            if regex {
                if let Ok(re) = regex::RegexBuilder::new(query)
                    .case_insensitive(!case_sensitive)
                    .build()
                {
                    for m in re.find_iter(&text) {
                        matches.push(SearchMatch {
                            block_idx,
                            start_offset: m.start(),
                            end_offset: m.end(),
                        });
                    }
                }
            } else {
                let haystack = if case_sensitive {
                    text.clone()
                } else {
                    text.to_lowercase()
                };
                let needle = if case_sensitive {
                    query.to_string()
                } else {
                    query.to_lowercase()
                };
                let mut start = 0;
                while let Some(pos) = haystack[start..].find(&needle) {
                    let abs = start + pos;
                    matches.push(SearchMatch {
                        block_idx,
                        start_offset: abs,
                        end_offset: abs + needle.len(),
                    });
                    // Advance past the current match to the next char boundary.
                    // Using `abs + 1` can land inside a multi-byte UTF-8 character.
                    start = haystack[abs..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| abs + i)
                        .unwrap_or(haystack.len());
                    if start >= haystack.len() {
                        break;
                    }
                }
            }
        }

        self.current_match_idx = if matches.is_empty() { None } else { Some(0) };
        self.search_matches = matches.clone();
        matches
    }

    /// Return the current search matches and the highlighted match index.
    pub fn get_search_state(&self) -> (&[SearchMatch], Option<usize>) {
        (&self.search_matches, self.current_match_idx)
    }

    /// Move to the next search match and select it.
    pub fn find_next(&mut self) -> EditResult {
        if self.search_matches.is_empty() {
            return self.make_result();
        }
        let idx = match self.current_match_idx {
            Some(i) => (i + 1) % self.search_matches.len(),
            None => 0,
        };
        self.current_match_idx = Some(idx);
        let m = &self.search_matches[idx];
        self.cursor = CursorState {
            block_idx: m.block_idx,
            offset: m.start_offset,
        };
        self.selection = Some(SelectionRange {
            start: CursorState {
                block_idx: m.block_idx,
                offset: m.start_offset,
            },
            end: CursorState {
                block_idx: m.block_idx,
                offset: m.end_offset,
            },
        });
        self.make_result()
    }

    /// Move to the previous search match and select it.
    pub fn find_prev(&mut self) -> EditResult {
        if self.search_matches.is_empty() {
            return self.make_result();
        }
        let idx = match self.current_match_idx {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.current_match_idx = Some(idx);
        let m = &self.search_matches[idx];
        self.cursor = CursorState {
            block_idx: m.block_idx,
            offset: m.start_offset,
        };
        self.selection = Some(SelectionRange {
            start: CursorState {
                block_idx: m.block_idx,
                offset: m.start_offset,
            },
            end: CursorState {
                block_idx: m.block_idx,
                offset: m.end_offset,
            },
        });
        self.make_result()
    }

    /// Replace the current match (or the next one if none is highlighted).
    pub fn replace_next(&mut self, replacement: &str) -> EditResult {
        if self.search_matches.is_empty() {
            return self.make_result();
        }
        // If no current match, jump to first
        if self.current_match_idx.is_none() {
            self.current_match_idx = Some(0);
        }
        let idx = self.current_match_idx.unwrap();
        let m = &self.search_matches[idx];

        // Set selection to the match, delete it, then insert replacement
        self.cursor = CursorState {
            block_idx: m.block_idx,
            offset: m.start_offset,
        };
        self.selection = Some(SelectionRange {
            start: CursorState {
                block_idx: m.block_idx,
                offset: m.start_offset,
            },
            end: CursorState {
                block_idx: m.block_idx,
                offset: m.end_offset,
            },
        });
        self.delete_selection_inner();
        self.insert_text(replacement);

        // Remove the replaced match from the list and adjust.
        self.search_matches.remove(idx);
        if self.search_matches.is_empty() {
            self.current_match_idx = None;
        } else if idx >= self.search_matches.len() {
            self.current_match_idx = Some(0);
        } else {
            self.current_match_idx = Some(idx);
        }
        self.make_result()
    }

    /// Replace all matches with the replacement text.
    /// Returns the number of replacements made.
    pub fn replace_all(
        &mut self,
        query: &str,
        replacement: &str,
        case_sensitive: bool,
        regex: bool,
    ) -> usize {
        // Find all matches first
        let matches = self.find(query, case_sensitive, regex);
        if matches.is_empty() {
            return 0;
        }
        let count = matches.len();

        // Replace from end to start to preserve offsets
        for m in matches.into_iter().rev() {
            self.cursor = CursorState {
                block_idx: m.block_idx,
                offset: m.start_offset,
            };
            self.selection = Some(SelectionRange {
                start: CursorState {
                    block_idx: m.block_idx,
                    offset: m.start_offset,
                },
                end: CursorState {
                    block_idx: m.block_idx,
                    offset: m.end_offset,
                },
            });
            self.delete_selection_inner();
            self.insert_text(replacement);
        }

        // Clear search state
        self.search_matches.clear();
        self.current_match_idx = None;
        count
    }

    /// Clear search state.
    pub fn clear_search(&mut self) {
        self.search_matches.clear();
        self.current_match_idx = None;
    }
}
