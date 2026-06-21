/// A parsed formula expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric literal.
    Number(f64),
    /// String literal (without surrounding quotes).
    Text(String),
    /// Boolean literal.
    Bool(bool),
    /// Cell reference, e.g. `A1`.
    CellRef { col: usize, row: usize },
    /// Range reference, e.g. `A1:B3`.
    Range {
        start_col: usize,
        start_row: usize,
        end_col: usize,
        end_row: usize,
    },
    /// Named reference.
    Name(String),
    /// Binary operator.
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Unary operator.
    UnaryOp { op: UnaryOp, operand: Box<Expr> },
    /// Function call, e.g. `SUM(A1:A10)`.
    Function { name: String, args: Vec<Expr> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Concat,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Percent,
}

// ---------------------------------------------------------------------------
// Value — runtime representation
// ---------------------------------------------------------------------------

/// Runtime value produced during formula evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Text(String),
    Bool(bool),
    Error(FormulaError),
    Array(Vec<Vec<Value>>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Number(0.0)
    }
}

impl Value {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::Text(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::Text(s) => !s.is_empty(),
            Value::Error(_) => false,
            Value::Array(a) => !a.is_empty(),
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            Value::Number(n) => format_number_display(*n),
            Value::Text(s) => s.clone(),
            Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.into(),
            Value::Error(e) => format!("{:?}", e),
            Value::Array(a) => {
                if a.is_empty() || a[0].is_empty() {
                    String::new()
                } else {
                    a[0][0].as_str()
                }
            }
        }
    }

    pub(crate) fn to_f64_or_error(&self) -> Result<f64, FormulaError> {
        self.as_f64().ok_or(FormulaError::Value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaError {
    /// Division by zero.
    DivZero,
    /// Invalid value / type mismatch.
    Value,
    /// Reference to an invalid cell or range.
    Ref,
    /// Unknown function name.
    Name,
    /// Not available (e.g. missing argument).
    NA,
    /// Circular reference detected.
    Circular,
    /// Generic error.
    #[allow(dead_code)]
    Error,
}

// ---------------------------------------------------------------------------
// Tokeniser
// ---------------------------------------------------------------------------

fn format_number_display(n: f64) -> String {
    if n == n.floor() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}
