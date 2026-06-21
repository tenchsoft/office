//! Shared modal geometry for paint, hit-test, and automation.
//!
//! Every modal that has pointer hit-targets or automation nodes must compute
//! its rectangle here so that paint, events, and automation stay in sync.

use tench_ui::kurbo::Rect;
use tench_ui::prelude::Size;

/// Computed geometry for the Find/Replace modal.
pub struct FindReplaceLayout {
    pub modal: Rect,
    pub query_field: Rect,
    pub close: Rect,
    pub btn_row_y: f64,
    pub btn_h: f64,
    pub find_next: Rect,
    pub find_prev: Rect,
    pub replace: Option<Rect>,
    pub replace_all: Option<Rect>,
    pub case_sensitive: Rect,
    pub regex: Rect,
}

/// Computed geometry for the Go To modal.
pub struct GotoLayout {
    pub modal: Rect,
    pub input_field: Rect,
    pub page_mode: Rect,
    pub line_mode: Rect,
}

/// Computed geometry for the Print Preview modal.
pub struct PrintPreviewLayout {
    pub modal: Rect,
    pub prev_btn: Rect,
    pub next_btn: Rect,
    pub print_btn: Rect,
    pub close: Rect,
    pub page_indicator: Rect,
    pub preview_page: Rect,
}

/// Computed geometry for info-style modals (Word Count, Special Character, etc.).
pub struct InfoModalLayout {
    pub modal: Rect,
    pub close: Rect,
}

// ── Constants ──────────────────────────────────────────────────────────────

const FIND_REPLACE_W: f64 = 400.0;
const FIND_REPLACE_H_FIND: f64 = 160.0;
const FIND_REPLACE_H_REPLACE: f64 = 200.0;
const GOTO_W: f64 = 320.0;
const GOTO_H: f64 = 160.0;
const PRINT_PREVIEW_W: f64 = 600.0;
const PRINT_PREVIEW_H: f64 = 480.0;
const INFO_MODAL_W: f64 = 420.0;
const INFO_MODAL_H: f64 = 190.0;

// ── Computation functions ──────────────────────────────────────────────────

pub fn compute_find_replace(size: Size, show_replace: bool) -> FindReplaceLayout {
    let modal_h = if show_replace {
        FIND_REPLACE_H_REPLACE
    } else {
        FIND_REPLACE_H_FIND
    };
    let modal_x = size.width / 2.0 - FIND_REPLACE_W / 2.0;
    let modal_y = size.height / 2.0 - modal_h / 2.0;
    let m = Rect::new(
        modal_x,
        modal_y,
        modal_x + FIND_REPLACE_W,
        modal_y + modal_h,
    );

    let query_field = Rect::new(m.x0 + 60.0, m.y0 + 38.0, m.x1 - 16.0, m.y0 + 60.0);
    let close = Rect::new(m.x1 - 28.0, m.y0 + 6.0, m.x1 - 10.0, m.y0 + 24.0);

    let btn_row_y = if show_replace {
        m.y0 + 104.0
    } else {
        m.y0 + 72.0
    };
    let btn_h = 28.0;
    let btn_gap = 8.0;
    let mut btn_x = m.x0 + 16.0;

    let find_next = Rect::new(btn_x, btn_row_y, btn_x + 72.0, btn_row_y + btn_h);
    btn_x += 72.0 + btn_gap;
    let find_prev = Rect::new(btn_x, btn_row_y, btn_x + 72.0, btn_row_y + btn_h);
    btn_x += 72.0 + btn_gap;

    let (replace, replace_all) = if show_replace {
        let r = Rect::new(btn_x, btn_row_y, btn_x + 64.0, btn_row_y + btn_h);
        btn_x += 64.0 + btn_gap;
        let ra = Rect::new(btn_x, btn_row_y, btn_x + 80.0, btn_row_y + btn_h);
        (Some(r), Some(ra))
    } else {
        (None, None)
    };

    let case_sensitive = Rect::new(m.x1 - 80.0, btn_row_y, m.x1 - 56.0, btn_row_y + btn_h);
    let regex = Rect::new(m.x1 - 52.0, btn_row_y, m.x1 - 16.0, btn_row_y + btn_h);

    FindReplaceLayout {
        modal: m,
        query_field,
        close,
        btn_row_y,
        btn_h,
        find_next,
        find_prev,
        replace,
        replace_all,
        case_sensitive,
        regex,
    }
}

