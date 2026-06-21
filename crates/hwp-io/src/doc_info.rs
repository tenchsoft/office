use crate::char_shape::CharShape;
use crate::error::HwpError;
use crate::header::Version;
use crate::para_shape::ParaShape;
use crate::record::*;

/// Parsed DocInfo — document-wide metadata and style definitions.
#[derive(Debug, Clone, Default)]
pub struct DocInfo {
    pub section_count: u16,
    pub fonts: Vec<FontEntry>,
    pub char_shapes: Vec<CharShape>,
    pub para_shapes: Vec<ParaShape>,
    pub styles: Vec<StyleEntry>,
    pub border_fill_count: u16,
    pub tab_def_count: u16,
    pub numbering_count: u16,
    pub bullet_count: u16,
    pub bin_data_entries: Vec<BinDataEntry>,
}

#[derive(Debug, Clone)]
pub struct FontEntry {
    pub name: String,
    pub lang: FontLang,
}

#[derive(Debug, Clone, Copy)]
pub enum FontLang {
    Korean,
    Latin,
    Hanja,
    Japanese,
    Other,
    Symbol,
    User,
}

#[derive(Debug, Clone)]
pub struct StyleEntry {
    pub name: String,
    pub eng_name: String,
    pub kind: u8,
    pub para_shape_id: u16,
    pub char_shape_id: u16,
}

#[derive(Debug, Clone)]
pub struct BinDataEntry {
    pub storage_type: u16,
    pub storage_id: u16,
    pub extension: String,
}

pub fn parse_doc_info(data: &[u8], _version: &Version) -> Result<DocInfo, HwpError> {
    let records = Record::parse_all(data)?;
    let mut info = DocInfo::default();

    for record in &records {
        let mut reader = RecordReader::new(&record.payload);

        match record.tag_id {
            TAG_DOCUMENT_PROPERTIES => {
                info.section_count = reader.read_u16();
            }
            TAG_ID_MAPPINGS => {
                let _face_name_count = reader.read_u16();
                let _border_fill_count = reader.read_u16();
                info.border_fill_count = _border_fill_count;
                let _char_shape_count = reader.read_u16();
                let _tab_def_count = reader.read_u16();
                info.tab_def_count = _tab_def_count;
                let _numbering_count = reader.read_u16();
                info.numbering_count = _numbering_count;
                let _bullet_count = reader.read_u16();
                info.bullet_count = _bullet_count;
                let _para_shape_count = reader.read_u16();
                let _style_count = reader.read_u16();
            }
            TAG_FACE_NAME => {
                let lang_type = reader.read_u8();
                let name = reader.read_string();
                let lang = match lang_type {
                    0 => FontLang::Korean,
                    1 => FontLang::Latin,
                    2 => FontLang::Hanja,
                    3 => FontLang::Japanese,
                    4 => FontLang::Other,
                    5 => FontLang::Symbol,
                    6 => FontLang::User,
                    _ => FontLang::Other,
                };
                info.fonts.push(FontEntry { name, lang });
            }
            TAG_CHAR_SHAPE => {
                info.char_shapes.push(CharShape::from_record(&mut reader));
            }
            TAG_PARA_SHAPE => {
                info.para_shapes.push(ParaShape::from_record(&mut reader));
            }
            TAG_STYLE => {
                let name = reader.read_string();
                let eng_name = reader.read_string();
                let kind = reader.read_u8();
                reader.read_u8();
                let para_shape_id = reader.read_u16();
                let char_shape_id = reader.read_u16();
                info.styles.push(StyleEntry {
                    name,
                    eng_name,
                    kind,
                    para_shape_id,
                    char_shape_id,
                });
            }
            TAG_BIN_DATA => {
                let flags = reader.read_u16();
                let storage_type = flags & 0x0F;
                let storage_id = reader.read_u16();
                let extension = reader.read_string();
                info.bin_data_entries.push(BinDataEntry {
                    storage_type,
                    storage_id,
                    extension,
                });
            }
            _ => {}
        }
    }

    Ok(info)
}
