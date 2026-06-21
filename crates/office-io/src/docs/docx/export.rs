use std::collections::HashMap;

use tench_document_core::{
    Alignment, BlockNode, ImageSource, InlineNode, Marks, Orientation, PageSetup, ParagraphAttrs,
    TenchDocument,
};

use crate::xml_util;

use super::util::mm_to_twip;

/// Collect all embedded images from the document for inclusion in the DOCX archive.
pub(crate) fn collect_embedded_images(
    doc: &TenchDocument,
    media_entries: &mut Vec<(String, Vec<u8>)>,
    rel_counter: &mut u32,
    extra_rels: &mut Vec<(String, String, String)>,
) {
    let mut idx: u32 = 1;
    for block in &doc.content {
        collect_images_from_block(block, media_entries, &mut idx, rel_counter, extra_rels);
    }
}

fn collect_images_from_block(
    block: &BlockNode,
    media_entries: &mut Vec<(String, Vec<u8>)>,
    idx: &mut u32,
    rel_counter: &mut u32,
    extra_rels: &mut Vec<(String, String, String)>,
) {
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
            for inline in content {
                collect_images_from_inline(inline, media_entries, idx, rel_counter, extra_rels);
            }
        }
        BlockNode::Image {
            source: ImageSource::Embedded { data },
            ..
        } => {
            let name = format!("word/media/image{}.png", idx);
            let rid = format!("rId{rel_counter}");
            *rel_counter += 1;
            *idx += 1;
            extra_rels.push((
                rid,
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
                    .to_string(),
                name.clone(),
            ));
            media_entries.push((name, data.clone()));
        }
        BlockNode::BlockQuote { content } => {
            for child in content {
                collect_images_from_block(child, media_entries, idx, rel_counter, extra_rels);
            }
        }
        BlockNode::Table { rows } => {
            for row in rows {
                for cell in &row.cells {
                    for b in &cell.content {
                        collect_images_from_block(b, media_entries, idx, rel_counter, extra_rels);
                    }
                }
            }
        }
        _ => {}
    }
}

fn collect_images_from_inline(
    node: &InlineNode,
    media_entries: &mut Vec<(String, Vec<u8>)>,
    idx: &mut u32,
    rel_counter: &mut u32,
    extra_rels: &mut Vec<(String, String, String)>,
) {
    if let InlineNode::InlineImage {
        source: ImageSource::Embedded { data },
        ..
    } = node
    {
        let name = format!("word/media/image{}.png", idx);
        let rid = format!("rId{rel_counter}");
        *rel_counter += 1;
        *idx += 1;
        extra_rels.push((
            rid,
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image".to_string(),
            name.clone(),
        ));
        media_entries.push((name, data.clone()));
    }
}

pub(crate) fn build_content_types(
    media_entries: &[(String, Vec<u8>)],
    has_header: bool,
    has_footer: bool,
) -> String {
    let mut overrides = String::from(
        r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>"#,
    );
    if has_header {
        overrides.push_str(
            r#"
  <Override PartName="/word/header1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/>"#,
        );
    }
    if has_footer {
        overrides.push_str(
            r#"
  <Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>"#,
        );
    }
    for (name, _) in media_entries {
        let ext = if name.ends_with(".png") {
            "png"
        } else if name.ends_with(".jpg") || name.ends_with(".jpeg") {
            "jpeg"
        } else {
            "bin"
        };
        if !overrides.contains(&format!("Extension=\"{ext}\"")) {
            let mime = match ext {
                "png" => "image/png",
                "jpeg" => "image/jpeg",
                _ => "application/octet-stream",
            };
            overrides.push_str(&format!(
                "\n  <Default Extension=\"{ext}\" ContentType=\"{mime}\"/>"
            ));
        }
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  {overrides}
</Types>"#
    )
}

