use std::collections::HashMap;

use tench_document_core::{
    Alignment, BlockNode, HeadersFooters, ImageSource, InlineNode, Margins, Marks, Orientation,
    PageSetup, PaperSize, ParagraphAttrs, TableCell, TableRow, TdmMetadata, TenchDocument,
};

use crate::xml_util;

use super::util::{extract_attr, twip_to_mm};

/// Parse relationship elements into a map of rId -> target.
pub(crate) fn parse_relationships(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<Relationship ") {
        rest = &rest[start + "<Relationship ".len()..];
        let end = rest
            .find('/')
            .or_else(|| rest.find('>'))
            .unwrap_or(rest.len());
        let tag = &rest[..end];
        let id = extract_attr(tag, "Id");
        let target = extract_attr(tag, "Target");
        if let (Some(id), Some(target)) = (id, target) {
            map.insert(id, target);
        }
        rest = &rest[end..];
    }
    map
}

/// Extract text from a header or footer XML.
pub(crate) fn extract_header_footer_text(xml: &str) -> String {
    let mut texts = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<w:t") {
        rest = &rest[start..];
        let open_end = rest.find('>').unwrap_or(0);
        let close_start = rest.find("</w:t>").unwrap_or(0);
        if open_end < close_start {
            texts.push(rest[open_end + 1..close_start].to_string());
        }
        rest = if close_start > 0 {
            &rest[close_start + "</w:t>".len()..]
        } else {
            break;
        };
    }
    texts.join("")
}

pub(crate) fn parse_document_xml_to_tdm(
    xml: &str,
    relationships: &HashMap<String, String>,
    media_images: &HashMap<String, Vec<u8>>,
) -> TenchDocument {
    let mut blocks = Vec::new();
    let mut page_setup = PageSetup::default();
    let metadata = TdmMetadata::default();

    // Parse page setup from sectPr.
    if let Some(sect_start) = xml.find("<w:sectPr") {
        let sect_end = xml.find("</w:sectPr>").unwrap_or(xml.len());
        let sect = &xml[sect_start..sect_end];
        page_setup = parse_page_setup(sect);
    }

    // Extract body content between <w:body> and the last <w:sectPr>.
    let body_start = xml
        .find("<w:body>")
        .map(|i| i + "<w:body>".len())
        .unwrap_or(0);
    let body_end = xml.rfind("<w:sectPr").unwrap_or(xml.len());
    let body = &xml[body_start..body_end];

    let mut rest = body;
    while !rest.is_empty() {
        // Check for table first (tables come before paragraphs in priority).
        if let Some(tbl_start) = rest.find("<w:tbl>") {
            let p_start = rest.find("<w:p");
            if p_start.is_none() || tbl_start < p_start.unwrap() {
                let tbl_end = rest.find("</w:tbl>").unwrap_or(rest.len());
                let tbl_xml = &rest[tbl_start..tbl_end + "</w:tbl>".len()];
                blocks.push(parse_table(tbl_xml, relationships, media_images));
                rest = &rest[tbl_end + "</w:tbl>".len()..];
                continue;
            }
        }

        if let Some(start) = rest.find("<w:p") {
            rest = &rest[start..];
            let open_end = rest.find('>').unwrap_or(rest.len());
            let close_start = rest.find("</w:p>").unwrap_or(rest.len());
            let paragraph = &rest[..close_start + "</w:p>".len()];
            let inner = &rest[open_end + 1..close_start];
            blocks.push(parse_paragraph_tdm(
                paragraph,
                inner,
                relationships,
                media_images,
            ));
            rest = &rest[close_start + "</w:p>".len()..];
        } else {
            break;
        }
    }

    if blocks.is_empty() {
        blocks.push(BlockNode::Paragraph {
            content: Vec::new(),
            attrs: ParagraphAttrs::default(),
        });
    }

    TenchDocument {
        metadata,
        page_setup,
        styles: Vec::new(),
        content: blocks,
        headers_footers: HeadersFooters::default(),
    }
}

