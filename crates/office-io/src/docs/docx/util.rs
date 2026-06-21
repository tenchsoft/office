/// Convert millimeters to twips (1 mm = 56.692913 twips).
pub(crate) fn mm_to_twip(mm: f32) -> i32 {
    (mm * 56.692913).round() as i32
}

/// Convert twips to millimeters.
pub(crate) fn twip_to_mm(twips: i32) -> f32 {
    twips as f32 / 56.692913
}

/// Extract an attribute value from an XML tag string.
pub(crate) fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let prefix = format!("{attr}=\"");
    let start = tag.find(&prefix)?;
    let rest = &tag[start + prefix.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
