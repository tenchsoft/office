use std::path::Path;

use tench_document_core::OfficeContent;

use crate::sheets::format as format_io;

/// Import a CSV file into OfficeContent with encoding detection and delimiter auto-detection.
pub fn import_csv(path: &Path) -> Result<OfficeContent, String> {
    let raw_bytes = std::fs::read(path).map_err(|e| format!("Failed to read CSV: {e}"))?;

    // Detect encoding and convert to UTF-8
    let text = detect_and_decode(&raw_bytes)?;

    // Auto-detect delimiter
    let delimiter = detect_delimiter(&text);

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();
    Ok(format_io::csv_to_workbook_content_with_delimiter(
        &text, &title, delimiter,
    ))
}

/// Import a TSV file into OfficeContent.
pub fn import_tsv(path: &Path) -> Result<OfficeContent, String> {
    let raw_bytes = std::fs::read(path).map_err(|e| format!("Failed to read TSV: {e}"))?;

    let text = detect_and_decode(&raw_bytes)?;

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();
    Ok(format_io::csv_to_workbook_content_with_delimiter(
        &text, &title, '\t',
    ))
}

/// Export OfficeContent to CSV bytes.
pub fn export_csv_bytes(content: &OfficeContent) -> Result<Vec<u8>, String> {
    Ok(format_io::workbook_to_csv(content).into_bytes())
}

/// Export OfficeContent to TSV bytes.
pub fn export_tsv_bytes(content: &OfficeContent) -> Result<Vec<u8>, String> {
    let csv = format_io::workbook_to_csv(content);
    Ok(csv.replace(',', "\t").into_bytes())
}

// ---------------------------------------------------------------------------
// Encoding detection
// ---------------------------------------------------------------------------

fn detect_and_decode(bytes: &[u8]) -> Result<String, String> {
    // Check for BOM (Byte Order Mark)
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        // UTF-8 BOM
        return String::from_utf8(bytes[3..].to_vec())
            .map_err(|e| format!("UTF-8 BOM decode error: {e}"));
    }

    if bytes.len() >= 2 {
        // UTF-16 BE BOM: FE FF
        if bytes[0] == 0xFE && bytes[1] == 0xFF {
            return decode_utf16_be(bytes);
        }
        // UTF-16 LE BOM: FF FE
        if bytes[0] == 0xFF && bytes[1] == 0xFE {
            return decode_utf16_le(bytes);
        }
    }

    // Try UTF-8 first
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return Ok(s);
    }

    // Try common encodings
    try_decode_fallback(bytes)
}

fn decode_utf16_be(bytes: &[u8]) -> Result<String, String> {
    let mut chars = Vec::new();
    let data = &bytes[2..]; // Skip BOM
    let mut i = 0;
    while i + 1 < data.len() {
        let code = u16::from_be_bytes([data[i], data[i + 1]]);
        if let Some(c) = char::from_u32(code as u32) {
            chars.push(c);
        }
        i += 2;
    }
    Ok(chars.into_iter().collect())
}

fn decode_utf16_le(bytes: &[u8]) -> Result<String, String> {
    let mut chars = Vec::new();
    let data = &bytes[2..]; // Skip BOM
    let mut i = 0;
    while i + 1 < data.len() {
        let code = u16::from_le_bytes([data[i], data[i + 1]]);
        if let Some(c) = char::from_u32(code as u32) {
            chars.push(c);
        }
        i += 2;
    }
    Ok(chars.into_iter().collect())
}

// type_complexity: complex function pointer type in encoding detection table
#[allow(clippy::type_complexity)]
fn try_decode_fallback(bytes: &[u8]) -> Result<String, String> {
    // Try common encodings by checking valid character ratio
    let candidates: Vec<(&str, fn(&[u8]) -> Option<String>)> = vec![
        ("EUC-KR", try_decode_euc_kr),
        ("Shift-JIS", try_decode_shift_jis),
        ("CP1252", try_decode_cp1252),
    ];

    let mut best: Option<(String, f32)> = None;

    for (_name, decoder) in candidates {
        if let Some(decoded) = decoder(bytes) {
            let ratio = valid_text_ratio(&decoded);
            if best.is_none() || ratio > best.as_ref().unwrap().1 {
                best = Some((decoded, ratio));
            }
        }
    }

    best.map(|(s, _)| s).ok_or_else(|| {
        // Last resort: lossy UTF-8
        String::from_utf8_lossy(bytes).to_string();
        "Could not detect file encoding. Attempted UTF-8, UTF-16, EUC-KR, Shift-JIS, and CP1252."
            .to_string()
    })
}

