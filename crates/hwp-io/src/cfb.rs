use crate::error::HwpError;

/// CFB (Compound File Binary) container for HWP files.
pub struct CfbContainer {
    comp: cfb::CompoundFile<std::io::Cursor<Vec<u8>>>,
}

impl CfbContainer {
    pub fn open(data: &[u8]) -> Result<Self, HwpError> {
        let cursor = std::io::Cursor::new(data.to_vec());
        let comp = cfb::CompoundFile::open(cursor)
            .map_err(|e| HwpError::Cfb(format!("Failed to open CFB: {}", e)))?;
        Ok(Self { comp })
    }

    pub fn read_header(&mut self) -> Result<super::header::FileHeader, HwpError> {
        let mut stream = self
            .comp
            .open_stream("/FileHeader")
            .map_err(|e| HwpError::Cfb(format!("FileHeader stream not found: {}", e)))?;
        let mut buf = [0u8; 256];
        std::io::Read::read_exact(&mut stream, &mut buf)
            .map_err(|e| HwpError::Cfb(format!("Failed to read FileHeader: {}", e)))?;
        super::header::FileHeader::parse(&buf)
    }

    pub fn read_doc_info(
        &mut self,
        header: &super::header::FileHeader,
    ) -> Result<Vec<u8>, HwpError> {
        let mut stream = self
            .comp
            .open_stream("/DocInfo")
            .map_err(|e| HwpError::Cfb(format!("DocInfo stream not found: {}", e)))?;
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut stream, &mut data)
            .map_err(|e| HwpError::Cfb(format!("Failed to read DocInfo: {}", e)))?;
        if header.is_compressed() {
            data = decompress(&data)?;
        }
        Ok(data)
    }

    pub fn read_sections(
        &mut self,
        header: &super::header::FileHeader,
    ) -> Result<Vec<Vec<u8>>, HwpError> {
        let mut sections = Vec::new();
        for i in 0.. {
            let path = format!("/BodyText/Section{}", i);
            let mut stream = match self.comp.open_stream(&path) {
                Ok(s) => s,
                Err(_) => break,
            };
            let mut data = Vec::new();
            std::io::Read::read_to_end(&mut stream, &mut data)
                .map_err(|e| HwpError::Cfb(format!("Failed to read {}: {}", path, e)))?;
            if header.is_compressed() {
                data = decompress(&data)?;
            }
            sections.push(data);
        }
        Ok(sections)
    }

    pub fn read_bin_data(
        &mut self,
        header: &super::header::FileHeader,
    ) -> Result<Vec<(String, Vec<u8>)>, HwpError> {
        let mut images = Vec::new();
        // Collect paths first to avoid borrow issues
        let paths: Vec<String> = self
            .comp
            .walk()
            .map(|e| e.path().to_string_lossy().to_string())
            .filter(|p| p.starts_with("/BinData/"))
            .collect();
        for path in paths {
            let mut stream = match self.comp.open_stream(&*path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut data = Vec::new();
            if std::io::Read::read_to_end(&mut stream, &mut data).is_ok() {
                if header.is_compressed() {
                    if let Ok(decompressed) = decompress(&data) {
                        data = decompressed;
                    }
                }
                let name = path.split('/').next_back().unwrap_or("unknown").to_string();
                images.push((name, data));
            }
        }
        Ok(images)
    }
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>, HwpError> {
    use flate2::read::DeflateDecoder;
    let mut decoder = DeflateDecoder::new(data);
    let mut decompressed = Vec::new();
    std::io::Read::read_to_end(&mut decoder, &mut decompressed)
        .map_err(|e| HwpError::Decompression(format!("Decompression failed: {}", e)))?;
    Ok(decompressed)
}

pub fn compress(data: &[u8]) -> Vec<u8> {
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    std::io::Write::write_all(&mut encoder, data).unwrap();
    encoder.finish().unwrap()
}
