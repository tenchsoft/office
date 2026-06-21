use super::*;
pub(crate) use super::automation_license::push_license_nodes;

pub(crate) fn kodocs_automation_nodes(
    state: &KodocsState,
    size: Size,
    base_id: u64,
) -> Vec<UiAutomationNode> {
    let mut nodes = Vec::new();
    let mut next_id = base_id.saturating_mul(1000);
    let width = size.width;
    let height = size.height;

    // --- Menu bar buttons ---
    let menu_names = ["파일", "편집", "보기", "삽입", "서식", "도구", "도움말"];
    let menu_ids = [
        "kodocs.menu.file",
        "kodocs.menu.edit",
        "kodocs.menu.view",
        "kodocs.menu.insert",
        "kodocs.menu.format",
        "kodocs.menu.tools",
        "kodocs.menu.help",
    ];
    let mut x = 12.0;
    for (i, name) in menu_names.iter().enumerate() {
        let w = match *name {
            "삽입" | "서식" | "도움말" => 50.0,
            _ => 42.0,
        };
        push_kodocs_node(
            &mut nodes,
            &mut next_id,
            "button",
            *name,
            menu_ids[i],
            Rect::new(x, 0.0, x + w, MENU_BAR_H),
        );
        x += w;
    }

    // Save status pill
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "status",
        if state.is_dirty() {
            "저장 안 됨"
        } else {
            "저장됨"
        },
        "kodocs.save_status",
        Rect::new(
            width - WINDOW_CONTROLS_W - 80.0,
            8.0,
            width - WINDOW_CONTROLS_W - 18.0,
            MENU_BAR_H - 8.0,
        ),
    );

    // Caption buttons (minimize / maximize-restore / close).
    for (control, debug_id, label) in [
        (
            WindowControl::Minimize,
            "kodocs.window.minimize",
            "Minimize",
        ),
        (
            WindowControl::MaximizeRestore,
            "kodocs.window.maximize",
            "Maximize",
        ),
        (WindowControl::Close, "kodocs.window.close", "Close"),
    ] {
        let rect = tench_ui::widgets::control_rect(width, MENU_BAR_H, control);
        push_kodocs_node(&mut nodes, &mut next_id, "button", label, debug_id, rect);
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

    // --- Toolbar buttons ---
    let toolbar_btn_y = MENU_BAR_H + 8.0;
    let toolbar_btn_h = 32.0;

    // Group 0: Undo/Redo (48px each in kodocs)
    let mut tx = 12.0;
    for (label, id, w) in [
        ("실행 취소", "kodocs.toolbar.undo", 48.0),
        ("다시 실행", "kodocs.toolbar.redo", 48.0),
    ] {
        push_kodocs_node(
            &mut nodes,
            &mut next_id,
            "button",
            label,
            id,
            Rect::new(tx, toolbar_btn_y, tx + w, toolbar_btn_y + toolbar_btn_h),
        );
        tx += w + 2.0;
    }
    tx += 14.0; // separator

    // Dropdown: Font Family (120px)
    let ff_x = tx;
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "dropdown",
        "글꼴",
        "kodocs.toolbar.font_family",
        Rect::new(
            ff_x,
            toolbar_btn_y,
            ff_x + 120.0,
            toolbar_btn_y + toolbar_btn_h,
        ),
    );
    tx += 120.0 + 2.0 + 14.0;

    // Dropdown: Font Size (62px)
    let fs_x = tx;
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "dropdown",
        "글꼴 크기",
        "kodocs.toolbar.font_size",
        Rect::new(
            fs_x,
            toolbar_btn_y,
            fs_x + 62.0,
            toolbar_btn_y + toolbar_btn_h,
        ),
    );
    tx += 62.0 + 2.0 + 14.0;

    // Dropdown: Paragraph Style (112px)
    let ps_x = tx;
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "dropdown",
        "단락 스타일",
        "kodocs.toolbar.paragraph_style",
        Rect::new(
            ps_x,
            toolbar_btn_y,
            ps_x + 112.0,
            toolbar_btn_y + toolbar_btn_h,
        ),
    );
    tx += 112.0 + 2.0 + 14.0;

    // Group 4: Format buttons
    for (label, id, w) in [
        ("B", "kodocs.toolbar.bold", 32.0),
        ("I", "kodocs.toolbar.italic", 32.0),
        ("U", "kodocs.toolbar.underline", 32.0),
        ("S", "kodocs.toolbar.strikethrough", 32.0),
        ("<>", "kodocs.toolbar.code", 32.0),
        ("x²", "kodocs.toolbar.superscript", 32.0),
        ("x_", "kodocs.toolbar.subscript", 32.0),
        ("HL", "kodocs.toolbar.highlight", 36.0),
    ] {
        push_kodocs_node(
            &mut nodes,
            &mut next_id,
            "button",
            label,
            id,
            Rect::new(tx, toolbar_btn_y, tx + w, toolbar_btn_y + toolbar_btn_h),
        );
        tx += w + 2.0;
    }
    tx += 12.0; // separator

    // Group 5: List buttons
    for (label, id, w) in [
        ("•", "kodocs.toolbar.bullet_list", 28.0),
        ("1.", "kodocs.toolbar.numbered_list", 28.0),
        ("☑", "kodocs.toolbar.checklist", 28.0),
        ("←", "kodocs.toolbar.outdent", 32.0),
        ("→", "kodocs.toolbar.indent", 32.0),
    ] {
        push_kodocs_node(
            &mut nodes,
            &mut next_id,
            "button",
            label,
            id,
            Rect::new(tx, toolbar_btn_y, tx + w, toolbar_btn_y + toolbar_btn_h),
        );
        tx += w + 2.0;
    }
    tx += 14.0;

    // Group 6: Alignment buttons
    for (label, id, w) in [
        ("좌", "kodocs.toolbar.align_left", 28.0),
        ("중", "kodocs.toolbar.align_center", 28.0),
        ("우", "kodocs.toolbar.align_right", 28.0),
        ("양", "kodocs.toolbar.align_justify", 28.0),
    ] {
        push_kodocs_node(
            &mut nodes,
            &mut next_id,
            "button",
            label,
            id,
            Rect::new(tx, toolbar_btn_y, tx + w, toolbar_btn_y + toolbar_btn_h),
        );
        tx += w + 2.0;
    }
    tx += 14.0;

    // Group 7: Insert buttons
    for (label, id, w) in [
        ("링크", "kodocs.toolbar.insert_link", 42.0),
        ("그림", "kodocs.toolbar.insert_image", 38.0),
        ("표", "kodocs.toolbar.insert_table", 28.0),
        ("줄", "kodocs.toolbar.horizontal_rule", 28.0),
        ("인용", "kodocs.toolbar.block_quote", 36.0),
    ] {
        push_kodocs_node(
            &mut nodes,
            &mut next_id,
            "button",
            label,
            id,
            Rect::new(tx, toolbar_btn_y, tx + w, toolbar_btn_y + toolbar_btn_h),
        );
        tx += w + 2.0;
    }
    tx += 14.0;

    // Group 8: Color buttons
    for (label, id, w) in [
        ("글자색", "kodocs.toolbar.text_color", 50.0),
        ("배경색", "kodocs.toolbar.highlight_color", 50.0),
    ] {
        push_kodocs_node(
            &mut nodes,
            &mut next_id,
            "button",
            label,
            id,
            Rect::new(tx, toolbar_btn_y, tx + w, toolbar_btn_y + toolbar_btn_h),
        );
        tx += w + 2.0;
    }

    // More overflow button (always present in automation tree)
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "button",
        "더보기",
        "kodocs.toolbar.more_overflow",
        Rect::new(tx, toolbar_btn_y, tx + 32.0, toolbar_btn_y + toolbar_btn_h),
    );

    // --- Main content area ---
    let main_y = MENU_BAR_H + TOOLBAR_H;
    let status_y = height - STATUS_BAR_H;
    let sidebar_open = state.show_style_panel || state.show_comments;
    let sidebar_w = if sidebar_open { STYLE_PANEL_W } else { 0.0 };
    let content_w = width - sidebar_w;

    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "title",
        state.title(),
        "kodocs.title_row",
        Rect::new(0.0, main_y, content_w, main_y + TITLE_ROW_H),
    );
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "ruler",
        "Ruler",
        "kodocs.ruler",
        Rect::new(
            0.0,
            main_y + TITLE_ROW_H,
            content_w,
            main_y + TITLE_ROW_H + RULER_H,
        ),
    );

    // --- Ruler markers ---
    let ruler_y = main_y + TITLE_ROW_H;
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "marker",
        "First Line Indent",
        "kodocs.ruler.indent.first_line",
        Rect::new(80.0, ruler_y + 4.0, 92.0, ruler_y + 20.0),
    );
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "marker",
        "Left Indent",
        "kodocs.ruler.indent.left",
        Rect::new(80.0, ruler_y + 16.0, 92.0, ruler_y + RULER_H),
    );
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "marker",
        "Left Margin",
        "kodocs.ruler.margin.left",
        Rect::new(64.0, ruler_y, 76.0, ruler_y + RULER_H),
    );
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "marker",
        "Right Indent",
        "kodocs.ruler.indent.right",
        Rect::new(
            content_w - 92.0,
            ruler_y + 16.0,
            content_w - 80.0,
            ruler_y + RULER_H,
        ),
    );
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "marker",
        "Right Margin",
        "kodocs.ruler.margin.right",
        Rect::new(
            content_w - 76.0,
            ruler_y,
            content_w - 64.0,
            ruler_y + RULER_H,
        ),
    );

    let workspace_y = main_y + TITLE_ROW_H + RULER_H;
    let tab_bar_h = if state.open_tabs.len() > 1 { 32.0 } else { 0.0 };
    let doc_y = workspace_y + tab_bar_h;
    if tab_bar_h > 0.0 {
        for (i, tab) in state.open_tabs.iter().enumerate() {
            let tab_w = 120.0;
            let tab_x = i as f64 * tab_w;
            push_kodocs_node(
                &mut nodes,
                &mut next_id,
                "tab",
                &tab.title,
                format!("kodocs.tab.{i}"),
                Rect::new(tab_x, workspace_y, tab_x + tab_w, doc_y),
            );
            push_kodocs_node(
                &mut nodes,
                &mut next_id,
                "button",
                format!("Close {}", tab.title),
                format!("kodocs.tab.close.{i}"),
                Rect::new(
                    tab_x + tab_w - 20.0,
                    workspace_y + 6.0,
                    tab_x + tab_w - 4.0,
                    workspace_y + 26.0,
                ),
            );
        }
    }

    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "document",
        "문서",
        "kodocs.document",
        Rect::new(0.0, doc_y, content_w, status_y),
    );

    if state.show_thumbnails && content_w > THUMB_PANEL_W + 480.0 {
        push_kodocs_node(
            &mut nodes,
            &mut next_id,
            "panel",
            "미리보기",
            "kodocs.thumbnails",
            Rect::new(0.0, main_y, THUMB_PANEL_W, status_y),
        );
    }

    if sidebar_open {
        let sidebar_x = content_w;
        if state.show_comments {
            push_kodocs_node(
                &mut nodes,
                &mut next_id,
                "panel",
                "메모",
                "kodocs.comments",
                Rect::new(sidebar_x, main_y, width, status_y),
            );
        } else {
            push_kodocs_node(
                &mut nodes,
                &mut next_id,
                "panel",
                "스타일",
                "kodocs.style_panel",
                Rect::new(sidebar_x, main_y, width, status_y),
            );
        }
    }

    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "status_bar",
        state.status_line(),
        "kodocs.status_bar",
        Rect::new(0.0, status_y, width, height),
    );

    // --- Status bar controls ---
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "button",
        "페이지",
        "kodocs.status_bar.page_indicator",
        Rect::new(
            width - 260.0,
            status_y + 4.0,
            width - 180.0,
            status_y + 24.0,
        ),
    );
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "button",
        "축소",
        "kodocs.status_bar.zoom_out",
        Rect::new(
            width - 160.0,
            status_y + 4.0,
            width - 136.0,
            status_y + 24.0,
        ),
    );
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "label",
        format!("{:.0}%", state.zoom),
        "kodocs.status_bar.zoom_label",
        Rect::new(width - 130.0, status_y + 4.0, width - 80.0, status_y + 24.0),
    );
    push_kodocs_node(
        &mut nodes,
        &mut next_id,
        "button",
        "확대",
        "kodocs.status_bar.zoom_in",
        Rect::new(width - 52.0, status_y + 4.0, width - 28.0, status_y + 24.0),
    );

    // --- Menu dropdown items ---
    if let Some(modal_name) = &state.active_modal {
        let items = chrome::menu_items_for(modal_name);
        let item_h = 26.0;
        let top_pad = 8.0;
        for (idx, item) in items.iter().enumerate() {
            let menu_key = modal_name.to_lowercase();
            let item_key = item.to_lowercase().replace(' ', "_");
            push_kodocs_node(
                &mut nodes,
                &mut next_id,
                "menu_item",
                *item,
                format!("kodocs.menu.{menu_key}.{item_key}"),
                Rect::new(
                    12.0,
                    MENU_BAR_H + top_pad + idx as f64 * item_h,
                    12.0 + 220.0,
                    MENU_BAR_H + top_pad + (idx + 1) as f64 * item_h,
                ),
            );
        }
    }

    push_modal_nodes(state, size, &mut nodes, &mut next_id);

    push_popup_nodes(state, size, &mut nodes, &mut next_id);

    // --- Style panel tabs ---
    if state.show_style_panel {
        let sidebar_x = content_w;
        let tab_h = 32.0;
        let tab_w = (STYLE_PANEL_W / 3.0).floor();
        let tabs = [
            ("스타일", "kodocs.style_panel.tab.style"),
            ("탐색", "kodocs.style_panel.tab.navigate"),
            ("AI", "kodocs.style_panel.tab.ai"),
        ];
        for (i, (label, id)) in tabs.iter().enumerate() {
            let tx = sidebar_x + i as f64 * tab_w;
            push_kodocs_node(
                &mut nodes,
                &mut next_id,
                "tab",
                *label,
                *id,
                Rect::new(tx, main_y, tx + tab_w, main_y + tab_h),
            );
        }
        // Save recovery copy button
        push_kodocs_node(
            &mut nodes,
            &mut next_id,
            "button",
            "스냅샷 저장",
            "kodocs.style_panel.action.save_recovery_copy",
            Rect::new(
                sidebar_x + 10.0,
                status_y - 50.0,
                sidebar_x + STYLE_PANEL_W - 10.0,
                status_y - 22.0,
            ),
        );
    }

    push_dropdown_nodes(state, width, (ff_x, fs_x, ps_x), &mut nodes, &mut next_id);

    nodes
}
