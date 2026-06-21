// ---------------------------------------------------------------------------
// Grid data helpers
// ---------------------------------------------------------------------------

use super::CellData;

pub fn col_letter(col: usize) -> String {
    let mut s = String::new();
    let mut n = col;
    loop {
        s.insert(0, (b'A' + (n % 26) as u8) as char);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    s
}

/// Parse column letters (A=0, B=1, ..., Z=25, AA=26, etc.) back to column index.
pub(super) fn parse_col_letters(s: &str) -> Result<usize, ()> {
    if s.is_empty() {
        return Err(());
    }
    let mut col = 0usize;
    for c in s.chars() {
        if !c.is_ascii_alphabetic() {
            return Err(());
        }
        col = col * 26 + (c.to_ascii_uppercase() as usize - b'A' as usize);
    }
    Ok(col)
}

/// Parse a cell reference like "A1" into (col, row).
pub(super) fn parse_cell_ref(s: &str) -> Option<(usize, usize)> {
    let s = s.trim();
    let mut col_end = 0;
    for c in s.chars() {
        if c.is_ascii_alphabetic() {
            col_end += 1;
        } else {
            break;
        }
    }
    if col_end == 0 || col_end >= s.len() {
        return None;
    }
    let col_str = &s[..col_end];
    let row_str = &s[col_end..];
    let col = parse_col_letters(col_str).ok()?;
    let row = row_str.parse::<usize>().ok()?;
    if row == 0 {
        return None;
    }
    Some((col, row - 1))
}

/// Mock grid for tests.
#[cfg(test)]
#[allow(dead_code)]
pub fn mock_grid() -> Vec<Vec<CellData>> {
    vec![
        vec![
            CellData::val("Item"),
            CellData::val("Q1"),
            CellData::val("Q2"),
            CellData::val("Q3"),
            CellData::val("Q4"),
        ],
        vec![
            CellData::val("Revenue"),
            CellData::val("12500"),
            CellData::val("15800"),
            CellData::val("14200"),
            CellData::val("18900"),
        ],
        vec![
            CellData::val("Expenses"),
            CellData::val("8200"),
            CellData::val("9100"),
            CellData::val("8800"),
            CellData::val("10200"),
        ],
        vec![
            CellData::val("Profit"),
            CellData::formula("=B2-B3"),
            CellData::formula("=C2-C3"),
            CellData::formula("=D2-D3"),
            CellData::formula("=E2-E3"),
        ],
        vec![
            CellData::val("Margin"),
            CellData::formula("=B4/B2"),
            CellData::formula("=C4/C2"),
            CellData::formula("=D4/D2"),
            CellData::formula("=E4/E2"),
        ],
        vec![
            CellData::val("Growth"),
            CellData::val(""),
            CellData::formula("=(C2-B2)/B2"),
            CellData::formula("=(D2-C2)/C2"),
            CellData::formula("=(E2-D2)/D2"),
        ],
        vec![
            CellData::val("Units Sold"),
            CellData::val("250"),
            CellData::val("316"),
            CellData::val("284"),
            CellData::val("378"),
        ],
        vec![
            CellData::val("Avg Price"),
            CellData::formula("=B2/B7"),
            CellData::formula("=C2/C7"),
            CellData::formula("=D2/D7"),
            CellData::formula("=E2/E7"),
        ],
    ]
}

pub(super) fn grid_to_csv(grid: &[Vec<CellData>]) -> String {
    grid.iter()
        .map(|row| {
            row.iter()
                .map(|cell| csv_escape(&cell.value))
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert grid to CSV with optional UTF-8 BOM prefix.
#[allow(dead_code)]
pub fn grid_to_csv_with_bom(grid: &[Vec<CellData>]) -> String {
    let mut csv = grid_to_csv(grid);
    csv.insert(0, '\u{FEFF}');
    csv
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub(super) fn format_number(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        format!("{value:.2}")
    }
}

/// Format a number with optional decimals, thousands separator, and currency symbol.
#[allow(dead_code)]
fn format_number_full(
    value: f64,
    decimals: u8,
    thousands_sep: bool,
    currency_symbol: Option<&str>,
) -> String {
    let int_part = value.abs() as i64;
    let frac_part = ((value.abs().fract()) * 10i64.pow(decimals as u32) as f64).round() as i64;

    let int_str = if thousands_sep {
        let s = int_part.to_string();
        let mut result = String::new();
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.insert(0, ',');
            }
            result.insert(0, c);
        }
        result
    } else {
        int_part.to_string()
    };

    let sign = if value < 0.0 { "-" } else { "" };
    let currency = currency_symbol.unwrap_or("");
    let frac_str = if decimals > 0 {
        format!(".{:0>width$}", frac_part, width = decimals as usize)
    } else {
        String::new()
    };

    format!("{sign}{currency}{int_str}{frac_str}")
}
