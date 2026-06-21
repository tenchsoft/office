// ---------------------------------------------------------------------------
// Cell data, ranges, clipboard, editing, autocomplete, formula types
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;

use super::col_letter;
use super::format::{CellFormat, NumberFormat};

/// Paste special mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteSpecialMode {
    All,
    ValuesOnly,
    FormatsOnly,
    FormulasOnly,
}

#[derive(Debug, Clone, Default)]
pub struct GridClipboard {
    /// Copied cell values.
    pub cells: Vec<Vec<CellData>>,
    /// Source row of the copied range.
    pub source_row: usize,
    /// Source col of the copied range.
    pub source_col: usize,
    /// Whether this is a cut operation.
    pub is_cut: bool,
}

/// Undo snapshot for the grid.
#[derive(Debug, Clone)]
pub struct GridSnapshot {
    pub(crate) grid: Vec<Vec<CellData>>,
    pub(crate) selected_row: usize,
    pub(crate) selected_col: usize,
}

pub(crate) const UNDO_LIMIT: usize = 100;

/// A cell range reference (e.g., A1:B3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRange {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

impl CellRange {
    /// Convenience constructor for a single-cell range.
    ///
    /// Public API used by named-range definitions and tests.
    #[allow(dead_code)] // public API — used by callers defining single-cell ranges
    pub fn single(row: usize, col: usize) -> Self {
        Self {
            start_row: row,
            start_col: col,
            end_row: row,
            end_col: col,
        }
    }

    pub fn to_address(&self) -> String {
        if self.start_row == self.end_row && self.start_col == self.end_col {
            format!("{}{}", col_letter(self.start_col), self.start_row + 1)
        } else {
            format!(
                "{}{}:{}{}",
                col_letter(self.start_col),
                self.start_row + 1,
                col_letter(self.end_col),
                self.end_row + 1
            )
        }
    }
}

/// A named range within the workbook.
#[derive(Debug, Clone)]
pub struct NamedRange {
    pub name: String,
    /// None = global scope, Some(idx) = sheet-scoped.
    pub sheet_idx: Option<usize>,
    pub range: CellRange,
}

/// Cell editing state for the currently edited cell.
#[derive(Debug, Clone)]
pub struct EditingCell {
    pub row: usize,
    pub col: usize,
    /// The draft text being edited (not yet committed to the cell).
    pub draft: String,
    /// Cursor position within the draft (byte offset).
    pub cursor_pos: usize,
    /// Original value before editing started (for Esc cancel).
    pub original_value: String,
    /// Whether this is a formula edit (draft starts with '=').
    pub is_formula_edit: bool,
    /// Autocomplete state for function names.
    pub autocomplete: Option<AutocompleteState>,
}

/// Autocomplete state for function name completion.
#[derive(Debug, Clone)]
pub struct AutocompleteState {
    /// The prefix typed so far (e.g. "SU").
    pub prefix: String,
    /// Filtered list of matching function names.
    pub candidates: Vec<&'static str>,
    /// Currently selected candidate index.
    pub selected_idx: usize,
    /// Position (x, y) for the popup relative to the cell.
    pub popup_offset: (f64, f64),
}

/// Function catalog entry for autocomplete and hints.
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: &'static str,
    pub signature: &'static str,
    pub description: &'static str,
    pub category: &'static str,
}

