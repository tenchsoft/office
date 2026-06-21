use super::state::{LicenseModalState, MenuAction, MenuItem, SheetsState};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

pub(crate) const MENU_NAMES: &[&str] = &[
    "File", "Edit", "View", "Insert", "Format", "Data", "Tools", "License", "Help",
];
pub(crate) const MENU_BAR_H: f64 = 28.0;
pub(crate) const MENU_ITEM_H: f64 = 24.0;
pub(crate) const MENU_PAD_X: f64 = 12.0;
pub(crate) const MENU_PAD_Y: f64 = 4.0;
pub(crate) const DROPDOWN_W: f64 = 220.0;
const SUBMENU_W: f64 = 160.0;
const SHORTCUT_PAD: f64 = 24.0;

pub fn paint_formula_bar(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    width: f64,
    top: f64,
    height: f64,
) {
    p.fill_rect(Rect::new(0.0, top, width, top + height), theme.surface);

    let ref_box = Rect::new(8.0, top + 6.0, 62.0, top + height - 6.0);
    p.fill_rounded_rect(ref_box, theme.background, 3.0);
    p.draw_text(
        &state.active_cell_ref(),
        18.0,
        top + 24.0,
        theme.primary,
        theme.font_size,
        FontWeight::BOLD,
        false,
    );

    let formula_input = Rect::new(68.0, top + 6.0, width - 48.0, top + height - 6.0);
    p.fill_rounded_rect(formula_input, theme.background, 3.0);
    p.draw_text(
        &state.formula_draft,
        78.0,
        top + 24.0,
        theme.on_surface,
        theme.font_size,
        FontWeight::NORMAL,
        false,
    );

    let toggle = Rect::new(width - 36.0, top + 6.0, width - 8.0, top + height - 6.0);
    p.fill_rounded_rect(toggle, theme.background, 3.0);
    p.draw_text(
        "CH",
        width - 30.0,
        top + 24.0,
        theme.secondary,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
}

pub fn paint_menu_bar(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    width: f64,
    top: f64,
) {
    p.fill_rect(Rect::new(0.0, top, width, top + MENU_BAR_H), theme.surface);
    let mut x = MENU_PAD_X;
    for (idx, name) in MENU_NAMES.iter().enumerate() {
        let is_open = state.menu_state.open_menu == Some(idx);
        let item_w = 54.0;
        if is_open {
            p.fill_rounded_rect(
                Rect::new(x - 4.0, top + 2.0, x + item_w, top + MENU_BAR_H - 2.0),
                theme.background,
                3.0,
            );
        }
        p.draw_text(
            name,
            x,
            top + 18.0,
            if is_open {
                theme.primary
            } else {
                theme.on_surface
            },
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        x += item_w;
    }
    // Notification label — only painted when the license is not active.
    // Two messages cycle every 5 seconds when an update is also pending.
    // Click handling lives in events/pointer.rs (looks up the same rect via
    // notification_label_rect).
    if let Some(msg) = notification_label_message(state) {
        let label_rect = notification_label_rect(top);
        p.draw_text(
            msg,
            label_rect.x0,
            top + 18.0,
            theme.primary,
            theme.font_size_small,
            FontWeight::BOLD,
            false,
        );
    }
    p.draw_text(
        &state.workbook_name,
        width - WINDOW_CONTROLS_W - 180.0,
        top + 18.0,
        theme.secondary,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );
}

/// Paint the open dropdown menu (if any). Called after the grid so it overlays.
pub fn paint_dropdown_menu(state: &SheetsState, p: &mut Painter<'_>, theme: &Theme, top: f64) {
    let Some(menu_idx) = state.menu_state.open_menu else {
        return;
    };
    let items = match state.menus.get(menu_idx) {
        Some(i) => i,
        None => return,
    };

    let menu_x = MENU_PAD_X + menu_idx as f64 * 54.0;
    let menu_y = top + MENU_BAR_H;

    // Count non-separator items for height, plus separators
    let total_h = MENU_PAD_Y * 2.0
        + items
            .iter()
            .map(|item| {
                if item.is_separator() {
                    9.0
                } else {
                    MENU_ITEM_H
                }
            })
            .sum::<f64>();

    let dropdown_rect = Rect::new(menu_x, menu_y, menu_x + DROPDOWN_W, menu_y + total_h);

    // Shadow
    p.fill_rounded_rect_with_shadow(
        dropdown_rect,
        4.0,
        theme.surface,
        (3.0, 3.0),
        8.0,
        Color::rgb8(0x00, 0x00, 0x00),
    );

    let mut item_y = menu_y + MENU_PAD_Y;
    for (item_idx, item) in items.iter().enumerate() {
        if item.is_separator() {
            let sep_y = item_y + 4.0;
            p.draw_line(
                Point::new(menu_x + 8.0, sep_y),
                Point::new(menu_x + DROPDOWN_W - 8.0, sep_y),
                theme.border,
                0.5,
            );
            item_y += 9.0;
            continue;
        }

        let is_hovered = state.menu_state.hovered_submenu == Some((menu_idx, item_idx));
        if is_hovered {
            p.fill_rounded_rect(
                Rect::new(
                    menu_x + 2.0,
                    item_y,
                    menu_x + DROPDOWN_W - 2.0,
                    item_y + MENU_ITEM_H,
                ),
                theme.primary,
                3.0,
            );
        }

        let text_color = if !item.enabled {
            theme.disabled
        } else if is_hovered {
            Color::WHITE
        } else {
            theme.on_surface
        };

        p.draw_text(
            &item.label,
            menu_x + 12.0,
            item_y + 16.0,
            text_color,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );

        // Shortcut text (right-aligned)
        if !item.shortcut.is_empty() {
            p.draw_text(
                &item.shortcut,
                menu_x + DROPDOWN_W - SHORTCUT_PAD - 60.0,
                item_y + 16.0,
                if is_hovered {
                    Color::rgb8(0xCC, 0xCC, 0xCC)
                } else {
                    theme.disabled
                },
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }

        // Submenu arrow
        if !item.submenu.is_empty() {
            p.draw_text(
                "\u{25B6}",
                menu_x + DROPDOWN_W - 16.0,
                item_y + 16.0,
                text_color,
                theme.font_size_small * 0.8,
                FontWeight::NORMAL,
                false,
            );
        }

        // Paint submenu if hovered
        if is_hovered && !item.submenu.is_empty() {
            paint_submenu(p, theme, menu_x + DROPDOWN_W, item_y, &item.submenu);
        }

        item_y += MENU_ITEM_H;
    }
}

fn paint_submenu(p: &mut Painter<'_>, theme: &Theme, x: f64, y: f64, items: &[MenuItem]) {
    let total_h = MENU_PAD_Y * 2.0
        + items
            .iter()
            .map(|item| {
                if item.is_separator() {
                    9.0
                } else {
                    MENU_ITEM_H
                }
            })
            .sum::<f64>();

    let rect = Rect::new(x, y, x + SUBMENU_W, y + total_h);

    p.fill_rounded_rect_with_shadow(
        rect,
        4.0,
        theme.surface,
        (3.0, 3.0),
        8.0,
        Color::rgb8(0x00, 0x00, 0x00),
    );

    let mut item_y = y + MENU_PAD_Y;
    for item in items {
        if item.is_separator() {
            let sep_y = item_y + 4.0;
            p.draw_line(
                Point::new(x + 8.0, sep_y),
                Point::new(x + SUBMENU_W - 8.0, sep_y),
                theme.border,
                0.5,
            );
            item_y += 9.0;
            continue;
        }

        let text_color = if item.enabled {
            theme.on_surface
        } else {
            theme.disabled
        };

        p.draw_text(
            &item.label,
            x + 12.0,
            item_y + 16.0,
            text_color,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        item_y += MENU_ITEM_H;
    }
}

/// Hit-test the menu bar: returns Some(menu_idx) if a menu name was clicked.
pub fn hit_menu_bar(x: f64, _y: f64) -> Option<usize> {
    let mut mx = MENU_PAD_X;
    for idx in 0..MENU_NAMES.len() {
        let item_w = 54.0;
        if x >= mx - 4.0 && x < mx + item_w {
            return Some(idx);
        }
        mx += item_w;
    }
    None
}

/// Hit-test the dropdown menu: returns Some(action) if an item was clicked.
pub fn hit_dropdown(
    state: &SheetsState,
    click_x: f64,
    click_y: f64,
    top: f64,
) -> Option<MenuAction> {
    let menu_idx = state.menu_state.open_menu?;
    let items = state.menus.get(menu_idx)?;
    let menu_x = MENU_PAD_X + menu_idx as f64 * 54.0;
    let menu_y = top + MENU_BAR_H;

    if click_x < menu_x || click_x > menu_x + DROPDOWN_W {
        return None;
    }

    let mut item_y = menu_y + MENU_PAD_Y;
    for item in items {
        if item.is_separator() {
            item_y += 9.0;
            continue;
        }
        if click_y >= item_y && click_y < item_y + MENU_ITEM_H {
            if item.enabled && item.submenu.is_empty() {
                return Some(item.action.clone());
            }
            return None;
        }
        item_y += MENU_ITEM_H;
    }
    None
}

pub fn dropdown_contains(state: &SheetsState, x: f64, y: f64, top: f64) -> bool {
    let Some(menu_idx) = state.menu_state.open_menu else {
        return false;
    };
    let Some(items) = state.menus.get(menu_idx) else {
        return false;
    };
    let menu_x = MENU_PAD_X + menu_idx as f64 * 54.0;
    let menu_y = top + MENU_BAR_H;
    let total_h = MENU_PAD_Y * 2.0
        + items
            .iter()
            .map(|item| {
                if item.is_separator() {
                    9.0
                } else {
                    MENU_ITEM_H
                }
            })
            .sum::<f64>();

    x >= menu_x && x <= menu_x + DROPDOWN_W && y >= menu_y && y <= menu_y + total_h
}

/// Determine which dropdown item is hovered.
pub fn hover_dropdown_item(
    state: &SheetsState,
    mx: f64,
    my: f64,
    top: f64,
) -> Option<(usize, usize)> {
    let menu_idx = state.menu_state.open_menu?;
    let items = state.menus.get(menu_idx)?;
    let menu_x = MENU_PAD_X + menu_idx as f64 * 54.0;
    let menu_y = top + MENU_BAR_H;

    if mx < menu_x || mx > menu_x + DROPDOWN_W {
        return None;
    }

    let mut item_y = menu_y + MENU_PAD_Y;
    for (item_idx, item) in items.iter().enumerate() {
        if item.is_separator() {
            item_y += 9.0;
            continue;
        }
        if my >= item_y && my < item_y + MENU_ITEM_H {
            return Some((menu_idx, item_idx));
        }
        item_y += MENU_ITEM_H;
    }
    None
}

/// Returns the message to display in the notification label, or None if the
/// label should be hidden (license is active).
///
/// Behavior matches the spec:
/// - license unauthenticated + no update available: fixed "$1/month" message
/// - license unauthenticated + update available: 2-message cycle, 5s each
/// - license authenticated: hidden
pub(crate) fn notification_label_message(state: &SheetsState) -> Option<&'static str> {
    if state.license_active {
        return None;
    }
    if state.update_available {
        // Cycle every 5 seconds.
        let cycle = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
            / 5)
            % 2;
        if cycle == 0 {
            Some("신규 업데이트 있음")
        } else {
            Some("월 $1 로 라이선스 활성화 가능")
        }
    } else {
        Some("월 $1 로 라이선스 활성화 가능")
    }
}

/// Geometry of the notification label inside the menu bar. `top` is the
/// menu bar's y offset (DOC_TAB_H). Used by both the painter and the click
/// handler so they stay in sync.
pub(crate) fn notification_label_rect(top: f64) -> Rect {
    // Park the label just to the right of the last menu entry. Menus start
    // at MENU_PAD_X and each is 54px wide.
    let label_x = MENU_PAD_X + MENU_NAMES.len() as f64 * 54.0 + 8.0;
    Rect::new(label_x, top, label_x + 220.0, top + MENU_BAR_H)
}

/// Geometry of the License activation modal. Centralized so paint, hit-test,
/// and automation stay in sync.
pub(crate) fn license_modal_rect(size: Size) -> Rect {
    let modal_w = 420.0;
    let modal_h = 220.0;
    Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    )
}

pub(crate) fn license_modal_close_rect(modal: Rect) -> Rect {
    Rect::new(
        modal.x1 - 86.0,
        modal.y1 - 44.0,
        modal.x1 - 16.0,
        modal.y1 - 16.0,
    )
}

pub(crate) fn license_modal_activate_rect(modal: Rect) -> Rect {
    Rect::new(
        modal.x1 - 170.0,
        modal.y1 - 44.0,
        modal.x1 - 94.0,
        modal.y1 - 16.0,
    )
}

pub(crate) fn license_modal_input_rect(modal: Rect) -> Rect {
    Rect::new(
        modal.x0 + 16.0,
        modal.y0 + 100.0,
        modal.x1 - 16.0,
        modal.y0 + 130.0,
    )
}

pub(crate) fn paint_license_modal(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    license_state: &LicenseModalState,
    license_active: bool,
) {
    let modal = license_modal_rect(size);

    // Semi-transparent backdrop
    p.fill_rect(
        Rect::new(0.0, 0.0, size.width, size.height),
        Color::rgba8(0, 0, 0, 100),
    );

    p.fill_rounded_rect(modal, theme.surface, 6.0);
    p.stroke_rounded_rect(modal, theme.border, 1.0, 6.0);

    // Title
    p.draw_text(
        "License",
        modal.x0 + 16.0,
        modal.y0 + 28.0,
        theme.on_surface,
        14.0,
        FontWeight::BOLD,
        false,
    );

    // Status line
    let (status_label, status_color) = if license_active {
        ("Status: Active", theme.primary)
    } else if license_state.busy {
        ("Status: Activating...", theme.secondary)
    } else {
        ("Status: Not activated", theme.secondary)
    };
    p.draw_text(
        status_label,
        modal.x0 + 16.0,
        modal.y0 + 56.0,
        status_color,
        12.0,
        FontWeight::NORMAL,
        false,
    );

    // License key label
    p.draw_text(
        "License key:",
        modal.x0 + 16.0,
        modal.y0 + 88.0,
        theme.secondary,
        11.0,
        FontWeight::NORMAL,
        false,
    );

    // License key input field
    let input_rect = license_modal_input_rect(modal);
    p.fill_rounded_rect(input_rect, theme.background, 4.0);
    p.stroke_rounded_rect(input_rect, theme.border, 1.0, 4.0);
    let display = if license_state.license_key_input.is_empty() {
        "TENCH-XXXX-XXXX-XXXX-XXXX".to_string()
    } else {
        license_state.license_key_input.clone()
    };
    let text_color = if license_state.license_key_input.is_empty() {
        theme.secondary
    } else {
        theme.on_surface
    };
    p.draw_text(
        &display,
        input_rect.x0 + 8.0,
        input_rect.y0 + 18.0,
        text_color,
        11.0,
        FontWeight::NORMAL,
        false,
    );

    // Optional status message (error or success)
    if !license_state.status_message.is_empty() {
        p.draw_text(
            &license_state.status_message,
            modal.x0 + 16.0,
            modal.y0 + 152.0,
            theme.primary,
            11.0,
            FontWeight::NORMAL,
            false,
        );
    }

    // Activate button
    let activate_rect = license_modal_activate_rect(modal);
    p.fill_rounded_rect(activate_rect, theme.primary, 4.0);
    p.draw_text(
        "Activate",
        activate_rect.x0 + 22.0,
        activate_rect.y0 + 18.0,
        Color::WHITE,
        11.0,
        FontWeight::BOLD,
        false,
    );

    // Close button
    let close_rect = license_modal_close_rect(modal);
    p.stroke_rounded_rect(close_rect, theme.border, 1.0, 4.0);
    p.draw_text(
        "Close",
        close_rect.x0 + 22.0,
        close_rect.y0 + 18.0,
        theme.on_surface,
        11.0,
        FontWeight::NORMAL,
        false,
    );
}