fn valid_text_ratio(text: &str) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let valid = text
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .count();
    valid as f32 / text.len().max(1) as f32
}

fn try_decode_euc_kr(bytes: &[u8]) -> Option<String> {
    // Basic EUC-KR decoder for common Korean text
    let mut result = String::new();
    let mut i = 0;
    let mut invalid = 0usize;
    let total = bytes.len();

    while i < bytes.len() {
        let b = bytes[i];
        if b < 0x80 {
            // ASCII
            result.push(b as char);
            i += 1;
        } else if (0xA1..=0xFE).contains(&b)
            && i + 1 < bytes.len()
            && (0xA1..=0xFE).contains(&bytes[i + 1])
        {
            // EUC-KR double byte - approximate with replacement
            let hi = bytes[i];
            let lo = bytes[i + 1];
            // Try to decode common Hangul syllables
            if let Some(c) = euc_kr_to_char(hi, lo) {
                result.push(c);
            } else {
                result.push('\u{FFFD}');
                invalid += 1;
            }
            i += 2;
        } else {
            result.push('\u{FFFD}');
            invalid += 1;
            i += 1;
        }
    }

    // If too many invalid chars, this encoding is probably wrong
    if invalid > total / 10 {
        None
    } else {
        Some(result)
    }
}

fn euc_kr_to_char(hi: u8, lo: u8) -> Option<char> {
    // Very simplified EUC-KR to Unicode mapping
    // For a production implementation, use a proper encoding crate
    let row = hi as u32 - 0xA1;
    let col = lo as u32 - 0xA1;
    // Hangul syllables in EUC-KR start at row 0xB0 (Hangul area)
    if (0xB0..=0xC8).contains(&hi) {
        // KS X 1001 Hangul area
        let idx = (row - 0xB0 + 16) * 94 + col;
        // Map to Unicode Hangul syllables (U+AC00 - U+D7A3)
        let unicode = 0xAC00 + idx;
        char::from_u32(unicode)
    } else {
        None
    }
}

fn try_decode_shift_jis(_bytes: &[u8]) -> Option<String> {
    // Shift-JIS is complex; for now return None to skip
    // A production implementation would use an encoding crate
    None
}

fn try_decode_cp1252(bytes: &[u8]) -> Option<String> {
    let mut result = String::new();
    for &b in bytes {
        if b < 0x80 {
            result.push(b as char);
        } else {
            // CP1252 mapping for 0x80-0xFF
            let c = match b {
                0x80 => '\u{20AC}', // Euro sign
                0x82 => '\u{201A}', // Single low-9 quotation mark
                0x83 => '\u{0192}', // Latin small letter f with hook
                0x84 => '\u{201E}', // Double low-9 quotation mark
                0x85 => '\u{2026}', // Horizontal ellipsis
                0x86 => '\u{2020}', // Dagger
                0x87 => '\u{2021}', // Double dagger
                0x88 => '\u{02C6}', // Modifier letter circumflex accent
                0x89 => '\u{2030}', // Per mille sign
                0x8A => '\u{0160}', // Latin capital letter S with caron
                0x8B => '\u{2039}', // Single left-pointing angle quotation mark
                0x8C => '\u{0152}', // Latin capital ligature OE
                0x8E => '\u{017D}', // Latin capital letter Z with caron
                0x91 => '\u{2018}', // Left single quotation mark
                0x92 => '\u{2019}', // Right single quotation mark
                0x93 => '\u{201C}', // Left double quotation mark
                0x94 => '\u{201D}', // Right double quotation mark
                0x95 => '\u{2022}', // Bullet
                0x96 => '\u{2013}', // En dash
                0x97 => '\u{2014}', // Em dash
                0x98 => '\u{02DC}', // Small tilde
                0x99 => '\u{2122}', // Trade mark sign
                0x9A => '\u{0161}', // Latin small letter s with caron
                0x9B => '\u{203A}', // Single right-pointing angle quotation mark
                0x9C => '\u{0153}', // Latin small ligature oe
                0x9E => '\u{017E}', // Latin small letter z with caron
                0x9F => '\u{0178}', // Latin capital letter Y with diaeresis
                0xA0..=0xFF => char::from_u32(b as u32)?,
                _ => '\u{FFFD}',
            };
            result.push(c);
        }
    }
    Some(result)
}

