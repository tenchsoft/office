use super::*;

pub(super) fn push_docs_modal_nodes(
    state: &DocsState,
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    // Comment modal automation nodes
    if let Some(comment_state) = &state.comment_modal {
        let il = compute_info_modal(size);
        push_docs_node(
            nodes,
            next_id,
            "dialog",
            "Add Comment",
            "docs.comment_modal",
            il.modal,
        );
        // Text input field with value
        let field_x = il.modal.x0 + 16.0;
        let field_y = il.modal.y0 + 38.0;
        let field_w = il.modal.width() - 32.0;
        let field_h = 28.0;
        push_docs_node(
            nodes,
            next_id,
            "textbox",
            "Comment text",
            "docs.comment_modal.text",
            Rect::new(field_x, field_y, field_x + field_w, field_y + field_h),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(comment_state.text.clone());
        }
        // Submit button (Enter)
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Submit",
            "docs.comment_modal.submit",
            Rect::new(
                il.modal.x1 - 170.0,
                il.modal.y1 - 44.0,
                il.modal.x1 - 94.0,
                il.modal.y1 - 14.0,
            ),
        );
        // Cancel button (Escape)
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Cancel",
            "docs.comment_modal.cancel",
            Rect::new(
                il.modal.x1 - 86.0,
                il.modal.y1 - 44.0,
                il.modal.x1 - 16.0,
                il.modal.y1 - 14.0,
            ),
        );
    } else if state.link_modal.is_some() {
        let il = compute_info_modal(size);
        push_docs_node(
            nodes,
            next_id,
            "dialog",
            "Insert link",
            "docs.modal.link",
            il.modal,
        );
        push_docs_node(
            nodes,
            next_id,
            "textbox",
            "URL",
            "docs.modal.link.url",
            Rect::new(
                il.modal.x0 + 20.0,
                il.modal.y0 + 54.0,
                il.modal.x1 - 20.0,
                il.modal.y0 + 84.0,
            ),
        );
        // Expose URL value from link modal state
        if let Some(node) = nodes.last_mut() {
            if let Some(link_state) = &state.link_modal {
                node.value = Some(link_state.url.clone());
            }
        }
        push_docs_node(
            nodes,
            next_id,
            "button",
            "OK",
            "docs.modal.link.ok",
            Rect::new(
                il.modal.x1 - 170.0,
                il.modal.y1 - 44.0,
                il.modal.x1 - 94.0,
                il.modal.y1 - 14.0,
            ),
        );
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Cancel",
            "docs.modal.link.cancel",
            Rect::new(
                il.modal.x1 - 86.0,
                il.modal.y1 - 44.0,
                il.modal.x1 - 16.0,
                il.modal.y1 - 14.0,
            ),
        );
    } else if state.find_replace.is_some() {
        let fr_state = state.find_replace.as_ref().unwrap();
        let fr = compute_find_replace(size, fr_state.show_replace);

        push_docs_node(
            nodes,
            next_id,
            "dialog",
            "Find and replace",
            "docs.modal.find_replace",
            fr.modal,
        );
        // Query field with value
        let mut query_node = UiAutomationNode {
            id: {
                *next_id = next_id.saturating_add(1);
                *next_id
            },
            debug_id: Some("docs.find.query".into()),
            role: "textbox".into(),
            label: Some("Query".into()),
            value: Some(fr_state.query.clone()),
            bounds: UiAutomationRect {
                x: fr.query_field.x0,
                y: fr.query_field.y0,
                width: fr.query_field.width(),
                height: fr.query_field.height(),
            },
            enabled: true,
            focused: false,
            hovered: false,
            children: Vec::new(),
        };
        if let Some(v) = query_node.value.as_mut() {
            v.truncate(256)
        }
        nodes.push(query_node);

        // Replace field with value (only when show_replace is true)
        if fr_state.show_replace {
            let mut replace_node = UiAutomationNode {
                id: {
                    *next_id = next_id.saturating_add(1);
                    *next_id
                },
                debug_id: Some("docs.find.replace_field".into()),
                role: "textbox".into(),
                label: Some("Replacement".into()),
                value: Some(fr_state.replacement.clone()),
                bounds: UiAutomationRect {
                    x: fr.query_field.x0,
                    y: fr.query_field.y0 + fr.query_field.height() + 8.0,
                    width: fr.query_field.width(),
                    height: fr.query_field.height(),
                },
                enabled: true,
                focused: false,
                hovered: false,
                children: Vec::new(),
            };
            if let Some(v) = replace_node.value.as_mut() {
                v.truncate(256)
            }
            nodes.push(replace_node);
        }

        // Find/Replace mode node
        let fr_mode_value = if fr_state.show_replace {
            "replace"
        } else {
            "find"
        };
        push_docs_node(
            nodes,
            next_id,
            "status",
            fr_mode_value,
            "docs.find.mode",
            Rect::new(0.0, 0.0, 0.0, 0.0),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(fr_mode_value.to_string());
        }

        // Match count node
        push_docs_node(
            nodes,
            next_id,
            "status",
            fr_state.matches.len().to_string(),
            "docs.find.match_count",
            Rect::new(0.0, 0.0, 0.0, 0.0),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(fr_state.matches.len().to_string());
        }

        // Current match node (1-based, 0 if none)
        let current_match_value = fr_state
            .current_match_idx
            .map(|i| (i + 1).to_string())
            .unwrap_or_else(|| "0".into());
        push_docs_node(
            nodes,
            next_id,
            "status",
            current_match_value.clone(),
            "docs.find.current_match",
            Rect::new(0.0, 0.0, 0.0, 0.0),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(current_match_value);
        }

        // Query cursor position
        push_docs_node(
            nodes,
            next_id,
            "status",
            fr_state.cursor_pos.to_string(),
            "docs.find.query_cursor",
            Rect::new(0.0, 0.0, 0.0, 0.0),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(fr_state.cursor_pos.to_string());
        }

        push_docs_node(
            nodes,
            next_id,
            "button",
            "Close",
            "docs.find.close",
            fr.close,
        );

        // Button row
        let btn_h = fr.btn_h;
        let btn_gap = 8.0;
        let mut btn_x = fr.modal.x0 + 16.0;

        push_docs_node(
            nodes,
            next_id,
            "button",
            "Find Next",
            "docs.find.next",
            Rect::new(btn_x, fr.btn_row_y, btn_x + 72.0, fr.btn_row_y + btn_h),
        );
        btn_x += 72.0 + btn_gap;

        push_docs_node(
            nodes,
            next_id,
            "button",
            "Find Previous",
            "docs.find.previous",
            Rect::new(btn_x, fr.btn_row_y, btn_x + 72.0, fr.btn_row_y + btn_h),
        );
        btn_x += 72.0 + btn_gap;

        if fr_state.show_replace {
            push_docs_node(
                nodes,
                next_id,
                "button",
                "Replace",
                "docs.find.replace",
                Rect::new(btn_x, fr.btn_row_y, btn_x + 64.0, fr.btn_row_y + btn_h),
            );
            btn_x += 64.0 + btn_gap;
            push_docs_node(
                nodes,
                next_id,
                "button",
                "Replace All",
                "docs.find.replace_all",
                Rect::new(btn_x, fr.btn_row_y, btn_x + 80.0, fr.btn_row_y + btn_h),
            );
        }

        push_docs_node(
            nodes,
            next_id,
            "toggle",
            if fr_state.case_sensitive {
                "Case sensitive (on)"
            } else {
                "Case sensitive"
            },
            "docs.find.case_sensitive",
            fr.case_sensitive,
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(
                if fr_state.case_sensitive {
                    "true"
                } else {
                    "false"
                }
                .to_string(),
            );
        }
        push_docs_node(
            nodes,
            next_id,
            "toggle",
            if fr_state.use_regex {
                "Regex (on)"
            } else {
                "Regex"
            },
            "docs.find.regex",
            fr.regex,
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(if fr_state.use_regex { "true" } else { "false" }.to_string());
        }

        // Search match nodes derived from FindReplaceState.matches
        for (idx, m) in fr_state.matches.iter().enumerate() {
            push_docs_node(
                nodes,
                next_id,
                "Label",
                format!("Search match {}", idx + 1),
                format!("docs.search.match.{idx}"),
                Rect::new(0.0, 0.0, 0.0, 0.0),
            );
            if let Some(node) = nodes.last_mut() {
                let state_str = if fr_state.current_match_idx == Some(idx) {
                    "current"
                } else {
                    "match"
                };
                node.value = Some(format!(
                    "{state_str}:{}:{}:{}",
                    m.block_idx, m.start_offset, m.end_offset
                ));
            }
        }
    } else if state.page_setup_dialog.is_some() {
        // Page setup uses a larger modal (420×380) than the default info modal.
        let ps_w = 420.0;
        let ps_h = 380.0;
        let ps_x0 = size.width / 2.0 - ps_w / 2.0;
        let ps_y0 = size.height / 2.0 - ps_h / 2.0;
        let ps_modal = Rect::new(ps_x0, ps_y0, ps_x0 + ps_w, ps_y0 + ps_h);
        push_docs_node(
            nodes,
            next_id,
            "dialog",
            "Page setup",
            "docs.modal.page_setup",
            ps_modal,
        );
        push_docs_node(
            nodes,
            next_id,
            "button",
            "OK",
            "docs.modal.page_setup.ok",
            Rect::new(
                ps_modal.x1 - 160.0,
                ps_modal.y1 - 44.0,
                ps_modal.x1 - 90.0,
                ps_modal.y1 - 16.0,
            ),
        );
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Cancel",
            "docs.modal.page_setup.cancel",
            Rect::new(
                ps_modal.x1 - 80.0,
                ps_modal.y1 - 44.0,
                ps_modal.x1 - 12.0,
                ps_modal.y1 - 16.0,
            ),
        );

        // Orientation buttons
        let right_x = ps_modal.x0 + ps_modal.width() / 2.0 + 20.0;
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Portrait",
            "docs.modal.page_setup.portrait",
            Rect::new(
                right_x,
                ps_modal.y0 + 80.0,
                right_x + 80.0,
                ps_modal.y0 + 104.0,
            ),
        );
        if let Some(node) = nodes.last_mut() {
            if let Some(dialog) = &state.page_setup_dialog {
                node.value = Some(
                    if dialog.orientation == Orientation::Portrait {
                        "selected"
                    } else {
                        "unselected"
                    }
                    .to_string(),
                );
            }
        }
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Landscape",
            "docs.modal.page_setup.landscape",
            Rect::new(
                right_x + 90.0,
                ps_modal.y0 + 80.0,
                right_x + 190.0,
                ps_modal.y0 + 104.0,
            ),
        );
        if let Some(node) = nodes.last_mut() {
            if let Some(dialog) = &state.page_setup_dialog {
                node.value = Some(
                    if dialog.orientation == Orientation::Landscape {
                        "selected"
                    } else {
                        "unselected"
                    }
                    .to_string(),
                );
            }
        }

        // Paper size options
        if let Some(dialog) = &state.page_setup_dialog {
            let sizes = state::PAPER_SIZES;
            let mut size_y = ps_modal.y0 + 90.0;
            for &size in sizes {
                let label = state::paper_size_label(&size);
                let is_selected = dialog.paper_size == size;
                let size_id = match size {
                    PaperSize::A4 => "docs.modal.page_setup.paper.a4",
                    PaperSize::Letter => "docs.modal.page_setup.paper.letter",
                    PaperSize::Legal => "docs.modal.page_setup.paper.legal",
                    PaperSize::A3 => "docs.modal.page_setup.paper.a3",
                    PaperSize::B5 => "docs.modal.page_setup.paper.b5",
                    _ => "docs.modal.page_setup.paper.custom",
                };
                push_docs_node(
                    nodes,
                    next_id,
                    "option",
                    label,
                    size_id,
                    Rect::new(
                        ps_modal.x0 + 20.0,
                        size_y - 2.0,
                        ps_modal.x0 + ps_modal.width() / 2.0 - 10.0,
                        size_y + 18.0,
                    ),
                );
                if let Some(node) = nodes.last_mut() {
                    node.value = Some(
                        if is_selected {
                            "selected"
                        } else {
                            "unselected"
                        }
                        .to_string(),
                    );
                }
                size_y += 24.0;
            }

            // Margin fields
            let margin_labels = [
                ("Top", "docs.modal.page_setup.margin.top", dialog.margin_top),
                (
                    "Bottom",
                    "docs.modal.page_setup.margin.bottom",
                    dialog.margin_bottom,
                ),
                (
                    "Left",
                    "docs.modal.page_setup.margin.left",
                    dialog.margin_left,
                ),
                (
                    "Right",
                    "docs.modal.page_setup.margin.right",
                    dialog.margin_right,
                ),
            ];
            let mut margin_y = ps_modal.y0 + 156.0;
            for (label, debug_id, value) in margin_labels {
                let is_editing = dialog.editing_margin_field
                    == Some(match label {
                        "Top" => 0,
                        "Bottom" => 1,
                        "Left" => 2,
                        "Right" => 3,
                        _ => 4,
                    });
                push_docs_node(
                    nodes,
                    next_id,
                    "textbox",
                    label,
                    debug_id,
                    Rect::new(right_x + 60.0, margin_y, right_x + 130.0, margin_y + 22.0),
                );
                if let Some(node) = nodes.last_mut() {
                    node.value = Some(if is_editing {
                        dialog.margin_edit_buffer.clone()
                    } else {
                        format!("{:.1}", value)
                    });
                }
                margin_y += 28.0;
            }
        }
    } else if state.word_count_modal {
        let il = compute_info_modal(size);
        push_docs_node(
            nodes,
            next_id,
            "dialog",
            "Word count",
            "docs.modal.word_count",
            il.modal,
        );
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Close",
            "docs.modal.word_count.close",
            il.close,
        );
    } else if let Some(goto_state) = &state.goto_modal {
        let gl = compute_goto(size);
        push_docs_node(
            nodes,
            next_id,
            "dialog",
            "Go to",
            "docs.modal.goto",
            gl.modal,
        );
        push_docs_node(
            nodes,
            next_id,
            "textbox",
            "Go to",
            "docs.goto.input",
            gl.input_field,
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(goto_state.input.clone());
        }

        // Goto mode node
        let goto_mode_value = match goto_state.mode {
            state::GotoMode::Page => "page",
            state::GotoMode::Line => "line",
        };
        push_docs_node(
            nodes,
            next_id,
            "status",
            goto_mode_value,
            "docs.goto.mode",
            Rect::new(0.0, 0.0, 0.0, 0.0),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(goto_mode_value.to_string());
        }

        // Goto cursor position
        push_docs_node(
            nodes,
            next_id,
            "status",
            goto_state.cursor_pos.to_string(),
            "docs.goto.cursor",
            Rect::new(0.0, 0.0, 0.0, 0.0),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(goto_state.cursor_pos.to_string());
        }

        // Goto mode buttons with selected state
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Page",
            "docs.goto.page_mode",
            gl.page_mode,
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(
                if matches!(goto_state.mode, state::GotoMode::Page) {
                    "selected"
                } else {
                    "unselected"
                }
                .to_string(),
            );
        }
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Line",
            "docs.goto.line_mode",
            gl.line_mode,
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(
                if matches!(goto_state.mode, state::GotoMode::Line) {
                    "selected"
                } else {
                    "unselected"
                }
                .to_string(),
            );
        }
    } else if state.print_preview.is_some() {
        let pp = compute_print_preview(size);
        push_docs_node(
            nodes,
            next_id,
            "dialog",
            "Print Preview",
            "docs.modal.print_preview",
            pp.modal,
        );
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Prev",
            "docs.print_preview.prev",
            pp.prev_btn,
        );
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Next",
            "docs.print_preview.next",
            pp.next_btn,
        );
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Print",
            "docs.print_preview.print",
            pp.print_btn,
        );
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Close",
            "docs.print_preview.close",
            pp.close,
        );
        if let Some(pp_state) = &state.print_preview {
            let num_pages = state.layout_cache.num_pages().max(1);
            let clamped_idx = pp_state.page_index.min(num_pages - 1);
            push_docs_node(
                nodes,
                next_id,
                "text",
                format!("Page {} of {}", clamped_idx + 1, num_pages),
                "docs.print_preview.page_indicator",
                pp.page_indicator,
            );
            if let Some(node) = nodes.last_mut() {
                node.value = Some(format!("{}:{}", clamped_idx, num_pages));
            }
            // Preview page node with page-map range
            let page_map = state.layout_cache.page_map();
            let start_block = page_map
                .get(clamped_idx)
                .map(|e| e.start_block)
                .unwrap_or(0);
            let end_block = page_map
                .get(clamped_idx + 1)
                .map(|e| e.start_block)
                .unwrap_or(state.current_document().content.len());
            push_docs_node(
                nodes,
                next_id,
                "image",
                "Preview page",
                "docs.print_preview.page",
                pp.preview_page,
            );
            if let Some(node) = nodes.last_mut() {
                node.value = Some(format!("{}:{}:{}", clamped_idx, start_block, end_block));
            }
        }
    } else if state.special_char_modal.is_some() {
        // Special character modal uses a larger modal (420×380).
        let sc_w = 420.0;
        let sc_h = 380.0;
        let sc_x0 = size.width / 2.0 - sc_w / 2.0;
        let sc_y0 = size.height / 2.0 - sc_h / 2.0;
        let sc_modal = Rect::new(sc_x0, sc_y0, sc_x0 + sc_w, sc_y0 + sc_h);
        push_docs_node(
            nodes,
            next_id,
            "dialog",
            "Special character",
            "docs.modal.special_char",
            sc_modal,
        );
        // Close button at bottom-right of modal
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Close",
            "docs.modal.special_char.close",
            Rect::new(
                sc_modal.x1 - 92.0,
                sc_modal.y1 - 42.0,
                sc_modal.x1 - 16.0,
                sc_modal.y1 - 14.0,
            ),
        );

        // Category tabs
        if let Some(sc_state) = &state.special_char_modal {
            let tab_y = sc_modal.y0 + 36.0;
            let mut tab_x = sc_modal.x0 + 12.0;
            for (idx, (cat_name, _chars)) in state::SPECIAL_CHAR_CATEGORIES.iter().enumerate() {
                let tab_w = cat_name.len() as f64 * 7.0 + 16.0;
                let cat_id = match *cat_name {
                    "Common Symbols" => "docs.modal.special_char.category.common_symbols",
                    "Arrows" => "docs.modal.special_char.category.arrows",
                    "Math" => "docs.modal.special_char.category.math",
                    "Currency" => "docs.modal.special_char.category.currency",
                    "Latin Extended" => "docs.modal.special_char.category.latin_extended",
                    "Punctuation" => "docs.modal.special_char.category.punctuation",
                    _ => continue,
                };
                push_docs_node(
                    nodes,
                    next_id,
                    "tab",
                    *cat_name,
                    cat_id,
                    Rect::new(tab_x, tab_y, tab_x + tab_w, tab_y + 22.0),
                );
                if let Some(node) = nodes.last_mut() {
                    node.value = Some(
                        if idx == sc_state.category_idx {
                            "selected"
                        } else {
                            "unselected"
                        }
                        .to_string(),
                    );
                }
                tab_x += tab_w + 4.0;
            }

            // Grid cells for the selected category
            if let Some((_cat_name, chars)) =
                state::SPECIAL_CHAR_CATEGORIES.get(sc_state.category_idx)
            {
                let grid_x = sc_modal.x0 + 16.0;
                let grid_y = tab_y + 30.0;
                let cell_size = 32.0;
                let cols = ((sc_w - 32.0) / cell_size) as usize;

                for (i, &ch) in chars.iter().enumerate() {
                    let col = i % cols;
                    let row = i / cols;
                    let cell_x = grid_x + col as f64 * cell_size;
                    let cell_y = grid_y + row as f64 * cell_size;
                    if cell_y + cell_size > sc_modal.y1 - 40.0 {
                        break;
                    }
                    push_docs_node(
                        nodes,
                        next_id,
                        "button",
                        ch.to_string(),
                        format!("docs.modal.special_char.cell.{}", i),
                        Rect::new(cell_x, cell_y, cell_x + cell_size, cell_y + cell_size),
                    );
                    if let Some(node) = nodes.last_mut() {
                        node.value = Some(ch.to_string());
                    }
                }
            }
        }
    }
}
