use super::*;

impl DocsApp {
    /// Pointer event handler extracted from Widget::on_pointer_event.
    pub(in crate::ui) fn handle_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        // Process any pending dialog results before handling the event
        self.process_dialog_results(ctx);

        match event {
            PointerEvent::Down(e) => {
                let (win_w, win_h) = self.state.last_window_size;
                // Clear stale overlays when a modal is active
                if self.state.any_modal_open() {
                    self.state.context_menu = None;
                    self.state.open_dropdown = None;
                    self.state.hovered_dropdown_item = None;
                }
                // Handle context menu clicks first
                if let Some(cm) = &self.state.context_menu {
                    let item_h = 28.0;
                    let menu_w = 200.0;
                    let items = cm.items();
                    let pt = Point::new(e.pos.x, e.pos.y);

                    for (idx, _) in items.iter().enumerate() {
                        let item_rect = Rect::new(
                            cm.x,
                            cm.y + idx as f64 * item_h,
                            cm.x + menu_w,
                            cm.y + (idx as f64 + 1.0) * item_h,
                        );
                        if item_rect.contains(pt) {
                            let item = items[idx].to_string();
                            self.state.context_menu = None;
                            self.handle_context_menu_item(&item, ctx);
                            return;
                        }
                    }
                    // Click outside context menu: close it
                    self.state.context_menu = None;
                    ctx.request_paint();
                    return;
                }

                if e.pos.y < MENU_BAR_H {
                    // Caption buttons (top-right) take priority over menus.
                    let (win_w, _) = self.state.last_window_size;
                    if let Some(ctrl) = window_control_at(e.pos.x, e.pos.y, win_w, MENU_BAR_H) {
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
                    // Notification label (between Help and title pill). Only
                    // hot when the label is currently visible.
                    if crate::ui::menu_bar::notification_label_message(&self.state).is_some() {
                        let label_rect = crate::ui::menu_bar::notification_label_rect(
                            Rect::new(0.0, 0.0, win_w, MENU_BAR_H),
                        );
                        if label_rect.contains(e.pos) {
                            // Open pricing page in default browser via shell plugin.
                            // tauri-plugin-shell::open is deprecated but still
                            // functional; migration to tauri-plugin-opener is a
                            // follow-up.
                            if let Some(handle) = &self.app_handle {
                                #[allow(deprecated)]
                                use tauri_plugin_shell::ShellExt;
                                #[allow(deprecated)]
                                let _ = handle.shell().open(
                                    "https://tenchsoft.com/pricing",
                                    None,
                                );
                            }
                            return;
                        }
                    }
                    if let Some(name) = menu_at(e.pos.x) {
                        self.state.active_modal = Some(name.to_string());
                        self.state.hovered_menu_item = None;
                        ctx.request_paint();
                    } else if self.state.active_modal.is_some()
                        || self.state.hovered_menu_item.is_some()
                    {
                        // Click in menu bar but not on any label: close menu
                        self.state.active_modal = None;
                        self.state.hovered_menu_item = None;
                        ctx.request_paint();
                    } else {
                        // Empty menu bar space: begin a window drag-move.
                        ctx.submit_window_action(WindowAction::StartDrag);
                    }
                    return;
                }

                // License modal click handling (Close button). The Activate
                // button currently delegates to the keyboard flow; wiring the
                // click → activate_license call is a follow-up.
                if let Some(_lic) = &self.state.license_modal.clone() {
                    let size = Size::new(self.state.last_window_size.0, self.state.last_window_size.1);
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
                                    "docs",
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

                // Handle print preview modal clicks
                if self.state.print_preview.is_some() {
                    let layout = layout::modal::compute_print_preview(Size::new(win_w, win_h));
                    let pt = Point::new(e.pos.x, e.pos.y);

                    // Close button (X) at top right
                    if layout.close.contains(pt) {
                        self.state.print_preview = None;
                        ctx.request_paint();
                        return;
                    }

                    // Prev button
                    if layout.prev_btn.contains(pt) {
                        if let Some(pp) = &mut self.state.print_preview {
                            pp.page_index = pp.page_index.saturating_sub(1);
                        }
                        ctx.request_paint();
                        return;
                    }

                    // Next button
                    if layout.next_btn.contains(pt) {
                        let num_pages = self.state.layout_cache.num_pages().max(1);
                        if let Some(pp) = &mut self.state.print_preview {
                            pp.page_index = (pp.page_index + 1).min(num_pages - 1);
                        }
                        ctx.request_paint();
                        return;
                    }

                    // Print button
                    if layout.print_btn.contains(pt) {
                        self.state
                            .show_toast("System print dialog not yet available in native mode");
                        ctx.request_paint();
                        return;
                    }

                    // Click outside modal: close
                    if !layout.modal.contains(pt) {
                        self.state.print_preview = None;
                        ctx.request_paint();
                    }
                    return;
                }

                // Handle word count modal clicks
                if self.state.word_count_modal {
                    let (win_w, win_h) = self.state.last_window_size;
                    let modal_w = 360.0;
                    let modal_h = 320.0;
                    let modal_x = win_w / 2.0 - modal_w / 2.0;
                    let modal_y = win_h / 2.0 - modal_h / 2.0;
                    let modal_rect =
                        Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);
                    let pt = Point::new(e.pos.x, e.pos.y);

                    // Close button
                    let close_btn = Rect::new(
                        modal_x + modal_w / 2.0 - 40.0,
                        modal_y + modal_h - 44.0,
                        modal_x + modal_w / 2.0 + 40.0,
                        modal_y + modal_h - 16.0,
                    );
                    if close_btn.contains(pt) || !modal_rect.contains(pt) {
                        self.state.word_count_modal = false;
                        ctx.request_paint();
                    }
                    return;
                }

                // Handle goto modal clicks
                if self.state.goto_modal.is_some() {
                    let layout = layout::modal::compute_goto(Size::new(win_w, win_h));
                    let modal_rect = layout.modal;
                    let pt = Point::new(e.pos.x, e.pos.y);

                    // Page button
                    if layout.page_mode.contains(pt) {
                        if let Some(goto) = &mut self.state.goto_modal {
                            goto.mode = state::GotoMode::Page;
                        }
                        ctx.request_paint();
                        return;
                    }

                    // Line button
                    if layout.line_mode.contains(pt) {
                        if let Some(goto) = &mut self.state.goto_modal {
                            goto.mode = state::GotoMode::Line;
                        }
                        ctx.request_paint();
                        return;
                    }

                    // Click outside modal: close
                    if !modal_rect.contains(pt) {
                        self.state.goto_modal = None;
                        ctx.request_paint();
                    }
                    return;
                }

                // Handle special character modal clicks
                if self.state.special_char_modal.is_some() {
                    let (win_w, win_h) = self.state.last_window_size;
                    let modal_w = 420.0;
                    let modal_h = 380.0;
                    let modal_x = win_w / 2.0 - modal_w / 2.0;
                    let modal_y = win_h / 2.0 - modal_h / 2.0;
                    let modal_rect =
                        Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);
                    let pt = Point::new(e.pos.x, e.pos.y);

                    // Close button (bottom-right of modal)
                    let close_btn = Rect::new(
                        modal_x + modal_w - 92.0,
                        modal_y + modal_h - 42.0,
                        modal_x + modal_w - 16.0,
                        modal_y + modal_h - 14.0,
                    );
                    if close_btn.contains(pt) {
                        self.state.special_char_modal = None;
                        ctx.request_paint();
                        return;
                    }

                    // Category tabs
                    let tab_y = modal_y + 36.0;
                    let mut tab_x = modal_x + 12.0;
                    for (idx, (cat_name, _chars)) in
                        state::SPECIAL_CHAR_CATEGORIES.iter().enumerate()
                    {
                        let tab_w = cat_name.len() as f64 * 7.0 + 16.0;
                        let tab_rect = Rect::new(tab_x, tab_y, tab_x + tab_w, tab_y + 22.0);
                        if tab_rect.contains(pt) {
                            if let Some(sc) = &mut self.state.special_char_modal {
                                sc.category_idx = idx;
                            }
                            ctx.request_paint();
                            return;
                        }
                        tab_x += tab_w + 4.0;
                    }

                    // Character grid
                    if let Some(sc_state) = &self.state.special_char_modal {
                        if let Some((_cat_name, chars)) =
                            state::SPECIAL_CHAR_CATEGORIES.get(sc_state.category_idx)
                        {
                            let grid_x = modal_x + 16.0;
                            let grid_y = tab_y + 30.0;
                            let cell_size = 32.0;
                            let cols = ((modal_w - 32.0) / cell_size) as usize;

                            for (i, &ch) in chars.iter().enumerate() {
                                let col = i % cols;
                                let row = i / cols;
                                let cell_x = grid_x + col as f64 * cell_size;
                                let cell_y = grid_y + row as f64 * cell_size;
                                if cell_y + cell_size > modal_y + modal_h - 40.0 {
                                    break;
                                }
                                let cell = Rect::new(
                                    cell_x,
                                    cell_y,
                                    cell_x + cell_size,
                                    cell_y + cell_size,
                                );
                                if cell.contains(pt) {
                                    let ch_str = ch.to_string();
                                    let result = self.engine().insert_text(&ch_str);
                                    self.state.apply_edit_result(result);
                                    self.state.special_char_modal = None;
                                    self.reset_cursor_blink();
                                    ctx.request_paint();
                                    return;
                                }
                            }
                        }
                    }

                    // Click outside modal: close
                    if !modal_rect.contains(pt) {
                        self.state.special_char_modal = None;
                        ctx.request_paint();
                    }
                    return;
                }

                // Handle link modal clicks
                if self.state.link_modal.is_some() {
                    let (win_w, win_h) = self.state.last_window_size;
                    // Check if clicking inside the modal area (OK/Cancel buttons)
                    let modal_w = 380.0;
                    let modal_h = 160.0;
                    let modal_x = win_w / 2.0 - modal_w / 2.0;
                    let modal_y = win_h / 2.0 - modal_h / 2.0;
                    let ok_btn = Rect::new(
                        modal_x + modal_w - 160.0,
                        modal_y + modal_h - 40.0,
                        modal_x + modal_w - 90.0,
                        modal_y + modal_h - 12.0,
                    );
                    let cancel_btn = Rect::new(
                        modal_x + modal_w - 80.0,
                        modal_y + modal_h - 40.0,
                        modal_x + modal_w - 12.0,
                        modal_y + modal_h - 12.0,
                    );

                    if ok_btn.contains(Point::new(e.pos.x, e.pos.y)) {
                        // OK: insert link with current URL
                        if let Some(link_state) = self.state.link_modal.take() {
                            if !link_state.url.is_empty() {
                                let result = self.engine().insert_link(&link_state.url);
                                self.state.apply_edit_result(result);
                            }
                        }
                        self.state.active_modal = None;
                        ctx.request_paint();
                        return;
                    }
                    if cancel_btn.contains(Point::new(e.pos.x, e.pos.y)) {
                        self.state.link_modal = None;
                        self.state.active_modal = None;
                        ctx.request_paint();
                        return;
                    }
                    // Click inside modal but not on buttons: ignore
                    let modal_rect =
                        Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);
                    if modal_rect.contains(Point::new(e.pos.x, e.pos.y)) {
                        return;
                    }
                    // Click outside modal: close
                    self.state.link_modal = None;
                    self.state.active_modal = None;
                    ctx.request_paint();
                    return;
                }

                // Handle page setup dialog clicks
                if self.state.page_setup_dialog.is_some() {
                    let (win_w, win_h) = self.state.last_window_size;
                    let modal_w = 420.0;
                    let modal_h = 380.0;
                    let modal_x = win_w / 2.0 - modal_w / 2.0;
                    let modal_y = win_h / 2.0 - modal_h / 2.0;
                    let modal_rect =
                        Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);

                    // Check OK button
                    let ok_btn = Rect::new(
                        modal_x + modal_w - 160.0,
                        modal_y + modal_h - 44.0,
                        modal_x + modal_w - 90.0,
                        modal_y + modal_h - 16.0,
                    );
                    if ok_btn.contains(Point::new(e.pos.x, e.pos.y)) {
                        if let Some(dialog_state) = self.state.page_setup_dialog.take() {
                            let result = self.engine().set_page_setup(dialog_state.to_page_setup());
                            self.state.apply_edit_result(result);
                        }
                        ctx.request_paint();
                        return;
                    }

                    // Check Cancel button
                    let cancel_btn = Rect::new(
                        modal_x + modal_w - 80.0,
                        modal_y + modal_h - 44.0,
                        modal_x + modal_w - 12.0,
                        modal_y + modal_h - 16.0,
                    );
                    if cancel_btn.contains(Point::new(e.pos.x, e.pos.y)) {
                        self.state.page_setup_dialog = None;
                        ctx.request_paint();
                        return;
                    }

                    // Check orientation toggle buttons (must match paint coordinates)
                    let right_x = modal_x + modal_w / 2.0 + 20.0;
                    let portrait_btn =
                        Rect::new(right_x, modal_y + 80.0, right_x + 80.0, modal_y + 104.0);
                    let landscape_btn = Rect::new(
                        right_x + 90.0,
                        modal_y + 80.0,
                        right_x + 190.0,
                        modal_y + 104.0,
                    );
                    if let Some(dialog) = &mut self.state.page_setup_dialog {
                        if portrait_btn.contains(Point::new(e.pos.x, e.pos.y)) {
                            dialog.orientation = Orientation::Portrait;
                            ctx.request_paint();
                            return;
                        }
                        if landscape_btn.contains(Point::new(e.pos.x, e.pos.y)) {
                            dialog.orientation = Orientation::Landscape;
                            ctx.request_paint();
                            return;
                        }

                        // Check paper size items (left column)
                        let sizes = state::PAPER_SIZES;
                        let mut size_y = modal_y + 90.0;
                        for &size in sizes {
                            let item_rect = Rect::new(
                                modal_x + 20.0,
                                size_y - 2.0,
                                modal_x + modal_w / 2.0 - 10.0,
                                size_y + 18.0,
                            );
                            if item_rect.contains(Point::new(e.pos.x, e.pos.y)) {
                                dialog.paper_size = size;
                                ctx.request_paint();
                                return;
                            }
                            size_y += 24.0;
                        }

                        // Check margin field clicks
                        let mut margin_y = modal_y + 156.0;
                        for field_idx in 0..4 {
                            let field_rect = Rect::new(
                                right_x + 60.0,
                                margin_y,
                                right_x + 130.0,
                                margin_y + 22.0,
                            );
                            if field_rect.contains(Point::new(e.pos.x, e.pos.y)) {
                                let current_value = match field_idx {
                                    0 => dialog.margin_top,
                                    1 => dialog.margin_bottom,
                                    2 => dialog.margin_left,
                                    3 => dialog.margin_right,
                                    _ => 0.0,
                                };
                                dialog.editing_margin_field = Some(field_idx);
                                dialog.margin_edit_buffer = format!("{:.1}", current_value);
                                ctx.request_paint();
                                return;
                            }
                            margin_y += 28.0;
                        }

                        // Click inside modal but not on a field: deselect margin field
                        dialog.editing_margin_field = None;
                    }

                    // Click outside modal: close without saving
                    if !modal_rect.contains(Point::new(e.pos.x, e.pos.y)) {
                        self.state.page_setup_dialog = None;
                        ctx.request_paint();
                    }
                    return;
                }

