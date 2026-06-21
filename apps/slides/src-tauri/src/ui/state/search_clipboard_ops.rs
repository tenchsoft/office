use super::*;

impl SlidesState {
    pub fn find_text(&mut self, query: &str) {
        self.find_replace.find_text = query.to_string();
        self.find_replace.matches.clear();
        self.find_replace.current_match = None;
        if query.is_empty() {
            return;
        }
        for (si, slide) in self.slides.iter().enumerate() {
            for (ei, elem) in slide.elements.iter().enumerate() {
                if let Some(text) = &elem.text {
                    if text.to_lowercase().contains(&query.to_lowercase()) {
                        self.find_replace.matches.push((si, ei));
                    }
                }
            }
        }
        if !self.find_replace.matches.is_empty() {
            self.find_replace.current_match = Some(self.find_replace.matches[0]);
            let (si, _ei) = self.find_replace.matches[0];
            self.select_slide(si);
        }
    }

    pub fn find_next(&mut self) {
        if self.find_replace.matches.is_empty() {
            return;
        }
        let current = self.find_replace.current_match.unwrap_or((0, 0));
        let idx = self
            .find_replace
            .matches
            .iter()
            .position(|m| *m == current)
            .unwrap_or(0);
        let next_idx = (idx + 1) % self.find_replace.matches.len();
        self.find_replace.current_match = Some(self.find_replace.matches[next_idx]);
        let (si, _ei) = self.find_replace.matches[next_idx];
        self.select_slide(si);
    }

    pub fn replace_current(&mut self) {
        if let Some((si, ei)) = self.find_replace.current_match {
            if let Some(slide) = self.slides.get_mut(si) {
                if let Some(elem) = slide.elements.get_mut(ei) {
                    if let Some(text) = &mut elem.text {
                        let query = &self.find_replace.find_text;
                        let replacement = &self.find_replace.replace_text;
                        *text = text.replace(query, replacement);
                        self.sync_content_from_slides();
                    }
                }
            }
        }
    }

    pub fn replace_all(&mut self) {
        let query = self.find_replace.find_text.clone();
        let replacement = self.find_replace.replace_text.clone();
        if query.is_empty() {
            return;
        }
        self.push_undo();
        for slide in &mut self.slides {
            for elem in &mut slide.elements {
                if let Some(text) = &mut elem.text {
                    *text = text.replace(&query, &replacement);
                }
            }
        }
        self.sync_content_from_slides();
        self.find_text(&query);
    }

    // ── Phase 8.3: Clipboard ────────────────────────────────────────

    pub fn copy_selected(&mut self) {
        let Some(idx) = self.selected_element else {
            return;
        };
        let Some(slide) = self.current_slide() else {
            return;
        };
        let Some(elem) = slide.elements.get(idx) else {
            return;
        };
        self.clipboard = Some(ClipboardData {
            elements: vec![elem.clone()],
            source_slide_index: Some(self.current_slide),
        });
    }

    pub fn cut_selected(&mut self) {
        self.copy_selected();
        self.delete_selected_element();
    }

    pub fn paste(&mut self) {
        let elements = match &self.clipboard {
            Some(data) => data.elements.clone(),
            None => return,
        };
        if elements.is_empty() {
            return;
        }
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            for mut elem in elements {
                elem.x += 20.0;
                elem.y += 20.0;
                slide.elements.push(elem);
            }
            self.selected_element = Some(slide.elements.len() - 1);
        }
        self.sync_content_from_slides();
    }

    // ── Phase 8.4: Speaker notes ────────────────────────────────────

    pub fn update_notes(&mut self, notes: String) {
        if let Some(slide) = self.current_slide_mut() {
            slide.notes = notes;
            self.sync_content_from_slides();
        }
    }

    // ── Phase 8.6: Version history ──────────────────────────────────

    pub fn save_version(&mut self) {
        let sig = slides_signature(&self.slides);
        self.version_history.push(VersionEntry {
            timestamp: format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
            slide_count: self.slides.len(),
            signature: sig,
        });
    }
}
