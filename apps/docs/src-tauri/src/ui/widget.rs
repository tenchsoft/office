use super::*;

impl Widget for DocsApp {
    fn measure(&mut self, _ctx: &mut MeasureCtx, _axis: Axis, available: f64) -> f64 {
        available
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let size = ctx.size();

        // Store window size for hit-test calculations
        self.state.last_window_size = (size.width, size.height);
        // Sync maximized flag so the caption buttons paint the correct glyph.
        self.state.window_maximized = ctx.global.window_maximized;

        // Ensure layout cache is warm before rendering
        self.state.ensure_layout_cache();

        // Sync undo/redo availability from the active session's engine
        let session = &self.sessions[self.active_session_idx];
        self.state.undo_available = session.engine.can_undo();
        self.state.redo_available = session.engine.can_redo();

        let p = &mut Painter::new(scene);
        let cache = &mut self.text_cache;
        let width = size.width;
        let height = size.height;

        p.fill_background(size, c_canvas_bg());
        paint_menu_bar(
            p,
            cache,
            Rect::new(0.0, 0.0, width, MENU_BAR_H),
            &self.state,
        );
        p.push_clip(Rect::new(0.0, MENU_BAR_H, width, MENU_BAR_H + TOOLBAR_H));
        paint_toolbar(
            p,
            cache,
            Rect::new(0.0, MENU_BAR_H, width, MENU_BAR_H + TOOLBAR_H),
            &self.state,
        );
        p.pop_clip();

        let main_y = MENU_BAR_H + TOOLBAR_H;
        let status_y = height - STATUS_BAR_H;
        let sidebar_open = self.state.show_style_panel || self.state.show_comments;
        let sidebar_w = if sidebar_open { STYLE_PANEL_W } else { 0.0 };
        let content_w = width - sidebar_w;

        if self.state.show_thumbnails && content_w > THUMB_PANEL_W + 480.0 {
            paint_thumbnails(
                p,
                cache,
                Rect::new(0.0, main_y, THUMB_PANEL_W, status_y),
                &self.state,
            );
        }

        paint_title_row(
            p,
            cache,
            Rect::new(0.0, main_y, content_w, main_y + TITLE_ROW_H),
            &self.state,
        );
        paint_ruler(
            p,
            cache,
            Rect::new(
                0.0,
                main_y + TITLE_ROW_H,
                content_w,
                main_y + TITLE_ROW_H + RULER_H,
            ),
            &self.state,
        );

        let workspace_y = main_y + TITLE_ROW_H + RULER_H;
        // Tab bar (only shown when multiple tabs are open)
        let tab_bar_h = if self.state.open_tabs.len() > 1 {
            32.0
        } else {
            0.0
        };
        let doc_y = workspace_y + tab_bar_h;
        if tab_bar_h > 0.0 {
            paint_tab_bar(
                p,
                cache,
                Rect::new(0.0, workspace_y, content_w, doc_y),
                &self.state,
            );
        }
        p.push_clip(Rect::new(0.0, doc_y, content_w, status_y));
        paint_document_area(
            p,
            cache,
            Rect::new(0.0, doc_y, content_w, status_y),
            &self.state,
        );
        p.pop_clip();

        if sidebar_open {
            let sidebar_layout = compute_sidebar(content_w, main_y, status_y);
            p.push_clip(sidebar_layout.area);
            if self.state.show_comments {
                paint_comment_panel(p, cache, sidebar_layout.area, &self.state);
            } else {
                paint_style_panel(p, cache, sidebar_layout.area, &self.state);
            }
            p.pop_clip();
        }

        paint_status_bar(
            p,
            cache,
            Rect::new(0.0, status_y, width, height),
            &self.state,
        );

        if let Some(link_state) = &self.state.link_modal {
            paint_link_modal(p, cache, size, link_state);
        } else if let Some(fr_state) = &self.state.find_replace {
            paint_find_replace_modal(p, cache, size, fr_state);
        } else if self.state.page_setup_dialog.is_some() {
            paint_page_setup_dialog(p, cache, size, &self.state);
        } else if self.state.print_preview.is_some() {
            paint_print_preview(p, cache, size, &self.state);
        } else if self.state.word_count_modal {
            paint_word_count_modal(p, cache, size, &self.state);
        } else if let Some(goto_state) = &self.state.goto_modal {
            paint_goto_modal(p, cache, size, goto_state, self.state.cursor_visible);
        } else if let Some(sc_state) = &self.state.special_char_modal {
            paint_special_char_modal(p, cache, size, sc_state);
        } else if let Some(modal) = &self.state.active_modal {
            paint_docs_modal(p, cache, size, modal, self.state.hovered_menu_item);
        }

        // Toolbar dropdowns render AFTER all other elements (Z-order fix)
        if self.state.open_dropdown.is_some() {
            let toolbar_rect = Rect::new(0.0, MENU_BAR_H, width, MENU_BAR_H + TOOLBAR_H);
            paint_toolbar_dropdowns(
                p,
                cache,
                toolbar_rect,
                &self.state,
                self.state.hovered_dropdown_item,
            );
        }

        paint_toolbar_tooltip(
            p,
            cache,
            Rect::new(0.0, MENU_BAR_H, width, MENU_BAR_H + TOOLBAR_H),
            &self.state,
        );

        // Paint context menu (on top of everything except toast)
        if let Some(cm) = &self.state.context_menu {
            paint_context_menu(p, cache, cm);
        }

        // Paint comment modal
        if let Some(comment_state) = &self.state.comment_modal {
            paint_comment_modal(p, cache, size, comment_state, self.state.cursor_visible);
        }

        if let Some((toast, _expiry)) = &self.state.toast {
            paint_docs_toast(p, cache, size, toast);
        }
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        self.handle_pointer_event(ctx, event);
    }

    fn on_text_event(&mut self, ctx: &mut EventCtx, event: &TextEvent) {
        self.handle_text_event_inner(ctx, event);
    }

    fn on_window_event(&mut self, ctx: &mut EventCtx, event: &WindowEvent) {
        self.handle_window_event_inner(ctx, event);
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        true
    }

    fn debug_id(&self) -> Option<&str> {
        Some("docs.root")
    }

    fn automation_children(&self, state: &WidgetState) -> Vec<UiAutomationNode> {
        automation::docs_automation_nodes(&self.state, state.size, state.id.to_raw())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
