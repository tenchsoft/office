use tench_document_core::{BlockNode, InlineNode, ListItem, Marks, ParagraphAttrs, TenchDocument};

pub fn markdown_to_tdm(markdown: &str) -> TenchDocument {
    let mut blocks: Vec<BlockNode> = Vec::new();
    let mut pending_bullets: Vec<ListItem> = Vec::new();
    let mut pending_ordered: Vec<ListItem> = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        if let Some(item) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            flush_ordered_tdm(&mut blocks, &mut pending_ordered);
            pending_bullets.push(ListItem {
                content: vec![InlineNode::Text {
                    text: item.to_string(),
                    marks: Marks::default(),
                }],
                children: Vec::new(),
            });
            continue;
        }

        if let Some((_, item)) = ordered_marker(trimmed) {
            flush_bullets_tdm(&mut blocks, &mut pending_bullets);
            pending_ordered.push(ListItem {
                content: vec![InlineNode::Text {
                    text: item.to_string(),
                    marks: Marks::default(),
                }],
                children: Vec::new(),
            });
            continue;
        }

        flush_bullets_tdm(&mut blocks, &mut pending_bullets);
        flush_ordered_tdm(&mut blocks, &mut pending_ordered);

        if trimmed.is_empty() {
            blocks.push(BlockNode::Paragraph {
                content: Vec::new(),
                attrs: ParagraphAttrs::default(),
            });
        } else if let Some((level, text)) = heading_marker(trimmed) {
            blocks.push(BlockNode::Heading {
                level,
                content: vec![InlineNode::Text {
                    text: text.to_string(),
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            });
        } else {
            blocks.push(BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: line.to_string(),
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            });
        }
    }

    flush_bullets_tdm(&mut blocks, &mut pending_bullets);
    flush_ordered_tdm(&mut blocks, &mut pending_ordered);

    if blocks.is_empty() {
        blocks.push(BlockNode::Paragraph {
            content: Vec::new(),
            attrs: ParagraphAttrs::default(),
        });
    }

    TenchDocument {
        content: blocks,
        ..TenchDocument::new("")
    }
}

/// Convert a `TenchDocument` to Markdown.
pub fn tdm_to_markdown(doc: &TenchDocument) -> String {
    let mut out = String::new();
    for (i, block) in doc.content.iter().enumerate() {
        if i > 0 && !out.ends_with("\n\n") {
            out.push('\n');
        }
        block_to_markdown(block, &mut out);
    }
    out.trim_end().to_string()
}

fn flush_bullets_tdm(blocks: &mut Vec<BlockNode>, pending: &mut Vec<ListItem>) {
    if !pending.is_empty() {
        blocks.push(BlockNode::BulletList {
            items: std::mem::take(pending),
        });
    }
}

fn flush_ordered_tdm(blocks: &mut Vec<BlockNode>, pending: &mut Vec<ListItem>) {
    if !pending.is_empty() {
        blocks.push(BlockNode::OrderedList {
            items: std::mem::take(pending),
            start: 1,
        });
    }
}

fn heading_marker(line: &str) -> Option<(u8, &str)> {
    let marker_len = line.chars().take_while(|ch| *ch == '#').count();
    if (1..=6).contains(&marker_len) && line.chars().nth(marker_len) == Some(' ') {
        Some((marker_len as u8, line[marker_len + 1..].trim()))
    } else {
        None
    }
}

fn ordered_marker(line: &str) -> Option<(&str, &str)> {
    let (marker, rest) = line.split_once(". ")?;
    marker
        .chars()
        .all(|ch| ch.is_ascii_digit())
        .then_some((marker, rest.trim()))
}

fn block_to_markdown(block: &BlockNode, out: &mut String) {
    match block {
        BlockNode::Paragraph { content, .. } => {
            out.push_str(&inline_text(content));
            out.push_str("\n\n");
        }
        BlockNode::Heading { level, content, .. } => {
            out.push_str(&"#".repeat(*level as usize));
            out.push(' ');
            out.push_str(&inline_text(content));
            out.push_str("\n\n");
        }
        BlockNode::BulletList { items } => {
            for item in items {
                out.push_str("- ");
                out.push_str(&inline_text(&item.content));
                out.push('\n');
            }
            out.push('\n');
        }
        BlockNode::OrderedList { items, .. } => {
            for (i, item) in items.iter().enumerate() {
                out.push_str(&format!("{}. ", i + 1));
                out.push_str(&inline_text(&item.content));
                out.push('\n');
            }
            out.push('\n');
        }
        BlockNode::BlockQuote { content } => {
            for child in content {
                block_to_markdown(child, out);
            }
        }
        BlockNode::CodeBlock { code, .. } => {
            out.push_str("```\n");
            out.push_str(code);
            out.push_str("\n```\n\n");
        }
        BlockNode::Table { rows } => {
            for row in rows {
                for (i, cell) in row.cells.iter().enumerate() {
                    if i > 0 {
                        out.push('\t');
                    }
                    for block in &cell.content {
                        block_to_markdown(block, out);
                    }
                }
                out.push('\n');
            }
        }
        BlockNode::HorizontalRule => {
            out.push_str("---\n\n");
        }
        BlockNode::PageBreak => {
            out.push_str("---\n\n");
        }
        BlockNode::Image { alt, .. } => {
            if let Some(a) = alt {
                out.push_str(a);
                out.push('\n');
            }
        }
        BlockNode::TaskList { items } => {
            for item in items {
                out.push_str(if item.checked { "[x] " } else { "[ ] " });
                out.push_str(&inline_text(&item.content));
                out.push('\n');
            }
            out.push('\n');
        }
        BlockNode::Footnote { number, content } => {
            out.push_str(&format!("[^{number}]: "));
            out.push_str(&inline_text(content));
            out.push_str("\n\n");
        }
    }
}

fn inline_text(nodes: &[InlineNode]) -> String {
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