                // Handle find/replace modal clicks
                if self.state.find_replace.is_some() {
                    let fr_state = self.state.find_replace.as_ref().unwrap();
                    let layout = layout::modal::compute_find_replace(
                        Size::new(win_w, win_h),
                        fr_state.show_replace,
                    );
                    let modal_rect = layout.modal;

                    // Close button (X) at top right
                    let close_rect = layout.close;
                    if close_rect.contains(Point::new(e.pos.x, e.pos.y)) {
                        self.engine().clear_search();
                        self.state.find_replace = None;
                        self.state.active_modal = None;
                        ctx.request_paint();
                        return;
                    }

                    // Button row
                    let _btn_y = layout.btn_row_y;
                    let _btn_h = layout.btn_h;

                    let find_next_rect = layout.find_next;
                    let find_prev_rect = layout.find_prev;
                    let replace_rect = layout.replace;
                    let replace_all_rect = layout.replace_all;

                    let aa_rect = layout.case_sensitive;
                    let regex_rect = layout.regex;

                    let pt = Point::new(e.pos.x, e.pos.y);

                    if find_next_rect.contains(pt) {
                        let result = self.engine().find_next();
                        self.state.apply_edit_result(result);
                        self.update_find_match_index();
                        self.auto_scroll_to_match();
                        ctx.request_paint();
                        return;
                    }
                    if find_prev_rect.contains(pt) {
                        let result = self.engine().find_prev();
                        self.state.apply_edit_result(result);
                        self.update_find_match_index();
                        self.auto_scroll_to_match();
                        ctx.request_paint();
                        return;
                    }
                    if let Some(rect) = replace_rect {
                        if rect.contains(pt) {
                            if let Some(fr) = &self.state.find_replace {
                                let replacement = fr.replacement.clone();
                                let result = self.engine().replace_next(&replacement);
                                self.state.apply_edit_result(result);
                                self.refresh_find_matches();
                                self.auto_scroll_to_match();
                            }
                            ctx.request_paint();
                            return;
                        }
                    }
                    if let Some(rect) = replace_all_rect {
                        if rect.contains(pt) {
                            if let Some(fr) = &self.state.find_replace {
                                let query = fr.query.clone();
                                let replacement = fr.replacement.clone();
                                let case_sensitive = fr.case_sensitive;
                                let use_regex = fr.use_regex;
                                let _count = self.engine().replace_all(
                                    &query,
                                    &replacement,
                                    case_sensitive,
                                    use_regex,
                                );
                            }
                            self.refresh_find_matches();
                            ctx.request_paint();
                            return;
                        }
                    }
                    if aa_rect.contains(pt) {
                        if let Some(fr) = &mut self.state.find_replace {
                            fr.case_sensitive = !fr.case_sensitive;
                            self.refresh_find_matches();
                        }
                        ctx.request_paint();
                        return;
                    }
                    if regex_rect.contains(pt) {
                        if let Some(fr) = &mut self.state.find_replace {
                            fr.use_regex = !fr.use_regex;
                            self.refresh_find_matches();
                        }
                        ctx.request_paint();
                        return;
                    }

                    // Click outside modal: close
                    if !modal_rect.contains(pt) {
                        self.engine().clear_search();
                        self.state.find_replace = None;
                        self.state.active_modal = None;
                        ctx.request_paint();
                    }
                    return;
                }

