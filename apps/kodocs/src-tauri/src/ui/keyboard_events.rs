use super::*;

// ---------------------------------------------------------------------------
// Keyboard event handling
// ---------------------------------------------------------------------------

use tench_ui::core::events::{LogicalKey, NamedKey, TextEvent};

impl KodocsApp {
    pub(crate) fn handle_text_event_inner(&mut self, ctx: &mut EventCtx, event: &TextEvent) {
        // Track Ctrl key state for scroll/zoom disambiguation
        if let TextEvent::Keyboard(kb) = event {
            self.state.ctrl_pressed = kb.modifiers.control;
        }

        let changed = match event {
            TextEvent::Keyboard(kb) => self.handle_keyboard(kb, ctx),
            TextEvent::Ime(ImeEvent::Commit(text)) => {
                let result = self.engine().insert_text(text);
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            TextEvent::ClipboardPaste(text) => {
                let result = self.engine().insert_text(text);
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            _ => false,
        };

        if changed {
            ctx.request_paint();
        }
    }

    fn handle_keyboard(&mut self, kb: &KeyboardEvent, ctx: &mut EventCtx) -> bool {
        if !kb.is_pressed {
            return false;
        }

        // If Hanja popup is open, route keyboard input to it
        if self.state.hanja_popup.is_some() {
            return self.handle_hanja_popup_keyboard(kb);
        }

        // If equation editor is open, route keyboard input to it
        if self.state.equation_editor.is_some() {
            return self.handle_equation_editor_keyboard(kb);
        }

        // If link modal is open, route keyboard input to it
        if self.state.link_modal.is_some() {
            return self.handle_link_modal_keyboard(kb);
        }

        // If license modal is open, route keyboard input to it
        if self.state.license_modal.is_some() {
            return self.handle_license_modal_keyboard(kb);
        }

        // If find/replace modal is open, route keyboard input to it
        if self.state.find_replace.is_some() {
            return self.handle_find_replace_keyboard(kb, ctx);
        }

        // If page setup dialog is open
        if self.state.page_setup_dialog.is_some() {
            if matches!(kb.logical_key, LogicalKey::Named(NamedKey::Escape)) {
                self.state.page_setup_dialog = None;
                return true;
            }
            return false;
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
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("a") => {
                    let result = self.engine().select_all();
                    self.state.apply_edit_result(result);
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("k") => {
                    self.state.link_modal = Some(LinkModalState {
                        url: String::new(),
                        display_text: String::new(),
                        cursor_pos: 0,
                    });
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("m") => {
                    self.state.comments_collapsed = !self.state.comments_collapsed;
                    true
                }
                LogicalKey::Character(c) if c.eq_ignore_ascii_case("t") => {
                    self.state.toast = Some(("변경 추적 전환".into(), 0.0));
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
                    self.state.zoom = (self.state.zoom + 10.0).min(200.0);
                    true
                }
                // Ctrl+Minus: zoom out
                LogicalKey::Character(c) if c == "-" => {
                    self.state.zoom = (self.state.zoom - 10.0).max(50.0);
                    true
                }
                // Ctrl+0: reset zoom
                LogicalKey::Character(c) if c == "0" => {
                    self.state.zoom = 100.0;
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

        // Non-modifier keys
        match &kb.logical_key {
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
            LogicalKey::Named(NamedKey::Enter) => {
                let result = self.engine().insert_text("\n");
                self.state.apply_edit_result(result);
                self.reset_cursor_blink();
                true
            }
            LogicalKey::Named(NamedKey::Tab) => {
                if kb.modifiers.control {
                    // Ctrl+Tab: switch tab
                    if self.state.open_tabs.len() > 1 {
                        let next = (self.state.active_tab_idx + 1) % self.state.open_tabs.len();
                        self.switch_to_tab(next);
                    }
                    true
                } else {
                    let result = self.engine().insert_text("    ");
                    self.state.apply_edit_result(result);
                    self.reset_cursor_blink();
                    true
                }
            }
            LogicalKey::Named(NamedKey::Escape) => {
                // Close any open modal/dropdown
                if self.state.active_modal.is_some() {
                    self.state.active_modal = None;
                    return true;
                }
                if self.state.open_dropdown.is_some() {
                    self.state.open_dropdown = None;
                    return true;
                }
                if self.state.find_replace.is_some() {
                    self.state.find_replace = None;
                    return true;
                }
                if self.state.link_modal.is_some() {
                    self.state.link_modal = None;
                    return true;
                }
                false
            }
            // F9: Hanja conversion
            LogicalKey::Named(NamedKey::F(9)) => {
                self.perform_hanja_conversion();
                true
            }
            // Regular character input
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
            _ => false,
        }
    }
}