fn parse_page_setup(sect: &str) -> PageSetup {
    let mut page_setup = PageSetup::default();

    // Parse pgSz.
    if let Some(pgsz_start) = sect.find("<w:pgSz") {
        let pgsz_end = sect[pgsz_start..].find('/').unwrap_or(0) + pgsz_start;
        let pgsz = &sect[pgsz_start..pgsz_end];
        if let Some(w) = extract_attr(pgsz, "w:w") {
            if let Some(h) = extract_attr(pgsz, "w:h") {
                let width_twip = w.parse::<i32>().unwrap_or(11906);
                let height_twip = h.parse::<i32>().unwrap_or(16838);
                let width_mm = twip_to_mm(width_twip);
                let height_mm = twip_to_mm(height_twip);
                page_setup.paper_size = match (width_mm.round() as i32, height_mm.round() as i32) {
                    (210, 297) => PaperSize::A4,
                    (297, 420) => PaperSize::A3,
                    (216, 279) => PaperSize::Letter,
                    (216, 356) => PaperSize::Legal,
                    (279, 432) => PaperSize::Tabloid,
                    (176, 250) => PaperSize::B5,
                    _ => PaperSize::Custom {
                        width_mm,
                        height_mm,
                    },
                };
            }
        }
        if let Some(orient) = extract_attr(pgsz, "w:orient") {
            page_setup.orientation = if orient == "landscape" {
                Orientation::Landscape
            } else {
                Orientation::Portrait
            };
        }
    }

    // Parse pgMar.
    if let Some(pgmar_start) = sect.find("<w:pgMar") {
        let pgmar_end = sect[pgmar_start..].find('/').unwrap_or(0) + pgmar_start;
        let pgmar = &sect[pgmar_start..pgmar_end];
        page_setup.margins = Margins {
            top: extract_attr(pgmar, "w:top")
                .and_then(|v| v.parse::<i32>().ok())
                .map(twip_to_mm)
                .unwrap_or(25.4),
            bottom: extract_attr(pgmar, "w:bottom")
                .and_then(|v| v.parse::<i32>().ok())
                .map(twip_to_mm)
                .unwrap_or(25.4),
            left: extract_attr(pgmar, "w:left")
                .and_then(|v| v.parse::<i32>().ok())
                .map(twip_to_mm)
                .unwrap_or(25.4),
            right: extract_attr(pgmar, "w:right")
                .and_then(|v| v.parse::<i32>().ok())
                .map(twip_to_mm)
                .unwrap_or(25.4),
        };
    }

    page_setup
}

/// Parse a table from OOXML.
fn parse_table(
    xml: &str,
    relationships: &HashMap<String, String>,
    media_images: &HashMap<String, Vec<u8>>,
) -> BlockNode {
    let mut rows = Vec::new();
    let mut rest = xml;
    while let Some(tr_start) = rest.find("<w:tr") {
        let tr_open_end = rest[tr_start..].find('>').unwrap_or(0) + tr_start;
        let tr_end = rest.find("</w:tr>").unwrap_or(rest.len());
        let tr_xml = &rest[tr_open_end + 1..tr_end];

        let mut cells = Vec::new();
        let mut tr_rest = tr_xml;
        while let Some(tc_start) = tr_rest.find("<w:tc") {
            let tc_open_end = tr_rest[tc_start..].find('>').unwrap_or(0) + tc_start;
            let tc_end = tr_rest.find("</w:tc>").unwrap_or(tr_rest.len());
            let tc_xml = &tr_rest[tc_start..tc_end + "</w:tc>".len()];
            let tc_inner = &tr_rest[tc_open_end + 1..tc_end];

            let mut cell = TableCell::default();

            // Parse tcPr for colspan/rowspan.
            if let Some(tcpr_start) = tc_xml.find("<w:tcPr") {
                let tcpr_end = tc_xml.find("</w:tcPr>").unwrap_or(tc_xml.len());
                let tcpr = &tc_xml[tcpr_start..tcpr_end];
                cell.colspan = extract_attr(tcpr, "w:gridSpan")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(1);
                if tcpr.contains("w:vMerge") {
                    let vmerge = &tcpr[tcpr.find("<w:vMerge").unwrap_or(0)..];
                    if extract_attr(vmerge, "w:val").is_some() {
                        cell.rowspan = 1; // restart
                    } else {
                        cell.rowspan = 0; // continue (merged)
                    }
                }
            }

            // Parse cell content as block nodes.
            let mut cell_blocks = Vec::new();
            let mut cell_rest = tc_inner;
            while let Some(p_start) = cell_rest.find("<w:p") {
                let p_open_end = cell_rest[p_start..].find('>').unwrap_or(0) + p_start;
                let p_end = cell_rest.find("</w:p>").unwrap_or(cell_rest.len());
                let p_xml = &cell_rest[p_start..p_end + "</w:p>".len()];
                let p_inner = &cell_rest[p_open_end + 1..p_end];
                cell_blocks.push(parse_paragraph_tdm(
                    p_xml,
                    p_inner,
                    relationships,
                    media_images,
                ));
                cell_rest = &cell_rest[p_end + "</w:p>".len()..];
            }
            cell.content = cell_blocks;
            cells.push(cell);
            tr_rest = &tr_rest[tc_end + "</w:tc>".len()..];
        }

        rows.push(TableRow { cells });
        rest = &rest[tr_end + "</w:tr>".len()..];
    }

    BlockNode::Table { rows }
}

