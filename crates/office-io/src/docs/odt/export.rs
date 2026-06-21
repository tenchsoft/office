use super::*;

pub fn export_odt_bytes(doc: &TenchDocument) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let stored = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o644);
    let deflated = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    zip_util::write_zip_file(&mut writer, "mimetype", ODT_MIMETYPE, stored)?;

    // Collect images and build automatic styles.
    let mut auto_styles = String::new();
    let mut style_counter: u32 = 1;
    let mut media_entries: Vec<(String, Vec<u8>)> = Vec::new();
    collect_odt_images(
        doc,
        &mut media_entries,
        &mut style_counter,
        &mut auto_styles,
    );

    let manifest = build_manifest_xml(&media_entries);
    zip_util::write_zip_file(&mut writer, "META-INF/manifest.xml", &manifest, deflated)?;

    let content_xml = build_content_xml_from_tdm(doc, &auto_styles);
    zip_util::write_zip_file(&mut writer, "content.xml", &content_xml, deflated)?;

    // Write embedded images.
    for (name, data) in &media_entries {
        writer
            .start_file(name, deflated)
            .map_err(|e| format!("Failed to add {name}: {e}"))?;
        writer
            .write_all(data)
            .map_err(|e| format!("Failed to write {name}: {e}"))?;
    }

    writer
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|error| format!("Failed to finish ODT archive: {error}"))
}

// ---------------------------------------------------------------------------
// Export helpers
// ---------------------------------------------------------------------------

fn collect_odt_images(
    doc: &TenchDocument,
    media_entries: &mut Vec<(String, Vec<u8>)>,
    style_counter: &mut u32,
    auto_styles: &mut String,
) {
    for block in &doc.content {
        collect_images_from_block(block, media_entries, style_counter, auto_styles);
    }
}

fn collect_images_from_block(
    block: &BlockNode,
    media_entries: &mut Vec<(String, Vec<u8>)>,
    style_counter: &mut u32,
    auto_styles: &mut String,
) {
    match block {
        BlockNode::Paragraph { content, attrs } | BlockNode::Heading { content, attrs, .. } => {
            maybe_emit_paragraph_style(attrs, style_counter, auto_styles);
            for inline in content {
                collect_images_from_inline(inline, media_entries, style_counter, auto_styles);
            }
        }
        BlockNode::Image {
            source: ImageSource::Embedded { data },
            ..
        } => {
            let name = format!("Pictures/image{}.png", media_entries.len() + 1);
            media_entries.push((name, data.clone()));
        }
        BlockNode::Table { rows } => {
            for row in rows {
                for cell in &row.cells {
                    for b in &cell.content {
                        collect_images_from_block(b, media_entries, style_counter, auto_styles);
                    }
                }
            }
        }
        BlockNode::BlockQuote { content } => {
            for child in content {
                collect_images_from_block(child, media_entries, style_counter, auto_styles);
            }
        }
        BlockNode::BulletList { items } | BlockNode::OrderedList { items, .. } => {
            for item in items {
                for inline in &item.content {
                    collect_images_from_inline(inline, media_entries, style_counter, auto_styles);
                }
            }
        }
        BlockNode::TaskList { items } => {
            for item in items {
                for inline in &item.content {
                    collect_images_from_inline(inline, media_entries, style_counter, auto_styles);
                }
            }
        }
        _ => {}
    }
}

fn collect_images_from_inline(
    node: &InlineNode,
    media_entries: &mut Vec<(String, Vec<u8>)>,
    style_counter: &mut u32,
    auto_styles: &mut String,
) {
    match node {
        InlineNode::Text { marks, .. } => {
            maybe_emit_text_style(marks, style_counter, auto_styles);
        }
        InlineNode::InlineImage {
            source: ImageSource::Embedded { data },
            ..
        } => {
            let name = format!("Pictures/image{}.png", media_entries.len() + 1);
            media_entries.push((name, data.clone()));
        }
        _ => {}
    }
}

