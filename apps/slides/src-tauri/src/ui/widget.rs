use super::*;

impl Widget for SlidesApp {
    fn measure(&mut self, _ctx: &mut MeasureCtx, _axis: Axis, available: f64) -> f64 {
        available
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        // Phase 11: Auto-dismiss toast after ~3 seconds.
        self.state.tick_toast();

        let size = ctx.size();
        let theme = ctx.theme();
        // Sync maximized flag so the caption buttons paint the correct glyph.
        self.state.window_maximized = ctx.global.window_maximized;
        let mut p = Painter::new(scene);

        p.fill_background(size, theme.background);
        paint_toolbar(
            &self.state,
            &mut p,
            theme,
            Rect::new(0.0, 0.0, size.width, TOOLBAR_H),
        );
        // Caption buttons sit at the top-right of the toolbar header.
        paint_window_controls(
            &mut p,
            size.width,
            TOOLBAR_H,
            self.state.window_maximized,
            self.state.window_control_hovered,
        );
        paint_filmstrip(
            &self.state,
            &mut p,
            theme,
            Rect::new(0.0, TOOLBAR_H, THUMB_W, size.height - NOTES_H),
        );
        paint_slide_canvas(
            &self.state,
            &mut p,
            theme,
            Rect::new(
                THUMB_W,
                TOOLBAR_H,
                size.width - PROPS_W,
                size.height - NOTES_H,
            ),
        );
        paint_properties(
            &self.state,
            &mut p,
            theme,
            Rect::new(
                size.width - PROPS_W,
                TOOLBAR_H,
                size.width,
                size.height - NOTES_H,
            ),
        );
        paint_notes(
            &self.state,
            &mut p,
            theme,
            Rect::new(0.0, size.height - NOTES_H, size.width, size.height),
        );

        if let Some(modal) = &self.state.active_modal {
            paint_modal(&mut p, theme, size, modal);
        }
        if let Some(toast) = &self.state.toast {
            paint_slides_toast(&mut p, theme, size, toast);
        }
        if self.state.show_animation_panel {
            paint_modal(&mut p, theme, size, &ActiveModal::AnimationPanel);
        }
        if self.state.presenting {
            paint_presenter_overlay(&mut p, theme, size, &self.state);
        }
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        self.process_dialog_results(ctx);
        match event {
            PointerEvent::Down(e) => self.handle_pointer_down(ctx, e),
            PointerEvent::Move(e) => self.handle_pointer_move(ctx, e),
            PointerEvent::Up(e) => self.handle_pointer_up(ctx, e),
            PointerEvent::Scroll(e) => self.handle_scroll(ctx, e),
            _ => {}
        }
    }