                if let Some(modal_name) = self.state.active_modal.clone() {
                    if is_info_modal(&modal_name) {
                        let (win_w, win_h) = self.state.last_window_size;
                        let size = Size::new(win_w, win_h);
                        let pt = Point::new(e.pos.x, e.pos.y);
                        if info_modal_close_rect(size).contains(pt)
                            || !info_modal_rect(size).contains(pt)
                        {
                            self.state.active_modal = None;
                            ctx.request_paint();
                        }
                        return;
                    }

                    // Check if a menu item was clicked
                    let modal_w = 220.0;
                    let item_h = 26.0;
                    let top_pad = 8.0;
                    let modal_x = 12.0; // matches menu bar left padding
                    let modal_y = MENU_BAR_H;
                    let rel_x = e.pos.x - modal_x;
                    let rel_y = e.pos.y - modal_y - top_pad;

                    if rel_x >= 0.0 && rel_x < modal_w && rel_y >= 0.0 {
                        let items = menu_items_for(&modal_name);
                        let idx = (rel_y / item_h) as usize;
                        if idx < items.len() {
                            let item = items[idx].to_string();
                            self.state.active_modal = None;
                            self.state.hovered_menu_item = None;
                            self.handle_menu_item(&item, ctx);
                            return;
                        }
                    }
                    // Click outside modal items closes it
                    self.state.active_modal = None;
                    self.state.hovered_menu_item = None;
                    ctx.request_paint();
                    return;
                }