fn parse_paragraph_tdm(
    paragraph: &str,
    inner: &str,
    relationships: &HashMap<String, String>,
    media_images: &HashMap<String, Vec<u8>>,
) -> BlockNode {
    let mut attrs = ParagraphAttrs::default();

    // Parse paragraph properties.
    if let Some(ppr_start) = inner.find("<w:pPr>") {
        let ppr_end = inner.find("</w:pPr>").unwrap_or(inner.len());
        let ppr = &inner[ppr_start + "<w:pPr>".len()..ppr_end];

        // Alignment.
        if let Some(jc_start) = ppr.find("<w:jc") {
            let jc_tag = &ppr[jc_start..];
            let tag_end = jc_tag
                .find('/')
                .or_else(|| jc_tag.find('>'))
                .unwrap_or(jc_tag.len());
            if let Some(val) = extract_attr(&jc_tag[..tag_end], "w:val") {
                attrs.alignment = match val.as_str() {
                    "center" => Alignment::Center,
                    "right" => Alignment::Right,
                    "both" => Alignment::Justify,
                    _ => Alignment::Left,
                };
            }
        }

        // Indentation.
        if let Some(ind_start) = ppr.find("<w:ind") {
            let ind_tag = &ppr[ind_start..];
            let ind_end = ind_tag
                .find('/')
                .or_else(|| ind_tag.find('>'))
                .unwrap_or(ind_tag.len());
            let ind = &ind_tag[..ind_end];
            attrs.indent_left = extract_attr(ind, "w:left")
                .and_then(|v| v.parse::<f32>().ok())
                .map(|tw| twip_to_mm(tw as i32))
                .unwrap_or(0.0);
            attrs.indent_right = extract_attr(ind, "w:right")
                .and_then(|v| v.parse::<f32>().ok())
                .map(|tw| twip_to_mm(tw as i32))
                .unwrap_or(0.0);
            attrs.indent_first_line = extract_attr(ind, "w:firstLine")
                .and_then(|v| v.parse::<f32>().ok())
                .map(|tw| twip_to_mm(tw as i32))
                .unwrap_or(0.0);
        }

        // Style.
        if let Some(pstyle_start) = ppr.find("<w:pStyle") {
            let tag = &ppr[pstyle_start..];
            let tag_end = tag.find('/').or_else(|| tag.find('>')).unwrap_or(tag.len());
            if let Some(val) = extract_attr(&tag[..tag_end], "w:val") {
                attrs.style_id = Some(val);
            }
        }
    }

    // Parse runs (text with marks).
    let mut inlines = parse_runs_tdm(inner, relationships, media_images);

    // Check for drawing elements (images).
    if let Some(img_node) = parse_drawing(inner, relationships, media_images) {
        inlines.push(img_node);
    }

    if let Some(level) = heading_level(paragraph) {
        BlockNode::Heading {
            level,
            content: inlines,
            attrs,
        }
    } else if let Some(style_id) = &attrs.style_id {
        // Map style IDs to semantic block types.
        match style_id.as_str() {
            "Quote" => BlockNode::BlockQuote {
                content: vec![BlockNode::Paragraph {
                    content: inlines,
                    attrs: attrs.clone(),
                }],
            },
            "ListParagraph" => {
                // Treat as paragraph for now; list detection requires numbering.xml.
                BlockNode::Paragraph {
                    content: inlines,
                    attrs,
                }
            }
            _ => BlockNode::Paragraph {
                content: inlines,
                attrs,
            },
        }
    } else {
        BlockNode::Paragraph {
            content: inlines,
            attrs,
        }
    }
}

fn parse_runs_tdm(
    inner: &str,
    _relationships: &HashMap<String, String>,
    _media_images: &HashMap<String, Vec<u8>>,
) -> Vec<InlineNode> {
    let mut nodes = Vec::new();
    let mut rest = inner;

    while let Some(start) = rest.find("<w:r") {
        rest = &rest[start..];
        let Some(close_start) = rest.find("</w:r>") else {
            break;
        };
        let run = &rest[..close_start + "</w:r>".len()];
        let marks = parse_run_marks_tdm(run);
        for text in parse_text_nodes(run) {
            if text.is_empty() {
                continue;
            }
            nodes.push(InlineNode::Text {
                text: xml_util::decode_xml(&text),
                marks: marks.clone(),
            });
        }
        if run.contains("<w:tab") {
            nodes.push(InlineNode::Text {
                text: "\t".to_string(),
                marks: Marks::default(),
            });
        }
        if run.contains("<w:br") && !run.contains("w:type=\"page\"") {
            nodes.push(InlineNode::HardBreak);
        }
        rest = &rest[close_start + "</w:r>".len()..];
    }

    nodes
}

