use crate::xml_util;
use tench_document_core::{
    BlockNode, ImageSource, InlineNode, ListItem, Marks, ParagraphAttrs, TableCell, TableRow,
    TdmMetadata, TenchDocument,
};

pub fn html_to_tdm(html: &str) -> TenchDocument {
    let sanitized = xml_util::remove_tag_block(html, "script");
    let sanitized = xml_util::remove_tag_block(&sanitized, "style");

    // Extract the body content.
    let body = extract_html_body(&sanitized);
    let blocks = parse_html_blocks(body);

    TenchDocument {
        content: if blocks.is_empty() {
            vec![BlockNode::Paragraph {
                content: Vec::new(),
                attrs: ParagraphAttrs::default(),
            }]
        } else {
            blocks
        },
        metadata: TdmMetadata::default(),
        ..TenchDocument::new("")
    }
}

/// Extract the content between `<body>` and `</body>`, or return the full HTML
/// if no body tag is found.
fn extract_html_body(html: &str) -> &str {
    let lower = html.to_ascii_lowercase();
    if let Some(start) = lower.find("<body") {
        // Find the end of the opening <body...> tag.
        let tag_end = html[start..]
            .find('>')
            .map(|i| start + i + 1)
            .unwrap_or(start);
        if let Some(end) = lower.find("</body>") {
            return &html[tag_end..end];
        }
        return &html[tag_end..];
    }
    html
}