/// Returns the catalog of all supported functions.
pub fn function_catalog() -> &'static [FunctionInfo] {
    &[
        FunctionInfo {
            name: "SUM",
            signature: "SUM(number1, [number2], ...)",
            description: "Sum of values",
            category: "Math",
        },
        FunctionInfo {
            name: "AVERAGE",
            signature: "AVERAGE(number1, [number2], ...)",
            description: "Average of values",
            category: "Math",
        },
        FunctionInfo {
            name: "AVG",
            signature: "AVG(number1, [number2], ...)",
            description: "Average of values (alias)",
            category: "Math",
        },
        FunctionInfo {
            name: "COUNT",
            signature: "COUNT(value1, [value2], ...)",
            description: "Count of numeric values",
            category: "Statistical",
        },
        FunctionInfo {
            name: "COUNTA",
            signature: "COUNTA(value1, [value2], ...)",
            description: "Count of non-empty values",
            category: "Statistical",
        },
        FunctionInfo {
            name: "COUNTBLANK",
            signature: "COUNTBLANK(range)",
            description: "Count of blank cells",
            category: "Statistical",
        },
        FunctionInfo {
            name: "MIN",
            signature: "MIN(number1, [number2], ...)",
            description: "Minimum value",
            category: "Statistical",
        },
        FunctionInfo {
            name: "MAX",
            signature: "MAX(number1, [number2], ...)",
            description: "Maximum value",
            category: "Statistical",
        },
        FunctionInfo {
            name: "ABS",
            signature: "ABS(number)",
            description: "Absolute value",
            category: "Math",
        },
        FunctionInfo {
            name: "ROUND",
            signature: "ROUND(number, digits)",
            description: "Round to digits",
            category: "Math",
        },
        FunctionInfo {
            name: "ROUNDUP",
            signature: "ROUNDUP(number, digits)",
            description: "Round up to digits",
            category: "Math",
        },
        FunctionInfo {
            name: "ROUNDDOWN",
            signature: "ROUNDDOWN(number, digits)",
            description: "Round down to digits",
            category: "Math",
        },
        FunctionInfo {
            name: "INT",
            signature: "INT(number)",
            description: "Round down to integer",
            category: "Math",
        },
        FunctionInfo {
            name: "MOD",
            signature: "MOD(number, divisor)",
            description: "Modulo",
            category: "Math",
        },
        FunctionInfo {
            name: "POWER",
            signature: "POWER(base, exponent)",
            description: "Raise to power",
            category: "Math",
        },
        FunctionInfo {
            name: "SQRT",
            signature: "SQRT(number)",
            description: "Square root",
            category: "Math",
        },
        FunctionInfo {
            name: "IF",
            signature: "IF(condition, true_val, false_val)",
            description: "Conditional value",
            category: "Logical",
        },
        FunctionInfo {
            name: "IFERROR",
            signature: "IFERROR(value, value_if_error)",
            description: "Return fallback on error",
            category: "Logical",
        },
        FunctionInfo {
            name: "IFNA",
            signature: "IFNA(value, value_if_na)",
            description: "Return fallback on #N/A",
            category: "Logical",
        },
        FunctionInfo {
            name: "LEN",
            signature: "LEN(text)",
            description: "Length of text",
            category: "Text",
        },
        FunctionInfo {
            name: "LEFT",
            signature: "LEFT(text, [num_chars])",
            description: "Left substring",
            category: "Text",
        },
        FunctionInfo {
            name: "RIGHT",
            signature: "RIGHT(text, [num_chars])",
            description: "Right substring",
            category: "Text",
        },
        FunctionInfo {
            name: "UPPER",
            signature: "UPPER(text)",
            description: "Convert to uppercase",
            category: "Text",
        },
        FunctionInfo {
            name: "LOWER",
            signature: "LOWER(text)",
            description: "Convert to lowercase",
            category: "Text",
        },
        FunctionInfo {
            name: "TRIM",
            signature: "TRIM(text)",
            description: "Remove extra spaces",
            category: "Text",
        },
        FunctionInfo {
            name: "CONCATENATE",
            signature: "CONCATENATE(text1, [text2], ...)",
            description: "Join text strings",
            category: "Text",
        },
        FunctionInfo {
            name: "SUBSTITUTE",
            signature: "SUBSTITUTE(text, old, new, [instance])",
            description: "Replace text",
            category: "Text",
        },
        FunctionInfo {
            name: "VLOOKUP",
            signature: "VLOOKUP(key, range, col_idx, [approx])",
            description: "Vertical lookup",
            category: "Lookup",
        },
        FunctionInfo {
            name: "HLOOKUP",
            signature: "HLOOKUP(key, range, row_idx, [approx])",
            description: "Horizontal lookup",
            category: "Lookup",
        },
        FunctionInfo {
            name: "TODAY",
            signature: "TODAY()",
            description: "Current date",
            category: "Date",
        },
        FunctionInfo {
            name: "NOW",
            signature: "NOW()",
            description: "Current date and time",
            category: "Date",
        },
        FunctionInfo {
            name: "YEAR",
            signature: "YEAR(date)",
            description: "Extract year",
            category: "Date",
        },
        FunctionInfo {
            name: "MONTH",
            signature: "MONTH(date)",
            description: "Extract month",
            category: "Date",
        },
        FunctionInfo {
            name: "DAY",
            signature: "DAY(date)",
            description: "Extract day",
            category: "Date",
        },
        FunctionInfo {
            name: "HOUR",
            signature: "HOUR(time)",
            description: "Extract hour",
            category: "Date",
        },
        FunctionInfo {
            name: "MINUTE",
            signature: "MINUTE(time)",
            description: "Extract minute",
            category: "Date",
        },
        FunctionInfo {
            name: "SECOND",
            signature: "SECOND(time)",
            description: "Extract second",
            category: "Date",
        },
    ]
}

