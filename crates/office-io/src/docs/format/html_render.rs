use crate::xml_util;
use tench_document_core::{BlockNode, InlineNode, TenchDocument};

pub fn tdm_to_html(doc: &TenchDocument) -> String {
    let body: String = doc
        .content
        .iter()
        .map(block_to_html)
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "<!doctype html>\n<html>\n<head><meta charset=\"utf-8\"><title>Tench Docs \
         Export</title></head>\n<body>\n{body}\n</body>\n</html>\n"
    )
}

fn block_to_html(block: &BlockNode) -> String {
    match block {
        BlockNode::Paragraph { content, .. } => {
            format!("<p>{}</p>", inline_to_html(content))
        }
        BlockNode::Heading { level, content, .. } => {
            let l = (*level as usize).clamp(1, 6);
            format!("<h{l}>{}</h{l}>", inline_to_html(content))
        }
        BlockNode::BulletList { items } => {
            let items_html: String = items
                .iter()
                .map(|i| format!("<li>{}</li>", inline_to_html(&i.content)))
                .collect();
            format!("<ul>{items_html}</ul>")
        }
        BlockNode::OrderedList { items, .. } => {
            let items_html: String = items
                .iter()
                .map(|i| format!("<li>{}</li>", inline_to_html(&i.content)))
                .collect();
            format!("<ol>{items_html}</ol>")
        }
        BlockNode::BlockQuote { content } => {
            let inner: String = content
                .iter()
                .map(block_to_html)
                .collect::<Vec<_>>()
                .join("\n");
            format!("<blockquote>{inner}</blockquote>")
        }
        BlockNode::CodeBlock { code, .. } => {
            format!("<pre><code>{}</code></pre>", xml_util::escape_xml(code))
        }
        BlockNode::Table { rows } => {
            let mut html = String::from("<table>");
            for row in rows {
                html.push_str("<tr>");
                for cell in &row.cells {
                    html.push_str("<td>");
                    for block in &cell.content {
                        html.push_str(&block_to_html(block));
                    }
                    html.push_str("</td>");
                }
                html.push_str("</tr>");
            }
            html.push_str("</table>");
            html
        }
        BlockNode::HorizontalRule => "<hr>".to_string(),
        BlockNode::PageBreak => "<hr>".to_string(),
        BlockNode::Image { source, alt, .. } => {
            let src = match source {
                tench_document_core::ImageSource::Referenced { path } => path.clone(),
                tench_document_core::ImageSource::Embedded { .. } => "embedded".to_string(),
            };
            let a = alt.as_deref().unwrap_or("");
            format!(
                "<img src=\"{}\" alt=\"{}\">",
                xml_util::escape_html(&src),
                xml_util::escape_html(a)
            )
        }
        BlockNode::TaskList { items } => {
            let items_html: String = items
                .iter()
                .map(|i| {
                    let checked = if i.checked { " checked" } else { "" };
                    format!(
                        "<li><input type=\"checkbox\"{checked}>{}</li>",
                        inline_to_html(&i.content)
                    )
                })
                .collect();
            format!("<ul>{items_html}</ul>")
        }
        BlockNode::Footnote { number, content } => {
            format!("<p>[{number}] {}</p>", inline_to_html(content))
        }
    }
}

fn inline_to_html(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            InlineNode::Text { text, marks } => {
                let mut t = xml_util::escape_html(text);
                if marks.bold {
                    t = format!("<strong>{t}</strong>");
                }
                if marks.italic {
                    t = format!("<em>{t}</em>");
                }
                if marks.underline {
                    t = format!("<u>{t}</u>");
                }
                if marks.strikethrough {
                    t = format!("<s>{t}</s>");
                }
                if marks.code {
                    t = format!("<code>{t}</code>");
                }
                out.push_str(&t);
            }
            InlineNode::Link { href, text, .. } => {
                out.push_str(&format!(
                    "<a href=\"{}\">{}</a>",
                    xml_util::escape_html(href),
                    xml_util::escape_html(text)
                ));
            }
            InlineNode::InlineImage { source, alt, .. } => {
                let src = match source {
                    tench_document_core::ImageSource::Referenced { path } => path.clone(),
                    tench_document_core::ImageSource::Embedded { .. } => "embedded".to_string(),
                };
                let a = alt.as_deref().unwrap_or("");
                out.push_str(&format!(
                    "<img src=\"{}\" alt=\"{}\">",
                    xml_util::escape_html(&src),
                    xml_util::escape_html(a)
                ));
            }
            InlineNode::HardBreak => out.push_str("<br>"),
        }
    }
    out
}
