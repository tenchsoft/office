use super::*;

impl Widget for KodocsApp {
    fn measure(&mut self, _ctx: &mut MeasureCtx, _axis: Axis, available: f64) -> f64 {
        available
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let size = ctx.size();

        // Ensure layout cache is warm before rendering
        self.state.ensure_layout_cache();
        // Sync maximized flag so the caption buttons paint the correct glyph.
        self.state.window_maximized = ctx.global.window_maximized;

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
        let dropdown_x = {
            p.push_clip(Rect::new(0.0, MENU_BAR_H, width, MENU_BAR_H + TOOLBAR_H));
            let x = paint_toolbar(
                p,
                cache,
                Rect::new(0.0, MENU_BAR_H, width, MENU_BAR_H + TOOLBAR_H),
                &self.state,
            );
            p.pop_clip();
            x
        };

        let main_y = MENU_BAR_H + TOOLBAR_H;
        let status_y = height - STATUS_BAR_H;
        let sidebar_open = self.state.show_style_panel || self.state.show_comments;
        let sidebar_w = if sidebar_open { STYLE_PANEL_W } else { 0.0 };
        let content_w = width - sidebar_w;

        if self.state.show_thumbnails && content_w > THUMB_PANEL_W + 480.0 {
            paint_thumbnails(p, cache, Rect::new(0.0, main_y, THUMB_PANEL_W, status_y));
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
            let sidebar_x = content_w;
            p.push_clip(Rect::new(sidebar_x, main_y, width, status_y));
            if self.state.show_comments {
                paint_comment_panel(
                    p,
                    cache,
                    Rect::new(sidebar_x, main_y, width, status_y),
                    &self.state,
                );
            } else {
                paint_style_panel(
                    p,
                    cache,
                    Rect::new(sidebar_x, main_y, width, status_y),
                    &self.state,
                );
            }
            p.pop_clip();
        }

        paint_status_bar(
            p,
            cache,
            Rect::new(0.0, status_y, width, height),
            &self.state,
        );

        // Modals
        if let Some(link_state) = &self.state.link_modal {
            paint_link_modal(p, cache, size, link_state);
        } else if let Some(fr_state) = &self.state.find_replace {
            paint_find_replace_modal(p, cache, size, fr_state);
        } else if self.state.page_setup_dialog.is_some() {
            paint_page_setup_dialog(p, cache, size, &self.state);
        } else if let Some(modal) = &self.state.active_modal {
            paint_docs_modal(p, cache, modal, self.state.hovered_menu_item);
        }

        // Context menu
        if let Some(ctx_menu) = &self.state.context_menu {
            paint_context_menu(p, cache, size, ctx_menu);
        }

        // Hanja conversion popup
        if let Some(hanja_state) = &self.state.hanja_popup {
            if !hanja_state.candidates.is_empty() {
                paint_hanja_popup(p, cache, size, hanja_state);
            }
        }

        // Equation editor dialog
        if self.state.equation_editor.is_some() {
            if let Some(eq_state) = &self.state.equation_editor {
                paint_equation_dialog(p, cache, size, eq_state);
            }
        }

        if let Some(toast) = &self.state.toast {
            paint_docs_toast(p, cache, size, &toast.0);
        }

        // Toolbar dropdowns rendered last for correct Z-order (on top of all modals)
        if self.state.open_dropdown.is_some() {
            let toolbar_rect = Rect::new(0.0, MENU_BAR_H, width, MENU_BAR_H + TOOLBAR_H);
            paint_toolbar_dropdowns(p, cache, toolbar_rect, dropdown_x, &self.state);
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
        Some("kodocs.root")
    }

    fn automation_children(&self, state: &WidgetState) -> Vec<UiAutomationNode> {
        let mut nodes =
            automation::kodocs_automation_nodes(&self.state, state.size, state.id.to_raw());
        if let Some(store) = &self.license_store {
            let next_id = nodes.last().map(|n| n.id).unwrap_or(0).saturating_add(1);
            automation::push_license_nodes(&mut nodes, next_id, store, "kodocs");
        }
        nodes
    }
}
