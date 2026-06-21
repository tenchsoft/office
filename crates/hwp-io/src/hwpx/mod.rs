mod reader;
mod writer;

use crate::error::HwpError;
use tench_document_core::tdm::TenchDocument;

pub fn read_hwpx(data: &[u8]) -> Result<TenchDocument, HwpError> {
    reader::read_hwpx(data)
}

pub fn write_hwpx(doc: &TenchDocument) -> Result<Vec<u8>, HwpError> {
    writer::write_hwpx(doc)
}