// ---------------------------------------------------------------------------
// Delimiter detection
// ---------------------------------------------------------------------------

fn detect_delimiter(text: &str) -> char {
    let first_line = text.lines().next().unwrap_or("");

    // Skip if the line is empty
    if first_line.is_empty() {
        return ',';
    }

    let delimiters = [',', '\t', ';', '|'];
    let mut best_delim = ',';
    let mut best_count = 0usize;

    // Parse the first line respecting quotes to count actual delimiters
    for &delim in &delimiters {
        let count = count_delimiters_in_line(first_line, delim);
        if count > best_count {
            best_count = count;
            best_delim = delim;
        }
    }

    best_delim
}

fn count_delimiters_in_line(line: &str, delim: char) -> usize {
    let mut count = 0;
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '"' {
            if in_quotes && chars.peek() == Some(&'"') {
                chars.next(); // Skip escaped quote
            } else {
                in_quotes = !in_quotes;
            }
        } else if ch == delim && !in_quotes {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use tench_document_core::OfficeContent;

    #[test]
    fn csv_round_trip() {
        let dir = std::env::temp_dir().join(format!("tench_csv_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.csv");
        std::fs::write(&path, "Name,Age\nAlice,30\nBob,25").unwrap();

        let content = import_csv(&path).expect("import");
        if let OfficeContent::Sheets(wb) = &content {
            assert!(wb.sheets[0].cells.iter().any(|c| {
                matches!(&c.value, tench_document_core::CellValue::String(s) if s == "Alice")
            }));
        } else {
            panic!("Expected Sheets");
        }

        let bytes = export_csv_bytes(&content).expect("export");
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("Alice"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn csv_utf8_bom_detection() {
        let dir = std::env::temp_dir().join(format!("tench_csv_bom_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bom_test.csv");

        let mut data = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        data.extend_from_slice(b"Name,Age\nAlice,30");
        std::fs::write(&path, &data).unwrap();

        let content = import_csv(&path).expect("import");
        if let OfficeContent::Sheets(wb) = &content {
            assert!(wb.sheets[0].cells.iter().any(|c| {
                matches!(&c.value, tench_document_core::CellValue::String(s) if s == "Alice")
            }));
        }

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn csv_delimiter_detection_comma() {
        assert_eq!(detect_delimiter("A,B,C,D"), ',');
    }

    #[test]
    fn csv_delimiter_detection_tab() {
        assert_eq!(detect_delimiter("A\tB\tC\tD"), '\t');
    }

    #[test]
    fn csv_delimiter_detection_semicolon() {
        assert_eq!(detect_delimiter("A;B;C;D"), ';');
    }

    #[test]
    fn csv_delimiter_detection_pipe() {
        assert_eq!(detect_delimiter("A|B|C|D"), '|');
    }

    #[test]
    fn csv_delimiter_with_quotes() {
        // Comma inside quotes should not count
        assert_eq!(detect_delimiter(r#"A,"B,C",D"#), ',');
        assert_eq!(count_delimiters_in_line(r#"A,"B,C",D"#, ','), 2);
    }

    #[test]
    fn cp1252_decoding() {
        // Test CP1252 euro sign
        let bytes = vec![0x80];
        let result = try_decode_cp1252(&bytes);
        assert_eq!(result.as_deref(), Some("\u{20AC}"));
    }
}
