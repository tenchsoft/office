use thiserror::Error;

#[derive(Error, Debug)]
pub enum HwpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CFB format error: {0}")]
    Cfb(String),
    #[error("Record parse error: {0}")]
    Record(String),
    #[error("Invalid header: {0}")]
    Header(String),
    #[error("Decompression error: {0}")]
    Decompression(String),
    #[error("XML error: {0}")]
    Xml(String),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Conversion error: {0}")]
    Conversion(String),
    #[error("Encrypted: {0}")]
    Encrypted(String),
    #[error("Unsupported: {0}")]
    Unsupported(String),
}
