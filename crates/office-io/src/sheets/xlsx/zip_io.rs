use super::*;

// ---------------------------------------------------------------------------
// ZIP utilities
// ---------------------------------------------------------------------------

pub(super) fn write_zip_bytes(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for (path, data) in files {
        writer
            .start_file(path, options)
            .map_err(|e| format!("ZIP start_file {path}: {e}"))?;
        writer
            .write_all(data)
            .map_err(|e| format!("ZIP write {path}: {e}"))?;
    }

    let cursor = writer.finish().map_err(|e| format!("ZIP finish: {e}"))?;
    Ok(cursor.into_inner())
}
