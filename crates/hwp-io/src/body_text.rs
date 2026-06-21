use crate::controls::Control;
use crate::doc_info::DocInfo;
use crate::error::HwpError;
use crate::header::Version;
use crate::record::*;

/// A parsed paragraph from HWP body text.
#[derive(Debug, Clone)]
pub struct HwpParagraph {
    pub text: String,
    pub char_shape_id: u16,
    pub para_shape_id: u16,
    pub style_id: u8,
    pub controls: Vec<Control>,
    pub char_shape_ranges: Vec<(usize, u16)>, // (char_offset, shape_id)
}

/// Parse all sections into paragraphs.
pub fn parse_sections(
    sections: &[Vec<u8>],
    doc_info: &DocInfo,
    version: &Version,
) -> Result<Vec<Vec<HwpParagraph>>, HwpError> {
    let mut all_sections = Vec::new();
    for section_data in sections {
        let paragraphs = parse_section(section_data, doc_info, version)?;
        all_sections.push(paragraphs);
    }
    Ok(all_sections)
}

fn parse_section(
    data: &[u8],
    _doc_info: &DocInfo,
    _version: &Version,
) -> Result<Vec<HwpParagraph>, HwpError> {
    let records = Record::parse_all(data)?;
    let mut paragraphs = Vec::new();
    let mut current_para: Option<HwpParagraph> = None;
    let mut i = 0;

    while i < records.len() {
        let record = &records[i];

        match record.tag_id {
            TAG_PARA_HEADER => {
                // Finish previous paragraph
                if let Some(para) = current_para.take() {
                    paragraphs.push(para);
                }

                let mut reader = RecordReader::new(&record.payload);
                let _char_count_and_flags = reader.read_u32();
                let _control_mask = reader.read_u32();
                let para_shape_id = reader.read_u16();
                let style_id = reader.read_u8();

                current_para = Some(HwpParagraph {
                    text: String::new(),
                    char_shape_id: 0,
                    para_shape_id,
                    style_id,
                    controls: Vec::new(),
                    char_shape_ranges: Vec::new(),
                });
            }
            TAG_PARA_TEXT => {
                if let Some(ref mut para) = current_para {
                    para.text = decode_para_text(&record.payload);
                }
            }
            TAG_PARA_CHAR_SHAPE => {
                if let Some(ref mut para) = current_para {
                    let mut reader = RecordReader::new(&record.payload);
                    while reader.remaining() >= 4 {
                        let offset = reader.read_u16() as usize;
                        let shape_id = reader.read_u16();
                        para.char_shape_ranges.push((offset, shape_id));
                    }
                    if !para.char_shape_ranges.is_empty() {
                        para.char_shape_id = para.char_shape_ranges[0].1;
                    }
                }
            }
            TAG_CTRL_HEADER => {
                // Parse control — look ahead for table/picture data
                if let Some(ref mut para) = current_para {
                    if let Some(ctrl) = crate::controls::parse_control(&records, &mut i) {
                        para.controls.push(ctrl);
                        continue; // Don't increment i, parse_control already advanced it
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    if let Some(para) = current_para.take() {
        paragraphs.push(para);
    }

    Ok(paragraphs)
}

/// Decode paragraph text from UTF-16LE with control character handling.
fn decode_para_text(data: &[u8]) -> String {
    if data.len() < 2 {
        return String::new();
    }

    let u16_iter = data
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]));

    let mut result = String::new();
    for code in u16_iter {
        match code {
            0x0000 => {}                            // NULL — skip
            0x000D => {}                            // Paragraph break — skip
            0x000A => result.push('\n'),            // Line break
            0x0009 => result.push('\t'),            // Tab
            0x001E => result.push('\u{00A0}'),      // Non-breaking space
            0x001F => result.push(' '),             // Fixed-width space
            0x0018 => result.push('-'),             // Hyphen
            0x0003..=0x0017 | 0x0019..=0x001D => {} // Extended/inline control characters — skip
            _ => {
                if let Some(ch) = char::from_u32(code as u32) {
                    result.push(ch);
                }
            }
        }
    }

    result
}