pub(crate) fn build_document_rels(extra_rels: &[(String, String, String)]) -> String {
    let rels: String = extra_rels
        .iter()
        .map(|(id, typ, target)| {
            format!(r#"  <Relationship Id="{id}" Type="{typ}" Target="{target}"/>"#)
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
{rels}
</Relationships>"#
    )
}

pub(crate) fn build_document_xml_from_tdm(
    doc: &TenchDocument,
    image_rid_map: &HashMap<String, String>,
    header_rid: Option<String>,
    footer_rid: Option<String>,
) -> String {
    let body: String = doc
        .content
        .iter()
        .map(|b| block_to_word_xml(b, image_rid_map))
        .collect::<Vec<_>>()
        .join("");

    let sect_pr = build_sect_pr(&doc.page_setup, header_rid, footer_rid);

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><w:body>{body}{sect_pr}</w:body></w:document>"#
    )
}

fn build_sect_pr(
    page_setup: &PageSetup,
    header_rid: Option<String>,
    footer_rid: Option<String>,
) -> String {
    let (w, h) = page_setup_size_twips(page_setup);
    let orient_attr = match page_setup.orientation {
        Orientation::Landscape => r#" w:orient="landscape""#,
        Orientation::Portrait => "",
    };
    let m = &page_setup.margins;
    let header_ref = header_rid
        .as_ref()
        .map(|rid| format!(r#"<w:headerReference w:type="default" r:id="{rid}"/>"#))
        .unwrap_or_default();
    let footer_ref = footer_rid
        .as_ref()
        .map(|rid| format!(r#"<w:footerReference w:type="default" r:id="{rid}"/>"#))
        .unwrap_or_default();

    format!(
        r#"<w:sectPr>{header_ref}{footer_ref}<w:pgSz w:w="{w}" w:h="{h}"{orient_attr}/><w:pgMar w:top="{}" w:right="{}" w:bottom="{}" w:left="{}" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>"#,
        mm_to_twip(m.top),
        mm_to_twip(m.right),
        mm_to_twip(m.bottom),
        mm_to_twip(m.left),
    )
}

fn page_setup_size_twips(page_setup: &PageSetup) -> (i32, i32) {
    let (w_mm, h_mm) = page_setup.paper_size.dimensions_mm();
    match page_setup.orientation {
        Orientation::Portrait => (mm_to_twip(w_mm), mm_to_twip(h_mm)),
        Orientation::Landscape => (mm_to_twip(h_mm), mm_to_twip(w_mm)),
    }
}

pub(crate) fn build_styles_xml(doc: &TenchDocument) -> String {
    let mut styles = String::new();

    // Default paragraph style.
    styles.push_str(
        r#"<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:pPr><w:spacing w:after="200" w:line="276"/></w:pPr><w:rPr><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr></w:style>"#,
    );

    // Title style.
    styles.push_str(
        r#"<w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:rPr><w:b/><w:sz w:val="56"/><w:szCs w:val="56"/></w:rPr></w:style>"#,
    );

    // Heading styles 1-9.
    for level in 1..=9 {
        let sz = match level {
            1 => 44,
            2 => 36,
            3 => 32,
            4 => 28,
            5 => 24,
            _ => 22,
        };
        styles.push_str(&format!(
            r#"<w:style w:type="paragraph" w:styleId="Heading{level}"><w:name w:val="heading {level}"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:pPr><w:keepNext/><w:keepLines/><w:spacing w:before="360" w:after="80"/></w:pPr><w:rPr><w:b/><w:sz w:val="{sz}"/><w:szCs w:val="{sz}"/></w:rPr></w:style>"#
        ));
    }

    // Quote style.
    styles.push_str(
        r#"<w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/><w:basedOn w:val="Normal"/><w:pPr><w:ind w:left="720"/></w:pPr><w:rPr><w:i/></w:rPr></w:style>"#,
    );

    // Code style.
    styles.push_str(
        r#"<w:style w:type="character" w:styleId="Code"><w:name w:val="Code"/><w:rPr><w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/></w:rPr></w:style>"#,
    );

    // User-defined styles.
    for style in &doc.styles {
        styles.push_str(&format!(
            r#"<w:style w:type="paragraph" w:styleId="{}"><w:name w:val="{}"/>{}</w:style>"#,
            xml_util::escape_xml(&style.id),
            xml_util::escape_xml(&style.name),
            style
                .parent_id
                .as_ref()
                .map(|p| format!(r#"<w:basedOn w:val="{}"/>"#, xml_util::escape_xml(p)))
                .unwrap_or_default()
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{styles}</w:styles>"#
    )
}

pub(crate) fn build_header_footer_xml(text: &str, kind: &str) -> String {
    let escaped = xml_util::escape_xml(text);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:{kind} xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>{escaped}</w:t></w:r></w:p></w:{kind}>"#
    )
}

fn block_to_word_xml(block: &BlockNode, image_rid_map: &HashMap<String, String>) -> String {
    match block {
        BlockNode::Heading {
            level,
            content,
            attrs,
            ..
        } => paragraph_with_attrs_xml(
            Some(&format!("Heading{level}")),
            &content
                .iter()
                .map(|n| inline_to_word_xml(n, image_rid_map))
                .collect::<Vec<_>>()
                .join(""),
            attrs,
        ),
        BlockNode::Paragraph { content, attrs } => paragraph_with_attrs_xml(
            None,
            &content
                .iter()
                .map(|n| inline_to_word_xml(n, image_rid_map))
                .collect::<Vec<_>>()
                .join(""),
            attrs,
        ),
        BlockNode::BulletList { items } => items
            .iter()
            .map(|item| {
                let runs = text_run_xml("• ", &Marks::default())
                    + &item
                        .content
                        .iter()
                        .map(|n| inline_to_word_xml(n, image_rid_map))
                        .collect::<Vec<_>>()
                        .join("");
                paragraph_xml(None, &runs)
            })
            .collect::<Vec<_>>()
            .join(""),
        BlockNode::OrderedList { items, .. } => items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let runs = text_run_xml(&format!("{}. ", index + 1), &Marks::default())
                    + &item
                        .content
                        .iter()
                        .map(|n| inline_to_word_xml(n, image_rid_map))
                        .collect::<Vec<_>>()
                        .join("");
                paragraph_xml(None, &runs)
            })
            .collect::<Vec<_>>()
            .join(""),
        BlockNode::BlockQuote { content } => paragraph_with_attrs_xml(
            Some("Quote"),
            &content
                .iter()
                .map(|b| {
                    b.inline_text_content()
                        .map(|t| text_run_xml(&t, &Marks::default()))
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
                .join(""),
            &ParagraphAttrs::default(),
        ),
        BlockNode::CodeBlock { code, .. } => paragraph_xml(
            None,
            &text_run_xml_with_style(code, &Marks::default(), "Code"),
        ),
        BlockNode::Table { rows } => {
            let col_count = rows.first().map(|r| r.cells.len()).unwrap_or(0).max(1);
            let tbl_width = col_count * 2000;
            let mut xml = format!(
                r#"<w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/><w:tblW w:w="{tbl_width}" w:type="dxa"/><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="000000"/><w:left w:val="single" w:sz="4" w:space="0" w:color="000000"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="000000"/><w:right w:val="single" w:sz="4" w:space="0" w:color="000000"/><w:insideH w:val="single" w:sz="4" w:space="0" w:color="000000"/><w:insideV w:val="single" w:sz="4" w:space="0" w:color="000000"/></w:tblBorders></w:tblPr><w:tblGrid>{}</w:tblGrid>"#,
                (0..col_count)
                    .map(|_| "<w:gridCol w:w=\"2000\"/>")
                    .collect::<Vec<_>>()
                    .join("")
            );
            for row in rows {
                xml.push_str("<w:tr>");
                for cell in &row.cells {
                    let grid_span = if cell.colspan > 1 {
                        format!(r#"<w:gridSpan w:val="{}"/>"#, cell.colspan)
                    } else {
                        String::new()
                    };
                    let vmerge = if cell.rowspan == 0 {
                        r#"<w:vMerge/>"#.to_string()
                    } else if cell.rowspan > 1 {
                        r#"<w:vMerge w:val="restart"/>"#.to_string()
                    } else {
                        String::new()
                    };
                    let tcpr = if grid_span.is_empty() && vmerge.is_empty() {
                        String::new()
                    } else {
                        format!("<w:tcPr>{grid_span}{vmerge}</w:tcPr>")
                    };
                    let cell_text: String = cell
                        .content
                        .iter()
                        .map(|b| block_to_word_xml(b, image_rid_map))
                        .collect();
                    xml.push_str(&format!("<w:tc>{tcpr}{cell_text}</w:tc>"));
                }
                xml.push_str("</w:tr>");
            }
            xml.push_str("</w:tbl>");
            xml
        }
        BlockNode::HorizontalRule => paragraph_xml(None, "<w:r><w:pict><v:rect/></w:pict></w:r>"),
        BlockNode::PageBreak => "<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>".to_string(),
        BlockNode::Image { source, alt, .. } => {
            let rid = find_image_rid(source, image_rid_map);
            if let Some(r_id) = rid {
                let cx = 500000; // Default 5cm in EMU
                let cy = 500000;
                let alt_text = alt.as_deref().unwrap_or("");
                let drawing = format!(
                    r#"<w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="1" name="Picture" descr="{}"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="0" name="{}"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{r_id}"/></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"/></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing>"#,
                    xml_util::escape_xml(alt_text),
                    xml_util::escape_xml(alt_text)
                );
                paragraph_xml(None, &format!("<w:r>{drawing}</w:r>"))
            } else {
                let text = alt.as_deref().unwrap_or("");
                paragraph_xml(None, &text_run_xml(text, &Marks::default()))
            }
        }
        BlockNode::TaskList { items } => items
            .iter()
            .map(|item| {
                let check = if item.checked { "[x] " } else { "[ ] " };
                let runs = text_run_xml(check, &Marks::default())
                    + &item
                        .content
                        .iter()
                        .map(|n| inline_to_word_xml(n, image_rid_map))
                        .collect::<Vec<_>>()
                        .join("");
                paragraph_xml(None, &runs)
            })
            .collect::<Vec<_>>()
            .join(""),
        BlockNode::Footnote { number, content } => {
            let runs = content
                .iter()
                .map(|n| inline_to_word_xml(n, image_rid_map))
                .collect::<Vec<_>>()
                .join("");
            let ref_run = text_run_xml(&format!("[{number}] "), &Marks::default());
            paragraph_xml(None, &(ref_run + &runs))
        }
    }
}

trait BlockTextExt {
    fn inline_text_content(&self) -> Option<String>;
}

impl BlockTextExt for BlockNode {
    fn inline_text_content(&self) -> Option<String> {
        match self {
            BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
                let t: String = content
                    .iter()
                    .map(|n| match n {
                        InlineNode::Text { text, .. } => text.clone(),
                        InlineNode::Link { text, .. } => text.clone(),
                        _ => String::new(),
                    })
                    .collect();
                Some(t)
            }
            _ => None,
        }
    }
}

fn find_image_rid(source: &ImageSource, image_rid_map: &HashMap<String, String>) -> Option<String> {
    match source {
        ImageSource::Embedded { .. } => image_rid_map.values().next().cloned(),
        ImageSource::Referenced { path } => {
            let target = if path.starts_with("word/media/") {
                path.trim_start_matches("word/").to_string()
            } else {
                path.clone()
            };
            image_rid_map.get(&target).cloned()
        }
    }
}

fn inline_to_word_xml(node: &InlineNode, image_rid_map: &HashMap<String, String>) -> String {
    match node {
        InlineNode::Text { text, marks } => text_run_xml(text, marks),
        InlineNode::HardBreak => "<w:r><w:br/></w:r>".to_string(),
        InlineNode::Link { text, .. } => text_run_xml(text, &Marks::default()),
        InlineNode::InlineImage { source, alt, .. } => {
            let rid = find_image_rid(source, image_rid_map);
            if let Some(r_id) = rid {
                let cx = 500000;
                let cy = 500000;
                let alt_text = alt.as_deref().unwrap_or("");
                let drawing = format!(
                    r#"<w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="1" name="Picture" descr="{}"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="0" name="{}"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{r_id}"/></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"/></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing>"#,
                    xml_util::escape_xml(alt_text),
                    xml_util::escape_xml(alt_text)
                );
                format!("<w:r>{drawing}</w:r>")
            } else {
                text_run_xml(alt.as_deref().unwrap_or(""), &Marks::default())
            }
        }
    }
}

fn paragraph_with_attrs_xml(style: Option<&str>, runs: &str, attrs: &ParagraphAttrs) -> String {
    let mut ppr = String::new();

    if let Some(s) = style {
        ppr.push_str(&format!(r#"<w:pStyle w:val="{s}"/>"#));
    }

    // Alignment.
    match attrs.alignment {
        Alignment::Left => {} // default
        Alignment::Center => ppr.push_str(r#"<w:jc w:val="center"/>"#),
        Alignment::Right => ppr.push_str(r#"<w:jc w:val="right"/>"#),
        Alignment::Justify => ppr.push_str(r#"<w:jc w:val="both"/>"#),
    }

    // Indentation.
    if attrs.indent_left != 0.0 || attrs.indent_right != 0.0 || attrs.indent_first_line != 0.0 {
        let left = mm_to_twip(attrs.indent_left);
        let right = mm_to_twip(attrs.indent_right);
        let first = mm_to_twip(attrs.indent_first_line);
        ppr.push_str(&format!(r#"<w:ind w:left="{left}" w:right="{right}""#));
        if first > 0 {
            ppr.push_str(&format!(r#" w:firstLine="{first}""#));
        }
        ppr.push_str("/>");
    }

    // Spacing.
    if attrs.space_before > 0.0 || attrs.space_after > 0.0 {
        let before = (attrs.space_before * 20.0) as i32;
        let after = (attrs.space_after * 20.0) as i32;
        ppr.push_str(&format!(
            r#"<w:spacing w:before="{before}" w:after="{after}"/>"#
        ));
    }

    let ppr_xml = if ppr.is_empty() {
        String::new()
    } else {
        format!("<w:pPr>{ppr}</w:pPr>")
    };

    let runs = if runs.is_empty() {
        "<w:r><w:t></w:t></w:r>"
    } else {
        runs
    };
    format!("<w:p>{ppr_xml}{runs}</w:p>")
}

fn paragraph_xml(style: Option<&str>, runs: &str) -> String {
    paragraph_with_attrs_xml(style, runs, &ParagraphAttrs::default())
}

fn text_run_xml(text: &str, marks: &Marks) -> String {
    text_run_xml_with_style(text, marks, "")
}

fn text_run_xml_with_style(text: &str, marks: &Marks, rstyle: &str) -> String {
    let props = run_props_xml(marks, rstyle);
    let space = if text.starts_with(' ') || text.ends_with(' ') {
        r#" xml:space="preserve""#
    } else {
        ""
    };
    format!(
        "<w:r>{props}<w:t{space}>{}</w:t></w:r>",
        xml_util::escape_xml(text)
    )
}

fn run_props_xml(marks: &Marks, rstyle: &str) -> String {
    let mut props = String::new();
    if marks.bold {
        props.push_str("<w:b/>");
    }
    if marks.italic {
        props.push_str("<w:i/>");
    }
    if marks.underline {
        props.push_str(r#"<w:u w:val="single"/>"#);
    }
    if marks.strikethrough {
        props.push_str("<w:strike/>");
    }
    if let Some(fs) = marks.font_size {
        let half_pts = (fs * 2.0) as i32;
        props.push_str(&format!(
            r#"<w:sz w:val="{half_pts}"/><w:szCs w:val="{half_pts}"/>"#
        ));
    }
    if let Some(ref color) = marks.text_color {
        let hex = color.trim_start_matches('#');
        props.push_str(&format!(r#"<w:color w:val="{hex}"/>"#));
    }
    if let Some(ref bg) = marks.background_color {
        let hex = bg.trim_start_matches('#');
        props.push_str(&format!(r#"<w:shd w:val="clear" w:fill="{hex}"/>"#));
    }
    if !rstyle.is_empty() {
        props.push_str(&format!(r#"<w:rStyle w:val="{rstyle}"/>"#));
    }

    if props.is_empty() {
        String::new()
    } else {
        format!("<w:rPr>{props}</w:rPr>")
    }
}

// ---------------------------------------------------------------------------
// Unit conversion
// ---------------------------------------------------------------------------
