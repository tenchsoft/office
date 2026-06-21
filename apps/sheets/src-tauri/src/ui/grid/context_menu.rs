use crate::ui::state::{ContextMenuTarget, SheetsState};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

const CTX_ITEM_H: f64 = 24.0;
const CTX_PAD_Y: f64 = 4.0;
const CTX_W: f64 = 200.0;

/// Paint the context menu overlay.
pub fn paint_context_menu(state: &SheetsState, p: &mut Painter<'_>, theme: &Theme) {
    let Some(ref ctx) = state.context_menu else {
        return;
    };

    let items = context_menu_items(&ctx.target);
    let total_h = CTX_PAD_Y * 2.0
        + items
            .iter()
            .map(|item| if item.is_empty() { 9.0 } else { CTX_ITEM_H })
            .sum::<f64>();

    let rect = Rect::new(ctx.x, ctx.y, ctx.x + CTX_W, ctx.y + total_h);

    // Shadow
    p.fill_rounded_rect_with_shadow(
        rect,
        4.0,
        theme.surface,
        (3.0, 3.0),
        8.0,
        Color::rgb8(0x00, 0x00, 0x00),
    );

    let mut item_y = ctx.y + CTX_PAD_Y;
    for item in &items {
        if item.is_empty() {
            let sep_y = item_y + 4.0;
            p.draw_line(
                Point::new(ctx.x + 8.0, sep_y),
                Point::new(ctx.x + CTX_W - 8.0, sep_y),
                theme.border,
                0.5,
            );
            item_y += 9.0;
            continue;
        }

        p.draw_text(
            item,
            ctx.x + 12.0,
            item_y + 16.0,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        item_y += CTX_ITEM_H;
    }
}

/// Build context menu items based on the target.
fn context_menu_items(target: &ContextMenuTarget) -> Vec<&'static str> {
    match target {
        ContextMenuTarget::Cell { .. } => vec![
            "Cut",
            "Copy",
            "Paste",
            "",
            "Format Cells",
            "",
            "Insert Row Above",
            "Insert Row Below",
            "Insert Column Left",
            "Insert Column Right",
            "",
            "Delete Row",
            "Delete Column",
            "",
            "Sort Ascending",
            "Sort Descending",
            "",
            "Set Print Area",
        ],
        ContextMenuTarget::RowHeader { .. } => vec![
            "Insert Row Above",
            "Insert Row Below",
            "",
            "Delete Row",
            "Hide Row",
            "Row Height",
        ],
        ContextMenuTarget::ColHeader { .. } => vec![
            "Insert Column Left",
            "Insert Column Right",
            "",
            "Delete Column",
            "Hide Column",
            "Column Width",
        ],
        ContextMenuTarget::SheetTab { .. } => {
            vec!["Rename", "Duplicate", "Move", "Delete", "Tab Color"]
        }
    }
}

/// Hit-test the context menu: returns the selected item index (non-separator).
pub fn hit_context_menu(state: &SheetsState, click_x: f64, click_y: f64) -> Option<usize> {
    let ctx = state.context_menu.as_ref()?;
    let items = context_menu_items(&ctx.target);
    if click_x < ctx.x || click_x > ctx.x + CTX_W {
        return None;
    }

    let mut item_y = ctx.y + CTX_PAD_Y;
    let mut real_idx = 0;
    for item in &items {
        if item.is_empty() {
            item_y += 9.0;
            continue;
        }
        if click_y >= item_y && click_y < item_y + CTX_ITEM_H {
            return Some(real_idx);
        }
        item_y += CTX_ITEM_H;
        real_idx += 1;
    }
    None
}

/// Get the label of a context menu item by real index.
pub fn context_menu_action(target: &ContextMenuTarget, idx: usize) -> &'static str {
    let items = context_menu_items(target);
    let mut real_idx = 0;
    for item in &items {
        if item.is_empty() {
            continue;
        }
        if real_idx == idx {
            return item;
        }
        real_idx += 1;
    }
    ""
}
