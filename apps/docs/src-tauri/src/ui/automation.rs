mod modal_nodes;

use modal_nodes::push_docs_modal_nodes;

use super::*;

pub(crate) fn docs_automation_nodes(
    state: &DocsState,
    size: Size,
    base_id: u64,
) -> Vec<UiAutomationNode> {
    let mut nodes = Vec::new();
    let mut next_id = base_id.saturating_mul(1000);
    let width = size.width;
    let height = size.height;

    let menus = [
        ("File", "docs.menu.file"),
        ("Edit", "docs.menu.edit"),
        ("View", "docs.menu.view"),
        ("Insert", "docs.menu.insert"),
        ("Format", "docs.menu.format"),
        ("Tools", "docs.menu.tools"),
        ("Help", "docs.menu.help"),
    ];
    let mut menu_x = 12.0;
    for (name, debug_id) in menus {
        let menu_w = match name {
            "Insert" | "Format" => 54.0,
            _ => 42.0,
        };
        push_docs_node(
            &mut nodes,
            &mut next_id,
            "button",
            name,
            debug_id,
            Rect::new(menu_x, 0.0, menu_x + menu_w, MENU_BAR_H),
        );
        menu_x += menu_w;
    }

    // Menu active state node
    let active_modal_value = state.active_modal.as_deref().unwrap_or("none");
    push_docs_node(
        &mut nodes,
        &mut next_id,
        "status",
        active_modal_value,
        "docs.menu.active",
        Rect::new(0.0, 0.0, 0.0, 0.0),
    );

    push_docs_node(
        &mut nodes,
        &mut next_id,
        "status",
        if state.is_dirty() { "Unsaved" } else { "Saved" },
        "docs.save_status",
        Rect::new(
            width - WINDOW_CONTROLS_W - 80.0,
            8.0,
            width - WINDOW_CONTROLS_W - 18.0,
            MENU_BAR_H - 8.0,
        ),
    );
    if let Some(node) = nodes.last_mut() {
        node.value = Some(if state.is_dirty() { "unsaved" } else { "saved" }.to_string());
    }

    // Caption buttons (minimize / maximize-restore / close).
    for (control, debug_id, label) in [
        (WindowControl::Minimize, "docs.window.minimize", "Minimize"),
        (
            WindowControl::MaximizeRestore,
            "docs.window.maximize",
            "Maximize",
        ),
        (WindowControl::Close, "docs.window.close", "Close"),
    ] {
        let rect = tench_ui::widgets::control_rect(width, MENU_BAR_H, control);
        push_docs_node(&mut nodes, &mut next_id, "button", label, debug_id, rect);
        if control == WindowControl::MaximizeRestore {
            if let Some(node) = nodes.last_mut() {
                node.value = Some(
                    if state.window_maximized {
                        "maximized"
                    } else {
                        "restored"
                    }
                    .to_string(),
                );
            }
        }
    }

    let toolbar_y = MENU_BAR_H + 8.0;
    let toolbar_h = 32.0;
    let mut toolbar_x = TOOLBAR_LEFT_PAD;
    let mut prev_group = 0;
    let mut first_item = true;
    for item in TOOLBAR_LAYOUT.iter() {
        if !first_item && item.group != prev_group {
            toolbar_x += SEPARATOR_W;
            if prev_group == 0 && item.group >= 4 {
                push_docs_node(
                    &mut nodes,
                    &mut next_id,
                    "dropdown",
                    "Font size",
                    "docs.toolbar.font_size",
                    Rect::new(
                        toolbar_x,
                        toolbar_y,
                        toolbar_x + FONT_SIZE_SELECT_W,
                        toolbar_y + toolbar_h,
                    ),
                );
                toolbar_x += FONT_SIZE_SELECT_W + BTN_GAP + SEPARATOR_W;
                push_docs_node(
                    &mut nodes,
                    &mut next_id,
                    "dropdown",
                    "Font family",
                    "docs.toolbar.font_family",
                    Rect::new(
                        toolbar_x,
                        toolbar_y,
                        toolbar_x + FONT_FAMILY_SELECT_W,
                        toolbar_y + toolbar_h,
                    ),
                );
                toolbar_x += FONT_FAMILY_SELECT_W + BTN_GAP + SEPARATOR_W;
                push_docs_node(
                    &mut nodes,
                    &mut next_id,
                    "dropdown",
                    "Paragraph style",
                    "docs.toolbar.paragraph_style",
                    Rect::new(
                        toolbar_x,
                        toolbar_y,
                        toolbar_x + PARAGRAPH_SELECT_W,
                        toolbar_y + toolbar_h,
                    ),
                );
                toolbar_x += PARAGRAPH_SELECT_W + BTN_GAP + SEPARATOR_W;
            }
            prev_group = item.group;
        }
        first_item = false;

        if let Some(debug_id) = toolbar_action_debug_id(&item.action) {
            push_docs_node(
                &mut nodes,
                &mut next_id,
                "button",
                item.tooltip,
                debug_id,
                Rect::new(
                    toolbar_x,
                    toolbar_y,
                    toolbar_x + item.width,
                    toolbar_y + toolbar_h,
                ),
            );
        }
        toolbar_x += item.width + BTN_GAP;
    }

    let main_y = MENU_BAR_H + TOOLBAR_H;
    let status_y = height - STATUS_BAR_H;
    let sidebar_open = state.show_style_panel || state.show_comments;
    let sidebar_w = if sidebar_open { STYLE_PANEL_W } else { 0.0 };
    let content_w = width - sidebar_w;
    let workspace_y = main_y + TITLE_ROW_H + RULER_H;
    let tab_bar_h = if state.open_tabs.len() > 1 { 32.0 } else { 0.0 };
    let doc_y = workspace_y + tab_bar_h;

    push_docs_node(
        &mut nodes,
        &mut next_id,
        "title",
        state.title(),
        "docs.title_row",
        Rect::new(0.0, main_y, content_w, main_y + TITLE_ROW_H),
    );
    push_docs_node(
        &mut nodes,
        &mut next_id,
        "ruler",
        "Ruler",
        "docs.ruler",
        Rect::new(
            0.0,
            main_y + TITLE_ROW_H,
            content_w,
            main_y + TITLE_ROW_H + RULER_H,
        ),
    );

    // Ruler drag handle nodes – geometry mirrors ruler_hit_test / paint_ruler
    {
        let ruler_rect = Rect::new(
            0.0,
            main_y + TITLE_ROW_H,
            content_w,
            main_y + TITLE_ROW_H + RULER_H,
        );
        let scale = state.zoom / 100.0;
        let doc = state.current_document();
        let setup = &doc.page_setup;
        let (page_w_raw, _) = setup.page_size_px();
        let page_w = page_w_raw * scale;
        let mm_to_px = 96.0 / 25.4;
        let margin_left_px = setup.margins.left as f64 * mm_to_px * scale;
        let margin_right_px = setup.margins.right as f64 * mm_to_px * scale;

        let track_w = page_w.min((ruler_rect.width() - 40.0).max(240.0));
        let track_x = ruler_rect.x0 + (ruler_rect.width() - track_w) / 2.0;
        let track_scale = track_w / page_w;

        let content_left = track_x + margin_left_px * track_scale;
        let content_right = track_x + (page_w - margin_right_px) * track_scale;

        let handle_size = 12.0;
        let half = handle_size / 2.0;
        let ruler_mid_y = ruler_rect.y0 + ruler_rect.height() / 2.0;

        // Left margin handle
        push_docs_node(
            &mut nodes,
            &mut next_id,
            "Slider",
            "Left margin",
            "docs.ruler.margin.left",
            Rect::new(
                content_left - half,
                ruler_mid_y - half,
                content_left + half,
                ruler_mid_y + half,
            ),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(format!("{:.1}", setup.margins.left));
            if state.ruler_drag == Some(RulerDragTarget::LeftMargin) {
                node.value = Some(format!("dragging:{:.1}", setup.margins.left));
            }
        }
        // Right margin handle
        push_docs_node(
            &mut nodes,
            &mut next_id,
            "Slider",
            "Right margin",
            "docs.ruler.margin.right",
            Rect::new(
                content_right - half,
                ruler_mid_y - half,
                content_right + half,
                ruler_mid_y + half,
            ),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(format!("{:.1}", setup.margins.right));
            if state.ruler_drag == Some(RulerDragTarget::RightMargin) {
                node.value = Some(format!("dragging:{:.1}", setup.margins.right));
            }
        }

        // Indent handles for the current paragraph
        if let Some(block) = doc.content.get(state.cursor().block_idx) {
            let (indent_left, indent_right, indent_first) = extract_indents_for_automation(block);

            let indent_left_x = content_left + indent_left as f64 * track_scale;
            let indent_right_x = content_right - indent_right as f64 * track_scale;

            // Left indent handle
            push_docs_node(
                &mut nodes,
                &mut next_id,
                "Slider",
                "Left indent",
                "docs.ruler.indent.left",
                Rect::new(
                    indent_left_x - half,
                    ruler_mid_y - half,
                    indent_left_x + half,
                    ruler_mid_y + half,
                ),
            );
            if let Some(node) = nodes.last_mut() {
                node.value = Some(format!("{:.1}", indent_left));
                if state.ruler_drag == Some(RulerDragTarget::IndentLeft) {
                    node.value = Some(format!("dragging:{:.1}", indent_left));
                }
            }
            // Right indent handle
            push_docs_node(
                &mut nodes,
                &mut next_id,
                "Slider",
                "Right indent",
                "docs.ruler.indent.right",
                Rect::new(
                    indent_right_x - half,
                    ruler_mid_y - half,
                    indent_right_x + half,
                    ruler_mid_y + half,
                ),
            );
            if let Some(node) = nodes.last_mut() {
                node.value = Some(format!("{:.1}", indent_right));
                if state.ruler_drag == Some(RulerDragTarget::IndentRight) {
                    node.value = Some(format!("dragging:{:.1}", indent_right));
                }
            }
            // First-line indent handle
            let first_indent_x = indent_left_x + indent_first as f64 * track_scale;
            push_docs_node(
                &mut nodes,
                &mut next_id,
                "Slider",
                "First line indent",
                "docs.ruler.indent.first_line",
                Rect::new(
                    first_indent_x - half,
                    ruler_mid_y - half,
                    first_indent_x + half,
                    ruler_mid_y + half,
                ),
            );
            if let Some(node) = nodes.last_mut() {
                node.value = Some(format!("{:.1}", indent_first));
                if state.ruler_drag == Some(RulerDragTarget::IndentFirstLine) {
                    node.value = Some(format!("dragging:{:.1}", indent_first));
                }
            }
        }
    }

    // Tab bar nodes (only when multiple tabs are open)
    if tab_bar_h > 0.0 {
        let tab_w: f64 = 160.0;
        let tab_x_start = 8.0;
        let close_btn_w = 20.0;
        for (idx, tab) in state.open_tabs.iter().enumerate() {
            let tab_x = tab_x_start + idx as f64 * (tab_w + 2.0);
            let tab_rect = Rect::new(tab_x, workspace_y, tab_x + tab_w, workspace_y + tab_bar_h);
            let is_active = idx == state.active_tab_idx;
            let value = if is_active {
                Some("active".to_string())
            } else {
                None
            };
            // Tab node
            let mut tab_node = UiAutomationNode {
                id: {
                    next_id = next_id.saturating_add(1);
                    next_id
                },
                debug_id: Some(format!("docs.tab.{idx}")),
                role: "tab".into(),
                label: Some(tab.title.clone()),
                value,
                bounds: UiAutomationRect {
                    x: tab_rect.x0,
                    y: tab_rect.y0,
                    width: tab_rect.width(),
                    height: tab_rect.height(),
                },
                enabled: true,
                focused: false,
                hovered: false,
                children: Vec::new(),
            };
            if tab.dirty {
                tab_node.label = Some(format!("{} *", tab.title));
            }
            nodes.push(tab_node);

            // Close button (only when more than one tab)
            if state.open_tabs.len() > 1 {
                let close_rect = Rect::new(
                    tab_rect.x1 - close_btn_w - 4.0,
                    tab_rect.y0 + (tab_bar_h - 14.0) / 2.0,
                    tab_rect.x1 - 4.0,
                    tab_rect.y0 + (tab_bar_h + 14.0) / 2.0,
                );
                push_docs_node(
                    &mut nodes,
                    &mut next_id,
                    "button",
                    "Close",
                    format!("docs.tab.{idx}.close"),
                    close_rect,
                );
            }
        }
    }

    push_docs_node(
        &mut nodes,
        &mut next_id,
        "document",
        "Document",
        "docs.document",
        Rect::new(0.0, doc_y, content_w, status_y),
    );

    push_document_state_nodes(
        state,
        size,
        doc_y,
        content_w,
        tab_bar_h,
        &mut nodes,
        &mut next_id,
    );

    push_status_nodes(state, width, height, status_y, &mut nodes, &mut next_id);

    // Track changes indicator in status bar
    if state.track_changes {
        push_docs_node(
            &mut nodes,
            &mut next_id,
            "status",
            "Track changes",
            "docs.track_changes_indicator",
            Rect::new(width - 280.0, status_y + 4.0, width - 180.0, height - 4.0),
        );
    }

    // Toast notification node
    if let Some((toast_msg, toast_expiry)) = &state.toast {
        let toast_w = 320.0;
        let toast_h = 36.0;
        let toast_x = (width - toast_w) / 2.0;
        let toast_y = height - STATUS_BAR_H - toast_h - 16.0;
        let expiry_state = if *toast_expiry > 0.0 {
            "scheduled"
        } else {
            "pending"
        };
        let mut toast_node = UiAutomationNode {
            id: {
                next_id = next_id.saturating_add(1);
                next_id
            },
            debug_id: Some("docs.toast".into()),
            role: "alert".into(),
            label: Some("Toast".into()),
            value: Some(format!("{expiry_state}:{toast_msg}")),
            bounds: UiAutomationRect {
                x: toast_x,
                y: toast_y,
                width: toast_w,
                height: toast_h,
            },
            enabled: true,
            focused: false,
            hovered: false,
            children: Vec::new(),
        };
        if let Some(ref mut v) = toast_node.value {
            v.truncate(256);
        }
        nodes.push(toast_node);
    }

    // Toolbar tooltip node
    if let Some(tooltip_text) = &state.hovered_tooltip {
        let tooltip_x = state.hovered_tooltip_x;
        push_docs_node(
            &mut nodes,
            &mut next_id,
            "tooltip",
            tooltip_text.clone(),
            "docs.toolbar.tooltip",
            Rect::new(
                tooltip_x - 40.0,
                MENU_BAR_H + TOOLBAR_H + 4.0,
                tooltip_x + 40.0,
                MENU_BAR_H + TOOLBAR_H + 28.0,
            ),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(tooltip_text.clone());
        }
    }

    // Modal backdrop node — present when any modal is open
    let any_modal = state.active_modal.is_some()
        || state.link_modal.is_some()
        || state.page_setup_dialog.is_some()
        || state.print_preview.is_some()
        || state.word_count_modal
        || state.goto_modal.is_some()
        || state.special_char_modal.is_some()
        || state.find_replace.is_some()
        || state.comment_modal.is_some();
    if any_modal {
        push_docs_node(
            &mut nodes,
            &mut next_id,
            "backdrop",
            "Modal backdrop",
            "docs.modal.backdrop",
            Rect::new(0.0, 0.0, width, height),
        );
    }

    if let Some(modal_name) = &state.active_modal {
        if menu_items_for(modal_name).is_empty() {
            if is_info_modal(modal_name) {
                let m = info_modal_rect(size);
                push_docs_node(
                    &mut nodes,
                    &mut next_id,
                    "dialog",
                    modal_name,
                    format!("docs.modal.{}", simple_docs_id(modal_name)),
                    m,
                );
            }
        } else {
            let items = menu_items_for(modal_name);
            let item_h = 26.0;
            let menu_key = simple_docs_id(modal_name);
            let menu_panel_w = 220.0;
            let menu_panel_h = 8.0 + items.len() as f64 * item_h + 8.0;

            // Menu panel node with bounds matching the dropdown
            push_docs_node(
                &mut nodes,
                &mut next_id,
                "panel",
                modal_name,
                format!("docs.menu.{menu_key}.panel"),
                Rect::new(
                    12.0,
                    MENU_BAR_H,
                    12.0 + menu_panel_w,
                    MENU_BAR_H + menu_panel_h,
                ),
            );

            for (idx, item) in items.iter().enumerate() {
                let is_hovered = state.hovered_menu_item == Some(idx);
                push_docs_node(
                    &mut nodes,
                    &mut next_id,
                    "MenuItem",
                    *item,
                    format!("docs.menu.{menu_key}.{}", simple_docs_id(item)),
                    Rect::new(
                        12.0,
                        MENU_BAR_H + 8.0 + idx as f64 * item_h,
                        12.0 + menu_panel_w,
                        MENU_BAR_H + 8.0 + (idx + 1) as f64 * item_h,
                    ),
                );
                if let Some(node) = nodes.last_mut() {
                    node.value = Some(if is_hovered { "hovered" } else { "idle" }.to_string());
                    node.hovered = is_hovered;
                }
            }

            // Hovered menu item status node
            let hovered_value = state
                .hovered_menu_item
                .and_then(|idx| items.get(idx).copied())
                .unwrap_or("none");
            push_docs_node(
                &mut nodes,
                &mut next_id,
                "status",
                hovered_value,
                format!("docs.menu.{menu_key}.hovered"),
                Rect::new(0.0, 0.0, 0.0, 0.0),
            );
            if let Some(node) = nodes.last_mut() {
                node.value = Some(hovered_value.to_string());
            }
        }
    }

    push_docs_modal_nodes(state, size, &mut nodes, &mut next_id);

    push_sidebar_nodes(state, content_w, main_y, status_y, &mut nodes, &mut next_id);

    // ── Thumbnail page previews ──
    if state.show_thumbnails && content_w > THUMB_PANEL_W + 480.0 {
        let num_pages = state.layout_cache.num_pages().max(1);
        let thumb_w = THUMB_PANEL_W - 24.0;
        let thumb_h = thumb_w * (state::PAGE_H / state::PAGE_W);
        let mut thumb_y = main_y + 12.0;
        for idx in 0..num_pages {
            if thumb_y + thumb_h + 20.0 > status_y {
                break;
            }
            push_docs_node(
                &mut nodes,
                &mut next_id,
                "thumbnail",
                format!("Page {}", idx + 1),
                format!("docs.thumbnail.page.{idx}"),
                Rect::new(12.0, thumb_y, 12.0 + thumb_w, thumb_y + thumb_h),
            );
            if let Some(node) = nodes.last_mut() {
                let is_active = idx + 1 == state.current_page;
                node.value = Some(if is_active {
                    format!("active:{}", idx + 1)
                } else {
                    format!("{}", idx + 1)
                });
            }
            thumb_y += thumb_h + 20.0;
        }
    }

    // ── Header/Footer editing fields ──
    if state.editing_header {
        push_docs_node(
            &mut nodes,
            &mut next_id,
            "TextInput",
            "Header",
            "docs.header_field",
            Rect::new(0.0, main_y, content_w, main_y + TITLE_ROW_H),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(state.header_text.clone());
        }
    }
    if state.editing_footer {
        push_docs_node(
            &mut nodes,
            &mut next_id,
            "TextInput",
            "Footer",
            "docs.footer_field",
            Rect::new(0.0, status_y - 30.0, content_w, status_y),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(state.footer_text.clone());
        }
    }

    // ── Context menu items ──
    if let Some(cm) = &state.context_menu {
        let item_h = 28.0;
        let menu_w = 200.0;
        for (idx, item) in cm.items().iter().enumerate() {
            let id = simple_docs_id(item);
            let is_hovered = cm.hovered_item == Some(idx);
            push_docs_node(
                &mut nodes,
                &mut next_id,
                "MenuItem",
                *item,
                format!("docs.context.{id}"),
                Rect::new(
                    cm.x,
                    cm.y + idx as f64 * item_h,
                    cm.x + menu_w,
                    cm.y + (idx as f64 + 1.0) * item_h,
                ),
            );
            if let Some(node) = nodes.last_mut() {
                node.value = Some(if is_hovered { "hovered" } else { "idle" }.to_string());
                node.hovered = is_hovered;
            }
        }
    }

    // ── Image resize handle nodes ──
    {
        let scale = state.zoom / 100.0;
        let doc = state.current_document();
        let setup = &doc.page_setup;
        let (page_w_raw, _) = setup.page_size_px();
        let page_w = page_w_raw * scale;
        let mm_to_px = 96.0 / 25.4;
        let margin_left = setup.margins.left as f64 * mm_to_px * scale;
        let margin_right = setup.margins.right as f64 * mm_to_px * scale;
        let margin_top = setup.margins.top as f64 * mm_to_px * scale;
        let header_h = state::HEADER_H * scale;

        let page_x = ((content_w - page_w) / 2.0).max(state::PAGE_MARGIN_X);
        let content_left = page_x + margin_left;
        let content_w_img = page_w - margin_left - margin_right;
        let content_top = doc_y + state::PAGE_MARGIN_Y + margin_top + header_h;

        let mut y = content_top;
        for (block_idx, block) in doc.content.iter().enumerate() {
            if let BlockNode::Image { width, height, .. } = block {
                let max_img_w = content_w_img;
                let img_w = width
                    .map(|w| (w as f64 * scale).min(max_img_w))
                    .unwrap_or(max_img_w * 0.6);
                let img_h = height.map(|h| h as f64 * scale).unwrap_or(img_w * 0.75);
                let img_x = content_left + (content_w_img - img_w) / 2.0;
                let img_y = y + 8.0 * scale;

                // Image block node
                let is_targeted = state.targeted_image_block == Some(block_idx)
                    || state
                        .image_resize_drag
                        .as_ref()
                        .is_some_and(|d| d.block_idx == block_idx);
                push_docs_node(
                    &mut nodes,
                    &mut next_id,
                    "image",
                    format!("Image {block_idx}"),
                    format!("docs.image_block.{block_idx}"),
                    Rect::new(img_x, img_y, img_x + img_w, img_y + img_h),
                );
                if let Some(node) = nodes.last_mut() {
                    node.value = Some(format!("{:.0}x{:.0}", img_w, img_h));
                }

                // Corner resize handles — only for the targeted image
                if is_targeted {
                    let handle_size = 8.0;
                    let active_handle = state
                        .image_resize_drag
                        .as_ref()
                        .filter(|d| d.block_idx == block_idx)
                        .map(|d| d.handle);
                    let corners = [
                        (0, "tl", img_x, img_y),
                        (1, "tr", img_x + img_w, img_y),
                        (2, "bl", img_x, img_y + img_h),
                        (3, "br", img_x + img_w, img_y + img_h),
                    ];
                    for (corner_idx, corner_id, cx, cy) in corners {
                        push_docs_node(
                            &mut nodes,
                            &mut next_id,
                            "Slider",
                            format!("Resize {corner_id}"),
                            format!("docs.image_resize.{block_idx}.{corner_id}"),
                            Rect::new(
                                cx - handle_size,
                                cy - handle_size,
                                cx + handle_size,
                                cy + handle_size,
                            ),
                        );
                        if let Some(node) = nodes.last_mut() {
                            node.value = Some(
                                if active_handle == Some(corner_idx) {
                                    "dragging"
                                } else {
                                    "idle"
                                }
                                .to_string(),
                            );
                        }
                    }
                }

                y = img_y + img_h + 16.0 * scale;
            } else {
                let text = extract_block_text(block);
                let line_h = 20.0 * scale;
                let chars_per_line = ((content_w_img / (7.0 * scale)).max(1.0)) as usize;
                let lines = if chars_per_line > 0 && !text.is_empty() {
                    text.chars().count().div_ceil(chars_per_line)
                } else {
                    1
                };
                y += lines as f64 * line_h + 12.0 * scale;
            }
        }
    }

    push_dropdown_nodes(state, &mut nodes, &mut next_id);

    nodes
}
