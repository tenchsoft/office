use super::*;

impl DocsApp {
    /// Text event handler extracted from Widget::on_text_event.
    pub(in crate::ui) fn handle_text_event_inner(&mut self, ctx: &mut EventCtx, event: &TextEvent) {
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

    /// Window event handler extracted from Widget::on_window_event.
    pub(in crate::ui) fn handle_window_event_inner(
        &mut self,
        ctx: &mut EventCtx,
        event: &WindowEvent,
    ) {
        match event {
            WindowEvent::AnimFrame(ts) => {
                // Only blink cursor when no modal is active
                let modal_active = self.state.active_modal.is_some()
                    || self.state.link_modal.is_some()
                    || self.state.page_setup_dialog.is_some()
                    || self.state.find_replace.is_some()
                    || self.state.comment_modal.is_some();
                let ticks = self.cursor_timer.update(*ts);
                if ticks > 0 && !modal_active {
                    self.state.cursor_visible = !self.state.cursor_visible;
                    ctx.request_paint();
                }
                // Autosave check
                if self.state.should_autosave((*ts) as f64) {
                    self.save_current_document();
                    self.state.mark_autosave_done((*ts) as f64);
                }
                // Toast auto-dismiss: set expiry on first frame, clear when expired
                if let Some((msg, expiry)) = &self.state.toast {
                    if *expiry == 0.0 {
                        self.state.toast = Some((msg.clone(), (*ts) as f64 + 3000.0));
                    } else if (*ts) as f64 >= *expiry {
                        self.state.toast = None;
                        ctx.request_paint();
                    }
                }
                // Keep requesting animation frames for cursor blink
                ctx.request_anim_frame();
            }
            WindowEvent::FileDrop { paths } => {
                // Ignore file drops while a modal is active to prevent
                // replacing document state behind the modal.
                if self.state.any_modal_open() {
                    return;
                }

                // Open the first .docx file from the dropped files
                if let Some(path) = paths.iter().find(|p| p.to_lowercase().ends_with(".docx")) {
                    self.open_file_from_path(path);
                    ctx.request_paint();
                } else if let Some(path) = paths.first() {
                    // Try opening any file
                    self.open_file_from_path(path);
                    ctx.request_paint();
                }
            }
            _ => {}
        }
    }
}