fn parse_run_marks_tdm(run: &str) -> Marks {
    let mut marks = Marks::default();

    // Extract rPr block.
    let rpr_start = run.find("<w:rPr>");
    let rpr = if let Some(start) = rpr_start {
        let end = run.find("</w:rPr>").unwrap_or(run.len());
        &run[start..end]
    } else {
        return marks;
    };

    if rpr.contains("<w:b") && !rpr.contains(r#"<w:b w:val="false""#) {
        marks.bold = true;
    }
    if rpr.contains("<w:i") && !rpr.contains(r#"<w:i w:val="false""#) {
        marks.italic = true;
    }
    if rpr.contains("<w:u") && !rpr.contains(r#"<w:u w:val="none""#) {
        marks.underline = true;
    }
    if rpr.contains("<w:strike") && !rpr.contains(r#"<w:strike w:val="false""#) {
        marks.strikethrough = true;
    }

    // Font size: <w:sz w:val="24"/> (half-points).
    if let Some(sz_start) = rpr.find("<w:sz") {
        let tag = &rpr[sz_start..];
        let tag_end = tag.find('/').or_else(|| tag.find('>')).unwrap_or(tag.len());
        if let Some(val) = extract_attr(&tag[..tag_end], "w:val") {
            if let Ok(half_pts) = val.parse::<f32>() {
                marks.font_size = Some(half_pts / 2.0);
            }
        }
    }

    // Font color: <w:color w:val="FF0000"/>.
    if let Some(color_start) = rpr.find("<w:color") {
        let tag = &rpr[color_start..];
        let tag_end = tag.find('/').or_else(|| tag.find('>')).unwrap_or(tag.len());
        if let Some(val) = extract_attr(&tag[..tag_end], "w:val") {
            marks.text_color = Some(format!("#{}", val));
        }
    }

    // Background/shading: <w:shd w:fill="FFFF00"/>.
    if let Some(shd_start) = rpr.find("<w:shd") {
        let tag = &rpr[shd_start..];
        let tag_end = tag.find('/').or_else(|| tag.find('>')).unwrap_or(tag.len());
        if let Some(val) = extract_attr(&tag[..tag_end], "w:fill") {
            if val != "auto" {
                marks.background_color = Some(format!("#{}", val));
            }
        }
    }

    marks
}

/// Parse a drawing element to extract an inline image.
fn parse_drawing(
    inner: &str,
    relationships: &HashMap<String, String>,
    media_images: &HashMap<String, Vec<u8>>,
) -> Option<InlineNode> {
    let draw_start = inner.find("<w:drawing")?;
    let draw_end = inner.find("</w:drawing>")?;
    let drawing = &inner[draw_start..draw_end + "</w:drawing>".len()];

    // Find the blip reference.
    let blip_start = drawing.find("<a:blip")?;
    let blip_tag = &drawing[blip_start..];
    let blip_end = blip_tag.find('/').or_else(|| blip_tag.find('>'))?;
    let embed_id = extract_attr(&blip_tag[..blip_end], "r:embed")?;

    // Resolve the target from relationships.
    let target = relationships.get(&embed_id)?;
    let media_path = if target.starts_with('/') {
        format!("word{}", target)
    } else {
        format!("word/{}", target)
    };

    // Try to get the image data.
    if let Some(data) = media_images.get(&media_path) {
        Some(InlineNode::InlineImage {
            source: ImageSource::Embedded { data: data.clone() },
            alt: None,
            width: None,
            height: None,
        })
    } else {
        // Fallback: reference by path.
        Some(InlineNode::InlineImage {
            source: ImageSource::Referenced { path: media_path },
            alt: None,
            width: None,
            height: None,
        })
    }
}

fn parse_text_nodes(run: &str) -> Vec<String> {
    let mut texts = Vec::new();
    let mut rest = run;

    while let Some(start) = rest.find("<w:t") {
        rest = &rest[start..];
        let Some(open_end) = rest.find('>') else {
            break;
        };
        let Some(close_start) = rest.find("</w:t>") else {
            break;
        };
        texts.push(rest[open_end + 1..close_start].to_string());
        rest = &rest[close_start + "</w:t>".len()..];
    }

    texts
}

fn heading_level(paragraph: &str) -> Option<u8> {
    for level in 1..=9 {
        if paragraph.contains(&format!(r#"w:val="Heading{level}""#))
            || paragraph.contains(&format!(r#"w:val="heading {level}""#))
        {
            return Some(level.min(6));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Export helpers
// ---------------------------------------------------------------------------