/// Parse top-level HTML block elements into TDM BlockNodes.
fn parse_html_blocks(html: &str) -> Vec<BlockNode> {
    let mut blocks = Vec::new();
    let mut pos = 0;

    while pos < html.len() {
        // Skip whitespace and text between block elements.
        while pos < html.len() && html.as_bytes()[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= html.len() {
            break;
        }

        // Try to match block-level elements.
        let remaining = &html[pos..];
        let lower = remaining.to_ascii_lowercase();

        if lower.starts_with("<table") {
            if let Some(end) = find_closing_tag(remaining, "table") {
                blocks.push(parse_html_table(&remaining[..end]));
                pos += end;
                continue;
            }
        }

        if lower.starts_with("<img") {
            if let Some(tag_end) = remaining.find('>') {
                blocks.push(parse_html_image(&remaining[..tag_end + 1]));
                pos += tag_end + 1;
                continue;
            }
        }

        let mut goto_next = false;
        for tag in &["h1", "h2", "h3", "h4", "h5", "h6"] {
            let open = format!("<{tag}");
            if lower.starts_with(open.as_str()) {
                if let Some(end) = find_closing_tag(remaining, tag) {
                    let level = tag[1..].parse::<u8>().unwrap_or(1);
                    let inner = extract_tag_inner(&remaining[..end], tag);
                    let content = parse_html_inline(inner);
                    blocks.push(BlockNode::Heading {
                        level,
                        content,
                        attrs: ParagraphAttrs::default(),
                    });
                    pos += end;
                    goto_next = true;
                    break;
                }
            }
        }
        if goto_next {
            continue;
        }

        if lower.starts_with("<p") {
            if let Some(end) = find_closing_tag(remaining, "p") {
                let inner = extract_tag_inner(&remaining[..end], "p");
                blocks.push(BlockNode::Paragraph {
                    content: parse_html_inline(inner),
                    attrs: parse_html_paragraph_attrs(&remaining[..end]),
                });
                pos += end;
                continue;
            }
        }

        if lower.starts_with("<ul") {
            if let Some(end) = find_closing_tag(remaining, "ul") {
                blocks.push(BlockNode::BulletList {
                    items: parse_html_list_items(&remaining[..end]),
                });
                pos += end;
                continue;
            }
        }

        if lower.starts_with("<ol") {
            if let Some(end) = find_closing_tag(remaining, "ol") {
                blocks.push(BlockNode::OrderedList {
                    items: parse_html_list_items(&remaining[..end]),
                    start: 1,
                });
                pos += end;
                continue;
            }
        }

        if lower.starts_with("<blockquote") {
            if let Some(end) = find_closing_tag(remaining, "blockquote") {
                let inner = extract_tag_inner(&remaining[..end], "blockquote");
                let children = parse_html_blocks(inner);
                blocks.push(BlockNode::BlockQuote { content: children });
                pos += end;
                continue;
            }
        }

        if lower.starts_with("<pre") {
            if let Some(end) = find_closing_tag(remaining, "pre") {
                let inner = extract_tag_inner(&remaining[..end], "pre");
                blocks.push(BlockNode::CodeBlock {
                    code: xml_util::decode_xml(inner).trim().to_string(),
                    language: None,
                });
                pos += end;
                continue;
            }
        }

        if lower.starts_with("<hr") {
            let tag_end = remaining.find('>').unwrap_or(remaining.len());
            blocks.push(BlockNode::HorizontalRule);
            pos += tag_end + 1;
            continue;
        }

        // Skip unknown opening tags.
        if remaining.starts_with('<') && !remaining.starts_with("</") {
            if let Some(tag_end) = remaining.find('>') {
                pos += tag_end + 1;
                continue;
            }
        }

        // Collect loose text until the next tag.
        let mut text_end = remaining.len();
        if let Some(angle) = remaining.find('<') {
            text_end = angle;
        }
        if text_end > 0 {
            let text = xml_util::decode_basic_entities(remaining[..text_end].trim());
            if !text.is_empty() {
                blocks.push(BlockNode::Paragraph {
                    content: vec![InlineNode::Text {
                        text,
                        marks: Marks::default(),
                    }],
                    attrs: ParagraphAttrs::default(),
                });
            }
            pos += text_end;
        } else {
            pos += 1;
        }
    }

    blocks
}

/// Parse inline HTML content (text, bold, italic, links, etc.) into InlineNodes.
fn parse_html_inline(html: &str) -> Vec<InlineNode> {
    let mut nodes = Vec::new();
    let mut pos = 0;

    while pos < html.len() {
        let remaining = &html[pos..];
        let lower = remaining.to_ascii_lowercase();

        // Bold: <strong> or <b>
        if lower.starts_with("<strong") || lower.starts_with("<b>") {
            let tag = if lower.starts_with("<strong") {
                "strong"
            } else {
                "b"
            };
            if let Some(end) = find_closing_tag(remaining, tag) {
                let inner = extract_tag_inner(&remaining[..end], tag);
                let style = if tag == "strong" {
                    extract_html_style(&remaining[..end])
                } else {
                    String::new()
                };
                let mut marks = parse_css_marks(&style);
                marks.bold = true;
                let inner_nodes = parse_html_inline(inner);
                wrap_inline_with_marks(inner_nodes, marks, &mut nodes);
                pos += end;
                continue;
            }
        }

        // Italic: <em> or <i>
        if lower.starts_with("<em") || lower.starts_with("<i>") {
            let tag = if lower.starts_with("<em") { "em" } else { "i" };
            if let Some(end) = find_closing_tag(remaining, tag) {
                let inner = extract_tag_inner(&remaining[..end], tag);
                let style = if tag == "em" {
                    extract_html_style(&remaining[..end])
                } else {
                    String::new()
                };
                let mut marks = parse_css_marks(&style);
                marks.italic = true;
                let inner_nodes = parse_html_inline(inner);
                wrap_inline_with_marks(inner_nodes, marks, &mut nodes);
                pos += end;
                continue;
            }
        }

        // Underline: <u>
        if lower.starts_with("<u>") {
            if let Some(end) = find_closing_tag(remaining, "u") {
                let inner = extract_tag_inner(&remaining[..end], "u");
                let inner_nodes = parse_html_inline(inner);
                wrap_inline_with_marks(
                    inner_nodes,
                    Marks {
                        underline: true,
                        ..Marks::default()
                    },
                    &mut nodes,
                );
                pos += end;
                continue;
            }
        }

        // Strikethrough: <s>, <del>, <strike>
        if lower.starts_with("<s>") || lower.starts_with("<del>") || lower.starts_with("<strike>") {
            let tag = if lower.starts_with("<del>") {
                "del"
            } else if lower.starts_with("<strike>") {
                "strike"
            } else {
                "s"
            };
            if let Some(end) = find_closing_tag(remaining, tag) {
                let inner = extract_tag_inner(&remaining[..end], tag);
                let inner_nodes = parse_html_inline(inner);
                wrap_inline_with_marks(
                    inner_nodes,
                    Marks {
                        strikethrough: true,
                        ..Marks::default()
                    },
                    &mut nodes,
                );
                pos += end;
                continue;
            }
        }

        // Inline code: <code>
        if lower.starts_with("<code>") {
            if let Some(end) = find_closing_tag(remaining, "code") {
                let inner = extract_tag_inner(&remaining[..end], "code");
                nodes.push(InlineNode::Text {
                    text: xml_util::decode_xml(inner),
                    marks: Marks {
                        code: true,
                        ..Marks::default()
                    },
                });
                pos += end;
                continue;
            }
        }

        // Link: <a href=...>
        if lower.starts_with("<a ") {
            if let Some(end) = find_closing_tag(remaining, "a") {
                let tag_content = &remaining[..end];
                let href = extract_html_attr(tag_content, "href").unwrap_or_default();
                let inner = extract_tag_inner(tag_content, "a");
                let link_text = xml_util::decode_basic_entities(&xml_util::strip_tags(inner));
                nodes.push(InlineNode::Link {
                    href,
                    title: None,
                    text: link_text,
                    marks: Marks::default(),
                });
                pos += end;
                continue;
            }
        }

        // Inline image: <img ...>
        if lower.starts_with("<img") {
            if let Some(tag_end) = remaining.find('>') {
                let tag = &remaining[..tag_end + 1];
                let src = extract_html_attr(tag, "src").unwrap_or_default();
                let alt = extract_html_attr(tag, "alt");
                nodes.push(InlineNode::InlineImage {
                    source: ImageSource::Referenced { path: src },
                    alt,
                    width: None,
                    height: None,
                });
                pos += tag_end + 1;
                continue;
            }
        }

        // Span with style
        if lower.starts_with("<span") {
            if let Some(end) = find_closing_tag(remaining, "span") {
                let style = extract_html_style(&remaining[..end]);
                let marks = parse_css_marks(&style);
                let inner = extract_tag_inner(&remaining[..end], "span");
                let inner_nodes = parse_html_inline(inner);
                wrap_inline_with_marks(inner_nodes, marks, &mut nodes);
                pos += end;
                continue;
            }
        }

        // <br>
        if lower.starts_with("<br") {
            let tag_end = remaining.find('>').unwrap_or(remaining.len());
            nodes.push(InlineNode::HardBreak);
            pos += tag_end + 1;
            continue;
        }

        // Skip unknown opening tags.
        if remaining.starts_with('<') && !remaining.starts_with("</") {
            if let Some(tag_end) = remaining.find('>') {
                pos += tag_end + 1;
                continue;
            }
        }

        // Collect text until the next tag.
        let mut text_end = remaining.len();
        if let Some(angle) = remaining.find('<') {
            text_end = angle;
        }
        if text_end > 0 {
            let text = xml_util::decode_basic_entities(&remaining[..text_end]);
            nodes.push(InlineNode::Text {
                text,
                marks: Marks::default(),
            });
            pos += text_end;
        } else {
            pos += 1;
        }
    }

    nodes
}

/// Apply marks to inline nodes, preserving any existing marks.
fn wrap_inline_with_marks(nodes: Vec<InlineNode>, marks: Marks, output: &mut Vec<InlineNode>) {
    for node in nodes {
        match node {
            InlineNode::Text {
                text,
                marks: existing,
            } => {
                output.push(InlineNode::Text {
                    text,
                    marks: merge_marks(existing, &marks),
                });
            }
            InlineNode::Link {
                href,
                title,
                text,
                marks: existing,
            } => {
                output.push(InlineNode::Link {
                    href,
                    title,
                    text,
                    marks: merge_marks(existing, &marks),
                });
            }
            other => output.push(other),
        }
    }
}

/// Merge two Marks, with `overlay` taking precedence for non-default values.
fn merge_marks(base: Marks, overlay: &Marks) -> Marks {
    Marks {
        bold: base.bold || overlay.bold,
        italic: base.italic || overlay.italic,
        underline: base.underline || overlay.underline,
        strikethrough: base.strikethrough || overlay.strikethrough,
        superscript: base.superscript || overlay.superscript,
        subscript: base.subscript || overlay.subscript,
        code: base.code || overlay.code,
        font_size: base.font_size.or(overlay.font_size),
        text_color: base
            .text_color
            .clone()
            .or_else(|| overlay.text_color.clone()),
        background_color: base
            .background_color
            .clone()
            .or_else(|| overlay.background_color.clone()),
        font_family: base
            .font_family
            .clone()
            .or_else(|| overlay.font_family.clone()),
    }
}

/// Parse a CSS inline style string into Marks.
fn parse_css_marks(style: &str) -> Marks {
    let mut marks = Marks::default();
    for prop in style.split(';') {
        let prop = prop.trim();
        if let Some((key, value)) = prop.split_once(':') {
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim();
            match key.as_str() {
                "font-weight" if value == "bold" || value.starts_with('7') || value == "bolder" => {
                    marks.bold = true;
                }
                "font-style" if value == "italic" || value == "oblique" => {
                    marks.italic = true;
                }
                "text-decoration" => {
                    let lower = value.to_ascii_lowercase();
                    if lower.contains("underline") {
                        marks.underline = true;
                    }
                    if lower.contains("line-through") {
                        marks.strikethrough = true;
                    }
                }
                "color" => {
                    marks.text_color = Some(normalize_css_color(value));
                }
                "background-color" => {
                    marks.background_color = Some(normalize_css_color(value));
                }
                "font-size" => {
                    marks.font_size = parse_css_font_size(value);
                }
                _ => {}
            }
        }
    }
    marks
}

/// Normalize a CSS color value to `#rrggbb` format.
fn normalize_css_color(value: &str) -> String {
    let value = value.trim();
    if value.starts_with('#') {
        return value.to_string();
    }
    // Handle rgb(r, g, b).
    if let Some(rest) = value.strip_prefix("rgb(") {
        if let Some(inner) = rest.strip_suffix(')') {
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parts[0].trim().parse::<u8>(),
                    parts[1].trim().parse::<u8>(),
                    parts[2].trim().parse::<u8>(),
                ) {
                    return format!("#{:02x}{:02x}{:02x}", r, g, b);
                }
            }
        }
    }
    // Named colors (basic set).
    match value {
        "black" => "#000000".to_string(),
        "white" => "#ffffff".to_string(),
        "red" => "#ff0000".to_string(),
        "green" => "#008000".to_string(),
        "blue" => "#0000ff".to_string(),
        "yellow" => "#ffff00".to_string(),
        "orange" => "#ffa500".to_string(),
        "purple" => "#800080".to_string(),
        "gray" | "grey" => "#808080".to_string(),
        _ => value.to_string(),
    }
}