                if e.pos.y >= MENU_BAR_H && e.pos.y <= MENU_BAR_H + TOOLBAR_H {
                    // Check if a dropdown is open and an item was clicked
                    if self.state.open_dropdown == Some(ToolbarDropdown::FontSize) {
                        if let Some(idx) = font_size_dropdown_hit(e.pos.x, e.pos.y) {
                            let size = FONT_SIZES[idx];
                            self.state.current_font_size = size;
                            let result = self.engine().set_font_size(size);
                            self.state.apply_edit_result(result);
                            self.state.open_dropdown = None;
                            ctx.request_paint();
                            return;
                        }
                        // Click outside dropdown items closes it
                        self.state.open_dropdown = None;
                        ctx.request_paint();
                        return;
                    }
                    if self.state.open_dropdown == Some(ToolbarDropdown::FontFamily) {
                        if let Some(idx) = font_family_dropdown_hit(e.pos.x, e.pos.y) {
                            let family = FONT_FAMILIES[idx].to_string();
                            self.state.current_font_family = family.clone();
                            let result = self.engine().set_font_family(family);
                            self.state.apply_edit_result(result);
                            self.state.open_dropdown = None;
                            ctx.request_paint();
                            return;
                        }
                        // Click outside dropdown items closes it
                        self.state.open_dropdown = None;
                        ctx.request_paint();
                        return;
                    }
                    if self.state.open_dropdown == Some(ToolbarDropdown::ParagraphStyle) {
                        if let Some(idx) = paragraph_style_dropdown_hit(e.pos.x, e.pos.y) {
                            let style = ParagraphStyle::all()[idx];
                            self.state.current_paragraph_style = style;
                            let block_type = match style {
                                ParagraphStyle::Paragraph => BlockType::Paragraph,
                                ParagraphStyle::Heading1 => BlockType::Heading(1),
                                ParagraphStyle::Heading2 => BlockType::Heading(2),
                                ParagraphStyle::Heading3 => BlockType::Heading(3),
                                ParagraphStyle::Heading4 => BlockType::Heading(4),
                                ParagraphStyle::Heading5 => BlockType::Heading(5),
                                ParagraphStyle::Heading6 => BlockType::Heading(6),
                                ParagraphStyle::BlockQuote => BlockType::BlockQuote,
                                ParagraphStyle::CodeBlock => BlockType::CodeBlock,
                            };
                            let result = self.engine().set_block_type(block_type);
                            self.state.apply_edit_result(result);
                            self.state.open_dropdown = None;
                            ctx.request_paint();
                            return;
                        }
                        self.state.open_dropdown = None;
                        ctx.request_paint();
                        return;
                    }

                    // Handle table grid picker clicks
                    if self.state.open_dropdown == Some(ToolbarDropdown::TableGrid) {
                        if let Some((rows, cols)) = table_grid_hit(e.pos.x, e.pos.y) {
                            let result = self.engine().insert_table(rows, cols);
                            self.state.apply_edit_result(result);
                            self.state.open_dropdown = None;
                            ctx.request_paint();
                            return;
                        }
                        self.state.open_dropdown = None;
                        ctx.request_paint();
                        return;
                    }

                    // Handle color picker clicks
                    if self.state.open_dropdown == Some(ToolbarDropdown::ColorPicker) {
                        if let Some(color) = color_picker_hit(e.pos.x, e.pos.y) {
                            let result = self.engine().set_text_color(color.clone());
                            self.state.selected_text_color = Some(color);
                            self.state.apply_edit_result(result);
                            self.state.open_dropdown = None;
                            ctx.request_paint();
                            return;
                        }
                        self.state.open_dropdown = None;
                        ctx.request_paint();
                        return;
                    }

                    // Handle mark/background color picker clicks
                    if self.state.open_dropdown == Some(ToolbarDropdown::MarkPicker) {
                        if let Some(color) = color_picker_hit(e.pos.x, e.pos.y) {
                            let result = self.engine().set_background_color(color.clone());
                            self.state.selected_bg_color = Some(color);
                            self.state.apply_edit_result(result);
                            self.state.open_dropdown = None;
                            ctx.request_paint();
                            return;
                        }
                        self.state.open_dropdown = None;
                        ctx.request_paint();
                        return;
                    }

                    if let Some(action) = toolbar_action_at(e.pos.x) {
                        if self.handle_toolbar_action(action) {
                            ctx.request_paint();
                        }
                    }
                    return;
                }

