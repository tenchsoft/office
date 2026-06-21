// ---------------------------------------------------------------------------
// Phase 6: Data operations types
// ---------------------------------------------------------------------------

use super::cell::CellRange;

/// Data validation types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataValidationType {
    #[default]
    WholeNumber,
    Decimal,
    List,
    Date,
    TextLength,
    Custom,
}

impl DataValidationType {
    pub fn label(&self) -> &str {
        match self {
            DataValidationType::WholeNumber => "Whole Number",
            DataValidationType::Decimal => "Decimal",
            DataValidationType::List => "List",
            DataValidationType::Date => "Date",
            DataValidationType::TextLength => "Text Length",
            DataValidationType::Custom => "Custom",
        }
    }

    pub const ALL: [DataValidationType; 6] = [
        DataValidationType::WholeNumber,
        DataValidationType::Decimal,
        DataValidationType::List,
        DataValidationType::Date,
        DataValidationType::TextLength,
        DataValidationType::Custom,
    ];
}

/// Data validation operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataValidationOperator {
    #[default]
    Between,
    NotBetween,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
}

impl DataValidationOperator {
    pub fn label(&self) -> &str {
        match self {
            DataValidationOperator::Between => "Between",
            DataValidationOperator::NotBetween => "Not Between",
            DataValidationOperator::Equal => "Equal To",
            DataValidationOperator::NotEqual => "Not Equal To",
            DataValidationOperator::GreaterThan => "Greater Than",
            DataValidationOperator::LessThan => "Less Than",
        }
    }

    pub const ALL: [DataValidationOperator; 6] = [
        DataValidationOperator::Between,
        DataValidationOperator::NotBetween,
        DataValidationOperator::Equal,
        DataValidationOperator::NotEqual,
        DataValidationOperator::GreaterThan,
        DataValidationOperator::LessThan,
    ];
}

/// A data validation rule.
#[derive(Debug, Clone)]
pub struct DataValidationRule {
    pub range: CellRange,
    pub validation_type: DataValidationType,
    pub operator: DataValidationOperator,
    pub value1: String,
    pub value2: String,
    pub error_message: String,
}

impl Default for DataValidationRule {
    fn default() -> Self {
        Self {
            range: CellRange::single(0, 0),
            validation_type: DataValidationType::default(),
            operator: DataValidationOperator::default(),
            value1: String::new(),
            value2: String::new(),
            error_message: String::new(),
        }
    }
}

/// State for the Data Validation dialog.
#[derive(Debug, Clone, Default)]
pub struct DataValidationDialogState {
    pub visible: bool,
    pub draft: DataValidationRule,
}
