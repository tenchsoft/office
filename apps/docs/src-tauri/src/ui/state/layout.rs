use tench_document_core::TenchDocument;

use super::super::document_text::extract_block_text;
use super::*;

impl DocsState {
    /// Ensure the layout cache is up to date for the current document and zoom.
    /// Call this before `paint_document_area` so the cache is warm.
    pub fn ensure_layout_cache(&mut self) {
        let zoom = self.zoom;
        let doc = &self.document;
        if self.layout_cache.is_valid(doc, zoom) {
            return;
        }

        let setup = &doc.page_setup;
        let scale = zoom / 100.0;
        let (page_w_raw, page_h_raw) = setup.page_size_px();
        let page_w = page_w_raw * scale;
        let page_h = page_h_raw * scale;
        let mm_to_px = 96.0 / 25.4;
        let margin_left = setup.margins.left as f64 * mm_to_px * scale;
        let margin_right = setup.margins.right as f64 * mm_to_px * scale;
        let margin_top = setup.margins.top as f64 * mm_to_px * scale;
        let margin_bottom = setup.margins.bottom as f64 * mm_to_px * scale;
        let content_w = page_w - margin_left - margin_right;
        let header_h = HEADER_H * scale;
        let footer_h = FOOTER_H * scale;
        let page_content_h = page_h - margin_top - margin_bottom - header_h - footer_h;

        let total_content_h = estimate_total_content_height_static(doc, content_w, scale);
        let num_pages = if total_content_h <= 0.0 {
            1
        } else {
            ((total_content_h / page_content_h).ceil() as usize).max(1)
        };

        let page_map = compute_page_map_static(doc, content_w, page_content_h, scale);
        let content_hash = DocumentLayoutCache::hash_document(doc);

        self.layout_cache
            .update(content_hash, zoom, page_map, total_content_h, num_pages);

        // Synchronize page_count and current_page with layout cache
        self.page_count = self.layout_cache.num_pages().max(1);
        self.current_page = self.current_page.clamp(1, self.page_count);
    }

    /// Set zoom level, clamped to valid range, and invalidate layout cache.
    pub fn set_zoom(&mut self, zoom: f64) {
        let clamped = zoom.clamp(50.0, 200.0);
        if (self.zoom - clamped).abs() < 0.001 {
            return;
        }
        self.zoom = clamped;
        self.layout_cache.invalidate();
        self.ensure_layout_cache();
    }
}

/// Estimate total content height for layout caching.
fn estimate_total_content_height_static(doc: &TenchDocument, content_w: f64, scale: f64) -> f64 {
    let line_h = 20.0 * scale;
    let block_spacing = 12.0 * scale;
    let chars_per_line = if content_w > 0.0 {
        ((content_w / (7.0 * scale.max(0.6))) as usize).max(24)
    } else {
        80
    };

    let mut total = 0.0;
    for block in &doc.content {
        total += estimate_block_height_static(block, chars_per_line, line_h, block_spacing, scale);
    }
    total
}

/// Compute page map for layout caching.
fn compute_page_map_static(
    doc: &TenchDocument,
    content_w: f64,
    page_content_h: f64,
    scale: f64,
) -> Vec<PageMapEntry> {
    let line_h = 20.0 * scale;
    let block_spacing = 12.0 * scale;
    let chars_per_line = if content_w > 0.0 {
        ((content_w / (7.0 * scale.max(0.6))) as usize).max(24)
    } else {
        80
    };

    let mut entries = Vec::new();
    let mut accumulated_h: f64 = 0.0;
    entries.push(PageMapEntry { start_block: 0 });

    for (i, block) in doc.content.iter().enumerate() {
        let block_h =
            estimate_block_height_static(block, chars_per_line, line_h, block_spacing, scale);
        if accumulated_h + block_h > page_content_h && accumulated_h > 0.0 {
            entries.push(PageMapEntry { start_block: i });
            accumulated_h = block_h;
        } else {
            accumulated_h += block_h;
        }
    }

    entries
}

/// Estimate block height for layout caching.
fn estimate_block_height_static(
    block: &tench_document_core::BlockNode,
    chars_per_line: usize,
    line_h: f64,
    block_spacing: f64,
    scale: f64,
) -> f64 {
    use tench_document_core::BlockNode;
    match block {
        BlockNode::Paragraph { content: _, .. } => {
            let text = extract_block_text(block);
            let lines = if text.is_empty() {
                1
            } else {
                text.chars().count().div_ceil(chars_per_line)
            };
            lines as f64 * line_h + block_spacing
        }
        BlockNode::Heading {
            content: _, level, ..
        } => {
            let text = extract_block_text(block);
            let heading_line_h = match level {
                1 => 36.0 * scale,
                2 => 32.0 * scale,
                3 => 28.0 * scale,
                _ => 20.0 * scale,
            };
            let lines = if text.is_empty() {
                1
            } else {
                text.chars().count().div_ceil(chars_per_line)
            };
            lines as f64 * heading_line_h + 8.0 * scale + block_spacing
        }
        BlockNode::CodeBlock { code, .. } => {
            let lines = code.lines().count().max(1);
            lines as f64 * 18.0 * scale + 24.0 * scale + block_spacing
        }
        BlockNode::HorizontalRule => 24.0 * scale + block_spacing,
        BlockNode::PageBreak => 24.0 * scale + block_spacing,
        BlockNode::Image { height, .. } => {
            height.unwrap_or(300.0) as f64 * scale + 24.0 * scale + block_spacing
        }
        BlockNode::BulletList { items } | BlockNode::OrderedList { items, .. } => {
            let total: f64 = items
                .iter()
                .map(|i| {
                    let text = extract_block_text(&BlockNode::Paragraph {
                        content: i.content.clone(),
                        attrs: Default::default(),
                    });
                    let lines = if text.is_empty() {
                        1
                    } else {
                        text.chars().count().div_ceil(chars_per_line)
                    };
                    lines as f64 * line_h + 4.0 * scale
                })
                .sum();
            total + 8.0 * scale + block_spacing
        }
        BlockNode::TaskList { items } => {
            let total: f64 = items
                .iter()
                .map(|i| {
                    let text = extract_block_text(&BlockNode::Paragraph {
                        content: i.content.clone(),
                        attrs: Default::default(),
                    });
                    let lines = if text.is_empty() {
                        1
                    } else {
                        text.chars().count().div_ceil(chars_per_line)
                    };
                    lines as f64 * line_h + 4.0 * scale
                })
                .sum();
            total + 8.0 * scale + block_spacing
        }
        BlockNode::BlockQuote { content } => {
            let inner: f64 = content
                .iter()
                .map(|b| {
                    estimate_block_height_static(b, chars_per_line, line_h, block_spacing, scale)
                })
                .sum();
            inner + 8.0 * scale + block_spacing
        }
        BlockNode::Table { rows } => {
            let row_h = 24.0 * scale;
            rows.len().max(1) as f64 * row_h + 12.0 * scale + block_spacing
        }
        BlockNode::Footnote { .. } => line_h * 2.0 + block_spacing,
    }
}
