use super::*;

impl DocsApp {
    pub(in crate::ui) fn reset_cursor_blink(&mut self) {
        self.state.cursor_visible = true;
        self.cursor_timer.reset();
    }

    pub(in crate::ui) fn handle_keyboard(
        &mut self,
        kb: &KeyboardEvent,
        ctx: &mut EventCtx,
    ) -> bool {
        if !kb.is_pressed {
            return false;
        }

        // If link modal is open, route keyboard input to it
        if self.state.link_modal.is_some() {
            return self.handle_link_modal_keyboard(kb);
        }

        // If comment modal is open, route keyboard input to it
        if self.state.comment_modal.is_some() {
            return self.handle_comment_modal_keyboard(kb, ctx);
        }

        // If find/replace modal is open, route keyboard input to it
        if self.state.find_replace.is_some() {
            return self.handle_find_replace_keyboard(kb, ctx);
        }

        // If page setup dialog is open and a margin field is being edited, route to it
        if self
            .state
            .page_setup_dialog
            .as_ref()
            .and_then(|d| d.editing_margin_field)
            .is_some()
        {
            return self.handle_page_setup_margin_keyboard(kb);
        }

        if self.handle_modal_keyboard(kb, ctx) {
            return true;
        }

        // If editing header/footer (and no modal is active), route text input
        if self.state.editing_header || self.state.editing_footer {
            return self.handle_header_footer_keyboard(kb);
        }

        if kb.modifiers.control {
            return match &kb.logical_key {
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("b") => {
                    let result = self.engine().toggle_mark(MarkType::Bold);
                    self.state.bold = !self.state.bold;
                    self.state.apply_edit_result(result);
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("i") => {
                    let result = self.engine().toggle_mark(MarkType::Italic);
                    self.state.italic = !self.state.italic;
                    self.state.apply_edit_result(result);
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("u") => {
                    let result = self.engine().toggle_mark(MarkType::Underline);
                    self.state.underline = !self.state.underline;
                    self.state.apply_edit_result(result);
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("z") => {
                    let result = self.engine().undo();
                    self.state.apply_edit_result(result);
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("y") => {
                    let result = self.engine().redo();
                    self.state.apply_edit_result(result);
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("s") => {
                    self.save_current_document();
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("k") => {
                    self.state.active_modal = Some("Hyperlink".into());
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("f") => {
                    self.open_find_replace(false);
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("h") => {
                    self.open_find_replace(true);
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("x") => {
                    self.cut_selection();
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("c") => {
                    self.copy_selection();
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("v") => {
                    self.paste_clipboard();
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("m") => {
                    self.state.show_comments = !self.state.show_comments;
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("t") => {
                    self.state.track_changes = !self.state.track_changes;
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("a") => {
                    let result = self.engine().select_all();
                    self.state.apply_edit_result(result);
                    true
                }
                // Ctrl+Arrow: word-level movement
                LogicalKey::Named(NamedKey::ArrowLeft) => {
                    let result = self.engine().move_cursor(MoveDirection::WordLeft);
                    self.state.apply_edit_result(result);
                    self.reset_cursor_blink();
                    true
                }
                LogicalKey::Named(NamedKey::ArrowRight) => {
                    let result = self.engine().move_cursor(MoveDirection::WordRight);
                    self.state.apply_edit_result(result);
                    self.reset_cursor_blink();
                    true
                }
                LogicalKey::Named(NamedKey::ArrowUp) => {
                    let result = self.engine().move_cursor(MoveDirection::DocStart);
                    self.state.apply_edit_result(result);
                    self.reset_cursor_blink();
                    true
                }
                LogicalKey::Named(NamedKey::ArrowDown) => {
                    let result = self.engine().move_cursor(MoveDirection::DocEnd);
                    self.state.apply_edit_result(result);
                    self.reset_cursor_blink();
                    true
                }
                // Ctrl+Home/End: document start/end
                LogicalKey::Named(NamedKey::Home) => {
                    let result = self.engine().move_cursor(MoveDirection::DocStart);
                    self.state.apply_edit_result(result);
                    self.reset_cursor_blink();
                    true
                }
                LogicalKey::Named(NamedKey::End) => {
                    let result = self.engine().move_cursor(MoveDirection::DocEnd);
                    self.state.apply_edit_result(result);
                    self.reset_cursor_blink();
                    true
                }
                // Ctrl+Plus: zoom in
                LogicalKey::Character(c) if c == "=" || c == "+" => {
                    self.state.set_zoom((self.state.zoom + 10.0).min(200.0));
                    true
                }
                // Ctrl+Minus: zoom out
                LogicalKey::Character(c) if c == "-" => {
                    self.state.set_zoom((self.state.zoom - 10.0).max(50.0));
                    true
                }
                // Ctrl+0: reset zoom
                LogicalKey::Character(c) if c == "0" => {
                    self.state.set_zoom(100.0);
                    true
                }
                // Ctrl+Tab: switch to next tab
                LogicalKey::Named(NamedKey::Tab) => {
                    if self.state.open_tabs.len() > 1 {
                        let next = (self.state.active_tab_idx + 1) % self.state.open_tabs.len();
                        self.switch_to_tab(next);
                    }
                    true
                }
                _ => false,
            };
        }

        if kb.modifiers.alt || kb.modifiers.super_key {
            return false;
        }

        // Handle Shift+Arrow for selection
        if kb.modifiers.shift {
            return match &kb.logical_key {
                LogicalKey::Named(NamedKey::ArrowLeft) => {
                    self.extend_selection(MoveDirection::Left);
                    true
                }
                LogicalKey::Named(NamedKey::ArrowRight) => {
                    self.extend_selection(MoveDirection::Right);
                    true
                }
                LogicalKey::Named(NamedKey::ArrowUp) => {
                    self.extend_selection(MoveDirection::Up);
                    true
                }
                LogicalKey::Named(NamedKey::ArrowDown) => {
                    self.extend_selection(MoveDirection::Down);
                    true
                }
                LogicalKey::Named(NamedKey::Home) => {
                    self.extend_selection(MoveDirection::Home);
                    true
                }
                LogicalKey::Named(NamedKey::End) => {
                    self.extend_selection(MoveDirection::End);
                    true
                }
                _ => false,
            };
        }

        match &kb.logical_key {
            LogicalKey::Character(c) => {
                let result = self.engine().insert_text(c);
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::Space) => {
                let result = self.engine().insert_text(" ");
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                let result = self.engine().insert_text("\n");
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                let result = self.engine().backspace();
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::Delete) => {
                let result = self.engine().delete_forward();
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::ArrowLeft) => {
                let result = self.engine().move_cursor(MoveDirection::Left);
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::ArrowRight) => {
                let result = self.engine().move_cursor(MoveDirection::Right);
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::ArrowUp) => {
                let result = self.engine().move_cursor(MoveDirection::Up);
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::ArrowDown) => {
                let result = self.engine().move_cursor(MoveDirection::Down);
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::Home) => {
                let result = self.engine().move_cursor(MoveDirection::Home);
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::End) => {
                let result = self.engine().move_cursor(MoveDirection::End);
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::Escape) => {
                if self.state.page_setup_dialog.is_some() {
                    self.state.page_setup_dialog = None;
                    return true;
                }
                if self.state.print_preview.is_some() {
                    self.state.print_preview = None;
                    return true;
                }
                if self.state.word_count_modal {
                    self.state.word_count_modal = false;
                    return true;
                }
                if self.state.goto_modal.is_some() {
                    self.state.goto_modal = None;
                    return true;
                }
                if self.state.special_char_modal.is_some() {
                    self.state.special_char_modal = None;
                    return true;
                }
                if self.state.active_modal.is_some() {
                    self.state.active_modal = None;
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    /// Extend selection from current cursor position in the given direction.
    pub(in crate::ui) fn extend_selection(&mut self, direction: MoveDirection) {
        // Save the anchor: the position before any shift-arrow extension started.
        // On the first shift-arrow, there is no anchor yet, so use the current cursor.
        // On subsequent shift-arrows, keep the existing anchor so the selection
        // extends from the original position.
        let anchor = self
            .state
            .selection_anchor
            .clone()
            .unwrap_or_else(|| self.state.cursor().clone());

        let result = self.engine().move_cursor(direction);
        let new_cursor = result.cursor.clone();
        self.state.apply_edit_result(result);

        // Set selection from anchor to new cursor position
        let sel_result = self.engine().select(anchor.clone(), new_cursor);
        self.state.selection_anchor = Some(anchor);
        self.state.apply_edit_result(sel_result);
        self.reset_cursor_blink();
    }

    /// Handle keyboard input for the link insertion modal.
    pub(in crate::ui) fn handle_link_modal_keyboard(&mut self, kb: &KeyboardEvent) -> bool {
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.link_modal = None;
                self.state.active_modal = None;
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                if let Some(link_state) = self.state.link_modal.take() {
                    if !link_state.url.is_empty() {
                        let result = self.engine().insert_link(&link_state.url);
                        self.state.apply_edit_result(result);
                    }
                }
                self.state.active_modal = None;
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                if let Some(link_state) = &mut self.state.link_modal {
                    if link_state.cursor_pos > 0 && !link_state.url.is_empty() {
                        let pos = link_state.cursor_pos - 1;
                        link_state.url.remove(pos);
                        link_state.cursor_pos = pos;
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::Delete) => {
                if let Some(link_state) = &mut self.state.link_modal {
                    if link_state.cursor_pos < link_state.url.len() {
                        link_state.url.remove(link_state.cursor_pos);
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::ArrowLeft) => {
                if let Some(link_state) = &mut self.state.link_modal {
                    link_state.cursor_pos = link_state.cursor_pos.saturating_sub(1);
                }
                true
            }
            LogicalKey::Named(NamedKey::ArrowRight) => {
                if let Some(link_state) = &mut self.state.link_modal {
                    link_state.cursor_pos = (link_state.cursor_pos + 1).min(link_state.url.len());
                }
                true
            }
            LogicalKey::Named(NamedKey::Home) => {
                if let Some(link_state) = &mut self.state.link_modal {
                    link_state.cursor_pos = 0;
                }
                true
            }
            LogicalKey::Named(NamedKey::End) => {
                if let Some(link_state) = &mut self.state.link_modal {
                    link_state.cursor_pos = link_state.url.len();
                }
                true
            }
            LogicalKey::Character(c) => {
                if let Some(link_state) = &mut self.state.link_modal {
                    link_state.url.insert_str(link_state.cursor_pos, c);
                    link_state.cursor_pos += c.len();
                }
                true
            }
            LogicalKey::Named(NamedKey::Space) => {
                if let Some(link_state) = &mut self.state.link_modal {
                    link_state.url.insert(link_state.cursor_pos, ' ');
                    link_state.cursor_pos += 1;
                }
                true
            }
            _ => false,
        }
    }

    /// Handle keyboard input for the comment input modal.
    pub(in crate::ui) fn handle_comment_modal_keyboard(
        &mut self,
        kb: &KeyboardEvent,
        ctx: &mut EventCtx,
    ) -> bool {
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.comment_modal = None;
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                if let Some(modal) = self.state.comment_modal.take() {
                    if !modal.text.is_empty() {
                        // Use current selection or cursor position as the comment range
                        let range = if let Some(sel) = &self.state.selection {
                            CommentRange {
                                block_idx: sel.start.block_idx,
                                start_offset: sel.start.offset,
                                end_offset: sel.end.offset,
                            }
                        } else {
                            let cursor = self.state.cursor();
                            CommentRange {
                                block_idx: cursor.block_idx,
                                start_offset: cursor.offset,
                                end_offset: cursor.offset,
                            }
                        };
                        let _comment = self.engine().add_comment(&modal.text, range);
                        let comments = self.engine().get_comments().to_vec();
                        self.state.update_comments(comments);
                        self.state.show_toast("Comment added");
                    }
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                if let Some(modal) = &mut self.state.comment_modal {
                    if modal.cursor_pos > 0 && !modal.text.is_empty() {
                        let pos = modal.cursor_pos - 1;
                        modal.text.remove(pos);
                        modal.cursor_pos = pos;
                    }
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::Delete) => {
                if let Some(modal) = &mut self.state.comment_modal {
                    if modal.cursor_pos < modal.text.len() {
                        modal.text.remove(modal.cursor_pos);
                    }
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::ArrowLeft) => {
                if let Some(modal) = &mut self.state.comment_modal {
                    modal.cursor_pos = modal.cursor_pos.saturating_sub(1);
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::ArrowRight) => {
                if let Some(modal) = &mut self.state.comment_modal {
                    modal.cursor_pos = (modal.cursor_pos + 1).min(modal.text.len());
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::Home) => {
                if let Some(modal) = &mut self.state.comment_modal {
                    modal.cursor_pos = 0;
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::End) => {
                if let Some(modal) = &mut self.state.comment_modal {
                    modal.cursor_pos = modal.text.len();
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Character(c) => {
                if let Some(modal) = &mut self.state.comment_modal {
                    modal.text.insert_str(modal.cursor_pos, c);
                    modal.cursor_pos += c.len();
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::Space) => {
                if let Some(modal) = &mut self.state.comment_modal {
                    modal.text.insert(modal.cursor_pos, ' ');
                    modal.cursor_pos += 1;
                }
                ctx.request_paint();
                true
            }
            _ => false,
        }
    }

    /// Handle keyboard input when editing header or footer text.
    pub(in crate::ui) fn handle_header_footer_keyboard(&mut self, kb: &KeyboardEvent) -> bool {
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                // Commit and exit header/footer editing
                if self.state.editing_header {
                    let text = self.state.header_text.clone();
                    let result = self.engine().set_default_header(text);
                    self.state.apply_edit_result(result);
                    self.state.editing_header = false;
                } else if self.state.editing_footer {
                    let text = self.state.footer_text.clone();
                    let result = self.engine().set_default_footer(text);
                    self.state.apply_edit_result(result);
                    self.state.editing_footer = false;
                }
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                // Commit and exit
                if self.state.editing_header {
                    let text = self.state.header_text.clone();
                    let result = self.engine().set_default_header(text);
                    self.state.apply_edit_result(result);
                    self.state.editing_header = false;
                } else if self.state.editing_footer {
                    let text = self.state.footer_text.clone();
                    let result = self.engine().set_default_footer(text);
                    self.state.apply_edit_result(result);
                    self.state.editing_footer = false;
                }
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                if self.state.editing_header {
                    self.state.header_text.pop();
                } else {
                    self.state.footer_text.pop();
                }
                true
            }
            LogicalKey::Character(c) => {
                if self.state.editing_header {
                    self.state.header_text.push_str(c);
                } else {
                    self.state.footer_text.push_str(c);
                }
                true
            }
            LogicalKey::Named(NamedKey::Space) => {
                if self.state.editing_header {
                    self.state.header_text.push(' ');
                } else {
                    self.state.footer_text.push(' ');
                }
                true
            }
            _ => false,
        }
    }

    /// Handle keyboard input for the page setup margin field editing.
    pub(in crate::ui) fn handle_page_setup_margin_keyboard(&mut self, kb: &KeyboardEvent) -> bool {
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                if let Some(dialog) = &mut self.state.page_setup_dialog {
                    dialog.editing_margin_field = None;
                }
                true
            }
            LogicalKey::Named(NamedKey::Enter) | LogicalKey::Named(NamedKey::Tab) => {
                // Commit the current field value and advance to next field
                if let Some(dialog) = &mut self.state.page_setup_dialog {
                    if let Some(field_idx) = dialog.editing_margin_field {
                        if let Ok(value) = dialog.margin_edit_buffer.parse::<f32>() {
                            match field_idx {
                                0 => dialog.margin_top = value,
                                1 => dialog.margin_bottom = value,
                                2 => dialog.margin_left = value,
                                3 => dialog.margin_right = value,
                                _ => {}
                            }
                        }
                        // Tab advances to next field, Enter closes editing
                        if kb.logical_key == LogicalKey::Named(NamedKey::Tab) {
                            let next = (field_idx + 1) % 4;
                            let next_value = match next {
                                0 => dialog.margin_top,
                                1 => dialog.margin_bottom,
                                2 => dialog.margin_left,
                                3 => dialog.margin_right,
                                _ => 0.0,
                            };
                            dialog.editing_margin_field = Some(next);
                            dialog.margin_edit_buffer = format!("{:.1}", next_value);
                        } else {
                            dialog.editing_margin_field = None;
                        }
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                if let Some(dialog) = &mut self.state.page_setup_dialog {
                    dialog.margin_edit_buffer.pop();
                }
                true
            }
            LogicalKey::Character(c) => {
                // Allow digits, decimal point, and minus sign
                if let Some(dialog) = &mut self.state.page_setup_dialog {
                    for ch in c.chars() {
                        if ch.is_ascii_digit() || ch == '.' || ch == '-' {
                            dialog.margin_edit_buffer.push(ch);
                        }
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Handle keyboard input for the find/replace modal.
    pub(in crate::ui) fn handle_find_replace_keyboard(
        &mut self,
        kb: &KeyboardEvent,
        _ctx: &mut EventCtx,
    ) -> bool {
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.engine().clear_search();
                self.state.find_replace = None;
                self.state.active_modal = None;
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                if let Some(fr) = &self.state.find_replace {
                    if fr.show_replace && !fr.replacement.is_empty() {
                        let replacement = fr.replacement.clone();
                        let result = self.engine().replace_next(&replacement);
                        self.state.apply_edit_result(result);
                        // Refresh search matches
                        self.refresh_find_matches();
                    } else {
                        let result = self.engine().find_next();
                        self.state.apply_edit_result(result);
                        self.update_find_match_index();
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                if let Some(fr) = &mut self.state.find_replace {
                    if fr.cursor_pos > 0 && !fr.query.is_empty() {
                        let pos = fr.cursor_pos - 1;
                        fr.query.remove(pos);
                        fr.cursor_pos = pos;
                        self.refresh_find_matches();
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::Delete) => {
                if let Some(fr) = &mut self.state.find_replace {
                    if fr.cursor_pos < fr.query.len() {
                        fr.query.remove(fr.cursor_pos);
                        self.refresh_find_matches();
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::ArrowLeft) => {
                if let Some(fr) = &mut self.state.find_replace {
                    fr.cursor_pos = fr.cursor_pos.saturating_sub(1);
                }
                true
            }
            LogicalKey::Named(NamedKey::ArrowRight) => {
                if let Some(fr) = &mut self.state.find_replace {
                    fr.cursor_pos = (fr.cursor_pos + 1).min(fr.query.len());
                }
                true
            }
            LogicalKey::Named(NamedKey::Home) => {
                if let Some(fr) = &mut self.state.find_replace {
                    fr.cursor_pos = 0;
                }
                true
            }
            LogicalKey::Named(NamedKey::End) => {
                if let Some(fr) = &mut self.state.find_replace {
                    fr.cursor_pos = fr.query.len();
                }
                true
            }
            LogicalKey::Character(c) => {
                if kb.modifiers.control {
                    return false;
                }
                if let Some(fr) = &mut self.state.find_replace {
                    fr.query.insert_str(fr.cursor_pos, c);
                    fr.cursor_pos += c.len();
                    self.refresh_find_matches();
                }
                true
            }
            LogicalKey::Named(NamedKey::Space) => {
                if let Some(fr) = &mut self.state.find_replace {
                    fr.query.insert(fr.cursor_pos, ' ');
                    fr.cursor_pos += 1;
                    self.refresh_find_matches();
                }
                true
            }
            _ => false,
        }
    }

    fn handle_modal_keyboard(&mut self, kb: &KeyboardEvent, ctx: &mut EventCtx) -> bool {
        if self.state.page_setup_dialog.is_some() {
            if matches!(kb.logical_key, LogicalKey::Named(NamedKey::Escape)) {
                self.state.page_setup_dialog = None;
                ctx.request_paint();
                return true;
            }
            return true;
        }

        if self.state.print_preview.is_some() {
            match &kb.logical_key {
                LogicalKey::Named(NamedKey::Escape) => {
                    self.state.print_preview = None;
                    ctx.request_paint();
                    true
                }
                LogicalKey::Named(NamedKey::ArrowLeft) => {
                    if let Some(pp) = &mut self.state.print_preview {
                        pp.page_index = pp.page_index.saturating_sub(1);
                    }
                    ctx.request_paint();
                    true
                }
                LogicalKey::Named(NamedKey::ArrowRight) => {
                    if let Some(pp) = &mut self.state.print_preview {
                        let max_page = pp.page_count.saturating_sub(1);
                        pp.page_index = (pp.page_index + 1).min(max_page);
                    }
                    ctx.request_paint();
                    true
                }
                _ => true,
            }
        } else if self.state.word_count_modal {
            if matches!(kb.logical_key, LogicalKey::Named(NamedKey::Escape)) {
                self.state.word_count_modal = false;
                ctx.request_paint();
            }
            true
        } else if self.state.goto_modal.is_some() {
            self.handle_goto_modal_keyboard(kb, ctx)
        } else if self.state.special_char_modal.is_some() {
            if matches!(kb.logical_key, LogicalKey::Named(NamedKey::Escape)) {
                self.state.special_char_modal = None;
                ctx.request_paint();
            }
            true
        } else {
            false
        }
    }

    fn handle_goto_modal_keyboard(&mut self, kb: &KeyboardEvent, ctx: &mut EventCtx) -> bool {
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.goto_modal = None;
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                if let Some(goto) = self.state.goto_modal.take() {
                    if let Ok(value) = goto.input.parse::<usize>() {
                        match goto.mode {
                            state::GotoMode::Page => self.go_to_page(value.max(1)),
                            state::GotoMode::Line => self.go_to_line(value.max(1)),
                        }
                    }
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                if let Some(goto) = &mut self.state.goto_modal {
                    if goto.cursor_pos > 0 {
                        goto.cursor_pos -= 1;
                        goto.input.remove(goto.cursor_pos);
                    }
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::Delete) => {
                if let Some(goto) = &mut self.state.goto_modal {
                    if goto.cursor_pos < goto.input.len() {
                        goto.input.remove(goto.cursor_pos);
                    }
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::ArrowLeft) => {
                if let Some(goto) = &mut self.state.goto_modal {
                    goto.cursor_pos = goto.cursor_pos.saturating_sub(1);
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::ArrowRight) => {
                if let Some(goto) = &mut self.state.goto_modal {
                    goto.cursor_pos = (goto.cursor_pos + 1).min(goto.input.len());
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::Home) => {
                if let Some(goto) = &mut self.state.goto_modal {
                    goto.cursor_pos = 0;
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Named(NamedKey::End) => {
                if let Some(goto) = &mut self.state.goto_modal {
                    goto.cursor_pos = goto.input.len();
                }
                ctx.request_paint();
                true
            }
            LogicalKey::Character(c) => {
                if let Some(goto) = &mut self.state.goto_modal {
                    for ch in c.chars().filter(|ch| ch.is_ascii_digit()) {
                        goto.input.insert(goto.cursor_pos, ch);
                        goto.cursor_pos += ch.len_utf8();
                    }
                }
                ctx.request_paint();
                true
            }
            _ => true,
        }
    }

    fn go_to_page(&mut self, page: usize) {
        let scale = self.state.zoom / 100.0;
        let doc = self.state.current_document();
        let (_page_w, page_h_raw) = doc.page_setup.page_size_px();
        let page_stride = page_h_raw * scale + state::PAGE_GAP * scale;
        let page_count = self.state.layout_cache.num_pages().max(1);
        let page_idx = page.saturating_sub(1).min(page_count - 1);

        self.state.current_page = page_idx + 1;
        self.state.scroll_y = page_idx as f64 * page_stride;
    }

    fn go_to_line(&mut self, line: usize) {
        let scale = self.state.zoom / 100.0;
        let line_h = 20.0 * scale;

        self.state.scroll_y = line.saturating_sub(1) as f64 * line_h;
    }
}
