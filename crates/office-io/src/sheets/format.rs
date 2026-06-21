use serde_json::json;
use tench_document_core::{CellContent, CellValue, OfficeContent, SheetContent, WorkbookContent};

const DEFAULT_ROWS: u32 = 1000;
const DEFAULT_COLS: u32 = 26;

/// Create an empty workbook with one sheet.
pub fn empty_workbook_content(title: &str) -> OfficeContent {
    let sheet_id = format!(
        "sheet_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    OfficeContent::Sheets(WorkbookContent {
        id: format!(
            "wb_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ),
        title: title.to_string(),
        sheets: vec![SheetContent {
            id: sheet_id.clone(),
            name: "Sheet1".to_string(),
            index: 0,
            cells: Vec::new(),
            row_count: Some(DEFAULT_ROWS),
            column_count: Some(DEFAULT_COLS),
        }],
        active_sheet_id: Some(sheet_id),
    })
}

/// Convert workbook content to plain text (CSV-like for the active sheet).
pub fn workbook_to_plain_text(content: &OfficeContent) -> String {
    let workbook = match content {
        OfficeContent::Sheets(wb) => wb,
        _ => return String::new(),
    };

    let active_sheet = workbook
        .active_sheet_id
        .as_ref()
        .and_then(|id| workbook.sheets.iter().find(|s| &s.id == id))
        .or_else(|| workbook.sheets.first());

    match active_sheet {
        Some(sheet) => sheet_to_csv(sheet),
        None => String::new(),
    }
}

/// Convert workbook content to CSV (active sheet).
pub fn workbook_to_csv(content: &OfficeContent) -> String {
    workbook_to_plain_text(content)
}

/// Convert CSV text to workbook content with a custom delimiter.
pub fn csv_to_workbook_content_with_delimiter(
    csv: &str,
    title: &str,
    delimiter: char,
) -> OfficeContent {
    let cells = parse_csv_cells_with_delimiter(csv, delimiter);
    let (rows, cols) = csv_dimensions_with_delimiter(csv, delimiter);

    let sheet_id = format!(
        "sheet_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    OfficeContent::Sheets(WorkbookContent {
        id: format!(
            "wb_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ),
        title: title.to_string(),
        sheets: vec![SheetContent {
            id: sheet_id.clone(),
            name: "Sheet1".to_string(),
            index: 0,
            cells,
            row_count: Some(rows.max(1000)),
            column_count: Some(cols.max(26)),
        }],
        active_sheet_id: Some(sheet_id),
    })
}

/// Convert CSV text to workbook content.
pub fn csv_to_workbook_content(csv: &str, title: &str) -> OfficeContent {
    csv_to_workbook_content_with_delimiter(csv, title, ',')
}

/// Convert a single sheet to CSV string.
pub fn sheet_to_csv(sheet: &SheetContent) -> String {
    if sheet.cells.is_empty() {
        return String::new();
    }

    let max_row = sheet
        .cells
        .iter()
        .filter_map(|c| row_from_address(&c.address))
        .max()
        .unwrap_or(0);
    let max_col = sheet
        .cells
        .iter()
        .filter_map(|c| col_from_address(&c.address))
        .max()
        .unwrap_or(0);

    let mut result = String::new();
    for row in 1..=max_row {
        for col in 1..=max_col {
            let address = address_from_row_col(row, col);
            if let Some(cell) = sheet.cells.iter().find(|c| c.address == address) {
                result.push_str(&cell_value_to_csv_string(&cell.value));
            }
            if col < max_col {
                result.push(',');
            }
        }
        result.push('\n');
    }
    result
}