    fn on_text_event(&mut self, ctx: &mut EventCtx, event: &TextEvent) {
        // Phase 3: Handle IME commit events for text editing
        if let TextEvent::Ime(ImeEvent::Commit(text)) = event {
            if self.state.text_edit.editing {
                for c in text.chars() {
                    self.state.text_edit_insert_char(c);
                }
                ctx.request_paint();
                return;
            }
        }

        let TextEvent::Keyboard(kb) = event else {
            return;
        };

        // Phase 9: Handle Space key release for pan mode
        if !kb.is_pressed {
            if matches!(kb.logical_key, LogicalKey::Named(NamedKey::Space)) && self.state.space_held
            {
                self.state.space_held = false;
                if self.state.interaction.mode == DragMode::Pan {
                    self.state.interaction.mode = DragMode::None;
                }
                ctx.request_paint();
            }
            return;
        }

        // Phase 3: Route keyboard input to text editing when active
        if self.state.text_edit.editing && !kb.modifiers.control && !kb.modifiers.alt {
            let changed = match &kb.logical_key {
                LogicalKey::Character(c) => {
                    for ch in c.chars() {
                        self.state.text_edit_insert_char(ch);
                    }
                    true
                }
                LogicalKey::Named(NamedKey::Backspace) => {
                    self.state.text_edit_backspace();
                    true
                }
                LogicalKey::Named(NamedKey::Delete) => {
                    self.state.text_edit_delete();
                    true
                }
                LogicalKey::Named(NamedKey::ArrowLeft) => {
                    self.state.text_edit_move_cursor(-1);
                    true
                }
                LogicalKey::Named(NamedKey::ArrowRight) => {
                    self.state.text_edit_move_cursor(1);
                    true
                }
                LogicalKey::Named(NamedKey::Enter) => {
                    self.state.end_text_edit();
                    true
                }
                LogicalKey::Named(NamedKey::Escape) => {
                    self.state.end_text_edit();
                    true
                }
                _ => false,
            };
            if changed {
                ctx.request_paint();
            }
            return;
        }

        let changed = match &kb.logical_key {
            // Phase 11: Arrow nudge for selected elements
            LogicalKey::Named(NamedKey::ArrowLeft)
                if self.state.selected_element.is_some() && !self.state.presenting =>
            {
                let delta = if kb.modifiers.shift { -10.0 } else { -1.0 };
                self.state.nudge_selected_element(delta, 0.0);
                true
            }
            LogicalKey::Named(NamedKey::ArrowRight)
                if self.state.selected_element.is_some() && !self.state.presenting =>
            {
                let delta = if kb.modifiers.shift { 10.0 } else { 1.0 };
                self.state.nudge_selected_element(delta, 0.0);
                true
            }
            LogicalKey::Named(NamedKey::ArrowUp)
                if self.state.selected_element.is_some() && !self.state.presenting =>
            {
                let delta = if kb.modifiers.shift { -10.0 } else { -1.0 };
                self.state.nudge_selected_element(0.0, delta);
                true
            }
            LogicalKey::Named(NamedKey::ArrowDown)
                if self.state.selected_element.is_some() && !self.state.presenting =>
            {
                let delta = if kb.modifiers.shift { 10.0 } else { 1.0 };
                self.state.nudge_selected_element(0.0, delta);
                true
            }
            LogicalKey::Named(NamedKey::ArrowUp) | LogicalKey::Named(NamedKey::PageUp) => {
                if self.state.presenting {
                    self.state.presentation_prev_slide()
                } else {
                    self.state.previous_slide()
                }
            }
            LogicalKey::Named(NamedKey::ArrowDown) | LogicalKey::Named(NamedKey::PageDown) => {
                if self.state.presenting {
                    self.state.presentation_next_slide()
                } else {
                    self.state.next_slide()
                }
            }
            LogicalKey::Named(NamedKey::Delete) | LogicalKey::Named(NamedKey::Backspace)
                if !self.state.text_edit.editing =>
            {
                self.state.delete_selected_element()
            }
            LogicalKey::Named(NamedKey::Delete) | LogicalKey::Named(NamedKey::Backspace) => false,
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("d") => {
                self.state.duplicate_selected_element()
            }
            LogicalKey::Character(c)
                if kb.modifiers.control && !kb.modifiers.shift && c.eq_ignore_ascii_case("s") =>
            {
                self.save_current_presentation();
                true
            }
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("p") => {
                if self.state.presenting {
                    self.state.stop_presentation();
                } else {
                    self.state.start_presentation();
                }
                true
            }
            // Phase 1.8: Undo/Redo
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("z") => {
                if kb.modifiers.shift {
                    self.state.redo()
                } else {
                    self.state.undo()
                }
            }
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("y") => {
                self.state.redo()
            }
            // Phase 8.3: Clipboard
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("c") => {
                self.state.copy_selected();
                false
            }
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("x") => {
                self.state.cut_selected();
                true
            }
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("v") => {
                self.state.paste();
                true
            }
            // Phase 9.6: Zoom
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("=") => {
                self.state.zoom_in();
                true
            }
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("-") => {
                self.state.zoom_out();
                true
            }
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("0") => {
                self.state.zoom_reset();
                true
            }
            // Phase 8.1: Find
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("f") => {
                self.state.active_modal = Some(ActiveModal::FindReplace {
                    find_text: String::new(),
                    replace_text: String::new(),
                });
                true
            }
            LogicalKey::Named(NamedKey::Escape) => {
                if self.state.presenting {
                    self.state.stop_presentation();
                } else if self.state.text_edit.editing {
                    self.state.end_text_edit();
                } else {
                    self.state.active_modal = None;
                }
                true
            }
            LogicalKey::Named(NamedKey::Enter) if self.state.presenting => {
                self.state.presentation_next_slide()
            }
            // Phase 6.5: Laser pointer toggle
            LogicalKey::Character(c) if self.state.presenting && c.eq_ignore_ascii_case("l") => {
                self.state.toggle_laser_pointer();
                true
            }
            LogicalKey::Named(NamedKey::Enter) => false,
            // Phase 9: Space for hand-pan
            LogicalKey::Named(NamedKey::Space) if !self.state.text_edit.editing => {
                self.state.space_held = true;
                true
            }
            // Phase 9: Ctrl+G toggle grid
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("g") => {
                self.state.show_grid = !self.state.show_grid;
                true
            }
            // Phase 10: Ctrl+Shift+S = Save As
            LogicalKey::Character(c)
                if kb.modifiers.control && kb.modifiers.shift && c.eq_ignore_ascii_case("s") =>
            {
                self.state.active_modal = Some(ActiveModal::SaveAs);
                true
            }
            // Phase 10: Ctrl+E = Export
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("e") => {
                self.state.active_modal = Some(ActiveModal::Export { format_index: 0 });
                true
            }
            // Phase 10: ? = Keyboard shortcuts help
            LogicalKey::Character(c) if c == "?" && !self.state.text_edit.editing => {
                self.state.active_modal = Some(ActiveModal::KeyboardShortcuts);
                true
            }
            _ => false,
        };

        if changed {
            ctx.request_paint();
        }
    }

    fn accessibility_tree(&self, state: &WidgetState) -> AccessibilityNode {
        AccessibilityNode {
            role: AccessRole::Window,
            label: Some("Tench Slides".to_string()),
            value: Some(self.state.status_line().to_string()),
            focused: state.is_focused,
            disabled: state.is_disabled,
            children: Vec::new(),
        }
    }

    fn automation_children(&self, state: &WidgetState) -> Vec<UiAutomationNode> {
        let mut nodes =
            automation::slides_automation_nodes(&self.state, state.size, state.id.to_raw());
        if let Some(store) = &self.license_store {
            let next_id = nodes.last().map(|n| n.id).unwrap_or(0).saturating_add(1);
            automation::push_license_nodes(&mut nodes, next_id, store, "slides");
        }
        nodes
    }

    fn debug_id(&self) -> Option<&str> {
        Some("slides.root")
    }
}
