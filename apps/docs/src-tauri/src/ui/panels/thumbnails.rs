use super::super::state::{
    c_accent, c_canvas_bg, c_page_bg, c_separator, c_text_dim, c_text_light, DocsState, PAGE_H,
    PAGE_W,
};
use tench_document_core::BlockNode;
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

pub fn paint_thumbnails(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect, state: &DocsState) {
    p.fill_rect(rect, c_canvas_bg());
    p.draw_line(
        Point::new(rect.x1 - 1.0, rect.y0),
        Point::new(rect.x1 - 1.0, rect.y1),
        c_separator(),
        1.0,
    );

    let num_pages = state.layout_cache.num_pages().max(1);
    let thumb_w = rect.width() - 24.0;
    let thumb_h = thumb_w * (PAGE_H / PAGE_W);
    let mut y = rect.y0 + 12.0;
    for idx in 0..num_pages {
        if y + thumb_h + 20.0 > rect.y1 {
            break;
        }
        let x = rect.x0 + 12.0;
        let thumb = Rect::new(x, y, x + thumb_w, y + thumb_h);

        // Highlight current page thumbnail
        let is_current = idx + 1 == state.current_page;
        p.fill_rounded_rect(thumb, c_page_bg(), 2.0);
        if is_current {
            p.stroke_rounded_rect(thumb, c_accent(), 2.0, 2.0);
        } else {
            p.stroke_rounded_rect(thumb, c_separator(), 1.0, 2.0);
        }

        // Draw miniature page content lines
        let line_h = 3.0;
        let pad = 8.0 * (thumb_w / PAGE_W);
        let mut line_y = thumb.y0 + pad;
        let page_map = state.layout_cache.page_map();
        let start_block = page_map.get(idx).map(|e| e.start_block).unwrap_or(0);
        let end_block = page_map
            .get(idx + 1)
            .map(|e| e.start_block)
            .unwrap_or(state.current_document().content.len());

        for block_idx in start_block..end_block {
            if line_y + line_h > thumb.y1 - pad {
                break;
            }
            let doc = state.current_document();
            if let Some(block) = doc.content.get(block_idx) {
                let is_heading = matches!(block, BlockNode::Heading { level, .. } if *level <= 2);
                let line_w = if is_heading {
                    thumb_w * 0.6
                } else {
                    thumb_w - pad * 2.0
                };
                let color = if is_heading {
                    c_text_light()
                } else {
                    c_text_dim()
                };
                p.fill_rect(
                    Rect::new(
                        thumb.x0 + pad,
                        line_y,
                        thumb.x0 + pad + line_w,
                        line_y + line_h,
                    ),
                    color,
                );
            }
            line_y += line_h + 2.0;
        }

        // Page number label
        p.draw_text_cached(
            cache,
            &format!("{}", idx + 1),
            x + thumb_w / 2.0 - 4.0,
            y + thumb_h + 4.0,
            if is_current { c_accent() } else { c_text_dim() },
            10.0,
            FontWeight::NORMAL,
            false,
            false,
        );
        y += thumb_h + 20.0;
    }
}