fn maybe_emit_paragraph_style(
    attrs: &ParagraphAttrs,
    style_counter: &mut u32,
    auto_styles: &mut String,
) {
    let mut fo = String::new();
    match attrs.alignment {
        Alignment::Center => fo.push_str("fo:text-align=\"center\" "),
        Alignment::Right => fo.push_str("fo:text-align=\"end\" "),
        Alignment::Justify => fo.push_str("fo:text-align=\"justify\" "),
        Alignment::Left => {}
    }
    if attrs.indent_left != 0.0 {
        fo.push_str(&format!(
            "fo:margin-left=\"{}cm\" ",
            attrs.indent_left / 10.0
        ));
    }
    if attrs.indent_right != 0.0 {
        fo.push_str(&format!(
            "fo:margin-right=\"{}cm\" ",
            attrs.indent_right / 10.0
        ));
    }
    if attrs.indent_first_line != 0.0 {
        fo.push_str(&format!(
            "fo:text-indent=\"{}cm\" ",
            attrs.indent_first_line / 10.0
        ));
    }
    if fo.is_empty() {
        return;
    }
    let name = format!("P{}", *style_counter);
    *style_counter += 1;
    auto_styles.push_str(&format!(
        "<style:style style:name=\"{name}\" style:family=\"paragraph\">\
         <style:paragraph-properties {fo}/></style:style>"
    ));
}

fn maybe_emit_text_style(marks: &Marks, style_counter: &mut u32, auto_styles: &mut String) {
    let mut fo = String::new();
    if marks.bold {
        fo.push_str("fo:font-weight=\"bold\" ");
    }
    if marks.italic {
        fo.push_str("fo:font-style=\"italic\" ");
    }
    if marks.underline {
        fo.push_str("style:text-underline-style=\"solid\" ");
    }
    if marks.strikethrough {
        fo.push_str("style:text-line-through-style=\"solid\" ");
    }
    if let Some(fs) = marks.font_size {
        fo.push_str(&format!("fo:font-size=\"{fs}pt\" "));
    }
    if let Some(ref color) = marks.text_color {
        let hex = color.trim_start_matches('#');
        fo.push_str(&format!("fo:color=\"#{hex}\" "));
    }
    if let Some(ref bg) = marks.background_color {
        let hex = bg.trim_start_matches('#');
        fo.push_str(&format!("fo:background-color=\"#{hex}\" "));
    }
    if fo.is_empty() {
        return;
    }
    let name = format!("T{}", *style_counter);
    *style_counter += 1;
    auto_styles.push_str(&format!(
        "<style:style style:name=\"{name}\" style:family=\"text\">\
         <style:text-properties {fo}/></style:style>"
    ));
}

fn build_content_xml_from_tdm(doc: &TenchDocument, auto_styles: &str) -> String {
    let body: String = doc
        .content
        .iter()
        .enumerate()
        .map(|(i, b)| block_to_odt_xml(b, i))
        .collect::<Vec<_>>()
        .join("");

    let styles_section = if auto_styles.is_empty() {
        String::new()
    } else {
        format!("<office:automatic-styles>{auto_styles}</office:automatic-styles>")
    };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><office:document-content office:version="1.2" xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:xlink="http://www.w3.org/1999/xlink">{styles_section}<office:body><office:text>{body}</office:text></office:body></office:document-content>"#
    )
}

