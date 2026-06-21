use super::xml::xml_unescape;
use super::*;

mod shared_strings;
mod sheet;
mod styles;
mod workbook;

use shared_strings::read_shared_strings;
use sheet::{
    apply_col_row_metadata, apply_conditional_formatting, apply_merge_cells, apply_page_setup,
    apply_tab_color, parse_sheet_xml_full,
};
use styles::{extract_attr, read_styles, XlsxStyle};
use workbook::read_sheet_info;

/// Import an XLSX file into OfficeContent.
pub fn import_xlsx(path: &Path) -> Result<OfficeContent, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Failed to open XLSX: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read XLSX: {e}"))?;

    // Apply archive limits
    crate::zip_util::check_archive_limits(&mut archive, &crate::zip_util::ArchiveLimits::desktop())
        .map_err(|e| format!("XLSX archive limit check failed: {e}"))?;

    // Read shared strings
    let shared_strings = read_shared_strings(&mut archive);

    // Read workbook.xml to get sheet names and tab colors
    let sheet_info = read_sheet_info(&mut archive);

    // Read styles.xml
    let styles = read_styles(&mut archive);

    // Read each sheet
    let mut sheets = Vec::new();
    for (idx, info) in sheet_info.iter().enumerate() {
        let sheet_path = format!("xl/worksheets/sheet{}.xml", idx + 1);
        let (cells, merge_cells, col_widths, row_heights, cond_fmts, page_setup) =
            if let Ok(mut sheet_file) = archive.by_name(&sheet_path) {
                let mut xml = String::new();
                sheet_file.read_to_string(&mut xml).unwrap_or_default();
                parse_sheet_xml_full(&xml, &shared_strings, &styles)
            } else {
                (
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    None,
                )
            };

        let sheet_id = format!("sheet_{}", idx);
        sheets.push(SheetContent {
            id: sheet_id,
            name: info.name.clone(),
            index: idx as u32,
            cells,
            row_count: Some(1000),
            column_count: Some(26),
        });

        // Store extra metadata in cells as style properties
        // Merge cells: add merge_range to existing cells or create placeholder cells
        if let Some(sheet) = sheets.last_mut() {
            apply_merge_cells(sheet, &merge_cells);
            apply_col_row_metadata(sheet, &col_widths, &row_heights);
            apply_conditional_formatting(sheet, &cond_fmts);
            apply_page_setup(sheet, page_setup.as_ref());
            apply_tab_color(sheet, info.tab_color.as_deref());
        }
    }

    if sheets.is_empty() {
        sheets.push(SheetContent {
            id: "sheet_0".to_string(),
            name: "Sheet1".to_string(),
            index: 0,
            cells: Vec::new(),
            row_count: Some(1000),
            column_count: Some(26),
        });
    }

    let active_id = sheets[0].id.clone();
    Ok(OfficeContent::Sheets(WorkbookContent {
        id: format!(
            "wb_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ),
        title: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string(),
        sheets,
        active_sheet_id: Some(active_id),
    }))
}
