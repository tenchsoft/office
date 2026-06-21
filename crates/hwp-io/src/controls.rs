use crate::record::*;

/// Control objects embedded in paragraphs (tables, images, etc.)
#[derive(Debug, Clone)]
pub enum Control {
    Table(TableControl),
    Picture(PictureControl),
    SectionDef(SectionDef),
    Header(String),
    Footer(String),
    Footnote(String),
    Endnote(String),
    TextBox(String),
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct TableControl {
    pub rows: u16,
    pub cols: u16,
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone)]
pub struct TableCell {
    pub col: u16,
    pub row: u16,
    pub colspan: u16,
    pub rowspan: u16,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct PictureControl {
    pub bin_data_id: u16,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone)]
pub struct SectionDef {
    pub width: i32,
    pub height: i32,
    pub left_margin: i32,
    pub right_margin: i32,
    pub top_margin: i32,
    pub bottom_margin: i32,
    pub landscape: bool,
}

/// Parse a control starting at the CTRL_HEADER record.
/// Returns the control and advances `index` past all child records.
pub fn parse_control(records: &[Record], index: &mut usize) -> Option<Control> {
    if *index >= records.len() {
        return None;
    }

    let ctrl_header = &records[*index];
    if ctrl_header.tag_id != TAG_CTRL_HEADER {
        return None;
    }

    // Read the 4-byte CHID from the control header payload
    if ctrl_header.payload.len() < 20 {
        return None;
    }

    let chid = &ctrl_header.payload[12..16];
    let chid_str = std::str::from_utf8(chid).unwrap_or("????");

    let mut end_idx = *index + 1;
    // Find the extent of child records (level > current)
    let current_level = ctrl_header.level;
    while end_idx < records.len() && records[end_idx].level > current_level {
        end_idx += 1;
    }

    let control = match chid_str {
        "tbl " => parse_table(records, *index + 1, end_idx),
        "$pic" => parse_picture(records, *index + 1, end_idx),
        "secd" => parse_section_def(records, *index + 1, end_idx),
        "head" => parse_text_control(records, *index + 1, end_idx).map(Control::Header),
        "foot" => parse_text_control(records, *index + 1, end_idx).map(Control::Footer),
        "fn  " => parse_text_control(records, *index + 1, end_idx).map(Control::Footnote),
        "en  " => parse_text_control(records, *index + 1, end_idx).map(Control::Endnote),
        "tcmt" => parse_text_control(records, *index + 1, end_idx).map(Control::TextBox),
        _ => Some(Control::Unknown(chid_str.to_string())),
    };

    *index = end_idx - 1; // Will be incremented by caller
    control
}

fn parse_table(records: &[Record], start: usize, end: usize) -> Option<Control> {
    let mut rows = 0u16;
    let mut cols = 0u16;
    let mut cells = Vec::new();

    for record in records.iter().take(end).skip(start) {
        if record.tag_id == TAG_TABLE {
            let mut reader = RecordReader::new(&record.payload);
            rows = reader.read_u16();
            cols = reader.read_u16();
            break;
        }
    }

    // Parse cells from LIST_HEADER records
    for i in start..end {
        if records[i].tag_id == TAG_LIST_HEADER && records[i].payload.len() >= 14 {
            let mut reader = RecordReader::new(&records[i].payload);
            reader.read_u16(); // para_count (or flags)
            let col = reader.read_u16();
            let row = reader.read_u16();
            let colspan = reader.read_u16();
            let rowspan = reader.read_u16();
            reader.read_i32(); // width
            reader.read_i32(); // height

            // Extract text from child paragraphs
            let mut text = String::new();
            for j in (i + 1)..end {
                if records[j].level <= records[i].level {
                    break;
                }
                if records[j].tag_id == TAG_PARA_TEXT {
                    text.push_str(&decode_cell_text(&records[j].payload));
                    text.push('\n');
                }
            }
            text = text.trim_end_matches('\n').to_string();

            cells.push(TableCell {
                col,
                row,
                colspan,
                rowspan,
                text,
            });
        }
    }

    Some(Control::Table(TableControl { rows, cols, cells }))
}

fn parse_picture(records: &[Record], start: usize, end: usize) -> Option<Control> {
    for record in records.iter().take(end).skip(start) {
        if record.tag_id == TAG_SHAPE_COMPONENT_PICTURE {
            let mut reader = RecordReader::new(&record.payload);
            // Skip: border_type(4) + border_thickness(4) + border_color(4)
            // + rect_left(4) + rect_right(4) + rect_top(4) + rect_bottom(4)
            // = 28 bytes before center offsets
            if reader.remaining() < 40 {
                return Some(Control::Picture(PictureControl {
                    bin_data_id: 0,
                    width: 0,
                    height: 0,
                }));
            }
            reader.read_bytes(28);
            let _center_x = reader.read_i32();
            let _center_y = reader.read_i32();
            // Next: width/height in HWPU units (1/7200 inch)
            // If not available, fall through to zero
            let width = if reader.remaining() >= 4 {
                reader.read_i32()
            } else {
                0
            };
            let height = if reader.remaining() >= 4 {
                reader.read_i32()
            } else {
                0
            };
            // Skip to bin_data_id
            // After dimensions there are more fields; bin_data_id is at a fixed offset
            // For robustness, scan for it in remaining payload
            let mut bin_data_id = 0u16;
            while reader.remaining() >= 4 {
                let candidate = reader.read_u16();
                if candidate > 0 && reader.remaining() >= 2 {
                    // Heuristic: bin_data_id is a small non-zero value
                    bin_data_id = candidate;
                    break;
                }
            }
            return Some(Control::Picture(PictureControl {
                bin_data_id,
                width,
                height,
            }));
        }
    }
    Some(Control::Picture(PictureControl {
        bin_data_id: 0,
        width: 0,
        height: 0,
    }))
}

fn parse_section_def(records: &[Record], start: usize, end: usize) -> Option<Control> {
    for record in records.iter().take(end).skip(start) {
        if record.tag_id == TAG_PAGE_DEF {
            let mut reader = RecordReader::new(&record.payload);
            let width = reader.read_i32();
            let height = reader.read_i32();
            let left_margin = reader.read_i32();
            let right_margin = reader.read_i32();
            let top_margin = reader.read_i32();
            let bottom_margin = reader.read_i32();
            reader.read_i32(); // header_offset
            reader.read_i32(); // footer_offset
            reader.read_i32(); // bookbinding_offset
            let flags = reader.read_u32();
            let landscape = (flags & 0x01) != 0;

            return Some(Control::SectionDef(SectionDef {
                width,
                height,
                left_margin,
                right_margin,
                top_margin,
                bottom_margin,
                landscape,
            }));
        }
    }
    None
}

fn parse_text_control(records: &[Record], start: usize, end: usize) -> Option<String> {
    let mut text = String::new();
    for record in records.iter().take(end).skip(start) {
        if record.tag_id == TAG_PARA_TEXT {
            text.push_str(&decode_cell_text(&record.payload));
        }
    }
    Some(text)
}

fn decode_cell_text(data: &[u8]) -> String {
    if data.len() < 2 {
        return String::new();
    }
    data.chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .filter(|&c| (0x20..0xD800).contains(&c) || c > 0xDFFF)
        .filter_map(|c| char::from_u32(c as u32))
        .collect()
}
