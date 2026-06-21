// ---------------------------------------------------------------------------
// Window event handling
// ---------------------------------------------------------------------------

use tench_ui::core::events::WindowEvent;
use tench_ui::prelude::*;

use super::KodocsApp;

impl KodocsApp {
    pub(super) fn handle_window_event_inner(&mut self, ctx: &mut EventCtx, event: &WindowEvent) {
        match event {
            WindowEvent::AnimFrame(ts) => {
                let ts_f64 = (*ts) as f64;

                let any_modal_active = self.state.active_modal.is_some()
                    || self.state.find_replace.is_some()
                    || self.state.link_modal.is_some()
                    || self.state.page_setup_dialog.is_some()
                    || self.state.goto_modal.is_some()
                    || self.state.special_char_modal.is_some()
                    || self.state.word_count_modal;
                if !any_modal_active {
                    let ticks = self.cursor_timer.update(*ts);
                    if ticks > 0 {
                        self.state.cursor_visible = !self.state.cursor_visible;
                        ctx.request_paint();
                    }
                }

                if let Some((msg, expiry)) = &self.state.toast {
                    if *expiry == 0.0 {
                        self.state.toast = Some((msg.clone(), ts_f64 + 3.0));
                        ctx.request_paint();
                    } else if ts_f64 >= *expiry {
                        self.state.toast = None;
                        ctx.request_paint();
                    }
                }

                self.process_dialog_results(ctx);

                if self.state.should_autosave(ts_f64) {
                    self.save_current_document();
                    self.state.mark_autosave_done(ts_f64);
                }

                ctx.request_anim_frame();
            }
            WindowEvent::Resize { width, height } => {
                self.state.last_window_size = (*width as f64, *height as f64);
                ctx.request_paint();
            }
            WindowEvent::Focused(gained) => {
                if *gained {
                    self.reset_cursor_blink();
                } else {
                    self.state.ctrl_pressed = false;
                }
                ctx.request_paint();
            }
            WindowEvent::FileDrop { paths } => {
                if let Some(path) = paths.first() {
                    self.open_file(path);
                }
            }
            _ => {}
        }
    }
}
