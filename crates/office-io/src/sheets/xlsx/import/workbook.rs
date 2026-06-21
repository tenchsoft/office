use super::*;

// ---------------------------------------------------------------------------
// Import helpers
// ---------------------------------------------------------------------------

pub(super) struct SheetInfo {
    pub(super) name: String,
    pub(super) tab_color: Option<String>,
}

pub(super) fn read_sheet_info(archive: &mut zip::ZipArchive<std::fs::File>) -> Vec<SheetInfo> {
    let Ok(mut file) = archive.by_name("xl/workbook.xml") else {
        return vec![SheetInfo {
            name: "Sheet1".to_string(),
            tab_color: None,
        }];
    };
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap_or_default();

    let mut infos = Vec::new();
    for token in xml.split('<') {
        let token = token.trim();
        if token.starts_with("sheet ") {
            let name = if let Some(start) = token.find("name=\"") {
                let rest = &token[start + 6..];
                if let Some(end) = rest.find('"') {
                    xml_unescape(&rest[..end])
                } else {
                    "Sheet1".to_string()
                }
            } else {
                "Sheet1".to_string()
            };
            let tab_color = if let Some(start) = token.find("tabColor=\"") {
                let rest = &token[start + 10..];
                rest.find('"').map(|end| rest[..end].to_string())
            } else {
                None
            };
            infos.push(SheetInfo { name, tab_color });
        }
    }
    if infos.is_empty() {
        infos.push(SheetInfo {
            name: "Sheet1".to_string(),
            tab_color: None,
        });
    }
    infos
}
