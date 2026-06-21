use crate::error::HwpError;
use crate::header::FileHeader;
use crate::record::*;
use tench_document_core::tdm::*;

pub fn convert(doc: &TenchDocument) -> Result<Vec<u8>, HwpError> {
    let header = FileHeader::default_v5();
    let doc_info_records = build_doc_info(doc);
    let section_records = build_section(doc);
    serialize_cfb(&header, &doc_info_records, &section_records)
}

fn build_doc_info(_doc: &TenchDocument) -> Vec<Record> {
    let mut records = Vec::new();
    let mut props = RecordWriter::new();
    props.write_u16(1);
    props.write_u16(1);
    props.write_u16(1);
    props.write_u16(1);
    props.write_u16(1);
    props.write_u16(1);
    props.write_u16(1);
    props.write_u32(0);
    props.write_u32(0);
    props.write_u32(0);
    records.push(Record {
        tag_id: TAG_DOCUMENT_PROPERTIES,
        level: 0,
        payload: props.into_payload(),
    });

    let mut mappings = RecordWriter::new();
    mappings.write_u16(1);
    mappings.write_u16(0);
    mappings.write_u16(1);
    mappings.write_u16(0);
    mappings.write_u16(0);
    mappings.write_u16(0);
    mappings.write_u16(1);
    mappings.write_u16(1);
    records.push(Record {
        tag_id: TAG_ID_MAPPINGS,
        level: 0,
        payload: mappings.into_payload(),
    });

    let mut font = RecordWriter::new();
    font.write_u8(1);
    font.write_string("Arial");
    records.push(Record {
        tag_id: TAG_FACE_NAME,
        level: 0,
        payload: font.into_payload(),
    });

    let mut cs = RecordWriter::new();
    for _ in 0..7 {
        cs.write_u16(0);
        cs.write_u8(0);
        cs.write_i8(0);
        cs.write_i8(0);
        cs.write_i8(0);
    }
    cs.write_i32(1000);
    cs.write_u32(0);
    cs.write_i8(0);
    cs.write_i8(0);
    cs.write_u32(0);
    cs.write_u32(0);
    cs.write_u32(0);
    cs.write_u32(0);
    records.push(Record {
        tag_id: TAG_CHAR_SHAPE,
        level: 0,
        payload: cs.into_payload(),
    });

    let mut ps = RecordWriter::new();
    ps.write_u32(0x04);
    ps.write_i32(0);
    ps.write_i32(0);
    ps.write_i32(0);
    ps.write_i32(0);
    ps.write_i32(0);
    ps.write_i32(160);
    ps.write_u16(0);
    ps.write_u16(0);
    ps.write_u16(0);
    records.push(Record {
        tag_id: TAG_PARA_SHAPE,
        level: 0,
        payload: ps.into_payload(),
    });

    let mut st = RecordWriter::new();
    st.write_string("바탕글");
    st.write_string("Normal");
    st.write_u8(0);
    st.write_u8(0);
    st.write_u16(0);
    st.write_u16(0);
    records.push(Record {
        tag_id: TAG_STYLE,
        level: 0,
        payload: st.into_payload(),
    });

    records
}

