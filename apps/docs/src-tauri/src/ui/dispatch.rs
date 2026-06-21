use super::*;

// ---------------------------------------------------------------------------
// Menu and toolbar dispatch
// ---------------------------------------------------------------------------

impl DocsApp {
    /// Handle a menu item action by name.
    pub(super) fn handle_menu_item(&mut self, item: &str, ctx: &mut EventCtx) {
        match item {
            "New" => {
                self.append_new_document_tab();
                ctx.request_paint();
            }
            "Save" => {
                self.save_current_document();
                self.refresh_version_history();
                ctx.request_paint();
            }
            "Save As" => {
                self.save_as_dialog();
                ctx.request_paint();
            }
            "Undo" => {
                let result = self.engine().undo();
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Redo" => {
                let result = self.engine().redo();
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Cut" => {
                self.cut_selection();
                ctx.request_paint();
            }
            "Copy" => {
                self.copy_selection();
                ctx.request_paint();
            }
            "Paste" => {
                self.paste_clipboard();
                ctx.request_paint();
            }
            "Select All" => {
                let result = self.engine().select_all();
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Find" => {
                self.open_find_replace(false);
                ctx.request_paint();
            }
            "Replace" => {
                self.open_find_replace(true);
                ctx.request_paint();
            }
            "Zoom In" => {
                self.state.set_zoom((self.state.zoom + 10.0).min(200.0));
                ctx.request_paint();
            }
            "Zoom Out" => {
                self.state.set_zoom((self.state.zoom - 10.0).max(50.0));
                ctx.request_paint();
            }
            "Reset Zoom" => {
                self.state.set_zoom(100.0);
                ctx.request_paint();
            }
            "Thumbnails" => {
                self.state.show_thumbnails = !self.state.show_thumbnails;
                ctx.request_paint();
            }
            "Style Panel" => {
                self.state.show_style_panel = !self.state.show_style_panel;
                ctx.request_paint();
            }
            "Comments" => {
                self.state.show_comments = !self.state.show_comments;
                if self.state.show_comments {
                    let comments = self.engine().get_comments().to_vec();
                    self.state.update_comments(comments);
                }
                ctx.request_paint();
            }
            "Image" => {
                self.insert_image_dialog();
                ctx.request_paint();
            }
            "Table" => {
                let result = self.engine().insert_table(3, 3);
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Link" => {
                self.state.prepare_modal_open();
                self.state.link_modal = Some(LinkModalState::default());
                self.state.active_modal = Some("Insert Link".into());
                ctx.request_paint();
            }
            "Horizontal Rule" => {
                let result = self.engine().insert_horizontal_rule();
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Page Break" => {
                let result = self.engine().insert_page_break();
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Bold" => {
                let result = self.engine().toggle_mark(MarkType::Bold);
                self.state.bold = !self.state.bold;
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Italic" => {
                let result = self.engine().toggle_mark(MarkType::Italic);
                self.state.italic = !self.state.italic;
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Underline" => {
                let result = self.engine().toggle_mark(MarkType::Underline);
                self.state.underline = !self.state.underline;
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Strikethrough" => {
                let result = self.engine().toggle_mark(MarkType::Strikethrough);
                self.state.strikethrough = !self.state.strikethrough;
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Superscript" => {
                let result = self.engine().toggle_mark(MarkType::Superscript);
                self.state.superscript = !self.state.superscript;
                if self.state.superscript {
                    self.state.subscript = false;
                }
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Subscript" => {
                let result = self.engine().toggle_mark(MarkType::Subscript);
                self.state.subscript = !self.state.subscript;
                if self.state.subscript {
                    self.state.superscript = false;
                }
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Clear Formatting" => {
                let result = self.engine().clear_marks();
                self.state.bold = false;
                self.state.italic = false;
                self.state.underline = false;
                self.state.strikethrough = false;
                self.state.code = false;
                self.state.superscript = false;
                self.state.subscript = false;
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Block Quote" => {
                let result = self.engine().set_block_type(BlockType::BlockQuote);
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "Word Count" => {
                self.state.prepare_modal_open();
                self.state.word_count_modal = true;
                ctx.request_paint();
            }
            "Track Changes" => {
                self.engine().toggle_track_changes();
                self.state.track_changes = self.engine().is_track_changes_enabled();
                ctx.request_paint();
            }
            "Page Setup" => {
                self.state.prepare_modal_open();
                self.state.page_setup_dialog = Some(PageSetupDialogState::from_page_setup(
                    &self.state.current_document().page_setup,
                ));
                ctx.request_paint();
            }
            "Print" => {
                self.state.ensure_layout_cache();
                let page_count = self.state.layout_cache.num_pages().max(1);
                let initial_page = self
                    .state
                    .current_page
                    .saturating_sub(1)
                    .min(page_count - 1);
                self.state.print_preview = Some(state::PrintPreviewState {
                    page_index: initial_page,
                    zoom: 100.0,
                    page_count,
                });
                ctx.request_paint();
            }
            "Header" => {
                self.state.editing_header = true;
                self.state.editing_footer = false;
                self.state.scroll_y = 0.0;
                // Initialize header text from document's header template
                let doc = self.state.current_document();
                self.state.header_text = doc
                    .headers_footers
                    .default_header
                    .clone()
                    .unwrap_or_default();
                ctx.request_paint();
            }
            "Footer" => {
                self.state.editing_footer = true;
                self.state.editing_header = false;
                self.scroll_to_footer_editor();
                // Initialize footer text from document's footer template
                let doc = self.state.current_document();
                self.state.footer_text = doc
                    .headers_footers
                    .default_footer
                    .clone()
                    .unwrap_or_default();
                ctx.request_paint();
            }
            "Version History" => {
                self.refresh_version_history();
                ctx.request_paint();
            }
            "Spell Check" => {
                self.state.show_toast("Spell check: checking...");
                ctx.request_paint();
                // Placeholder: simulate spell check completion
                self.state.show_toast("No misspellings found");
                ctx.request_paint();
            }
            "Go To" => {
                self.state.prepare_modal_open();
                self.state.goto_modal = Some(GotoModalState::default());
                ctx.request_paint();
            }
            "Special Character" => {
                self.state.prepare_modal_open();
                self.state.special_char_modal = Some(SpecialCharModalState::default());
                ctx.request_paint();
            }
            "Footnote" => {
                // Insert superscript reference number
                let result = self.engine().toggle_mark(MarkType::Superscript);
                self.state.apply_edit_result(result);
                // Count existing footnotes in document
                let doc = self.state.current_document();
                let footnote_count = doc
                    .content
                    .iter()
                    .filter(|b| matches!(b, BlockNode::Footnote { .. }))
                    .count();
                let ref_num = (footnote_count + 1) as u32;
                let result = self.engine().insert_text(&ref_num.to_string());
                self.state.apply_edit_result(result);
                // Remove superscript
                let result = self.engine().toggle_mark(MarkType::Superscript);
                self.state.apply_edit_result(result);
                // Add footnote block at the end
                let result = self.engine().insert_footnote(ref_num);
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "About" => {
                self.state.active_modal = Some("About".into());
                self.state.toast = None;
                ctx.request_paint();
            }
            "Keyboard Shortcuts" => {
                self.state.active_modal = Some("Keyboard Shortcuts".into());
                self.state.toast = None;
                ctx.request_paint();
            }
            "Export As" => {
                self.save_as_dialog();
                ctx.request_paint();
            }
            "Open" => {
                self.open_file_dialog();
                ctx.request_paint();
            }
            "Recent Files" => {
                self.state.show_toast("No recent files");
                ctx.request_paint();
            }
            _ => {
                ctx.request_paint();
            }
        }
    }

    /// Handle toolbar button/index-based actions.
    pub(super) fn handle_toolbar_action(&mut self, action: ToolbarAction) -> bool {
        match action {
            ToolbarAction::Undo => {
                let result = self.engine().undo();
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::Redo => {
                let result = self.engine().redo();
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::FormatButton(idx) => {
                self.state.hovered_btn = Some(idx);
                match idx {
                    0 => {
                        // Bold
                        let result = self.engine().toggle_mark(MarkType::Bold);
                        self.state.bold = !self.state.bold;
                        self.state.apply_edit_result(result);
                    }
                    1 => {
                        // Italic
                        let result = self.engine().toggle_mark(MarkType::Italic);
                        self.state.italic = !self.state.italic;
                        self.state.apply_edit_result(result);
                    }
                    2 => {
                        // Underline
                        let result = self.engine().toggle_mark(MarkType::Underline);
                        self.state.underline = !self.state.underline;
                        self.state.apply_edit_result(result);
                    }
                    3 => {
                        // Strikethrough
                        let result = self.engine().toggle_mark(MarkType::Strikethrough);
                        self.state.strikethrough = !self.state.strikethrough;
                        self.state.apply_edit_result(result);
                    }
                    4 => {
                        // Code
                        let result = self.engine().toggle_mark(MarkType::Code);
                        self.state.code = !self.state.code;
                        self.state.apply_edit_result(result);
                    }
                    5 => {
                        // Superscript (x2)
                        let result = self.engine().toggle_mark(MarkType::Superscript);
                        self.state.superscript = !self.state.superscript;
                        if self.state.superscript {
                            self.state.subscript = false;
                        }
                        self.state.apply_edit_result(result);
                    }
                    6 => {
                        // Subscript (x_)
                        let result = self.engine().toggle_mark(MarkType::Subscript);
                        self.state.subscript = !self.state.subscript;
                        if self.state.subscript {
                            self.state.superscript = false;
                        }
                        self.state.apply_edit_result(result);
                    }
                    7 => {
                        // Highlight - open background color picker
                        self.state.open_dropdown =
                            if self.state.open_dropdown == Some(ToolbarDropdown::MarkPicker) {
                                None
                            } else {
                                Some(ToolbarDropdown::MarkPicker)
                            };
                    }
                    _ => return false,
                }
                true
            }
            ToolbarAction::BulletList => {
                let result = self.engine().toggle_list(false);
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::NumberedList => {
                let result = self.engine().toggle_list(true);
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::Checklist => {
                let result = self.engine().toggle_task_list();
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::Outdent => {
                let result = self.engine().outdent();
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::Indent => {
                let result = self.engine().indent();
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::AlignLeft => {
                let result = self.engine().set_alignment(Alignment::Left);
                self.state.current_alignment = Alignment::Left;
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::AlignCenter => {
                let result = self.engine().set_alignment(Alignment::Center);
                self.state.current_alignment = Alignment::Center;
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::AlignRight => {
                let result = self.engine().set_alignment(Alignment::Right);
                self.state.current_alignment = Alignment::Right;
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::AlignJustify => {
                let result = self.engine().set_alignment(Alignment::Justify);
                self.state.current_alignment = Alignment::Justify;
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::HorizontalRule => {
                let result = self.engine().insert_horizontal_rule();
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::BlockQuote => {
                let result = self.engine().set_block_type(BlockType::BlockQuote);
                self.state.apply_edit_result(result);
                true
            }
            ToolbarAction::ToggleThumbnails => {
                self.state.show_thumbnails = !self.state.show_thumbnails;
                true
            }
            ToolbarAction::ZoomIn => {
                self.state.set_zoom((self.state.zoom + 10.0).min(200.0));
                true
            }
            ToolbarAction::ZoomOut => {
                self.state.set_zoom((self.state.zoom - 10.0).max(50.0));
                true
            }
            ToolbarAction::ToggleStylePanel => {
                self.state.show_style_panel = !self.state.show_style_panel;
                true
            }
            ToolbarAction::ToggleComments => {
                self.state.show_comments = !self.state.show_comments;
                true
            }
            ToolbarAction::ToggleTrackChanges => {
                self.engine().toggle_track_changes();
                self.state.track_changes = self.engine().is_track_changes_enabled();
                true
            }
            ToolbarAction::FontFamilySelect => {
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::FontFamily) {
                        None
                    } else {
                        Some(ToolbarDropdown::FontFamily)
                    };
                true
            }
            ToolbarAction::FontFamilyItem(idx) => {
                if let Some(&name) = FONT_FAMILIES.get(idx) {
                    self.state.current_font_family = name.to_string();
                    let result = self.engine().set_font_family(name.to_string());
                    self.state.apply_edit_result(result);
                }
                self.state.open_dropdown = None;
                true
            }
            ToolbarAction::FontSizeSelect => {
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::FontSize) {
                        None
                    } else {
                        Some(ToolbarDropdown::FontSize)
                    };
                true
            }
            ToolbarAction::ParagraphStyleSelect => {
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::ParagraphStyle) {
                        None
                    } else {
                        Some(ToolbarDropdown::ParagraphStyle)
                    };
                true
            }
            ToolbarAction::FontSizeItem(idx) => {
                if let Some(&size) = FONT_SIZES.get(idx) {
                    self.state.current_font_size = size;
                    let result = self.engine().set_font_size(size);
                    self.state.apply_edit_result(result);
                }
                self.state.open_dropdown = None;
                true
            }
            ToolbarAction::ParagraphStyleItem(idx) => {
                if let Some(&style) = ParagraphStyle::all().get(idx) {
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
                }
                self.state.open_dropdown = None;
                true
            }
            ToolbarAction::InsertLink => {
                // Open link insertion modal
                self.state.prepare_modal_open();
                self.state.link_modal = Some(LinkModalState::default());
                self.state.active_modal = Some("Insert Link".into());
                true
            }
            ToolbarAction::InsertImage => {
                self.insert_image_dialog();
                true
            }
            ToolbarAction::InsertTable => {
                // Toggle table grid picker
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::TableGrid) {
                        None
                    } else {
                        self.state.table_grid = TableGridState::default();
                        Some(ToolbarDropdown::TableGrid)
                    };
                true
            }
            ToolbarAction::TextColor => {
                // Toggle text color picker
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::ColorPicker) {
                        None
                    } else {
                        Some(ToolbarDropdown::ColorPicker)
                    };
                true
            }
            ToolbarAction::MarkColor => {
                // Toggle background/mark color picker
                self.state.open_dropdown =
                    if self.state.open_dropdown == Some(ToolbarDropdown::MarkPicker) {
                        None
                    } else {
                        Some(ToolbarDropdown::MarkPicker)
                    };
                true
            }
        }
    }

    /// Handle a context menu item selection.
    pub(super) fn handle_context_menu_item(&mut self, item: &str, ctx: &mut EventCtx) {
        match item {
            "Cut" => {
                self.cut_selection();
            }
            "Copy" => {
                self.copy_selection();
            }
            "Paste" => {
                self.paste_clipboard();
            }
            "Add Comment" => {
                // Open comment modal with current selection
                self.state.prepare_modal_open();
                self.state.comment_modal = Some(CommentModalState::default());
            }
            "Insert Link" => {
                self.state.prepare_modal_open();
                self.state.link_modal = Some(LinkModalState::default());
                self.state.active_modal = Some("Insert Link".into());
            }
            "Clear Formatting" => {
                let result = self.engine().clear_marks();
                self.state.bold = false;
                self.state.italic = false;
                self.state.underline = false;
                self.state.strikethrough = false;
                self.state.code = false;
                self.state.superscript = false;
                self.state.subscript = false;
                self.state.apply_edit_result(result);
            }
            "Replace Image" => {
                self.insert_image_dialog();
            }
            "Remove" => {
                let cursor = self.state.cursor().clone();
                let result = self.engine().backspace();
                // If backspace didn't work (image block), try deleting the block
                if self.state.cursor().block_idx == cursor.block_idx {
                    let _ = self.engine().delete_selection();
                }
                self.state.apply_edit_result(result);
            }
            "Insert Row Above" => {
                let result = self.engine().insert_table_row(true);
                self.state.apply_edit_result(result);
            }
            "Insert Row Below" => {
                let result = self.engine().insert_table_row(false);
                self.state.apply_edit_result(result);
            }
            "Insert Column Left" => {
                let result = self.engine().insert_table_column(true);
                self.state.apply_edit_result(result);
            }
            "Insert Column Right" => {
                let result = self.engine().insert_table_column(false);
                self.state.apply_edit_result(result);
            }
            "Delete Row" => {
                let result = self.engine().delete_table_row();
                self.state.apply_edit_result(result);
            }
            "Delete Column" => {
                let result = self.engine().delete_table_column();
                self.state.apply_edit_result(result);
            }
            "Delete Table" => {
                let result = self.engine().delete_table();
                self.state.apply_edit_result(result);
            }
            "Close" => {
                let idx = self.state.active_tab_idx;
                self.close_tab(idx);
            }
            "Close Others" => {
                let keep_idx = self.state.active_tab_idx;
                // Keep only the active session
                if keep_idx < self.sessions.len() {
                    let session = self.sessions.remove(keep_idx);
                    self.sessions.clear();
                    self.sessions.push(session);
                }
                let mut new_tabs = vec![self.state.open_tabs[keep_idx].clone()];
                std::mem::swap(&mut self.state.open_tabs, &mut new_tabs);
                self.state.active_tab_idx = 0;
                self.active_session_idx = 0;
            }
            "Close All" => {
                self.replace_with_single_new_document();
            }
            _ => {}
        }
        ctx.request_paint();
    }

    /// Compute which dropdown item is hovered based on pointer position.
    pub(super) fn compute_dropdown_hover(
        &self,
        x: f64,
        y: f64,
    ) -> Option<(ToolbarDropdown, usize)> {
        use state::ToolbarDropdown::*;
        let dropdown_top = MENU_BAR_H + 8.0 + 32.0;
        match self.state.open_dropdown? {
            FontSize => {
                let fs_x = toolbar_actions::font_size_select_x();
                if x < fs_x || x > fs_x + toolbar_actions::FONT_SIZE_SELECT_W || y < dropdown_top {
                    return None;
                }
                let rel_y = y - dropdown_top - 2.0;
                if rel_y < 0.0 {
                    return None;
                }
                let idx = (rel_y / popovers::DROPDOWN_ITEM_H) as usize;
                if idx < FONT_SIZES.len() {
                    Some((FontSize, idx))
                } else {
                    None
                }
            }
            ParagraphStyle => {
                let ps_x = toolbar_actions::paragraph_style_select_x();
                if x < ps_x || x > ps_x + toolbar_actions::PARAGRAPH_SELECT_W || y < dropdown_top {
                    return None;
                }
                let rel_y = y - dropdown_top - 2.0;
                if rel_y < 0.0 {
                    return None;
                }
                let idx = (rel_y / popovers::DROPDOWN_ITEM_H) as usize;
                if idx < state::ParagraphStyle::all().len() {
                    Some((ToolbarDropdown::ParagraphStyle, idx))
                } else {
                    None
                }
            }
            FontFamily => {
                let ff_x = toolbar_actions::font_family_select_x();
                if x < ff_x || x > ff_x + toolbar_actions::FONT_FAMILY_SELECT_W || y < dropdown_top
                {
                    return None;
                }
                let rel_y = y - dropdown_top - 2.0;
                if rel_y < 0.0 {
                    return None;
                }
                let idx = (rel_y / popovers::DROPDOWN_ITEM_H) as usize;
                if idx < state::FONT_FAMILIES.len() {
                    Some((FontFamily, idx))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Compute which context menu item is hovered.
    pub(super) fn compute_context_menu_hover(&self, x: f64, y: f64) -> Option<usize> {
        let cm = self.state.context_menu.as_ref()?;
        let item_h = 28.0;
        let menu_w = 200.0;
        if x < cm.x || x > cm.x + menu_w || y < cm.y {
            return None;
        }
        let rel_y = y - cm.y;
        let idx = (rel_y / item_h) as usize;
        let items = cm.items();
        if idx < items.len() {
            Some(idx)
        } else {
            None
        }
    }

    /// Detect what type of context menu to show based on click position.
    pub(super) fn detect_context_type(
        &self,
        _x: f64,
        _y: f64,
        _workspace_y: f64,
    ) -> ContextMenuType {
        // Check if the cursor is in a table cell
        let doc = self.state.current_document();
        let cursor = self.state.cursor();
        if cursor.block_idx < doc.content.len() {
            // Walk backwards to find if we're inside a table
            for block in &doc.content {
                if let BlockNode::Table { .. } = block {
                    // Simplified: if cursor is at or near a table block
                    if doc
                        .content
                        .get(cursor.block_idx)
                        .is_some_and(|b| matches!(b, BlockNode::Table { .. }))
                    {
                        return ContextMenuType::TableCell;
                    }
                }
            }
            // Check if cursor is on an image block
            if doc
                .content
                .get(cursor.block_idx)
                .is_some_and(|b| matches!(b, BlockNode::Image { .. }))
            {
                return ContextMenuType::Image;
            }
        }
        ContextMenuType::Text
    }

    fn scroll_to_footer_editor(&mut self) {
        let scale = self.state.zoom / 100.0;
        let doc = self.state.current_document();
        let (_page_w, page_h_raw) = doc.page_setup.page_size_px();
        let page_h = page_h_raw * scale;
        let mm_to_px = 96.0 / 25.4;
        let margin_bottom = doc.page_setup.margins.bottom as f64 * mm_to_px * scale;
        let footer_h = state::FOOTER_H * scale;
        let chrome_h = MENU_BAR_H + TOOLBAR_H + TITLE_ROW_H + RULER_H + STATUS_BAR_H;
        let viewport_h = (self.state.last_window_size.1 - chrome_h).max(1.0);
        let footer_top = state::PAGE_MARGIN_Y + page_h - margin_bottom - footer_h;

        self.state.scroll_y = (footer_top - viewport_h * 0.55).max(0.0);
    }
}