/// Parse a CSS font-size value into points.
fn parse_css_font_size(value: &str) -> Option<f32> {
    let value = value.trim();
    if let Some(pt) = value.strip_suffix("pt") {
        return pt.trim().parse::<f32>().ok();
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse::<f32>().ok().map(|v| v * 0.75);
    }
    if let Some(em) = value.strip_suffix("em") {
        return em.trim().parse::<f32>().ok().map(|v| v * 12.0);
    }
    if let Some(rem) = value.strip_suffix("rem") {
        return rem.trim().parse::<f32>().ok().map(|v| v * 12.0);
    }
    // Absolute sizes.
    match value {
        "xx-small" => Some(7.0),
        "x-small" => Some(8.0),
        "small" => Some(10.0),
        "medium" => Some(12.0),
        "large" => Some(14.0),
        "x-large" => Some(18.0),
        "xx-large" => Some(24.0),
        _ => value.parse::<f32>().ok(),
    }
}

/// Find the end position (exclusive) of a closing tag, handling nested tags.
fn find_closing_tag(html: &str, tag: &str) -> Option<usize> {
    let close = format!("</{tag}>");
    let lower = html.to_ascii_lowercase();
    let open_exact = format!("<{tag}>");
    let open_prefix = format!("<{tag} ");
    let mut depth = 0;
    let mut pos = 0;
    while pos < lower.len() {
        if lower[pos..].starts_with(&open_exact) || lower[pos..].starts_with(&open_prefix) {
            depth += 1;
            // Skip past the opening tag (find the closing '>').
            let tag_end = lower[pos..].find('>').unwrap_or(0);
            pos += tag_end + 1;
        } else if lower[pos..].starts_with(&close) {
            depth -= 1;
            if depth == 0 {
                return Some(pos + close.len());
            }
            pos += close.len();
        } else {
            pos += 1;
        }
    }
    // If depth > 0 but we've consumed everything, return the full length.
    if depth > 0 {
        Some(html.len())
    } else {
        None
    }
}

