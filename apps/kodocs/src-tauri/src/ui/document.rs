//! TDM-based rich text document renderer.

use super::state::{
    c_accent, c_page_bg, c_separator, c_text_dark, c_text_dim, c_text_light, KodocsState, FOOTER_H,
    HEADER_H, PAGE_GAP, PAGE_MARGIN_X, PAGE_MARGIN_Y, PAGE_PAD_X, PAGE_W,
};
use tench_document_core::{
    Alignment, BlockNode, ChangeType, HeadersFooters, ImageSource, InlineNode, TableRow,
};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

mod paint;
mod render;

pub use paint::paint_document_area;
