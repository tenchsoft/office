use super::*;

pub fn import_odt(path: &Path) -> Result<TenchDocument, String> {
    let file = File::open(path)
        .map_err(|error| format!("Failed to open ODT {}: {error}", path.display()))?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| format!("Failed to read ODT zip: {error}"))?;
    zip_util::check_archive_limits(&mut archive, &zip_util::ArchiveLimits::desktop())
        .map_err(|error| format!("ODT archive rejected by safety limits: {error}"))?;
    let mut content = String::new();
    archive
        .by_name("content.xml")
        .map_err(|error| format!("ODT missing content.xml: {error}"))?
        .read_to_string(&mut content)
        .map_err(|error| format!("Failed to read ODT content.xml: {error}"))?;

    // Collect embedded images from the archive.
    let mut media_images: HashMap<String, Vec<u8>> = HashMap::new();
    for idx in 0..archive.len() {
        if let Ok(mut entry) = archive.by_index(idx) {
            let name = entry.name().to_string();
            if name.starts_with("Pictures/") || name.starts_with("media/") {
                let mut data = Vec::new();
                if entry.read_to_end(&mut data).is_ok() {
                    media_images.insert(name, data);
                }
            }
        }
    }

    Ok(parse_content_xml_to_tdm(&content, &media_images))
}

// ---------------------------------------------------------------------------
// Import helpers
// ---------------------------------------------------------------------------

fn parse_content_xml_to_tdm(xml: &str, media_images: &HashMap<String, Vec<u8>>) -> TenchDocument {
    let mut blocks = Vec::new();
    let mut position = 0;

    // Parse automatic styles for later reference.
    let _auto_styles = parse_auto_styles(xml);

    while position < xml.len() {
        // Check for table first.
        let tbl_start = xml[position..]
            .find("<table:table")
            .map(|offset| position + offset);
        let h_start = xml[position..]
            .find("<text:h")
            .map(|offset| position + offset);
        let p_start = xml[position..]
            .find("<text:p")
            .map(|offset| position + offset);

        if let Some(ts) = tbl_start {
            let earliest = [h_start, p_start]
                .into_iter()
                .flatten()
                .min()
                .unwrap_or(xml.len());
            if ts < earliest {
                if let Some((block, next)) = parse_table_tdm(xml, ts, media_images) {
                    blocks.push(block);
                    position = next;
                    continue;
                } else {
                    position = ts + 1;
                    continue;
                }
            }
        }

        match (h_start, p_start) {
            (Some(h), Some(p)) if h < p => {
                if let Some((block, next)) = parse_heading_tdm(xml, h) {
                    blocks.push(block);
                    position = next;
                } else {
                    break;
                }
            }
            (Some(h), None) => {
                if let Some((block, next)) = parse_heading_tdm(xml, h) {
                    blocks.push(block);
                    position = next;
                } else {
                    break;
                }
            }
            (_, Some(p)) => {
                if let Some((block, next)) = parse_paragraph_tdm(xml, p, media_images) {
                    blocks.push(block);
                    position = next;
                } else {
                    break;
                }
            }
            (None, None) => break,
        }
    }

    if blocks.is_empty() {
        blocks.push(BlockNode::Paragraph {
            content: Vec::new(),
            attrs: ParagraphAttrs::default(),
        });
    }

    TenchDocument {
        content: blocks,
        metadata: TdmMetadata::default(),
        ..TenchDocument::new("")
    }
}

fn parse_auto_styles(xml: &str) -> HashMap<String, HashMap<String, String>> {
    let mut styles = HashMap::new();
    let mut pos = 0;
    while let Some(start) = xml[pos..].find("<style:style") {
        let abs_start = pos + start;
        let tag_end = xml[abs_start..].find('>').unwrap_or(0) + abs_start;
        let tag = &xml[abs_start..tag_end];
        let name = extract_odt_attr(tag, "style:name").unwrap_or_default();
        let close = xml[abs_start..].find("</style:style>").unwrap_or(0) + abs_start;
        let inner = &xml[tag_end + 1..close];

        let mut props = HashMap::new();
        // Parse text-properties.
        if let Some(tp_start) = inner.find("<style:text-properties") {
            let tp_end = inner[tp_start..]
                .find('/')
                .or_else(|| inner[tp_start..].find('>'))
                .unwrap_or(0);
            let tp_tag = &inner[tp_start..tp_start + tp_end];
            for attr in &[
                "fo:font-weight",
                "fo:font-style",
                "style:text-underline-style",
                "style:text-line-through-style",
                "fo:font-size",
                "fo:color",
                "fo:background-color",
            ] {
                if let Some(val) = extract_odt_attr(tp_tag, attr) {
                    props.insert(attr.to_string(), val);
                }
            }
        }
        // Parse paragraph-properties.
        if let Some(pp_start) = inner.find("<style:paragraph-properties") {
            let pp_end = inner[pp_start..]
                .find('/')
                .or_else(|| inner[pp_start..].find('>'))
                .unwrap_or(0);
            let pp_tag = &inner[pp_start..pp_start + pp_end];
            for attr in &[
                "fo:text-align",
                "fo:margin-left",
                "fo:margin-right",
                "fo:text-indent",
            ] {
                if let Some(val) = extract_odt_attr(pp_tag, attr) {
                    props.insert(attr.to_string(), val);
                }
            }
        }

        styles.insert(name, props);
        pos = close + "</style:style>".len();
    }
    styles
}