                // Check ruler area for margin/indent marker drag
                let main_y = MENU_BAR_H + TOOLBAR_H;
                let sidebar_open = self.state.show_style_panel || self.state.show_comments;
                let sidebar_w = if sidebar_open {
                    state::STYLE_PANEL_W
                } else {
                    0.0
                };
                let content_w = self.state.last_window_size.0 - sidebar_w;
                let ruler_rect = Rect::new(
                    0.0,
                    main_y + TITLE_ROW_H,
                    content_w,
                    main_y + TITLE_ROW_H + RULER_H,
                );

                // Check tab bar clicks (only if multiple tabs)
                if self.state.open_tabs.len() > 1 {
                    let tab_bar_y = main_y + TITLE_ROW_H + RULER_H;
                    let tab_bar_h = 32.0;
                    if e.pos.y >= tab_bar_y && e.pos.y <= tab_bar_y + tab_bar_h {
                        let tab_w: f64 = 160.0;
                        let tab_x_start = 8.0;
                        let close_btn_w = 20.0;
                        for idx in 0..self.state.open_tabs.len() {
                            let tab_x = tab_x_start + idx as f64 * (tab_w + 2.0);
                            let tab_rect =
                                Rect::new(tab_x, tab_bar_y, tab_x + tab_w, tab_bar_y + tab_bar_h);
                            if tab_rect.contains(Point::new(e.pos.x, e.pos.y)) {
                                // Right-click on tab: show tab context menu
                                if e.button == tench_ui::core::events::PointerButton::Secondary {
                                    self.state.context_menu = Some(ContextMenuState {
                                        x: e.pos.x,
                                        y: e.pos.y,
                                        menu_type: ContextMenuType::Tab,
                                        hovered_item: None,
                                    });
                                    ctx.request_paint();
                                    return;
                                }
                                // Check if close button was clicked
                                let close_rect = Rect::new(
                                    tab_rect.x1 - close_btn_w - 4.0,
                                    tab_rect.y0 + (tab_bar_h - 14.0) / 2.0,
                                    tab_rect.x1 - 4.0,
                                    tab_rect.y0 + (tab_bar_h + 14.0) / 2.0,
                                );
                                if close_rect.contains(Point::new(e.pos.x, e.pos.y)) {
                                    self.close_tab(idx);
                                } else if idx != self.state.active_tab_idx {
                                    self.switch_to_tab(idx);
                                }
                                ctx.request_paint();
                                return;
                            }
                        }
                        // Click in tab bar but not on a tab: ignore
                        return;
                    }
                }
                if e.pos.y >= ruler_rect.y0 && e.pos.y <= ruler_rect.y1 {
                    if let Some((target, _)) = ruler_hit_test(e.pos.x, ruler_rect, &self.state) {
                        self.state.ruler_drag = Some(target);
                        ctx.request_paint();
                        return;
                    }
                }

