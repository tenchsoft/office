use crate::body_text::HwpParagraph;
use crate::controls::{Control, PictureControl, SectionDef, TableControl};
use crate::doc_info::DocInfo;
use crate::error::HwpError;
use crate::header::FileHeader;
use tench_document_core::tdm::*;

/// Convert parsed HWP data into a TenchDocument.
pub fn convert(
    sections: Vec<Vec<HwpParagraph>>,
    doc_info: DocInfo,
    images: Vec<(String, Vec<u8>)>,
    _header: &FileHeader,
) -> Result<TenchDocument, HwpError> {
    let mut blocks = Vec::new();
    let mut page_setup = PageSetup::default();

    for section in &sections {
        for para in section {
            // Process controls first (section defs, tables, images, etc.)
            for ctrl in &para.controls {
                if let Control::SectionDef(sec) = ctrl {
                    page_setup = section_def_to_page_setup(sec);
                }
            }
            for ctrl in &para.controls {
                match ctrl {
                    Control::Table(table) => {
                        blocks.push(table_to_block(table));
                    }
                    Control::Picture(pic) => {
                        if let Some(block) = picture_to_block(pic, &images) {
                            blocks.push(block);
                        }
                    }
                    Control::Header(text) | Control::Footer(text) => {
                        // Headers/footers are metadata, not content blocks
                        let _ = text;
                    }
                    Control::Footnote(text) => {
                        blocks.push(BlockNode::Footnote {
                            number: 0,
                            content: vec![InlineNode::Text {
                                text: text.clone(),
                                marks: Marks::default(),
                            }],
                        });
                    }
                    Control::TextBox(text) if !text.trim().is_empty() => {
                        // TextBox content becomes a paragraph
                        blocks.push(BlockNode::Paragraph {
                            attrs: ParagraphAttrs::default(),
                            content: vec![InlineNode::Text {
                                text: text.clone(),
                                marks: Marks::default(),
                            }],
                        });
                    }
                    _ => {}
                }
            }
            if !para.text.trim().is_empty() {
                blocks.push(paragraph_to_block(para, &doc_info));
            }
        }
    }

    let title = extract_title(&sections);
    let mut doc = TenchDocument::new(&title);
    doc.content = blocks;
    doc.page_setup = page_setup;
    Ok(doc)
}

fn section_def_to_page_setup(sec: &SectionDef) -> PageSetup {
    let width_mm = sec.width as f64 / 7200.0 * 25.4;
    let height_mm = sec.height as f64 / 7200.0 * 25.4;
    let paper_size = if (width_mm - 210.0).abs() < 5.0 && (height_mm - 297.0).abs() < 5.0 {
        PaperSize::A4
    } else if (width_mm - 215.9).abs() < 5.0 && (height_mm - 279.4).abs() < 5.0 {
        PaperSize::Letter
    } else {
        PaperSize::A4
    };
    PageSetup {
        paper_size,
        orientation: if sec.landscape {
            Orientation::Landscape
        } else {
            Orientation::Portrait
        },
        margins: Margins {
            top: sec.top_margin as f32 / 7200.0 * 25.4,
            bottom: sec.bottom_margin as f32 / 7200.0 * 25.4,
            left: sec.left_margin as f32 / 7200.0 * 25.4,
            right: sec.right_margin as f32 / 7200.0 * 25.4,
        },
    }
}

fn paragraph_to_block(para: &HwpParagraph, doc_info: &DocInfo) -> BlockNode {
    let alignment = get_alignment(para, doc_info);
    let (indent_left, indent_right, indent_first_line, line_height, space_before, space_after) =
        get_para_attrs(para, doc_info);
    let attrs = ParagraphAttrs {
        alignment,
        indent_left,
        indent_right,
        indent_first_line,
        line_height,
        space_before,
        space_after,
        style_id: None,
    };
    let inline = text_to_inline_nodes(para, doc_info);
    let para_shape = doc_info.para_shapes.get(para.para_shape_id as usize);
    if let Some(shape) = para_shape {
        if shape.heading_level > 0 && shape.heading_level <= 6 {
            return BlockNode::Heading {
                level: shape.heading_level,
                attrs,
                content: inline,
            };
        }
        // Handle numbered/bulleted lists
        if shape.numbering_id > 0 {
            return BlockNode::OrderedList {
                items: vec![ListItem {
                    content: inline.clone(),
                    children: Vec::new(),
                }],
                start: 1,
            };
        }
    }
    BlockNode::Paragraph {
        attrs,
        content: inline,
    }
}

fn get_alignment(para: &HwpParagraph, doc_info: &DocInfo) -> Alignment {
    let para_shape = doc_info.para_shapes.get(para.para_shape_id as usize);
    if let Some(shape) = para_shape {
        match shape.alignment {
            0 => Alignment::Justify,
            1 => Alignment::Left,
            2 => Alignment::Right,
            3 => Alignment::Center,
            _ => Alignment::Left,
        }
    } else {
        Alignment::Left
    }
}