fn cell_value_to_csv_string(value: &CellValue) -> String {
    match value {
        CellValue::String(s) => {
            if s.contains(',') || s.contains('"') || s.contains('\n') {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.clone()
            }
        }
        CellValue::Number(n) => {
            if *n == (*n as i64) as f64 {
                format!("{}", *n as i64)
            } else {
                format!("{n}")
            }
        }
        CellValue::Boolean(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        CellValue::Empty => String::new(),
    }
}

fn parse_csv_cells_with_delimiter(csv: &str, delimiter: char) -> Vec<CellContent> {
    let mut cells = Vec::new();
    for (row_idx, line) in csv.lines().enumerate() {
        let row = (row_idx + 1) as u32;
        let fields = parse_csv_line_with_delimiter(line, delimiter);
        for (col_idx, field) in fields.iter().enumerate() {
            if field.is_empty() {
                continue;
            }
            let col = (col_idx + 1) as u32;
            let address = address_from_row_col(row, col);
            let value = parse_cell_value(field);
            cells.push(CellContent {
                address,
                value,
                formula: None,
                style: json!({}),
            });
        }
    }
    cells
}

fn parse_csv_line_with_delimiter(line: &str, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if in_quotes {
            if ch == '"' {
                if i + 1 < chars.len() && chars[i + 1] == '"' {
                    current.push('"');
                    i += 2;
                    continue;
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(ch);
            }
        } else if ch == '"' {
            in_quotes = true;
        } else if ch == delimiter {
            fields.push(current.clone());
            current.clear();
        } else {
            current.push(ch);
        }
        i += 1;
    }
    fields.push(current);
    fields
}

fn parse_cell_value(s: &str) -> CellValue {
    if s.is_empty() {
        return CellValue::Empty;
    }
    if s == "TRUE" || s == "true" {
        return CellValue::Boolean(true);
    }
    if s == "FALSE" || s == "false" {
        return CellValue::Boolean(false);
    }
    if let Ok(n) = s.parse::<f64>() {
        return CellValue::Number(n);
    }
    CellValue::String(s.to_string())
}

fn csv_dimensions_with_delimiter(csv: &str, delimiter: char) -> (u32, u32) {
    let mut max_row = 0u32;
    let mut max_col = 0u32;
    for (row_idx, line) in csv.lines().enumerate() {
        max_row = (row_idx + 1) as u32;
        let fields = parse_csv_line_with_delimiter(line, delimiter);
        max_col = max_col.max(fields.len() as u32);
    }
    (max_row, max_col)
}

/// Convert column number (1-based) to letter(s): 1=A, 2=B, ..., 26=Z, 27=AA
pub fn col_to_letter(col: u32) -> String {
    let mut result = String::new();
    let mut c = col;
    while c > 0 {
        c -= 1;
        result.push((b'A' + (c % 26) as u8) as char);
        c /= 26;
    }
    result.chars().rev().collect()
}

/// Convert letter(s) to column number (1-based): A=1, B=2, ..., Z=26, AA=27
pub fn letter_to_col(letters: &str) -> Option<u32> {
    let mut col = 0u32;
    for ch in letters.chars() {
        if !ch.is_ascii_alphabetic() {
            return None;
        }
        col = col * 26 + (ch.to_ascii_uppercase() as u32 - b'A' as u32 + 1);
    }
    Some(col)
}

/// Build a cell address from row and column numbers.
pub fn address_from_row_col(row: u32, col: u32) -> String {
    format!("{}{}", col_to_letter(col), row)
}

/// Extract row number from a cell address like "A1".
pub fn row_from_address(address: &str) -> Option<u32> {
    let digits: String = address.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// Extract column number from a cell address like "A1".
pub fn col_from_address(address: &str) -> Option<u32> {
    let letters: String = address
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect();
    letter_to_col(&letters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_workbook_has_one_sheet() {
        let content = empty_workbook_content("Test");
        let wb = match &content {
            OfficeContent::Sheets(wb) => wb,
            _ => panic!("Expected Sheets content"),
        };
        assert_eq!(wb.sheets.len(), 1);
        assert_eq!(wb.sheets[0].name, "Sheet1");
        assert!(wb.sheets[0].cells.is_empty());
    }

    #[test]
    fn csv_round_trip() {
        let csv = "Name,Age,City\nAlice,30,Seoul\nBob,25,Busan";
        let content = csv_to_workbook_content(csv, "Test");
        let text = workbook_to_plain_text(&content);
        assert!(text.contains("Alice"));
        assert!(text.contains("30"));
        assert!(text.contains("Bob"));
    }

    #[test]
    fn col_letter_round_trip() {
        assert_eq!(col_to_letter(1), "A");
        assert_eq!(col_to_letter(26), "Z");
        assert_eq!(col_to_letter(27), "AA");
        assert_eq!(col_to_letter(28), "AB");
        assert_eq!(letter_to_col("A"), Some(1));
        assert_eq!(letter_to_col("Z"), Some(26));
        assert_eq!(letter_to_col("AA"), Some(27));
    }

    #[test]
    fn address_parsing() {
        assert_eq!(row_from_address("A1"), Some(1));
        assert_eq!(col_from_address("A1"), Some(1));
        assert_eq!(row_from_address("AB123"), Some(123));
        assert_eq!(col_from_address("AB123"), Some(28));
        assert_eq!(address_from_row_col(1, 1), "A1");
        assert_eq!(address_from_row_col(5, 28), "AB5");
    }

    #[test]
    fn csv_with_quotes() {
        let csv = r#"Name,Note
Alice,"Hello, World"
Bob,"She said ""hi"""#;
        let content = csv_to_workbook_content(csv, "Quoted");
        let text = workbook_to_plain_text(&content);
        assert!(text.contains("Hello, World"));
    }
}
