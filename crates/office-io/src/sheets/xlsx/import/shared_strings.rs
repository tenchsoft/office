use super::*;

// ---------------------------------------------------------------------------
// Shared strings
// ---------------------------------------------------------------------------

pub(super) fn read_shared_strings(archive: &mut zip::ZipArchive<std::fs::File>) -> Vec<String> {
    let Ok(mut file) = archive.by_name("xl/sharedStrings.xml") else {
        return Vec::new();
    };
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap_or_default();

    let mut strings = Vec::new();
    let mut in_t = false;
    let mut current = String::new();

    for token in xml.split('<') {
        let token = token.trim();
        if token.starts_with("t>") || token.starts_with("t ") {
            in_t = true;
            current.clear();
            if let Some(text) = token.strip_prefix("t>") {
                current.push_str(text);
            }
        } else if in_t && token.starts_with('/') {
            strings.push(xml_unescape(&current));
            in_t = false;
        } else if in_t {
            current.push('<');
            current.push_str(token);
        }
    }
    strings
}