/// Extract paragraph attributes from ParaShape.
fn get_para_attrs(
    para: &HwpParagraph,
    doc_info: &DocInfo,
) -> (f32, f32, f32, Option<f32>, f32, f32) {
    let para_shape = doc_info.para_shapes.get(para.para_shape_id as usize);
    if let Some(shape) = para_shape {
        let mm_to_px = (96.0 / 25.4) as f32;
        let indent_left = shape.margin_left as f32 / 7200.0 * 25.4 * mm_to_px;
        let indent_right = shape.margin_right as f32 / 7200.0 * 25.4 * mm_to_px;
        let indent_first_line = shape.indent as f32 / 7200.0 * 25.4 * mm_to_px;
        let line_height = match shape.line_spacing_type {
            0 => Some(shape.line_spacing as f32 / 100.0), // ratio
            1 => Some(shape.line_spacing as f32 / 100.0), // fixed – approximate
            _ => Some(1.6),
        };
        let space_before = shape.margin_top as f32 / 7200.0 * 25.4 * mm_to_px;
        let space_after = shape.margin_bottom as f32 / 7200.0 * 25.4 * mm_to_px;
        (
            indent_left,
            indent_right,
            indent_first_line,
            line_height,
            space_before,
            space_after,
        )
    } else {
        (0.0, 0.0, 0.0, Some(1.6), 0.0, 0.0)
    }
}

fn text_to_inline_nodes(para: &HwpParagraph, doc_info: &DocInfo) -> Vec<InlineNode> {
    if para.text.trim().is_empty() {
        return Vec::new();
    }

    // If we have per-character shape ranges, use them for rich formatting
    if !para.char_shape_ranges.is_empty() {
        return build_rich_inline_nodes(para, doc_info);
    }

    // Fallback: single shape for the whole paragraph
    let char_shape = doc_info.char_shapes.get(para.char_shape_id as usize);
    let marks = build_marks_from_shape(char_shape);
    vec![InlineNode::Text {
        text: para.text.clone(),
        marks,
    }]
}

/// Build inline nodes with per-character formatting using char_shape_ranges.
fn build_rich_inline_nodes(para: &HwpParagraph, doc_info: &DocInfo) -> Vec<InlineNode> {
    let mut nodes = Vec::new();
    let text = &para.text;
    let ranges = &para.char_shape_ranges;

    let mut current_start = 0usize;
    for (i, (offset, shape_id)) in ranges.iter().enumerate() {
        let end = *offset.min(&text.len());
        if end > current_start {
            let slice = text[current_start..end].to_string();
            if !slice.is_empty() {
                let shape = doc_info.char_shapes.get(*shape_id as usize);
                let marks = build_marks_from_shape(shape);
                nodes.push(InlineNode::Text { text: slice, marks });
            }
        }
        current_start = end;

        // Handle last range: extend to end of text
        if i == ranges.len() - 1 && current_start < text.len() {
            let slice = text[current_start..].to_string();
            if !slice.is_empty() {
                let shape = doc_info.char_shapes.get(*shape_id as usize);
                let marks = build_marks_from_shape(shape);
                nodes.push(InlineNode::Text { text: slice, marks });
            }
        }
    }

    if nodes.is_empty() {
        nodes.push(InlineNode::Text {
            text: text.clone(),
            marks: Marks::default(),
        });
    }
    nodes
}

/// Build Marks from a CharShape reference.
fn build_marks_from_shape(char_shape: Option<&crate::char_shape::CharShape>) -> Marks {
    let mut marks = Marks::default();
    if let Some(shape) = char_shape {
        marks.bold = shape.bold;
        marks.italic = shape.italic;
        marks.underline = shape.underline_type > 0;
        if shape.base_size > 0 {
            marks.font_size = Some(shape.base_size as f32 / 100.0);
        }
        let (r, g, b) = crate::char_shape::CharShape::color_to_rgb(shape.text_color);
        if r > 0 || g > 0 || b > 0 {
            marks.text_color = Some(format!("#{:02X}{:02X}{:02X}", r, g, b));
        }
    }
    marks
}

fn table_to_block(table: &TableControl) -> BlockNode {
    let mut rows = Vec::new();
    for row_idx in 0..table.rows as usize {
        let mut row_cells = Vec::new();
        for cell in &table.cells {
            if cell.row as usize == row_idx {
                row_cells.push(TableCell {
                    content: vec![BlockNode::Paragraph {
                        attrs: ParagraphAttrs::default(),
                        content: vec![InlineNode::Text {
                            text: cell.text.clone(),
                            marks: Marks::default(),
                        }],
                    }],
                    colspan: cell.colspan.max(1) as u32,
                    rowspan: cell.rowspan.max(1) as u32,
                });
            }
        }
        rows.push(TableRow { cells: row_cells });
    }
    BlockNode::Table { rows }
}

fn picture_to_block(pic: &PictureControl, images: &[(String, Vec<u8>)]) -> Option<BlockNode> {
    let idx = pic.bin_data_id as usize;
    let data = if idx < images.len() {
        images[idx].1.clone()
    } else {
        Vec::new()
    };
    let alt: Option<String> = if data.is_empty() {
        Some("[Image]".into())
    } else {
        None
    };
    Some(BlockNode::Image {
        source: ImageSource::Embedded { data },
        alt,
        width: None,
        height: None,
    })
}

fn extract_title(sections: &[Vec<HwpParagraph>]) -> String {
    for section in sections {
        for para in section {
            let trimmed = para.text.trim();
            if !trimmed.is_empty() && trimmed.len() < 100 {
                return trimmed.to_string();
            }
        }
    }
    "Untitled".into()
}