fn build_section(doc: &TenchDocument) -> Vec<Record> {
    let mut records = Vec::new();
    let mut ctrl_hdr = RecordWriter::new();
    ctrl_hdr.write_bytes(&[0; 20]);
    records.push(Record {
        tag_id: TAG_CTRL_HEADER,
        level: 0,
        payload: ctrl_hdr.into_payload(),
    });

    // Write page definition from document page_setup
    let mut pd = RecordWriter::new();
    let ps = &doc.page_setup;
    let (w_mm, h_mm) = ps.paper_size.dimensions_mm();
    let mm_to_hwpu = |mm: f32| -> i32 { (mm as f64 * 7200.0 / 25.4) as i32 };
    let (w, h) = match ps.orientation {
        tench_document_core::Orientation::Portrait => (w_mm, h_mm),
        tench_document_core::Orientation::Landscape => (h_mm, w_mm),
    };
    pd.write_i32(mm_to_hwpu(w));
    pd.write_i32(mm_to_hwpu(h));
    pd.write_i32(mm_to_hwpu(ps.margins.left));
    pd.write_i32(mm_to_hwpu(ps.margins.right));
    pd.write_i32(mm_to_hwpu(ps.margins.top));
    pd.write_i32(mm_to_hwpu(ps.margins.bottom));
    pd.write_i32(0); // header_offset
    pd.write_i32(0); // footer_offset
    pd.write_i32(0); // bookbinding_offset
    let flags: u32 = if matches!(ps.orientation, tench_document_core::Orientation::Landscape) {
        0x01
    } else {
        0x00
    };
    pd.write_u32(flags);
    records.push(Record {
        tag_id: TAG_PAGE_DEF,
        level: 1,
        payload: pd.into_payload(),
    });

    for block in &doc.content {
        match block {
            BlockNode::Table { rows } => {
                write_table_records(&mut records, rows);
            }
            BlockNode::Image {
                source,
                alt,
                width,
                height,
            } => {
                write_image_records(
                    &mut records,
                    source,
                    alt.as_deref().unwrap_or(""),
                    width.unwrap_or(0.0),
                    height.unwrap_or(0.0),
                );
            }
            BlockNode::Footnote { number, content } => {
                write_footnote_records(&mut records, *number, content);
            }
            BlockNode::HorizontalRule => {
                // Horizontal rule as a paragraph with a separator
                write_text_paragraph(&mut records, "—".repeat(40));
            }
            BlockNode::PageBreak => {
                // Page break as a section definition control
                write_page_break_record(&mut records);
            }
            BlockNode::BlockQuote { content } => {
                for child in content {
                    let text = extract_block_text(child);
                    if !text.is_empty() {
                        write_text_paragraph(&mut records, text);
                    }
                }
            }
            BlockNode::BulletList { items } | BlockNode::OrderedList { items, .. } => {
                for item in items {
                    let text: String = item
                        .content
                        .iter()
                        .filter_map(|n| match n {
                            InlineNode::Text { text, .. } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect();
                    if !text.is_empty() {
                        write_text_paragraph(&mut records, text);
                    }
                }
            }
            BlockNode::TaskList { items } => {
                for item in items {
                    let text: String = item
                        .content
                        .iter()
                        .filter_map(|n| match n {
                            InlineNode::Text { text, .. } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect();
                    if !text.is_empty() {
                        write_text_paragraph(&mut records, text);
                    }
                }
            }
            BlockNode::CodeBlock { code, .. } => {
                if !code.is_empty() {
                    write_text_paragraph(&mut records, code.clone());
                }
            }
            _ => {
                let text = extract_block_text(block);
                if !text.is_empty() {
                    write_text_paragraph(&mut records, text);
                }
            }
        }
    }
    records
}

fn extract_block_text(block: &BlockNode) -> String {
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => content
            .iter()
            .filter_map(|n| match n {
                InlineNode::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect(),
        _ => String::new(),
    }
}

/// Write a simple text paragraph as PARA_HEADER + PARA_TEXT records.
fn write_text_paragraph(records: &mut Vec<Record>, text: String) {
    let mut ph = RecordWriter::new();
    ph.write_u32(text.encode_utf16().count() as u32);
    ph.write_u32(0);
    ph.write_u16(0);
    ph.write_u8(0);
    records.push(Record {
        tag_id: TAG_PARA_HEADER,
        level: 0,
        payload: ph.into_payload(),
    });
    let mut pt = RecordWriter::new();
    for ch in text.encode_utf16() {
        pt.write_u16(ch);
    }
    records.push(Record {
        tag_id: TAG_PARA_TEXT,
        level: 1,
        payload: pt.into_payload(),
    });
}

/// Write a table as HWP table control records.
fn write_table_records(records: &mut Vec<Record>, rows: &[TableRow]) {
    if rows.is_empty() {
        return;
    }
    let row_count = rows.len() as u16;
    let col_count = rows[0].cells.len() as u16;

    // CTRL_HEADER for table
    let mut ctrl = RecordWriter::new();
    ctrl.write_bytes(&[0; 12]);
    ctrl.write_bytes(b"tbl ");
    records.push(Record {
        tag_id: TAG_CTRL_HEADER,
        level: 0,
        payload: ctrl.into_payload(),
    });

    // TABLE record
    let mut tbl = RecordWriter::new();
    tbl.write_u16(row_count);
    tbl.write_u16(col_count);
    // Property flags, column widths, etc.
    tbl.write_u16(0); // flags
    tbl.write_u16(0); // property
    for _ in 0..col_count {
        tbl.write_i32(3000); // default column width in HWPU
    }
    records.push(Record {
        tag_id: TAG_TABLE,
        level: 1,
        payload: tbl.into_payload(),
    });

    // Write cells
    for (row_idx, row) in rows.iter().enumerate() {
        for (col_idx, cell) in row.cells.iter().enumerate() {
            let mut lh = RecordWriter::new();
            lh.write_u16(1); // para_count
            lh.write_u16(col_idx as u16);
            lh.write_u16(row_idx as u16);
            lh.write_u16(1); // colspan
            lh.write_u16(1); // rowspan
            lh.write_i32(3000); // width
            lh.write_i32(500); // height
            records.push(Record {
                tag_id: TAG_LIST_HEADER,
                level: 2,
                payload: lh.into_payload(),
            });

            // Cell paragraph - extract text from cell content blocks
            let text: String = cell
                .content
                .iter()
                .map(extract_block_text)
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            let mut ph = RecordWriter::new();
            ph.write_u32(text.encode_utf16().count() as u32);
            ph.write_u32(0);
            ph.write_u16(0);
            ph.write_u8(0);
            records.push(Record {
                tag_id: TAG_PARA_HEADER,
                level: 3,
                payload: ph.into_payload(),
            });
            let mut pt = RecordWriter::new();
            for ch in text.encode_utf16() {
                pt.write_u16(ch);
            }
            records.push(Record {
                tag_id: TAG_PARA_TEXT,
                level: 4,
                payload: pt.into_payload(),
            });
        }
    }
}

/// Write an image as HWP picture control records.
fn write_image_records(
    records: &mut Vec<Record>,
    source: &ImageSource,
    alt: &str,
    width: f32,
    height: f32,
) {
    // CTRL_HEADER for picture
    let mut ctrl = RecordWriter::new();
    ctrl.write_bytes(&[0; 12]);
    ctrl.write_bytes(b"$pic");
    records.push(Record {
        tag_id: TAG_CTRL_HEADER,
        level: 0,
        payload: ctrl.into_payload(),
    });

    // SHAPE_COMPONENT_PICTURE record
    let mut pic = RecordWriter::new();
    // Border type, thickness, color (12 bytes)
    pic.write_i32(0); // border_type
    pic.write_i32(0); // border_thickness
    pic.write_u32(0); // border_color
                      // Bounding rect (16 bytes)
    pic.write_i32(0); // left
    pic.write_i32(0); // right
    pic.write_i32(0); // top
    pic.write_i32(0); // bottom
                      // Center offsets
    pic.write_i32(0); // center_x
    pic.write_i32(0); // center_y
                      // Dimensions in HWPU (1/7200 inch)
    let px_to_hwpu = |px: f32| -> i32 { (px * 7200.0 / 96.0) as i32 };
    pic.write_i32(px_to_hwpu(width));
    pic.write_i32(px_to_hwpu(height));
    // bin_data_id (placeholder 0)
    pic.write_u16(0);
    pic.write_u16(0); // pic_type
                      // Remaining fields
    pic.write_u32(0); // flags
    pic.write_u32(0); // effects
    records.push(Record {
        tag_id: TAG_SHAPE_COMPONENT_PICTURE,
        level: 1,
        payload: pic.into_payload(),
    });

    // Alt text as a paragraph
    if !alt.is_empty() {
        write_text_paragraph(records, alt.to_string());
    }

    let _ = source; // Image data would need to be stored in BinData stream
}

/// Write a footnote control.
fn write_footnote_records(records: &mut Vec<Record>, number: u32, content: &[InlineNode]) {
    let mut ctrl = RecordWriter::new();
    ctrl.write_bytes(&[0; 12]);
    ctrl.write_bytes(b"fn  ");
    records.push(Record {
        tag_id: TAG_CTRL_HEADER,
        level: 0,
        payload: ctrl.into_payload(),
    });

    let text: String = content
        .iter()
        .filter_map(|n| match n {
            InlineNode::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    let _ = number;
    if !text.is_empty() {
        write_text_paragraph(records, text);
    }
}

/// Write a page break as a section definition.
fn write_page_break_record(records: &mut Vec<Record>) {
    let mut ctrl = RecordWriter::new();
    ctrl.write_bytes(&[0; 12]);
    ctrl.write_bytes(b"secd");
    records.push(Record {
        tag_id: TAG_CTRL_HEADER,
        level: 0,
        payload: ctrl.into_payload(),
    });
    // Minimal page def
    let mut pd = RecordWriter::new();
    pd.write_i32(0);
    pd.write_i32(0);
    pd.write_i32(0);
    pd.write_i32(0);
    pd.write_i32(0);
    pd.write_i32(0);
    pd.write_i32(0);
    pd.write_i32(0);
    pd.write_i32(0);
    pd.write_u32(0);
    records.push(Record {
        tag_id: TAG_PAGE_DEF,
        level: 1,
        payload: pd.into_payload(),
    });
}

fn serialize_cfb(
    header: &FileHeader,
    doc_info_records: &[Record],
    section_records: &[Record],
) -> Result<Vec<u8>, HwpError> {
    use crate::cfb::compress;
    let header_bytes = header.to_bytes();
    let mut di = Vec::new();
    for r in doc_info_records {
        di.extend_from_slice(&r.to_bytes());
    }
    let di_c = compress(&di);
    let mut sd = Vec::new();
    for r in section_records {
        sd.extend_from_slice(&r.to_bytes());
    }
    let sd_c = compress(&sd);

    let mut comp = cfb::CompoundFile::create(std::io::Cursor::new(Vec::new()))
        .map_err(|e| HwpError::Cfb(format!("CFB create failed: {}", e)))?;
    {
        let mut s = comp
            .create_stream("/FileHeader")
            .map_err(|e| HwpError::Cfb(format!("create_stream failed: {}", e)))?;
        std::io::Write::write_all(&mut s, &header_bytes)
            .map_err(|e| HwpError::Cfb(format!("write failed: {}", e)))?;
    }
    {
        let mut s = comp
            .create_stream("/DocInfo")
            .map_err(|e| HwpError::Cfb(format!("create_stream failed: {}", e)))?;
        std::io::Write::write_all(&mut s, &di_c)
            .map_err(|e| HwpError::Cfb(format!("write failed: {}", e)))?;
    }
    {
        // Create parent storage entry first, then the stream
        comp.create_storage("/BodyText")
            .map_err(|e| HwpError::Cfb(format!("create_storage failed: {}", e)))?;
        let mut s = comp
            .create_stream("/BodyText/Section0")
            .map_err(|e| HwpError::Cfb(format!("create_stream failed: {}", e)))?;
        std::io::Write::write_all(&mut s, &sd_c)
            .map_err(|e| HwpError::Cfb(format!("write failed: {}", e)))?;
    }
    comp.flush()
        .map_err(|e| HwpError::Cfb(format!("CFB flush failed: {}", e)))?;
    let cursor = comp.into_inner();
    Ok(cursor.into_inner())
}
