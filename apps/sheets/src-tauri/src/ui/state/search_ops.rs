use super::*;

impl SheetsState {
    pub fn find(&mut self) {
        let query = self.find_replace.find_text.clone();
        if query.is_empty() {
            self.find_replace.matches.clear();
            self.find_replace.current_match = None;
            return;
        }

        let case_sensitive = self.find_replace.case_sensitive;
        let use_regex = self.find_replace.use_regex;
        let search_formulas = self.find_replace.search_in_formulas;

        let mut matches = Vec::new();

        for (r, row) in self.grid.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let text = if search_formulas || !cell.is_formula {
                    &cell.value
                } else {
                    continue;
                };

                let found = if use_regex {
                    regex::RegexBuilder::new(&query)
                        .case_insensitive(!case_sensitive)
                        .build()
                        .map(|re| re.is_match(text))
                        .unwrap_or(false)
                } else {
                    let hay = if case_sensitive {
                        text.to_string()
                    } else {
                        text.to_lowercase()
                    };
                    let needle = if case_sensitive {
                        query.clone()
                    } else {
                        query.to_lowercase()
                    };
                    hay.contains(&needle)
                };

                if found {
                    matches.push(GridSearchMatch { row: r, col: c });
                }
            }
        }

        self.find_replace.current_match = if matches.is_empty() { None } else { Some(0) };
        self.find_replace.matches = matches;
    }

    /// Move to the next search match.
    pub fn find_next(&mut self) -> bool {
        if self.find_replace.matches.is_empty() {
            return false;
        }
        let idx = match self.find_replace.current_match {
            Some(i) => (i + 1) % self.find_replace.matches.len(),
            None => 0,
        };
        self.find_replace.current_match = Some(idx);
        let m = &self.find_replace.matches[idx];
        self.select_cell(m.row, m.col);
        true
    }

    /// Move to the previous search match.
    pub fn find_prev(&mut self) -> bool {
        if self.find_replace.matches.is_empty() {
            return false;
        }
        let idx = match self.find_replace.current_match {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.find_replace.current_match = Some(idx);
        let m = &self.find_replace.matches[idx];
        self.select_cell(m.row, m.col);
        true
    }

    /// Replace the current match and advance to the next.
    pub fn replace_next(&mut self) -> bool {
        let idx = match self.find_replace.current_match {
            Some(i) if i < self.find_replace.matches.len() => i,
            _ => return false,
        };
        let match_row = self.find_replace.matches[idx].row;
        let match_col = self.find_replace.matches[idx].col;

        self.push_undo();
        if let Some(row) = self.grid.get_mut(match_row) {
            if let Some(cell) = row.get_mut(match_col) {
                let query = &self.find_replace.find_text;
                let replacement = &self.find_replace.replace_text;
                if self.find_replace.use_regex {
                    if let Ok(re) = regex::RegexBuilder::new(query)
                        .case_insensitive(!self.find_replace.case_sensitive)
                        .build()
                    {
                        cell.value = re
                            .replace_all(&cell.value, replacement.as_str())
                            .to_string();
                    }
                } else {
                    let case_insensitive = !self.find_replace.case_sensitive;
                    if case_insensitive {
                        let hay = cell.value.to_lowercase();
                        let needle = query.to_lowercase();
                        if let Some(pos) = hay.find(&needle) {
                            cell.value = format!(
                                "{}{}{}",
                                &cell.value[..pos],
                                replacement,
                                &cell.value[pos + needle.len()..]
                            );
                        }
                    } else if let Some(pos) = cell.value.find(query) {
                        cell.value = format!(
                            "{}{}{}",
                            &cell.value[..pos],
                            replacement,
                            &cell.value[pos + query.len()..]
                        );
                    }
                }
                cell.is_formula = cell.value.starts_with('=');
            }
        }
        self.find_replace.matches.remove(idx);
        if self.find_replace.matches.is_empty() {
            self.find_replace.current_match = None;
        } else if idx >= self.find_replace.matches.len() {
            self.find_replace.current_match = Some(0);
        }
        self.sync_content_from_grid();
        true
    }

    /// Replace all matches.
    pub fn replace_all(&mut self) -> usize {
        self.find();
        let count = self.find_replace.matches.len();
        if count == 0 {
            return 0;
        }

        self.push_undo();
        let query = self.find_replace.find_text.clone();
        let replacement = self.find_replace.replace_text.clone();
        let case_sensitive = self.find_replace.case_sensitive;
        let use_regex = self.find_replace.use_regex;

        for m in &self.find_replace.matches {
            if let Some(row) = self.grid.get_mut(m.row) {
                if let Some(cell) = row.get_mut(m.col) {
                    if use_regex {
                        if let Ok(re) = regex::RegexBuilder::new(&query)
                            .case_insensitive(!case_sensitive)
                            .build()
                        {
                            cell.value = re
                                .replace_all(&cell.value, replacement.as_str())
                                .to_string();
                        }
                    } else {
                        let case_insensitive = !case_sensitive;
                        if case_insensitive {
                            let hay = cell.value.to_lowercase();
                            let needle = query.to_lowercase();
                            if hay.contains(&needle) {
                                let mut result = String::new();
                                let mut last = 0;
                                let lower = cell.value.to_lowercase();
                                while let Some(pos) = lower[last..].find(&needle) {
                                    let abs = last + pos;
                                    result.push_str(&cell.value[last..abs]);
                                    result.push_str(&replacement);
                                    last = abs + needle.len();
                                }
                                result.push_str(&cell.value[last..]);
                                cell.value = result;
                            }
                        } else {
                            cell.value = cell.value.replace(&query, &replacement);
                        }
                    }
                    cell.is_formula = cell.value.starts_with('=');
                }
            }
        }

        self.find_replace.matches.clear();
        self.find_replace.current_match = None;
        self.sync_content_from_grid();
        count
    }

    /// Check if a cell is a search match.
    pub fn is_search_match(&self, row: usize, col: usize) -> bool {
        self.find_replace
            .matches
            .iter()
            .any(|m| m.row == row && m.col == col)
    }

    /// Check if a cell is the current search match.
    pub fn is_current_match(&self, row: usize, col: usize) -> bool {
        self.find_replace
            .current_match
            .and_then(|idx| self.find_replace.matches.get(idx))
            .is_some_and(|m| m.row == row && m.col == col)
    }
}