pub fn compute_goto(size: Size) -> GotoLayout {
    let modal_x = size.width / 2.0 - GOTO_W / 2.0;
    let modal_y = size.height / 2.0 - GOTO_H / 2.0;
    let m = Rect::new(modal_x, modal_y, modal_x + GOTO_W, modal_y + GOTO_H);

    let input_field = Rect::new(m.x0 + 16.0, m.y0 + 70.0, m.x1 - 16.0, m.y0 + 98.0);
    let page_mode = Rect::new(m.x0 + 16.0, m.y0 + 38.0, m.x0 + 80.0, m.y0 + 58.0);
    let line_mode = Rect::new(m.x0 + 86.0, m.y0 + 38.0, m.x0 + 150.0, m.y0 + 58.0);

    GotoLayout {
        modal: m,
        input_field,
        page_mode,
        line_mode,
    }
}

pub fn compute_print_preview(size: Size) -> PrintPreviewLayout {
    let modal_x = size.width / 2.0 - PRINT_PREVIEW_W / 2.0;
    let modal_y = size.height / 2.0 - PRINT_PREVIEW_H / 2.0;
    let m = Rect::new(
        modal_x,
        modal_y,
        modal_x + PRINT_PREVIEW_W,
        modal_y + PRINT_PREVIEW_H,
    );

    let btn_y = modal_y + PRINT_PREVIEW_H - 40.0;
    let btn_h = 28.0;

    let prev_btn = Rect::new(modal_x + 16.0, btn_y, modal_x + 72.0, btn_y + btn_h);
    let next_btn = Rect::new(modal_x + 80.0, btn_y, modal_x + 136.0, btn_y + btn_h);
    let print_btn = Rect::new(
        modal_x + PRINT_PREVIEW_W - 100.0,
        btn_y,
        modal_x + PRINT_PREVIEW_W - 16.0,
        btn_y + btn_h,
    );
    let close = Rect::new(
        modal_x + PRINT_PREVIEW_W - 30.0,
        modal_y + 6.0,
        modal_x + PRINT_PREVIEW_W - 10.0,
        modal_y + 24.0,
    );
    let page_indicator = Rect::new(
        modal_x + PRINT_PREVIEW_W / 2.0 - 60.0,
        modal_y + PRINT_PREVIEW_H - 30.0,
        modal_x + PRINT_PREVIEW_W / 2.0 + 60.0,
        modal_y + PRINT_PREVIEW_H - 14.0,
    );
    let preview_page = Rect::new(
        modal_x + 24.0,
        modal_y + 12.0,
        modal_x + PRINT_PREVIEW_W - 24.0,
        btn_y - 8.0,
    );

    PrintPreviewLayout {
        modal: m,
        prev_btn,
        next_btn,
        print_btn,
        close,
        page_indicator,
        preview_page,
    }
}

pub fn compute_info_modal(size: Size) -> InfoModalLayout {
    let modal_x = size.width / 2.0 - INFO_MODAL_W / 2.0;
    let modal_y = size.height / 2.0 - INFO_MODAL_H / 2.0;
    let m = Rect::new(
        modal_x,
        modal_y,
        modal_x + INFO_MODAL_W,
        modal_y + INFO_MODAL_H,
    );
    let close = Rect::new(m.x1 - 92.0, m.y1 - 42.0, m.x1 - 16.0, m.y1 - 14.0);

    InfoModalLayout { modal: m, close }
}
