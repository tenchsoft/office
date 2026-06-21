use super::super::state::{
    c_separator, c_status_bg, c_text_dim, c_text_light, c_toolbar_bg_compat, DocsState,
};
use tench_document_core::BlockNode;
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

pub fn paint_status_bar(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect, state: &DocsState) {
    p.fill_rect(rect, c_status_bg());
    p.draw_line(
        Point::new(rect.x0, rect.y0),
        Point::new(rect.x1, rect.y0),
        c_separator(),
        1.0,
    );
    let y = rect.y0 + rect.height() / 2.0;

    // Cursor position: Page X · Line Y · Col Z
    let cursor = state.cursor();
    let doc = state.current_document();
    let mut line_num = 1;
    let mut col_num = 1;
    for (i, block) in doc.content.iter().enumerate() {
        if i == cursor.block_idx {
            col_num = cursor.offset + 1;
            break;
        }
        let block_text = extract_block_text_for_status(block);
        line_num += block_text.lines().count().max(1);
    }
    let cursor_pos_text = format!(
        "Page {} · Line {} · Col {}",
        state.current_page, line_num, col_num
    );
    p.draw_text_cached(
        cache,
        &cursor_pos_text,
        rect.x0 + 12.0,
        y,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );

    p.draw_text_cached(
        cache,
        &format!("{} words", state.word_count),
        rect.x0 + 180.0,
        y,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    p.draw_text_cached(
        cache,
        &format!("{} characters", state.character_count()),
        rect.x0 + 260.0,
        y,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    p.draw_text_cached(
        cache,
        state.status_line(),
        rect.x0 + 380.0,
        y,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    paint_status_button(
        p,
        cache,
        Rect::new(
            rect.x1 - 252.0,
            rect.y0 + 4.0,
            rect.x1 - 174.0,
            rect.y1 - 4.0,
        ),
        &format!("Page {}/{}", state.current_page, state.page_count),
    );
    p.draw_text_cached(
        cache,
        &state.language,
        rect.x1 - 344.0,
        y,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    paint_status_button(
        p,
        cache,
        Rect::new(
            rect.x1 - 160.0,
            rect.y0 + 4.0,
            rect.x1 - 136.0,
            rect.y1 - 4.0,
        ),
        "-",
    );
    p.draw_text_cached(
        cache,
        &format!("Zoom {:.0}%", state.zoom),
        rect.x1 - 96.0,
        y,
        c_text_light(),
        11.0,
        FontWeight::NORMAL,
        true,
        false,
    );
    paint_status_button(
        p,
        cache,
        Rect::new(rect.x1 - 52.0, rect.y0 + 4.0, rect.x1 - 28.0, rect.y1 - 4.0),
        "+",
    );
    if state.track_changes {
        p.draw_text_cached(
            cache,
            "Track changes",
            rect.x1 - 170.0,
            y,
            c_text_light(),
            11.0,
            FontWeight::BOLD,
            false,
            false,
        );
    }
}

/// Extract text from a block for status bar display.
fn extract_block_text_for_status(block: &tench_document_core::BlockNode) -> String {
    use tench_document_core::InlineNode;
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
            let mut out = String::new();
            for node in content {
                match node {
                    InlineNode::Text { text, .. } => out.push_str(text),
                    InlineNode::Link { text, .. } => out.push_str(text),
                    _ => {}
                }
            }
            out
        }
        BlockNode::CodeBlock { code, .. } => code.clone(),
        _ => String::new(),
    }
}

fn paint_status_button(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect, label: &str) {
    p.fill_rounded_rect(rect, c_toolbar_bg_compat(), 4.0);
    p.stroke_rounded_rect(rect, c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        label,
        rect.x0 + rect.width() / 2.0,
        rect.y0 + rect.height() / 2.0,
        c_text_light(),
        11.0,
        FontWeight::NORMAL,
        true,
        false,
    );
}
