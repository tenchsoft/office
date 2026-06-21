use crate::error::HwpError;

/// Parsed record from HWP binary stream.
#[derive(Debug, Clone)]
pub struct Record {
    pub tag_id: u16,
    pub level: u16,
    pub payload: Vec<u8>,
}

impl Record {
    /// Parse all records from a stream (DocInfo or BodyText section).
    pub fn parse_all(data: &[u8]) -> Result<Vec<Self>, HwpError> {
        let mut records = Vec::new();
        let mut offset = 0;

        while offset + 4 <= data.len() {
            let header = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            let tag_id = (header & 0x3FF) as u16;
            let level = ((header >> 10) & 0x3FF) as u16;
            let size = ((header >> 20) & 0xFFF) as usize;

            offset += 4;

            let payload_size = if size == 0xFFF {
                if offset + 4 > data.len() {
                    break;
                }
                let ext_size = u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]) as usize;
                offset += 4;
                ext_size
            } else {
                size
            };

            if offset + payload_size > data.len() {
                // Truncated record — take what's available
                let payload = data[offset..].to_vec();
                records.push(Record {
                    tag_id,
                    level,
                    payload,
                });
                break;
            }

            let payload = data[offset..offset + payload_size].to_vec();
            offset += payload_size;

            records.push(Record {
                tag_id,
                level,
                payload,
            });
        }

        Ok(records)
    }

    /// Build a record into bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let payload_len = self.payload.len();
        if payload_len < 0xFFF {
            let header =
                (self.tag_id as u32) | ((self.level as u32) << 10) | ((payload_len as u32) << 20);
            buf.extend_from_slice(&header.to_le_bytes());
        } else {
            let header = (self.tag_id as u32) | ((self.level as u32) << 10) | (0xFFFu32 << 20);
            buf.extend_from_slice(&header.to_le_bytes());
            buf.extend_from_slice(&(payload_len as u32).to_le_bytes());
        }
        buf.extend_from_slice(&self.payload);
        buf
    }
}

/// Helper to read typed values from a record payload.
pub struct RecordReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> RecordReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    pub fn read_u8(&mut self) -> u8 {
        if self.offset < self.data.len() {
            let v = self.data[self.offset];
            self.offset += 1;
            v
        } else {
            0
        }
    }

    pub fn read_i8(&mut self) -> i8 {
        self.read_u8() as i8
    }

    pub fn read_u16(&mut self) -> u16 {
        if self.offset + 2 <= self.data.len() {
            let v = u16::from_le_bytes([self.data[self.offset], self.data[self.offset + 1]]);
            self.offset += 2;
            v
        } else {
            0
        }
    }

    pub fn read_i16(&mut self) -> i16 {
        self.read_u16() as i16
    }

    pub fn read_u32(&mut self) -> u32 {
        if self.offset + 4 <= self.data.len() {
            let v = u32::from_le_bytes([
                self.data[self.offset],
                self.data[self.offset + 1],
                self.data[self.offset + 2],
                self.data[self.offset + 3],
            ]);
            self.offset += 4;
            v
        } else {
            0
        }
    }

    pub fn read_i32(&mut self) -> i32 {
        self.read_u32() as i32
    }

    pub fn read_bytes(&mut self, len: usize) -> &'a [u8] {
        if self.offset + len <= self.data.len() {
            let slice = &self.data[self.offset..self.offset + len];
            self.offset += len;
            slice
        } else {
            &[]
        }
    }

    /// Read a length-prefixed UTF-16LE string (HWPUNIT BSTR).
    pub fn read_string(&mut self) -> String {
        let len = self.read_u16() as usize;
        if len == 0 || self.offset + len * 2 > self.data.len() {
            return String::new();
        }
        let bytes = &self.data[self.offset..self.offset + len * 2];
        self.offset += len * 2;
        let u16_vec: Vec<u16> = bytes
            .chunks(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&u16_vec)
    }
}

/// Helper to build record payloads.
pub struct RecordWriter {
    buf: Vec<u8>,
}

impl RecordWriter {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn write_i8(&mut self, v: i8) {
        self.buf.push(v as u8);
    }

    pub fn write_u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_i16(&mut self, v: i16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    pub fn write_string(&mut self, s: &str) {
        let u16_vec: Vec<u16> = s.encode_utf16().collect();
        self.write_u16(u16_vec.len() as u16);
        for c in u16_vec {
            self.buf.extend_from_slice(&c.to_le_bytes());
        }
    }

    pub fn into_payload(self) -> Vec<u8> {
        self.buf
    }
}

// Tag ID constants
pub const TAG_DOCUMENT_PROPERTIES: u16 = 16;
pub const TAG_ID_MAPPINGS: u16 = 17;
pub const TAG_BIN_DATA: u16 = 18;
pub const TAG_FACE_NAME: u16 = 19;
pub const TAG_BORDER_FILL: u16 = 20;
pub const TAG_CHAR_SHAPE: u16 = 21;
pub const TAG_TAB_DEF: u16 = 22;
pub const TAG_NUMBERING: u16 = 23;
pub const TAG_BULLET: u16 = 24;
pub const TAG_PARA_SHAPE: u16 = 25;
pub const TAG_STYLE: u16 = 26;
pub const TAG_DOC_DATA: u16 = 27;
pub const TAG_PARA_HEADER: u16 = 66;
pub const TAG_PARA_TEXT: u16 = 67;
pub const TAG_PARA_CHAR_SHAPE: u16 = 68;
pub const TAG_PARA_LINE_SEG: u16 = 69;
pub const TAG_PARA_RANGE_TAG: u16 = 70;
pub const TAG_CTRL_HEADER: u16 = 71;
pub const TAG_LIST_HEADER: u16 = 72;
pub const TAG_PAGE_DEF: u16 = 73;
pub const TAG_TABLE: u16 = 77;
pub const TAG_SHAPE_COMPONENT_PICTURE: u16 = 85;
