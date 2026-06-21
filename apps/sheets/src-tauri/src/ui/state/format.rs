// ---------------------------------------------------------------------------
// Phase 5: Cell formatting types
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;

/// Horizontal alignment within a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HorizontalAlignment {
    Left,
    #[default]
    Center,
    Right,
}

/// Vertical alignment within a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalAlignment {
    Top,
    #[default]
    Middle,
    Bottom,
}

/// Number format for a cell.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NumberFormat {
    #[default]
    General,
    Number {
        decimals: u8,
        thousands_sep: bool,
    },
    Currency {
        symbol: String,
        decimals: u8,
    },
    Percentage {
        decimals: u8,
    },
    Date,
    Text,
}

/// Cell formatting attributes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CellFormat {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub text_color: Option<Color>,
    pub bg_color: Option<Color>,
    pub h_align: Option<HorizontalAlignment>,
    pub v_align: Option<VerticalAlignment>,
    pub number_format: NumberFormat,
}

/// A merged cell region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergedCell {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

/// Conditional formatting rule.
#[derive(Debug, Clone)]
pub struct ConditionalFormatRule {
    /// Column to apply the condition to.
    pub col: usize,
    /// Row range (start, end).
    pub row_range: (usize, usize),
    /// Condition operator.
    pub condition: ConditionOp,
    /// Value to compare against.
    pub value: f64,
    /// Second value for "between" condition.
    pub value2: Option<f64>,
    /// Background color when condition is met.
    pub bg_color: Color,
    /// Text color when condition is met.
    pub text_color: Color,
}

/// Condition operator for conditional formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConditionOp {
    #[default]
    GreaterThan,
    LessThan,
    EqualTo,
    Between,
}

impl ConditionOp {
    pub fn label(&self) -> &str {
        match self {
            ConditionOp::GreaterThan => "Greater than",
            ConditionOp::LessThan => "Less than",
            ConditionOp::EqualTo => "Equal to",
            ConditionOp::Between => "Between",
        }
    }

    pub const ALL: [ConditionOp; 4] = [
        ConditionOp::GreaterThan,
        ConditionOp::LessThan,
        ConditionOp::EqualTo,
        ConditionOp::Between,
    ];
}

/// Tab index for the Format Cells dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormatCellsTab {
    #[default]
    Number,
    Alignment,
    Font,
    Border,
    Fill,
}

/// State for the Format Cells dialog.
#[derive(Debug, Clone, Default)]
pub struct FormatCellsState {
    pub visible: bool,
    pub active_tab: FormatCellsTab,
    pub draft: CellFormat,
}

/// State for the Conditional Format dialog.
#[derive(Debug, Clone, Default)]
pub struct ConditionalFormatDialogState {
    pub visible: bool,
    pub condition: ConditionOp,
    pub value_text: String,
    pub value2_text: String,
    pub bg_color: Option<Color>,
    pub text_color: Option<Color>,
}
