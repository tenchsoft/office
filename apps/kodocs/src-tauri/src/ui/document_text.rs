// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Extract plain text from a block node.
pub(super) fn extract_block_text(block: &tench_document_core::BlockNode) -> String {
    match block {
        tench_document_core::BlockNode::Paragraph { content, .. }
        | tench_document_core::BlockNode::Heading { content, .. } => extract_inline_text(content),
        tench_document_core::BlockNode::CodeBlock { code, .. } => code.clone(),
        tench_document_core::BlockNode::BlockQuote { content } => content
            .iter()
            .map(extract_block_text)
            .collect::<Vec<_>>()
            .join("\n"),
        tench_document_core::BlockNode::BulletList { items }
        | tench_document_core::BlockNode::OrderedList { items, .. } => items
            .iter()
            .map(|i| extract_inline_text(&i.content))
            .collect::<Vec<_>>()
            .join("\n"),
        tench_document_core::BlockNode::TaskList { items } => items
            .iter()
            .map(|i| extract_inline_text(&i.content))
            .collect::<Vec<_>>()
            .join("\n"),
        tench_document_core::BlockNode::HorizontalRule
        | tench_document_core::BlockNode::PageBreak => String::new(),
        tench_document_core::BlockNode::Footnote { number, content } => {
            let text = extract_inline_text(content);
            if text.trim().is_empty() {
                format!("[{number}]")
            } else {
                format!("[{number}] {text}")
            }
        }
        tench_document_core::BlockNode::Image { alt, .. } => alt.clone().unwrap_or_default(),
        tench_document_core::BlockNode::Table { rows } => rows
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
    }
}

pub(super) fn extract_inline_text(nodes: &[tench_document_core::InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            tench_document_core::InlineNode::Text { text, .. } => out.push_str(text),
            tench_document_core::InlineNode::Link { text, .. } => out.push_str(text),
            tench_document_core::InlineNode::InlineImage { alt, .. } => {
                if let Some(a) = alt {
                    out.push_str(a);
                }
            }
            tench_document_core::InlineNode::HardBreak => out.push('\n'),
        }
    }
    out
}