                // Sidebar hit-testing (when sidebar is open)
                let sidebar_open = self.state.show_style_panel || self.state.show_comments;
                if sidebar_open {
                    let win_h = self.state.last_window_size.1;
                    let status_y = win_h - STATUS_BAR_H;
                    // compute_sidebar expects content_w = window width minus sidebar width
                    let sidebar_layout =
                        super::layout::sidebar::compute_sidebar(content_w, main_y, status_y);
                    let pt = Point::new(e.pos.x, e.pos.y);

                    // Comments collapse header click (when show_comments is true)
                    // Check this BEFORE sidebar tabs because the comments header
                    // overlaps the tab area when the comments panel is open.
                    if self.state.show_comments {
                        let sx = sidebar_layout.area.x0;
                        let sw = sidebar_layout.area.width();
                        let header_rect =
                            Rect::new(sx + 12.0, main_y + 12.0, sx + sw - 12.0, main_y + 42.0);
                        if header_rect.contains(pt) {
                            self.state.comments_collapsed = !self.state.comments_collapsed;
                            ctx.request_paint();
                            return;
                        }
                    }

                    // Check sidebar tab clicks (top 36px of sidebar)
                    // Only when NOT showing comments (comments panel has its own header)
                    if !self.state.show_comments {
                        for (idx, tab_rect) in sidebar_layout.tabs.iter().enumerate() {
                            if tab_rect.contains(pt) {
                                self.state.sidebar_tab = match idx {
                                    0 => SidebarTab::Style,
                                    1 => SidebarTab::Navigate,
                                    _ => SidebarTab::Ai,
                                };
                                ctx.request_paint();
                                return;
                            }
                        }
                    }
                }

                // Click in document area: set cursor position or start image resize
                if e.button == tench_ui::core::events::PointerButton::Primary {
                    let workspace_y = main_y + TITLE_ROW_H + RULER_H;
                    if e.pos.y >= workspace_y {
                        // Close any open dropdown when clicking in the document area
                        if self.state.open_dropdown.is_some() {
                            self.state.open_dropdown = None;
                        }
                        // Check for image resize handle first
                        if let Some(mut drag) =
                            self.hit_test_image_resize_handle(e.pos.x, e.pos.y, workspace_y)
                        {
                            drag.current_width = drag.start_width;
                            drag.current_height = drag.start_height;
                            self.state.targeted_image_block = Some(drag.block_idx);
                            self.state.image_resize_drag = Some(drag);
                            ctx.request_paint();
                            return;
                        }
                        if let Some(cursor) = self.click_to_cursor(e.pos.x, e.pos.y, workspace_y) {
                            self.drag_start = Some(cursor.clone());
                            let result = self.engine().select(cursor.clone(), cursor);
                            self.state.apply_edit_result(result);
                            self.reset_cursor_blink();
                            ctx.request_paint();
                        }
                    }
                }