/// Extract the inner content between opening and closing tags.
fn extract_tag_inner<'a>(html: &'a str, tag: &str) -> &'a str {
    let open_end = html.find('>').unwrap_or(0) + 1;
    let close = format!("</{tag}>");
    let close_start = html.rfind(&close).unwrap_or(html.len());
    if close_start > open_end {
        &html[open_end..close_start]
    } else {
        ""
    }
}

/// Extract the value of an HTML attribute from a tag string.
fn extract_html_attr(tag: &str, attr: &str) -> Option<String> {
    let prefix = format!("{attr}=\"");
    let start = tag.find(&prefix)?;
    let rest = &tag[start + prefix.len()..];
    let end = rest.find('"')?;
    Some(xml_util::decode_basic_entities(&rest[..end]))
}

/// Extract the `style` attribute value from an HTML tag.
fn extract_html_style(tag: &str) -> String {
    extract_html_attr(tag, "style").unwrap_or_default()
}

/// Parse paragraph attributes from a `<p>` tag.
fn parse_html_paragraph_attrs(tag: &str) -> ParagraphAttrs {
    let mut attrs = ParagraphAttrs::default();
    let style = extract_html_style(tag);
    for prop in style.split(';') {
        let prop = prop.trim();
        if let Some((key, value)) = prop.split_once(':') {
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim();
            match key.as_str() {
                "text-align" => {
                    attrs.alignment = match value {
                        "center" => tench_document_core::Alignment::Center,
                        "right" => tench_document_core::Alignment::Right,
                        "justify" => tench_document_core::Alignment::Justify,
                        _ => tench_document_core::Alignment::Left,
                    };
                }
                "margin-left" => {
                    if let Some(val) = parse_css_length(value) {
                        attrs.indent_left = val;
                    }
                }
                "margin-right" => {
                    if let Some(val) = parse_css_length(value) {
                        attrs.indent_right = val;
                    }
                }
                "text-indent" => {
                    if let Some(val) = parse_css_length(value) {
                        attrs.indent_first_line = val;
                    }
                }
                _ => {}
            }
        }
    }
    attrs
}

