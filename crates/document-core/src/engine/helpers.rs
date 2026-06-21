use super::*;

// ---------------------------------------------------------------------------
// Free helper functions
// ---------------------------------------------------------------------------

pub(super) fn block_text_of(content: &[BlockNode], idx: usize) -> String {
    content.get(idx).map(extract_block_text).unwrap_or_default()
}

fn extract_block_text(block: &BlockNode) -> String {
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
            extract_inline_text(content)
        }
        BlockNode::CodeBlock { code, .. } => code.clone(),
        BlockNode::BlockQuote { content } => content
            .iter()
            .map(extract_block_text)
            .collect::<Vec<_>>()
            .join("\n"),
        BlockNode::BulletList { items } | BlockNode::OrderedList { items, .. } => items
            .iter()
            .map(|i| extract_inline_text(&i.content))
            .collect::<Vec<_>>()
            .join("\n"),
        BlockNode::TaskList { items } => items
            .iter()
            .map(|i| extract_inline_text(&i.content))
            .collect::<Vec<_>>()
            .join("\n"),
        BlockNode::Table { rows } => rows
            .iter()
            .flat_map(|r| r.cells.iter())
            .map(|c| {
                c.content
                    .iter()
                    .map(extract_block_text)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\t"),
        BlockNode::HorizontalRule | BlockNode::PageBreak => String::new(),
        BlockNode::Image { alt, .. } => alt.clone().unwrap_or_default(),
        BlockNode::Footnote { content, .. } => extract_inline_text(content),
    }
}

fn extract_inline_text(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            InlineNode::Text { text, .. } => out.push_str(text),
            InlineNode::Link { text, .. } => out.push_str(text),
            InlineNode::InlineImage { alt, .. } => {
                if let Some(a) = alt {
                    out.push_str(a);
                }
            }
            InlineNode::HardBreak => out.push('\n'),
        }
    }
    out
}

pub(super) fn set_block_alignment(block: &mut BlockNode, alignment: Alignment) {
    match block {
        BlockNode::Paragraph { attrs, .. } | BlockNode::Heading { attrs, .. } => {
            attrs.alignment = alignment;
        }
        _ => {}
    }
}

pub(super) fn adjust_block_indent(block: &mut BlockNode, delta: f32) {
    match block {
        BlockNode::Paragraph { attrs, .. } | BlockNode::Heading { attrs, .. } => {
            attrs.indent_left = (attrs.indent_left + delta).max(0.0);
        }
        _ => {}
    }
}

pub(super) fn set_block_indent_left(block: &mut BlockNode, indent: f32) {
    match block {
        BlockNode::Paragraph { attrs, .. } | BlockNode::Heading { attrs, .. } => {
            attrs.indent_left = indent.max(0.0);
        }
        _ => {}
    }
}

pub(super) fn set_block_indent_right(block: &mut BlockNode, indent: f32) {
    match block {
        BlockNode::Paragraph { attrs, .. } | BlockNode::Heading { attrs, .. } => {
            attrs.indent_right = indent.max(0.0);
        }
        _ => {}
    }
}

pub(super) fn set_block_indent_first_line(block: &mut BlockNode, indent: f32) {
    match block {
        BlockNode::Paragraph { attrs, .. } | BlockNode::Heading { attrs, .. } => {
            attrs.indent_first_line = indent;
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Helpers for clipboard
// ---------------------------------------------------------------------------

/// HTML-escape plain text.
pub(super) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Trim a block's text content to only the selected range.
pub(super) fn trim_block(block: &mut BlockNode, start: usize, end: usize) {
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
            let mut offset = 0;
            let mut new_content = Vec::new();
            for node in content.drain(..) {
                match node {
                    InlineNode::Text { text, marks } => {
                        let node_start = offset;
                        let node_end = offset + text.len();
                        // Calculate overlap
                        let overlap_start = node_start.max(start);
                        let overlap_end = node_end.min(end);
                        if overlap_start < overlap_end {
                            let text_start = overlap_start - node_start;
                            let text_end = overlap_end - node_start;
                            if let Some(slice) = text.get(text_start..text_end) {
                                new_content.push(InlineNode::Text {
                                    text: slice.to_string(),
                                    marks,
                                });
                            }
                        }
                        offset = node_end;
                    }
                    InlineNode::InlineImage { .. } | InlineNode::HardBreak => {
                        if offset >= start && offset < end {
                            new_content.push(node);
                        }
                        offset += 1;
                    }
                    InlineNode::Link { text, .. } => {
                        let node_start = offset;
                        let node_end = offset + text.len();
                        let overlap_start = node_start.max(start);
                        let overlap_end = node_end.min(end);
                        if overlap_start < overlap_end {
                            let text_start = overlap_start - node_start;
                            let text_end = overlap_end - node_start;
                            if let Some(slice) = text.get(text_start..text_end) {
                                new_content.push(InlineNode::Text {
                                    text: slice.to_string(),
                                    marks: Marks::default(),
                                });
                            }
                        }
                        offset = node_end;
                    }
                }
            }
            *content = new_content;
        }
        BlockNode::CodeBlock { code, .. } => {
            let end = end.min(code.len());
            if start < end {
                *code = code[start..end].to_string();
            } else {
                code.clear();
            }
        }
        _ => {}
    }
}

/// Get plain text from a block node.
pub(super) fn block_text_of_from_node(node: &BlockNode) -> String {
    match node {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
            let mut s = String::new();
            for inline in content {
                match inline {
                    InlineNode::Text { text, .. } => s.push_str(text),
                    InlineNode::HardBreak => s.push('\n'),
                    InlineNode::InlineImage { .. } => {}
                    InlineNode::Link { text, .. } => s.push_str(text),
                }
            }
            s
        }
        BlockNode::CodeBlock { code, .. } => code.clone(),
        _ => String::new(),
    }
}

/// Insert an inline node at a specific offset within a content vector.
pub(super) fn insert_inline_at(content: &mut Vec<InlineNode>, offset: usize, node: InlineNode) {
    let mut current_offset = 0;
    for i in 0..content.len() {
        match &content[i] {
            InlineNode::Text { text, .. } => {
                let node_end = current_offset + text.len();
                if offset >= current_offset && offset <= node_end {
                    if offset == current_offset {
                        content.insert(i, node);
                        return;
                    } else if offset == node_end {
                        content.insert(i + 1, node);
                        return;
                    } else {
                        // Split the text node
                        let left_len = offset - current_offset;
                        let right_len = text.len() - left_len;
                        if let InlineNode::Text { text, marks } = content.remove(i) {
                            let left = text[..left_len].to_string();
                            let right = text[text.len() - right_len..].to_string();
                            content.insert(
                                i,
                                InlineNode::Text {
                                    text: left,
                                    marks: marks.clone(),
                                },
                            );
                            content.insert(i + 1, node);
                            content.insert(i + 2, InlineNode::Text { text: right, marks });
                            return;
                        }
                    }
                }
                current_offset = node_end;
            }
            InlineNode::HardBreak | InlineNode::InlineImage { .. } => {
                if offset == current_offset {
                    content.insert(i, node);
                    return;
                }
                current_offset += 1;
            }
            InlineNode::Link { text, .. } => {
                let node_end = current_offset + text.len();
                if offset >= current_offset && offset <= node_end {
                    if offset == current_offset {
                        content.insert(i, node);
                        return;
                    } else if offset == node_end {
                        content.insert(i + 1, node);
                        return;
                    }
                    // Don't split links for simplicity, insert before
                    content.insert(i, node);
                    return;
                }
                current_offset = node_end;
            }
        }
    }
    // If we get here, append at end
    content.push(node);
}