                // Right-click: show context menu
                if e.button == tench_ui::core::events::PointerButton::Secondary {
                    let workspace_y = main_y + TITLE_ROW_H + RULER_H;
                    if e.pos.y >= workspace_y {
                        // Determine context type based on what's under the cursor
                        let menu_type = self.detect_context_type(e.pos.x, e.pos.y, workspace_y);
                        self.state.context_menu = Some(ContextMenuState {
                            x: e.pos.x,
                            y: e.pos.y,
                            menu_type,
                            hovered_item: None,
                        });
                        ctx.request_paint();
                    }
                }
            }
            PointerEvent::Move(e) => {
                // Track caption button hover for visual feedback.
                let (win_w, _) = self.state.last_window_size;
                let new_hover = window_control_at(e.pos.x, e.pos.y, win_w, MENU_BAR_H);
                if new_hover != self.state.window_control_hovered {
                    self.state.window_control_hovered = new_hover;
                    ctx.request_paint();
                }
                // Handle image resize drag
                if self.state.image_resize_drag.is_some()
                    && e.buttons
                        .has(tench_ui::core::events::PointerButton::Primary)
                {
                    if let Some(drag) = &mut self.state.image_resize_drag {
                        let dx = e.pos.x - drag.start_x;
                        let dy = e.pos.y - drag.start_y;
                        // Compute new dimensions based on which handle is dragged
                        let (new_w, new_h) = match drag.handle {
                            // Bottom-right: expand both
                            3 => (drag.start_width + dx, drag.start_height + dy),
                            // Bottom-left: expand height, contract width
                            2 => (drag.start_width - dx, drag.start_height + dy),
                            // Top-right: contract height, expand width
                            1 => (drag.start_width + dx, drag.start_height - dy),
                            // Top-left: contract both
                            _ => (drag.start_width - dx, drag.start_height - dy),
                        };
                        // Enforce minimum size
                        let new_w = new_w.max(40.0);
                        let new_h = new_h.max(30.0);
                        drag.current_width = new_w;
                        drag.current_height = new_h;
                        let block_idx = drag.block_idx;
                        // Update the image block dimensions in the document
                        let result =
                            self.engine()
                                .set_image_size(block_idx, new_w as f32, new_h as f32);
                        self.state.apply_edit_result(result);
                        ctx.request_paint();
                    }
                    // Don't process other move handlers during image resize
                    return;
                }

                if let Some(ref modal) = self.state.active_modal {
                    let modal_x = 12.0;
                    let modal_w = 220.0;
                    let item_h = 26.0;
                    let top_pad = 8.0;
                    let items = menu_items_for(modal);
                    let modal_h = top_pad + items.len() as f64 * item_h + top_pad;
                    let modal_top = MENU_BAR_H;
                    let modal_bottom = modal_top + modal_h;

                    // Only track hover when the pointer is inside the real menu rectangle
                    if e.pos.x >= modal_x
                        && e.pos.x < modal_x + modal_w
                        && e.pos.y >= modal_top
                        && e.pos.y < modal_bottom
                    {
                        let rel_y = e.pos.y - MENU_BAR_H - top_pad;
                        let inside_rows = rel_y >= 0.0 && rel_y < items.len() as f64 * item_h;
                        let new_hover = if inside_rows {
                            let idx = (rel_y / item_h) as usize;
                            if idx < items.len() {
                                Some(idx)
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        if new_hover != self.state.hovered_menu_item {
                            self.state.hovered_menu_item = new_hover;
                            ctx.request_paint();
                        }
                    } else if self.state.hovered_menu_item.is_some() {
                        self.state.hovered_menu_item = None;
                        ctx.request_paint();
                    }
                } else if self.state.hovered_menu_item.is_some() {
                    self.state.hovered_menu_item = None;
                    ctx.request_paint();
                }

                // Update context menu hover
                if self.state.context_menu.is_some() {
                    let new_hover = self.compute_context_menu_hover(e.pos.x, e.pos.y);
                    if let Some(cm) = &mut self.state.context_menu {
                        if cm.hovered_item != new_hover {
                            cm.hovered_item = new_hover;
                            ctx.request_paint();
                        }
                    }
                }

                // Update table grid hover state
                if self.state.open_dropdown == Some(ToolbarDropdown::TableGrid) {
                    let (next_row, next_col) = table_grid_hit(e.pos.x, e.pos.y).unwrap_or((0, 0));
                    if self.state.table_grid.hover_row != next_row
                        || self.state.table_grid.hover_col != next_col
                    {
                        self.state.table_grid.hover_row = next_row;
                        self.state.table_grid.hover_col = next_col;
                        ctx.request_paint();
                    }
                }

                // Update dropdown item hover highlight
                if self.state.open_dropdown.is_some() {
                    let new_hover = self.compute_dropdown_hover(e.pos.x, e.pos.y);
                    if new_hover != self.state.hovered_dropdown_item {
                        self.state.hovered_dropdown_item = new_hover;
                        ctx.request_paint();
                    }
                }

                // Update toolbar tooltip (suppressed when modal/dropdown/context-menu is active)
                let toolbar_y0 = MENU_BAR_H;
                let toolbar_y1 = MENU_BAR_H + TOOLBAR_H;
                if e.pos.y >= toolbar_y0
                    && e.pos.y <= toolbar_y1
                    && !self.state.any_modal_open()
                    && self.state.open_dropdown.is_none()
                    && self.state.context_menu.is_none()
                {
                    let tooltip_info = toolbar_tooltip_at(e.pos.x);
                    let new_tooltip = tooltip_info.map(|(t, _)| t.to_string());
                    if new_tooltip != self.state.hovered_tooltip {
                        self.state.hovered_tooltip = new_tooltip;
                        self.state.hovered_tooltip_x =
                            tooltip_info.map(|(_, x)| x).unwrap_or(e.pos.x);
                        ctx.request_paint();
                    }
                } else if self.state.hovered_tooltip.is_some() {
                    self.state.hovered_tooltip = None;
                    ctx.request_paint();
                }

                // Handle ruler drag
                if self.state.ruler_drag.is_some()
                    && e.buttons
                        .has(tench_ui::core::events::PointerButton::Primary)
                {
                    let main_y = MENU_BAR_H + TOOLBAR_H;
                    let sidebar_open = self.state.show_style_panel || self.state.show_comments;
                    let sidebar_w = if sidebar_open {
                        state::STYLE_PANEL_W
                    } else {
                        0.0
                    };
                    let content_w = self.state.last_window_size.0 - sidebar_w;
                    let ruler_rect = Rect::new(
                        0.0,
                        main_y + TITLE_ROW_H,
                        content_w,
                        main_y + TITLE_ROW_H + RULER_H,
                    );
                    if let Some(target) = self.state.ruler_drag {
                        match target {
                            RulerDragTarget::LeftMargin => {
                                let mm = ruler_drag_to_margin(e.pos.x, ruler_rect, &self.state);
                                let existing = self.state.current_document().page_setup.margins;
                                let result =
                                    self.engine().set_margins(tench_document_core::Margins {
                                        left: mm,
                                        ..existing
                                    });
                                self.state.apply_edit_result(result);
                            }
                            RulerDragTarget::RightMargin => {
                                let mm = ruler_drag_to_margin(e.pos.x, ruler_rect, &self.state);
                                let existing = self.state.current_document().page_setup.margins;
                                let result =
                                    self.engine().set_margins(tench_document_core::Margins {
                                        right: mm,
                                        ..existing
                                    });
                                self.state.apply_edit_result(result);
                            }
                            RulerDragTarget::IndentLeft => {
                                let indent = ruler_drag_to_indent(e.pos.x, ruler_rect, &self.state);
                                let result = self.engine().set_indent_left(indent);
                                self.state.apply_edit_result(result);
                            }
                            RulerDragTarget::IndentRight => {
                                let indent = ruler_drag_to_indent(e.pos.x, ruler_rect, &self.state);
                                let result = self.engine().set_indent_right(indent);
                                self.state.apply_edit_result(result);
                            }
                            RulerDragTarget::IndentFirstLine => {
                                let indent = ruler_drag_to_indent(e.pos.x, ruler_rect, &self.state);
                                let result = self.engine().set_indent_first_line(indent);
                                self.state.apply_edit_result(result);
                            }
                        }
                        ctx.request_paint();
                    }
                }

                if e.buttons
                    .has(tench_ui::core::events::PointerButton::Primary)
                {
                    if let Some(start) = self.drag_start.clone() {
                        let main_y = MENU_BAR_H + TOOLBAR_H;
                        let workspace_y = main_y + TITLE_ROW_H + RULER_H;
                        if e.pos.y >= workspace_y {
                            if let Some(end_cursor) =
                                self.click_to_cursor(e.pos.x, e.pos.y, workspace_y)
                            {
                                let result = self.engine().select(start, end_cursor);
                                self.state.apply_edit_result(result);
                                ctx.request_paint();
                            }
                        }
                    }
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
                if e.modifiers.control {
                    // Ctrl+wheel: zoom
                    if e.delta.y > 0.0 {
                        self.state.set_zoom((self.state.zoom + 10.0).min(200.0));
                    } else if e.delta.y < 0.0 {
                        self.state.set_zoom((self.state.zoom - 10.0).max(50.0));
                    }
                } else {
                    // Plain wheel: scroll
                    self.state.scroll_y = (self.state.scroll_y + e.delta.y).max(0.0);
                    // Update current_page based on scroll position
                    let scale = self.state.zoom / 100.0;
                    let page_stride = state::PAGE_H * scale + 20.0 * scale;
                    if page_stride > 0.0 {
                        self.state.current_page =
                            ((self.state.scroll_y / page_stride).floor() as usize + 1).max(1);
                    }
                }
                ctx.request_paint();
            }
            _ => {}
        }
    }
}