/// Parse a CSS length value to a numeric value (approximate mm).
fn parse_css_length(value: &str) -> Option<f32> {
    let value = value.trim();
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse::<f32>().ok().map(|v| v * 0.264583);
    }
    if let Some(em) = value.strip_suffix("em") {
        return em.trim().parse::<f32>().ok().map(|v| v * 3.175);
    }
    if let Some(cm) = value.strip_suffix("cm") {
        return cm.trim().parse::<f32>().ok().map(|v| v * 10.0);
    }
    if let Some(mm) = value.strip_suffix("mm") {
        return mm.trim().parse::<f32>().ok();
    }
    value.parse::<f32>().ok()
}

/// Parse an HTML `<table>` element into a Table BlockNode.
fn parse_html_table(html: &str) -> BlockNode {
    let mut rows = Vec::new();
    let mut pos = 0;
    let lower = html.to_ascii_lowercase();

    while pos < lower.len() {
        if let Some(tr_start) = lower[pos..].find("<tr") {
            let abs = pos + tr_start;
            if let Some(tr_end) = find_closing_tag(&html[abs..], "tr") {
                let tr_html = &html[abs..abs + tr_end];
                rows.push(parse_html_table_row(tr_html));
                pos = abs + tr_end;
                continue;
            }
        }
        break;
    }

    BlockNode::Table { rows }
}

/// Parse an HTML `<tr>` element into a TableRow.
fn parse_html_table_row(html: &str) -> TableRow {
    let mut cells = Vec::new();
    let mut pos = 0;
    let lower = html.to_ascii_lowercase();

    while pos < lower.len() {
        let cell_tag = if lower[pos..].starts_with("<th") {
            "th"
        } else if lower[pos..].starts_with("<td") {
            "td"
        } else {
            pos += 1;
            continue;
        };

        let abs = pos;
        if let Some(cell_end) = find_closing_tag(&html[abs..], cell_tag) {
            let cell_html = &html[abs..abs + cell_end];
            let inner = extract_tag_inner(cell_html, cell_tag);
            let cell_blocks = parse_html_blocks(inner);
            let cell = TableCell {
                content: cell_blocks,
                ..TableCell::default()
            };
            cells.push(cell);
            pos = abs + cell_end;
            continue;
        }
        break;
    }

    TableRow { cells }
}

/// Parse an HTML `<img>` tag into an Image BlockNode.
fn parse_html_image(tag: &str) -> BlockNode {
    let src = extract_html_attr(tag, "src").unwrap_or_default();
    let alt = extract_html_attr(tag, "alt");
    BlockNode::Image {
        source: ImageSource::Referenced { path: src },
        alt,
        width: extract_html_attr(tag, "width").and_then(|v| v.parse::<f32>().ok()),
        height: extract_html_attr(tag, "height").and_then(|v| v.parse::<f32>().ok()),
    }
}

/// Parse `<li>` elements from a list HTML fragment.
fn parse_html_list_items(html: &str) -> Vec<ListItem> {
    let mut items = Vec::new();
    let mut pos = 0;
    let lower = html.to_ascii_lowercase();

    while pos < lower.len() {
        if let Some(li_start) = lower[pos..].find("<li") {
            let abs = pos + li_start;
            if let Some(li_end) = find_closing_tag(&html[abs..], "li") {
                let inner = extract_tag_inner(&html[abs..abs + li_end], "li");
                let content = parse_html_inline(inner);
                items.push(ListItem {
                    content,
                    children: Vec::new(),
                });
                pos = abs + li_end;
                continue;
            }
        }
        break;
    }

    items
}

// ---------------------------------------------------------------------------
// Markdown helpers
// ---------------------------------------------------------------------------
