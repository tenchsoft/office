use crate::record::RecordReader;

/// Paragraph shape definition from HWP DocInfo.
#[derive(Debug, Clone)]
pub struct ParaShape {
    pub alignment: u8, // 0=both, 1=left, 2=right, 3=center, 4=distribute
    pub margin_left: i32,
    pub margin_right: i32,
    pub indent: i32,
    pub margin_top: i32,
    pub margin_bottom: i32,
    pub line_spacing: i32,
    pub line_spacing_type: u8, // 0=ratio, 1=fixed, 2=space_only, 3=minimum
    pub tab_def_id: u16,
    pub numbering_id: u16,
    pub border_fill_id: u16,
    pub heading_level: u8,
}

impl Default for ParaShape {
    fn default() -> Self {
        Self {
            alignment: 1, // left
            margin_left: 0,
            margin_right: 0,
            indent: 0,
            margin_top: 0,
            margin_bottom: 0,
            line_spacing: 160,    // 160%
            line_spacing_type: 0, // ratio
            tab_def_id: 0,
            numbering_id: 0,
            border_fill_id: 0,
            heading_level: 0,
        }
    }
}

impl ParaShape {
    pub fn from_record(reader: &mut RecordReader) -> Self {
        let flags1 = reader.read_u32();
        let line_spacing_type = (flags1 & 0x03) as u8;
        let alignment = ((flags1 >> 2) & 0x07) as u8;
        let heading_level = ((flags1 >> 25) & 0x07) as u8;

        let doubled_margin_left = reader.read_i32();
        let doubled_margin_right = reader.read_i32();
        let indent = reader.read_i32();
        let doubled_margin_top = reader.read_i32();
        let doubled_margin_bottom = reader.read_i32();
        let line_spacing = reader.read_i32();

        let tab_def_id = reader.read_u16();
        let numbering_id = reader.read_u16();
        let border_fill_id = reader.read_u16();

        Self {
            alignment,
            margin_left: doubled_margin_left / 2,
            margin_right: doubled_margin_right / 2,
            indent,
            margin_top: doubled_margin_top / 2,
            margin_bottom: doubled_margin_bottom / 2,
            line_spacing,
            line_spacing_type,
            tab_def_id,
            numbering_id,
            border_fill_id,
            heading_level,
        }
    }
}