fn block_to_odt_xml(block: &BlockNode, _idx: usize) -> String {
    match block {
        BlockNode::Heading {
            level,
            content,
            attrs,
            ..
        } => {
            let ppr = paragraph_props_odt(attrs);
            let text = inline_to_odt_spans(content);
            format!(r#"<text:h text:outline-level="{level}"{ppr}>{text}</text:h>"#)
        }
        BlockNode::Paragraph { content, attrs } => {
            let ppr = paragraph_props_odt(attrs);
            let text = inline_to_odt_spans(content);
            format!("<text:p{ppr}>{text}</text:p>")
        }
        BlockNode::BulletList { items } => items
            .iter()
            .map(|item| {
                let text = inline_to_odt_spans(&item.content);
                format!("<text:p text:style-name=\"Bullet\">• {text}</text:p>")
            })
            .collect::<Vec<_>>()
            .join(""),
        BlockNode::OrderedList { items, .. } => items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let text = inline_to_odt_spans(&item.content);
                format!(
                    "<text:p text:style-name=\"Numbering\">{}. {text}</text:p>",
                    index + 1
                )
            })
            .collect::<Vec<_>>()
            .join(""),
        BlockNode::BlockQuote { content } => content
            .iter()
            .map(|b| {
                let text = block_plain_text(b);
                format!(
                    "<text:p text:style-name=\"Quote\">{}</text:p>",
                    xml_util::escape_xml(&text)
                )
            })
            .collect::<Vec<_>>()
            .join(""),
        BlockNode::CodeBlock { code, .. } => {
            format!(
                "<text:p text:style-name=\"Preformatted\">{}</text:p>",
                xml_util::escape_xml(code)
            )
        }
        BlockNode::Table { rows } => {
            let col_count = rows.first().map(|r| r.cells.len()).unwrap_or(0).max(1);
            let columns: String = (0..col_count).map(|_| "<table:table-column/>").collect();
            let row_xml: String = rows
                .iter()
                .map(|row| {
                    let cells: String = row
                        .cells
                        .iter()
                        .map(|cell| {
                            let text: String = cell
                                .content
                                .iter()
                                .map(block_plain_text)
                                .collect::<Vec<_>>()
                                .join(" ");
                            let colspan_attr = if cell.colspan > 1 {
                                format!(" table:number-columns-spanned=\"{}\"", cell.colspan)
                            } else {
                                String::new()
                            };
                            format!(
                                "<table:table-cell{colspan_attr}><text:p>{}</text:p></table:table-cell>",
                                xml_util::escape_xml(&text)
                            )
                        })
                        .collect();
                    format!("<table:table-row>{cells}</table:table-row>")
                })
                .collect();
            format!("<table:table>{columns}{row_xml}</table:table>")
        }
        BlockNode::HorizontalRule => "<text:p>---</text:p>".to_string(),
        BlockNode::PageBreak => "<text:p></text:p>".to_string(),
        BlockNode::Image { source, alt, .. } => {
            let href = match source {
                ImageSource::Embedded { .. } => {
                    format!("Pictures/image{}.png", image_idx_for_source(source))
                }
                ImageSource::Referenced { path } => path.clone(),
            };
            let alt_text = alt.as_deref().unwrap_or("");
            format!(
                "<text:p><draw:image xlink:href=\"{href}\" xlink:type=\"simple\" draw:alt=\"{}\"/></text:p>",
                xml_util::escape_xml(alt_text)
            )
        }
        BlockNode::TaskList { items } => items
            .iter()
            .map(|item| {
                let check = if item.checked { "[x] " } else { "[ ] " };
                format!(
                    "<text:p>{}{}</text:p>",
                    xml_util::escape_xml(check),
                    inline_to_odt_spans(&item.content)
                )
            })
            .collect::<Vec<_>>()
            .join(""),
        BlockNode::Footnote { number, content } => {
            let text = inline_to_odt_spans(content);
            format!("<text:p>[{number}] {text}</text:p>")
        }
    }
}

fn paragraph_props_odt(attrs: &ParagraphAttrs) -> String {
    let mut props = String::new();
    match attrs.alignment {
        Alignment::Center => props.push_str(" fo:text-align=\"center\""),
        Alignment::Right => props.push_str(" fo:text-align=\"end\""),
        Alignment::Justify => props.push_str(" fo:text-align=\"justify\""),
        Alignment::Left => {}
    }
    if attrs.indent_left != 0.0 {
        props.push_str(&format!(
            " fo:margin-left=\"{}cm\"",
            attrs.indent_left / 10.0
        ));
    }
    if props.is_empty() {
        String::new()
    } else {
        " text:style-name=\"CustomPara\"".to_string()
    }
}

