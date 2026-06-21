// ---------------------------------------------------------------------------
// Modal keyboard handlers
// ---------------------------------------------------------------------------

use tench_ui::core::events::{KeyboardEvent, LogicalKey, NamedKey};
use tench_ui::prelude::*;

use super::{equation_editor, KodocsApp};

impl KodocsApp {
    pub(super) fn handle_link_modal_keyboard(&mut self, kb: &KeyboardEvent) -> bool {
        if !kb.is_pressed {
            return false;
        }
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.link_modal = None;
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                if let Some(link_state) = self.state.link_modal.take() {
                    if !link_state.url.is_empty() {
                        let result = self.engine().insert_link(&link_state.url);
                        self.state.apply_edit_result(result);
                    }
                }
                true
            }
            LogicalKey::Character(c) => {
                if let Some(link_state) = &mut self.state.link_modal {
                    link_state.url.push_str(c);
                }
                true
            }
            LogicalKey::Named(NamedKey::Backspace) | LogicalKey::Named(NamedKey::Delete) => {
                if let Some(link_state) = &mut self.state.link_modal {
                    link_state.url.pop();
                }
                true
            }
            LogicalKey::Named(NamedKey::ArrowLeft)
            | LogicalKey::Named(NamedKey::ArrowRight)
            | LogicalKey::Named(NamedKey::Home)
            | LogicalKey::Named(NamedKey::End) => true,
            LogicalKey::Named(NamedKey::Space) => {
                if let Some(link_state) = &mut self.state.link_modal {
                    link_state.url.push(' ');
                }
                true
            }
            _ => false,
        }
    }

    pub(super) fn handle_find_replace_keyboard(
        &mut self,
        kb: &KeyboardEvent,
        _ctx: &mut EventCtx,
    ) -> bool {
        if !kb.is_pressed {
            return false;
        }
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.engine().clear_search();
                self.state.find_replace = None;
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                self.refresh_find_matches();
                let result = self.engine().find_next();
                self.state.apply_edit_result(result);
                self.update_find_match_index();
                true
            }
            LogicalKey::Character(c) => {
                if let Some(fr) = &mut self.state.find_replace {
                    fr.query.push_str(c);
                    self.refresh_find_matches();
                }
                true
            }
            LogicalKey::Named(NamedKey::Backspace) | LogicalKey::Named(NamedKey::Delete) => {
                if let Some(fr) = &mut self.state.find_replace {
                    fr.query.pop();
                    self.refresh_find_matches();
                }
                true
            }
            LogicalKey::Named(NamedKey::ArrowLeft)
            | LogicalKey::Named(NamedKey::ArrowRight)
            | LogicalKey::Named(NamedKey::Home)
            | LogicalKey::Named(NamedKey::End) => true,
            LogicalKey::Named(NamedKey::Space) => {
                if let Some(fr) = &mut self.state.find_replace {
                    fr.query.push(' ');
                    self.refresh_find_matches();
                }
                true
            }
            _ => false,
        }
    }

    pub(super) fn handle_hanja_popup_keyboard(&mut self, kb: &KeyboardEvent) -> bool {
        if !kb.is_pressed {
            return false;
        }
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.hanja_popup = None;
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                if let Some(hanja_state) = self.state.hanja_popup.take() {
                    if let Some(candidate) = hanja_state.candidates.get(hanja_state.selected_idx) {
                        let hanja_text = candidate.split('(').next().unwrap_or(candidate).trim();
                        let result = self.engine().insert_text(hanja_text);
                        self.state.apply_edit_result(result);
                        self.reset_cursor_blink();
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::ArrowUp) => {
                if let Some(hanja_state) = &mut self.state.hanja_popup {
                    if hanja_state.selected_idx > 0 {
                        hanja_state.selected_idx -= 1;
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::ArrowDown) => {
                if let Some(hanja_state) = &mut self.state.hanja_popup {
                    if hanja_state.selected_idx + 1 < hanja_state.candidates.len() {
                        hanja_state.selected_idx += 1;
                    }
                }
                true
            }
            _ => false,
        }
    }

    pub(super) fn handle_equation_editor_keyboard(&mut self, kb: &KeyboardEvent) -> bool {
        if !kb.is_pressed {
            return false;
        }
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.equation_editor = None;
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                if let Some(eq_state) = self.state.equation_editor.take() {
                    if !eq_state.input.is_empty() {
                        let preview = equation_editor::render_equation_preview(&eq_state.input);
                        let result = self.engine().insert_text(&preview);
                        self.state.apply_edit_result(result);
                        self.reset_cursor_blink();
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                if let Some(eq_state) = &mut self.state.equation_editor {
                    if eq_state.cursor_pos > 0 {
                        let before = eq_state.input[..eq_state.cursor_pos - 1].to_string();
                        let after = eq_state.input[eq_state.cursor_pos..].to_string();
                        eq_state.input = format!("{before}{after}");
                        eq_state.cursor_pos -= 1;
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::Delete) => {
                if let Some(eq_state) = &mut self.state.equation_editor {
                    if eq_state.cursor_pos < eq_state.input.len() {
                        let before = eq_state.input[..eq_state.cursor_pos].to_string();
                        let after = eq_state.input[eq_state.cursor_pos + 1..].to_string();
                        eq_state.input = format!("{before}{after}");
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::ArrowLeft) => {
                if let Some(eq_state) = &mut self.state.equation_editor {
                    if eq_state.cursor_pos > 0 {
                        eq_state.cursor_pos -= 1;
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::ArrowRight) => {
                if let Some(eq_state) = &mut self.state.equation_editor {
                    if eq_state.cursor_pos < eq_state.input.len() {
                        eq_state.cursor_pos += 1;
                    }
                }
                true
            }
            LogicalKey::Character(c) => {
                if let Some(eq_state) = &mut self.state.equation_editor {
                    eq_state.input.insert_str(eq_state.cursor_pos, c);
                    eq_state.cursor_pos += c.len();
                }
                true
            }
            LogicalKey::Named(NamedKey::Space) => {
                if let Some(eq_state) = &mut self.state.equation_editor {
                    eq_state.input.insert(eq_state.cursor_pos, ' ');
                    eq_state.cursor_pos += 1;
                }
                true
            }
            _ => false,
        }
    }
}