/// Colors used for formula reference highlighting.
pub(crate) const FORMULA_REF_COLORS: &[Color] = &[
    Color::rgb8(0x42, 0xA5, 0xF5), // blue
    Color::rgb8(0xFF, 0x70, 0x43), // orange
    Color::rgb8(0x66, 0xBB, 0x6A), // green
    Color::rgb8(0xAB, 0x47, 0xBC), // purple
];

/// Parsed formula reference for color highlighting.
#[derive(Debug, Clone)]
pub struct FormulaRef {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
    pub color_idx: usize,
}

#[derive(Debug, Clone)]
pub struct CellData {
    pub value: String,
    pub is_formula: bool,
    /// Evaluated display value (computed from formula or same as value for plain cells).
    pub display_value: String,
    /// Cell formatting attributes.
    pub format: CellFormat,
}

impl CellData {
    pub fn val(value: &str) -> Self {
        Self {
            value: value.into(),
            is_formula: false,
            display_value: value.into(),
            format: CellFormat::default(),
        }
    }

    pub fn formula(value: &str) -> Self {
        Self {
            value: value.into(),
            is_formula: true,
            display_value: String::new(),
            format: CellFormat::default(),
        }
    }

    pub fn display(&self) -> &str {
        if self.is_formula && !self.display_value.is_empty() {
            &self.display_value
        } else {
            &self.value
        }
    }

    /// Get the formatted display value, applying number formatting.
    pub fn formatted_display(&self) -> String {
        let raw = self.display();
        if raw.is_empty() {
            return String::new();
        }
        match &self.format.number_format {
            NumberFormat::General => raw.to_string(),
            NumberFormat::Number {
                decimals,
                thousands_sep,
            } => {
                if let Ok(num) = raw.parse::<f64>() {
                    format_number_full(num, *decimals, *thousands_sep, None)
                } else {
                    raw.to_string()
                }
            }
            NumberFormat::Currency { symbol, decimals } => {
                if let Ok(num) = raw.parse::<f64>() {
                    format_number_full(num, *decimals, false, Some(symbol))
                } else {
                    raw.to_string()
                }
            }
            NumberFormat::Percentage { decimals } => {
                if let Ok(num) = raw.parse::<f64>() {
                    format!("{:.prec$}%", num * 100.0, prec = *decimals as usize)
                } else {
                    raw.to_string()
                }
            }
            NumberFormat::Date | NumberFormat::Text => raw.to_string(),
        }
    }
}

/// Format a number with optional decimals, thousands separator, and currency symbol.
pub(crate) fn format_number_full(
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