fn parse_table_tdm(
    xml: &str,
    start: usize,
    media_images: &HashMap<String, Vec<u8>>,
) -> Option<(BlockNode, usize)> {
    let segment = &xml[start..];
    let close = segment.find("</table:table>")?;
    let tbl_xml = &segment[..close + "</table:table>".len()];

    let mut rows = Vec::new();
    let mut row_pos = 0;
    while let Some(tr_start) = tbl_xml[row_pos..].find("<table:table-row") {
        let abs = row_pos + tr_start;
        let tr_open_end = tbl_xml[abs..].find('>').unwrap_or(0) + abs;
        let tr_close = tbl_xml[abs..].find("</table:table-row>")?;
        let tr_inner = &tbl_xml[tr_open_end + 1..abs + tr_close];

        let mut cells = Vec::new();
        let mut cell_pos = 0;
        while let Some(tc_start) = tr_inner[cell_pos..].find("<table:table-cell") {
            let abs_tc = cell_pos + tc_start;
            let tc_open_end = tr_inner[abs_tc..].find('>').unwrap_or(0) + abs_tc;
            let tc_close = tr_inner[abs_tc..].find("</table:table-cell>")?;

            let tc_tag = &tr_inner[abs_tc..tc_open_end];
            let tc_inner = &tr_inner[tc_open_end + 1..abs_tc + tc_close];

            let mut cell = TableCell {
                colspan: extract_odt_attr(tc_tag, "table:number-columns-spanned")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(1),
                ..TableCell::default()
            };

            // Parse cell content as paragraphs.
            let mut cell_blocks = Vec::new();
            let mut cp = 0;
            while let Some(p_start) = tc_inner[cp..].find("<text:p") {
                let abs_p = cp + p_start;
                if let Some((block, next)) = parse_paragraph_tdm(tc_inner, abs_p, media_images) {
                    cell_blocks.push(block);
                    cp = next;
                } else {
                    break;
                }
            }
            cell.content = cell_blocks;
            cells.push(cell);
            cell_pos = abs_tc + tc_close + "</table:table-cell>".len();
        }

        rows.push(TableRow { cells });
        row_pos = abs + tr_close + "</table:table-row>".len();
    }

    Some((
        BlockNode::Table { rows },
        start + close + "</table:table>".len(),
    ))
}

fn parse_heading_tdm(xml: &str, start: usize) -> Option<(BlockNode, usize)> {
    let segment = &xml[start..];
    let open_end = segment.find('>')?;
    let close_start = segment.find("</text:h>")?;

    let open_tag = &segment[..open_end + 1];
    let inner = &segment[open_end + 1..close_start];
    let level = outline_level(open_tag);

    let mut attrs = ParagraphAttrs::default();
    if let Some(style_name) = extract_odt_attr(open_tag, "text:style-name") {
        attrs.style_id = Some(style_name);
    }

    let content = parse_inline_content(inner);

    Some((
        BlockNode::Heading {
            level,
            content,
            attrs,
        },
        start + close_start + "</text:h>".len(),
    ))
}

fn parse_paragraph_tdm(
    xml: &str,
    start: usize,
    media_images: &HashMap<String, Vec<u8>>,
) -> Option<(BlockNode, usize)> {
    let segment = &xml[start..];
    let open_end = segment.find('>')?;
    let close_start = segment.find("</text:p>")?;

    let open_tag = &segment[..open_end + 1];
    let inner = &segment[open_end + 1..close_start];

    let mut attrs = ParagraphAttrs::default();
    if let Some(style_name) = extract_odt_attr(open_tag, "text:style-name") {
        attrs.style_id = Some(style_name);
    }

    // Check for text alignment in inline attributes.
    if let Some(align) = extract_odt_attr(open_tag, "fo:text-align") {
        attrs.alignment = match align.as_str() {
            "center" => Alignment::Center,
            "end" | "right" => Alignment::Right,
            "justify" => Alignment::Justify,
            _ => Alignment::Left,
        };
    }

    let mut content = parse_inline_content(inner);

    // Check for draw:image within paragraph.
    if let Some(img_start) = inner.find("<draw:image") {
        let img_end = inner
            .find("</draw:image>")
            .or_else(|| inner[img_start..].find("/>"))?;
        let img_tag = &inner[img_start..img_end + 2];
        if let Some(href) = extract_odt_attr(img_tag, "xlink:href") {
            let source = if let Some(data) = media_images.get(&href) {
                ImageSource::Embedded { data: data.clone() }
            } else {
                ImageSource::Referenced { path: href }
            };
            content.push(InlineNode::InlineImage {
                source,
                alt: extract_odt_attr(img_tag, "draw:alt"),
                width: None,
                height: None,
            });
        }
    }

    Some((
        BlockNode::Paragraph { content, attrs },
        start + close_start + "</text:p>".len(),
    ))
}