fn inline_to_odt_spans(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            InlineNode::Text { text, marks } => {
                let escaped = xml_util::escape_xml(text);
                if has_marks(marks) {
                    let style_attrs = marks_to_odt_attrs(marks);
                    out.push_str(&format!(
                        "<text:span text:style-name=\"CustomSpan\"><text:span{style_attrs}>{escaped}</text:span></text:span>"
                    ));
                } else {
                    out.push_str(&escaped);
                }
            }
            InlineNode::Link { href, text, .. } => {
                out.push_str(&format!(
                    "<text:a xlink:href=\"{}\" xlink:type=\"simple\">{}</text:a>",
                    xml_util::escape_xml(href),
                    xml_util::escape_xml(text)
                ));
            }
            InlineNode::HardBreak => out.push_str("<text:line-break/>"),
            InlineNode::InlineImage { source, alt, .. } => {
                let href = match source {
                    ImageSource::Embedded { .. } => {
                        format!("Pictures/image{}.png", image_idx_for_source(source))
                    }
                    ImageSource::Referenced { path } => path.clone(),
                };
                let alt_text = alt.as_deref().unwrap_or("");
                out.push_str(&format!(
                    "<draw:image xlink:href=\"{href}\" xlink:type=\"simple\" draw:alt=\"{}\"/>",
                    xml_util::escape_xml(alt_text)
                ));
            }
        }
    }
    out
}

fn has_marks(marks: &Marks) -> bool {
    marks.bold
        || marks.italic
        || marks.underline
        || marks.strikethrough
        || marks.font_size.is_some()
        || marks.text_color.is_some()
        || marks.background_color.is_some()
}

fn marks_to_odt_attrs(marks: &Marks) -> String {
    let mut attrs = String::new();
    if marks.bold {
        attrs.push_str(" fo:font-weight=\"bold\"");
    }
    if marks.italic {
        attrs.push_str(" fo:font-style=\"italic\"");
    }
    if marks.underline {
        attrs.push_str(" style:text-underline-style=\"solid\"");
    }
    if marks.strikethrough {
        attrs.push_str(" style:text-line-through-style=\"solid\"");
    }
    if let Some(fs) = marks.font_size {
        attrs.push_str(&format!(" fo:font-size=\"{fs}pt\""));
    }
    if let Some(ref color) = marks.text_color {
        let hex = color.trim_start_matches('#');
        attrs.push_str(&format!(" fo:color=\"#{hex}\""));
    }
    if let Some(ref bg) = marks.background_color {
        let hex = bg.trim_start_matches('#');
        attrs.push_str(&format!(" fo:background-color=\"#{hex}\""));
    }
    attrs
}

fn image_idx_for_source(_source: &ImageSource) -> usize {
    // Simplified: use a counter in production
    1
}

fn inline_text(nodes: &[InlineNode]) -> String {
    nodes
        .iter()
        .map(|n| match n {
            InlineNode::Text { text, .. } => text.clone(),
            InlineNode::Link { text, .. } => text.clone(),
            InlineNode::HardBreak => "\n".to_string(),
            InlineNode::InlineImage { alt, .. } => alt.clone().unwrap_or_default(),
        })
        .collect()
}

fn block_plain_text(block: &BlockNode) -> String {
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
            inline_text(content)
        }
        BlockNode::CodeBlock { code, .. } => code.clone(),
        BlockNode::Image { alt, .. } => alt.clone().unwrap_or_default(),
        _ => String::new(),
    }
}

fn build_manifest_xml(media_entries: &[(String, Vec<u8>)]) -> String {
    let mut entries = String::from(
        r#"<manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.text"/>
  <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>"#,
    );
    for (name, _) in media_entries {
        let mime = if name.ends_with(".png") {
            "image/png"
        } else if name.ends_with(".jpg") || name.ends_with(".jpeg") {
            "image/jpeg"
        } else {
            "application/octet-stream"
        };
        entries.push_str(&format!(
            "\n  <manifest:file-entry manifest:full-path=\"{name}\" manifest:media-type=\"{mime}\"/>"
        ));
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
  {entries}
</manifest:manifest>"#
    )
}
