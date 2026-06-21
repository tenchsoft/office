use crate::record::RecordReader;

/// Character shape definition from HWP DocInfo.
#[derive(Debug, Clone)]
pub struct CharShape {
    pub font_ids: [u16; 7], // ko, en, cn, jp, other, symbol, user
    pub base_size: i32,
    pub bold: bool,
    pub italic: bool,
    pub underline_type: u8,
    pub text_color: u32,
    pub underline_color: u32,
    pub shade_color: u32,
    pub shadow_color: u32,
}

impl Default for CharShape {
    fn default() -> Self {
        Self {
            font_ids: [0; 7],
            base_size: 1000, // 10pt in 0.1pt units
            bold: false,
            italic: false,
            underline_type: 0,
            text_color: 0x00000000,
            underline_color: 0x00000000,
            shade_color: 0x00000000,
            shadow_color: 0x00000000,
        }
    }
}

impl CharShape {
    pub fn from_record(reader: &mut RecordReader) -> Self {
        let mut font_ids = [0u16; 7];
        for font_id in font_ids.iter_mut() {
            *font_id = reader.read_u16();
            reader.read_u8(); // letter_width_expansion
            reader.read_i8(); // letter_spacing
            reader.read_i8(); // relative_size
            reader.read_i8(); // position
        }
        let base_size = reader.read_i32();
        let flags = reader.read_u32();
        let bold = (flags & (1 << 1)) != 0;
        let italic = (flags & (1 << 0)) != 0;
        let underline_type = ((flags >> 2) & 0x03) as u8;
        reader.read_i8(); // shadow_x
        reader.read_i8(); // shadow_y
        let text_color = reader.read_u32();
        let underline_color = reader.read_u32();
        let shade_color = reader.read_u32();
        let shadow_color = reader.read_u32();

        Self {
            font_ids,
            base_size,
            bold,
            italic,
            underline_type,
            text_color,
            underline_color,
            shade_color,
            shadow_color,
        }
    }

    /// Convert HWP COLORREF (0x00BBGGRR) to RGB tuple.
    pub fn color_to_rgb(color: u32) -> (u8, u8, u8) {
        let r = (color & 0xFF) as u8;
        let g = ((color >> 8) & 0xFF) as u8;
        let b = ((color >> 16) & 0xFF) as u8;
        (r, g, b)
    }

    /// Convert RGB to HWP COLORREF.
    pub fn rgb_to_color(r: u8, g: u8, b: u8) -> u32 {
        r as u32 | ((g as u32) << 8) | ((b as u32) << 16)
    }
}