fn parse_inline_content(inner: &str) -> Vec<InlineNode> {
    let mut nodes = Vec::new();
    let mut pos = 0;
    let mut text_buf = String::new();

    while pos < inner.len() {
        // Check for text:span.
        if let Some(span_start) = inner[pos..].find("<text:span") {
            let abs = pos + span_start;
            // Flush accumulated text before the span.
            if abs > pos {
                let chunk = xml_util::decode_xml(&inner[pos..abs]);
                text_buf.push_str(&chunk);
            }
            let open_end = inner[abs..].find('>').unwrap_or(0) + abs;
            let close = inner[abs..].find("</text:span>").unwrap_or(inner.len());
            let span_inner = &inner[open_end + 1..close];

            // Extract style from the span tag.
            let span_tag = &inner[abs..open_end];
            let marks = parse_span_marks(span_tag);

            // Recursively parse inner content.
            let inner_nodes = parse_inline_content(span_inner);
            for node in inner_nodes {
                match node {
                    InlineNode::Text { text, .. } => {
                        nodes.push(InlineNode::Text {
                            text,
                            marks: marks.clone(),
                        });
                    }
                    other => nodes.push(other),
                }
            }

            pos = close + "</text:span>".len();
            continue;
        }

        // Check for text:line-break.
        if let Some(br_start) = inner[pos..].find("<text:line-break") {
            let abs = pos + br_start;
            if abs > pos {
                let chunk = xml_util::decode_xml(&inner[pos..abs]);
                text_buf.push_str(&chunk);
            }
            flush_text(&mut text_buf, &mut nodes, &Marks::default());
            nodes.push(InlineNode::HardBreak);
            pos = abs + "<text:line-break/>".len().min(inner.len() - abs);
            continue;
        }

        // Check for text:a (link).
        if let Some(a_start) = inner[pos..].find("<text:a") {
            let abs = pos + a_start;
            if abs > pos {
                let chunk = xml_util::decode_xml(&inner[pos..abs]);
                text_buf.push_str(&chunk);
            }
            let open_end = inner[abs..].find('>').unwrap_or(0) + abs;
            let close = inner[abs..].find("</text:a>").unwrap_or(inner.len());
            let a_tag = &inner[abs..open_end];
            let a_inner = &inner[open_end + 1..close];
            let href = extract_odt_attr(a_tag, "xlink:href").unwrap_or_default();
            let link_text = xml_util::decode_xml(&xml_util::strip_tags(a_inner));
            flush_text(&mut text_buf, &mut nodes, &Marks::default());
            nodes.push(InlineNode::Link {
                href,
                title: None,
                text: link_text,
                marks: Marks::default(),
            });
            pos = close + "</text:a>".len();
            continue;
        }

        // No more tags — consume remaining text.
        let remaining = xml_util::decode_xml(&inner[pos..]);
        text_buf.push_str(&remaining);
        break;
    }

    flush_text(&mut text_buf, &mut nodes, &Marks::default());
    nodes
}

fn parse_span_marks(span_tag: &str) -> Marks {
    let mut marks = Marks::default();
    if let Some(weight) = extract_odt_attr(span_tag, "fo:font-weight") {
        marks.bold = weight == "bold";
    }
    if let Some(style) = extract_odt_attr(span_tag, "fo:font-style") {
        marks.italic = style == "italic";
    }
    if let Some(ul) = extract_odt_attr(span_tag, "style:text-underline-style") {
        marks.underline = ul == "solid";
    }
    if let Some(st) = extract_odt_attr(span_tag, "style:text-line-through-style") {
        marks.strikethrough = st == "solid";
    }
    if let Some(fs) = extract_odt_attr(span_tag, "fo:font-size") {
        marks.font_size = fs.trim_end_matches("pt").parse::<f32>().ok();
    }
    if let Some(color) = extract_odt_attr(span_tag, "fo:color") {
        marks.text_color = Some(color);
    }
    if let Some(bg) = extract_odt_attr(span_tag, "fo:background-color") {
        marks.background_color = Some(bg);
    }
    marks
}

fn flush_text(buf: &mut String, nodes: &mut Vec<InlineNode>, marks: &Marks) {
    if !buf.is_empty() {
        nodes.push(InlineNode::Text {
            text: std::mem::take(buf),
            marks: marks.clone(),
        });
    }
}

fn outline_level(open_tag: &str) -> u8 {
    let Some((_, rest)) = open_tag.split_once("text:outline-level=\"") else {
        return 1;
    };
    let Some((level, _)) = rest.split_once('"') else {
        return 1;
    };
    level.parse::<u8>().unwrap_or(1).clamp(1, 6)
}

fn extract_odt_attr(tag: &str, attr: &str) -> Option<String> {
    let prefix = format!("{attr}=\"");
    let start = tag.find(&prefix)?;
    let rest = &tag[start + prefix.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
