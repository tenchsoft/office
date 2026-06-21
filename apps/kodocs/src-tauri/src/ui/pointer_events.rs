use super::*;

// ---------------------------------------------------------------------------
// Pointer event handling
// ---------------------------------------------------------------------------

impl KodocsApp {
    pub(crate) fn handle_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        // Process any pending dialog results before handling the event
        self.process_dialog_results(ctx);

        match event {
            PointerEvent::Down(e) => {
                let x = e.pos.x;
                let y = e.pos.y;

                self.state.ctrl_pressed = false;

                // Check menu bar click
                if y < MENU_BAR_H {
                    let win_w = self.state.last_window_size.0;
                    if let Some(ctrl) = window_control_at(x, y, win_w, MENU_BAR_H) {
                        match ctrl {
                            WindowControl::Close => ctx.submit_window_action(WindowAction::Close),
                            WindowControl::Minimize => {
                                ctx.submit_window_action(WindowAction::Minimize)
                            }
                            WindowControl::MaximizeRestore => {
                                ctx.submit_window_action(WindowAction::ToggleMaximize)
                            }
                        }
                        return;
                    }
                    // Notification label (between 도움말 and title pill). Only
                    // hot when the label is currently visible.
                    if crate::ui::menu_bar::notification_label_message(&self.state).is_some() {
                        let label_rect = crate::ui::menu_bar::notification_label_rect(
                            Rect::new(0.0, 0.0, win_w, MENU_BAR_H),
                        );
                        if label_rect.contains(e.pos) {
                            // Open pricing page in default browser via shell plugin.
                            if let Some(handle) = &self.app_handle {
                                #[allow(deprecated)]
                                use tauri_plugin_shell::ShellExt;
                                #[allow(deprecated)]
                                let _ = handle.shell().open("https://tenchsoft.com/pricing", None);
                            }
                            return;
                        }
                    }
                    if let Some(menu_name) = menu_at(x) {
                        self.state.active_modal = Some(menu_name.to_string());
                        ctx.request_paint();
                        return;
                    }
                    // Empty menu bar space: begin a window drag-move.
                    ctx.submit_window_action(WindowAction::StartDrag);
                    return;
                }

                // Check toolbar click
                if (MENU_BAR_H..MENU_BAR_H + TOOLBAR_H).contains(&y) {
                    // Check if a dropdown is open and we clicked on an item
                    if self.state.open_dropdown.is_some() {
                        // Check font family dropdown hit
                        if self.state.open_dropdown == Some(ToolbarDropdown::FontFamily) {
                            if let Some(idx) = font_family_dropdown_hit(x, y) {
                                self.handle_toolbar_action(ToolbarAction::FontFamilyItem(idx));
                                ctx.request_paint();
                                return;
                            }
                        }

                        // Check font size dropdown hit
                        if let Some(idx) = font_size_dropdown_hit(x, y) {
                            self.handle_toolbar_action(ToolbarAction::FontSizeItem(idx));
                            ctx.request_paint();
                            return;
                        }

                        // Check paragraph style dropdown hit
                        if let Some(idx) = paragraph_style_dropdown_hit(x, y) {
                            self.handle_toolbar_action(ToolbarAction::ParagraphStyleItem(idx));
                            ctx.request_paint();
                            return;
                        }

                        // Check table grid hit
                        if let Some((rows, cols)) = table_grid_hit(x, y) {
                            let result = self.engine().insert_table(rows, cols);
                            self.state.apply_edit_result(result);
                            self.state.open_dropdown = None;
                            ctx.request_paint();
                            return;
                        }

                        // Check color picker hit
                        if let Some(color) = color_picker_hit(x, y) {
                            match self.state.open_dropdown {
                                Some(ToolbarDropdown::ColorPicker) => {
                                    self.state.selected_text_color = Some(color.clone());
                                    let result = self.engine().set_text_color(color);
                                    self.state.apply_edit_result(result);
                                }
                                Some(ToolbarDropdown::MarkPicker) => {
                                    self.state.selected_bg_color = Some(color.clone());
                                    let result = self.engine().set_background_color(color);
                                    self.state.apply_edit_result(result);
                                }
                                _ => {}
                            }
                            self.state.open_dropdown = None;
                            ctx.request_paint();
                            return;
                        }

                        // Close dropdown if clicked elsewhere
                        self.state.open_dropdown = None;
                    }

                    if let Some(action) = toolbar_action_at(x) {
                        self.handle_toolbar_action(action);
                        ctx.request_paint();
                        return;
                    }
                    return;
                }

                // Check modal click
                if self.state.active_modal.is_some() {
                    if let Some(modal_title) = &self.state.active_modal.clone() {
                        let items = menu_items_for(modal_title);
                        let item_h = 26.0;
                        let modal_x = 12.0;
                        let modal_y = MENU_BAR_H;
                        if x >= modal_x && x < modal_x + 220.0 && y >= modal_y + 8.0 {
                            let rel_y = y - modal_y - 8.0;
                            let idx = (rel_y / item_h) as usize;
                            if idx < items.len() {
                                let item = items[idx].to_string();
                                self.state.active_modal = None;
                                self.handle_menu_item(&item, ctx);
                                return;
                            }
                        }
                    }
                    // Click outside modal closes it
                    self.state.active_modal = None;
                    ctx.request_paint();
                    return;
                }

                // License modal click handling (Close and Activate buttons).
                if self.state.license_modal.is_some() {
                    let size = Size::new(
                        self.state.last_window_size.0,
                        self.state.last_window_size.1,
                    );
                    let modal = crate::ui::chrome::license_modal_rect(size);
                    let close_rect = crate::ui::chrome::license_modal_close_rect(modal);
                    if close_rect.contains(e.pos) {
                        self.state.license_modal = None;
                        ctx.request_paint();
                        return;
                    }
                    let activate_rect = crate::ui::chrome::license_modal_activate_rect(modal);
                    if activate_rect.contains(e.pos) {
                        if let Some(store) = self.license_store.clone() {
                            let key = self
                                .state
                                .license_modal
                                .as_ref()
                                .map(|m| m.license_key_input.clone())
                                .unwrap_or_default();
                            if !key.is_empty() {
                                match tench_update_client::activate_license(
                                    &store,
                                    None,
                                    &key,
                                    "kodocs",
                                    env!("CARGO_PKG_VERSION"),
                                ) {
                                    Ok(()) => {
                                        if let Some(m) = &mut self.state.license_modal {
                                            m.status_message = "Activated".into();
                                            m.busy = false;
                                        }
                                    }
                                    Err(err) => {
                                        if let Some(m) = &mut self.state.license_modal {
                                            m.status_message =
                                                format!("Activation failed: {err}");
                                            m.busy = false;
                                        }
                                    }
                                }
                            }
                        }
                        ctx.request_paint();
                        return;
                    }
                }

                if self.handle_hanja_popup_click(ctx, x, y) {
                    return;
                }
                if self.handle_equation_editor_click(ctx, x, y) {
                    return;
                }
                if self.handle_link_modal_click(ctx, x, y) {
                    return;
                }
                if self.handle_find_replace_click(ctx, x, y) {
                    return;
                }
                if self.handle_page_setup_click(ctx, x, y) {
                    return;
                }

                // Status bar click -> zoom buttons
                let status_y = self.state.last_window_size.1 - STATUS_BAR_H;
                if y >= status_y {
                    let w = self.state.last_window_size.0;
                    // Zoom out button
                    if x >= w - 160.0 && x <= w - 136.0 {
                        self.state.zoom = (self.state.zoom - 10.0).max(50.0);
                        ctx.request_paint();
                        return;
                    }
                    // Zoom in button
                    if x >= w - 52.0 && x <= w - 28.0 {
                        self.state.zoom = (self.state.zoom + 10.0).min(200.0);
                        ctx.request_paint();
                        return;
                    }
                    return;
                }

                // Document area click -> place cursor
                let main_y = MENU_BAR_H + TOOLBAR_H;
                let title_y = main_y + TITLE_ROW_H;
                let workspace_y = main_y + TITLE_ROW_H + RULER_H;
                let tab_bar_h = if self.state.open_tabs.len() > 1 {
                    32.0
                } else {
                    0.0
                };
                let doc_y = workspace_y + tab_bar_h;

                // Right-click -> context menu
                if e.button == tench_ui::core::events::PointerButton::Secondary {
                    let menu_type = if y >= workspace_y && y < doc_y && tab_bar_h > 0.0 {
                        state::ContextMenuType::Tab
                    } else if y > doc_y {
                        // Check if click is on an image or table block
                        let doc = self.state.current_document();
                        let mut is_image = false;
                        let mut is_table = false;
                        if let Some(cursor) = self.click_to_cursor(x, y, doc_y) {
                            if cursor.block_idx < doc.content.len() {
                                match &doc.content[cursor.block_idx] {
                                    tench_document_core::BlockNode::Image { .. } => is_image = true,
                                    tench_document_core::BlockNode::Table { .. } => is_table = true,
                                    _ => {}
                                }
                            }
                        }
                        if is_image {
                            state::ContextMenuType::Image
                        } else if is_table {
                            state::ContextMenuType::TableCell
                        } else {
                            state::ContextMenuType::Text
                        }
                    } else {
                        state::ContextMenuType::Text
                    };
                    self.state.context_menu = Some(state::ContextMenuState {
                        x,
                        y,
                        menu_type,
                        hovered_item: None,
                    });
                    ctx.request_paint();
                    return;
                }

                // Left-click on context menu item
                if self.state.context_menu.is_some() {
                    if let Some(ctx_menu) = self.state.context_menu.take() {
                        let items = state::context_menu_items(&ctx_menu.menu_type);
                        let item_h = 28.0;
                        let menu_w = 200.0;
                        if x >= ctx_menu.x && x <= ctx_menu.x + menu_w && y >= ctx_menu.y {
                            let rel_y = y - ctx_menu.y;
                            let idx = (rel_y / item_h) as usize;
                            if idx < items.len() {
                                self.handle_context_menu_item(
                                    items[idx], ctx_menu.x, ctx_menu.y, ctx,
                                );
                                ctx.request_paint();
                                return;
                            }
                        }
                    }
                    ctx.request_paint();
                    return;
                }

                // Check image resize handle hit
                if let Some(handle) = self.hit_test_image_resize_handle(x, y) {
                    let doc = self.state.current_document();
                    if let Some(cursor) = self.click_to_cursor(x, y, doc_y) {
                        if cursor.block_idx < doc.content.len() {
                            if let tench_document_core::BlockNode::Image { width, height, .. } =
                                &doc.content[cursor.block_idx]
                            {
                                self.state.image_resize_drag = Some(state::ImageResizeDrag {
                                    block_idx: cursor.block_idx,
                                    handle,
                                    start_width: width.unwrap_or(200.0) as f64,
                                    start_height: height.unwrap_or(200.0) as f64,
                                    start_x: x,
                                    start_y: y,
                                });
                            }
                        }
                    }
                    ctx.request_paint();
                    return;
                }

                // Ruler click -> start ruler drag
                if y >= title_y && y < workspace_y {
                    let ruler_rect =
                        Rect::new(0.0, title_y, self.state.last_window_size.0, workspace_y);
                    if let Some((target, _)) = ruler_hit_test(x, ruler_rect, &self.state) {
                        self.state.ruler_drag = Some(target);
                    }
                } else if y > doc_y {
                    if let Some(cursor) = self.click_to_cursor(x, y, doc_y) {
                        let result = self.engine().select(cursor.clone(), cursor);
                        self.state.apply_edit_result(result);
                        self.reset_cursor_blink();
                    }
                } else if tab_bar_h > 0.0 && y >= workspace_y && y < doc_y {
                    // Tab bar click – switch tab
                    let tab_width = 160.0;
                    let idx = (x / tab_width).min(self.state.open_tabs.len() as f64 - 1.0) as usize;
                    self.switch_to_tab(idx);
                }
                ctx.request_paint();
            }
            PointerEvent::Move(e) => {
                let x = e.pos.x;
                let y = e.pos.y;

                // Track caption button hover for visual feedback.
                let win_w = self.state.last_window_size.0;
                let new_hover = window_control_at(x, y, win_w, MENU_BAR_H);
                if new_hover != self.state.window_control_hovered {
                    self.state.window_control_hovered = new_hover;
                    ctx.request_paint();
                }

                // Handle image resize drag
                if self.state.image_resize_drag.is_some()
                    && e.buttons
                        .has(tench_ui::core::events::PointerButton::Primary)
                {
                    let drag_data = self.state.image_resize_drag;
                    if let Some(drag) = drag_data {
                        let dx = x - drag.start_x;
                        let dy = y - drag.start_y;
                        let new_w = (drag.start_width + dx).max(20.0);
                        let new_h = (drag.start_height + dy).max(20.0);
                        let result = self.engine().set_image_size(
                            drag.block_idx,
                            new_w as f32,
                            new_h as f32,
                        );
                        self.state.apply_edit_result(result);
                        self.state.image_resize_drag = Some(drag);
                        ctx.request_paint();
                        return;
                    }
                }

                // Handle ruler drag
                if self.state.ruler_drag.is_some()
                    && e.buttons
                        .has(tench_ui::core::events::PointerButton::Primary)
                {
                    let main_y = MENU_BAR_H + TOOLBAR_H;
                    let ruler_rect = Rect::new(
                        0.0,
                        main_y + TITLE_ROW_H,
                        10000.0,
                        main_y + TITLE_ROW_H + RULER_H,
                    );
                    match self.state.ruler_drag.unwrap() {
                        RulerDragTarget::LeftMargin => {
                            let mm = ruler_drag_to_margin(x, ruler_rect, &self.state);
                            let existing = self.state.current_document().page_setup.margins;
                            let result = self.engine().set_margins(tench_document_core::Margins {
                                left: mm,
                                ..existing
                            });
                            self.state.apply_edit_result(result);
                        }
                        RulerDragTarget::RightMargin => {
                            let mm = ruler_drag_to_margin(x, ruler_rect, &self.state);
                            let existing = self.state.current_document().page_setup.margins;
                            let result = self.engine().set_margins(tench_document_core::Margins {
                                right: mm,
                                ..existing
                            });
                            self.state.apply_edit_result(result);
                        }
                        RulerDragTarget::IndentLeft => {
                            let indent = ruler_drag_to_indent(x, ruler_rect, &self.state);
                            let result = self.engine().set_indent_left(indent);
                            self.state.apply_edit_result(result);
                        }
                        RulerDragTarget::IndentRight => {
                            let indent = ruler_drag_to_indent(x, ruler_rect, &self.state);
                            let result = self.engine().set_indent_right(indent);
                            self.state.apply_edit_result(result);
                        }
                        RulerDragTarget::IndentFirstLine => {
                            let indent = ruler_drag_to_indent(x, ruler_rect, &self.state);
                            let result = self.engine().set_indent_first_line(indent);
                            self.state.apply_edit_result(result);
                        }
                    }
                    ctx.request_paint();
                    return;
                }

                // Handle text selection drag
                if e.buttons
                    .has(tench_ui::core::events::PointerButton::Primary)
                {
                    if let Some(start) = self.drag_start.clone() {
                        let main_y = MENU_BAR_H + TOOLBAR_H;
                        let workspace_y = main_y + TITLE_ROW_H + RULER_H;
                        let tab_bar_h = if self.state.open_tabs.len() > 1 {
                            32.0
                        } else {
                            0.0
                        };
                        let doc_y = workspace_y + tab_bar_h;
                        if y >= doc_y {
                            if let Some(end_cursor) = self.click_to_cursor(x, y, doc_y) {
                                let result = self.engine().select(start, end_cursor);
                                self.state.apply_edit_result(result);
                                ctx.request_paint();
                            }
                        }
                    }
                }

                // Menu modal hover highlighting
                if let Some(modal_title) = &self.state.active_modal {
                    let items = menu_items_for(modal_title);
                    let item_h = 26.0;
                    let modal_x = 12.0;
                    let modal_y = MENU_BAR_H;
                    let modal_w = 220.0;
                    if x >= modal_x && x < modal_x + modal_w && y >= modal_y + 8.0 {
                        let rel_y = y - modal_y - 8.0;
                        let idx = (rel_y / item_h) as usize;
                        self.state.hovered_menu_item =
                            if idx < items.len() { Some(idx) } else { None };
                    } else {
                        self.state.hovered_menu_item = None;
                    }
                    ctx.request_paint();
                }

                // Context menu hover highlighting
                if let Some(ctx_menu) = &mut self.state.context_menu {
                    let items = state::context_menu_items(&ctx_menu.menu_type);
                    let item_h = 28.0;
                    let menu_w = 200.0;
                    if x >= ctx_menu.x && x <= ctx_menu.x + menu_w && y >= ctx_menu.y {
                        let rel_y = y - ctx_menu.y;
                        let idx = (rel_y / item_h) as usize;
                        ctx_menu.hovered_item = if idx < items.len() { Some(idx) } else { None };
                    } else {
                        ctx_menu.hovered_item = None;
                    }
                    ctx.request_paint();
                }

                // Toolbar tooltip
                if (MENU_BAR_H..MENU_BAR_H + TOOLBAR_H).contains(&y)
                    && self.state.open_dropdown.is_none()
                    && self.state.active_modal.is_none()
                {
                    self.state.hovered_tooltip = self.compute_toolbar_tooltip(x, y);
                    self.state.hovered_tooltip_x = x;
                } else {
                    self.state.hovered_tooltip = None;
                }
            }
            PointerEvent::Up(_) => {
                self.drag_start = None;
                if self.state.ruler_drag.is_some() {
                    self.state.ruler_drag = None;
                    ctx.request_paint();
                }
                if self.state.image_resize_drag.is_some() {
                    self.state.image_resize_drag = None;
                    ctx.request_paint();
                }
            }
            PointerEvent::Scroll(e) => {
                if self.state.ctrl_pressed {
                    if e.delta.y > 0.0 {
                        self.state.zoom = (self.state.zoom + 10.0).min(200.0);
                    } else if e.delta.y < 0.0 {
                        self.state.zoom = (self.state.zoom - 10.0).max(50.0);
                    }
                } else {
                    self.state.scroll_y = (self.state.scroll_y + e.delta.y).max(0.0);
                }
                ctx.request_paint();
            }
            _ => {}
        }
    }
}
