use super::*;

impl SheetsState {
    pub fn add_data_validation_rule(&mut self, rule: DataValidationRule) {
        self.data_validation_rules.push(rule);
    }

    /// Validate a cell value against all applicable rules.
    /// Returns the first error message if validation fails, or None if valid.
    pub fn validate_cell(&self, row: usize, col: usize, value: &str) -> Option<String> {
        for rule in &self.data_validation_rules {
            if row >= rule.range.start_row
                && row <= rule.range.end_row
                && col >= rule.range.start_col
                && col <= rule.range.end_col
            {
                if let Some(err) = self.check_validation(rule, value) {
                    return Some(err);
                }
            }
        }
        None
    }

    /// Check a single validation rule against a value.
    fn check_validation(&self, rule: &DataValidationRule, value: &str) -> Option<String> {
        let num = value.parse::<f64>().ok();
        match rule.validation_type {
            DataValidationType::WholeNumber => {
                if let Some(n) = num {
                    if n.fract() != 0.0 {
                        return Some(rule.error_message.clone());
                    }
                } else if !value.is_empty() {
                    return Some(rule.error_message.clone());
                }
            }
            DataValidationType::Decimal => {
                if num.is_none() && !value.is_empty() {
                    return Some(rule.error_message.clone());
                }
            }
            DataValidationType::List => {
                let allowed: Vec<&str> = rule.value1.split(',').map(|s| s.trim()).collect();
                if !value.is_empty() && !allowed.contains(&value) {
                    return Some(rule.error_message.clone());
                }
            }
            DataValidationType::Date => {
                // Basic date validation: non-empty value should be parseable
                if !value.is_empty() && value.parse::<f64>().is_err() {
                    // Accept any non-empty string as a date placeholder
                }
            }
            DataValidationType::TextLength => {
                if !value.is_empty() {
                    let len = value.len() as f64;
                    if let Some(err) = self.check_operator(rule, len) {
                        return Some(err);
                    }
                }
            }
            DataValidationType::Custom => {
                // Custom validation is a placeholder
            }
        }
        // For numeric types, also check operator constraints
        if matches!(
            rule.validation_type,
            DataValidationType::WholeNumber | DataValidationType::Decimal
        ) {
            if let Some(n) = num {
                if let Some(err) = self.check_operator(rule, n) {
                    return Some(err);
                }
            }
        }
        None
    }

    /// Check numeric operator constraint.
    fn check_operator(&self, rule: &DataValidationRule, value: f64) -> Option<String> {
        let v1 = rule.value1.parse::<f64>().unwrap_or(f64::NAN);
        let v2 = rule.value2.parse::<f64>().unwrap_or(f64::NAN);
        let failed = match rule.operator {
            DataValidationOperator::Between => !(value >= v1 && value <= v2),
            DataValidationOperator::NotBetween => value >= v1 && value <= v2,
            DataValidationOperator::Equal => (value - v1).abs() > f64::EPSILON,
            DataValidationOperator::NotEqual => (value - v1).abs() < f64::EPSILON,
            DataValidationOperator::GreaterThan => value <= v1,
            DataValidationOperator::LessThan => value >= v1,
        };
        if failed {
            Some(rule.error_message.clone())
        } else {
            None
        }
    }
}
