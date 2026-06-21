use super::{BinOp, Expr, FormulaError, UnaryOp, Value};

/// Cell data needed for formula evaluation.
pub trait CellProvider {
    /// Get the raw value string at (row, col).
    #[allow(dead_code)]
    fn get_cell_value(&self, row: usize, col: usize) -> Option<&str>;
    /// Get the evaluated display value at (row, col).
    fn get_cell_display(&self, row: usize, col: usize) -> Option<&str>;
    /// Resolve a named reference to a range. Returns (start_row, start_col, end_row, end_col).
    fn resolve_name(&self, name: &str) -> Option<(usize, usize, usize, usize)>;
}

/// Evaluate a parsed formula against a cell provider.
pub fn evaluate(expr: &Expr, provider: &dyn CellProvider) -> Value {
    match expr {
        Expr::Number(n) => Value::Number(*n),
        Expr::Text(s) => Value::Text(s.clone()),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::CellRef { col, row } => {
            let val = provider.get_cell_display(*row, *col);
            match val {
                Some("") => Value::Number(0.0),
                Some(s) => s
                    .parse::<f64>()
                    .map(Value::Number)
                    .unwrap_or(Value::Text(s.into())),
                None => Value::Error(FormulaError::Ref),
            }
        }
        Expr::Range {
            start_col,
            start_row,
            end_col,
            end_row,
        } => {
            // Collect values in the range as a flat list for functions
            let mut vals = Vec::new();
            for r in *start_row..=*end_row {
                for c in *start_col..=*end_col {
                    let val = provider.get_cell_display(r, c);
                    match val {
                        Some(s) if !s.is_empty() => {
                            if let Ok(n) = s.parse::<f64>() {
                                vals.push(Value::Number(n));
                            } else {
                                vals.push(Value::Text(s.into()));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Value::Array(vec![vals])
        }
        Expr::Name(name) => {
            if let Some((sr, sc, er, ec)) = provider.resolve_name(name) {
                let mut vals = Vec::new();
                for r in sr..=er {
                    for c in sc..=ec {
                        let val = provider.get_cell_display(r, c);
                        match val {
                            Some(s) if !s.is_empty() => {
                                if let Ok(n) = s.parse::<f64>() {
                                    vals.push(Value::Number(n));
                                } else {
                                    vals.push(Value::Text(s.into()));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                if vals.len() == 1 {
                    vals.pop().unwrap()
                } else {
                    Value::Array(vec![vals])
                }
            } else {
                Value::Error(FormulaError::Name)
            }
        }
        Expr::BinOp { op, left, right } => {
            let lv = evaluate(left, provider);
            let rv = evaluate(right, provider);
            eval_binop(*op, lv, rv)
        }
        Expr::UnaryOp { op, operand } => {
            let v = evaluate(operand, provider);
            match op {
                UnaryOp::Neg => match v.to_f64_or_error() {
                    Ok(n) => Value::Number(-n),
                    Err(e) => Value::Error(e),
                },
                UnaryOp::Percent => match v.to_f64_or_error() {
                    Ok(n) => Value::Number(n / 100.0),
                    Err(e) => Value::Error(e),
                },
            }
        }
        Expr::Function { name, args } => eval_function(name, args, provider),
    }
}

// ---------------------------------------------------------------------------
// Binary operators
// ---------------------------------------------------------------------------

fn eval_binop(op: BinOp, lv: Value, rv: Value) -> Value {
    match op {
        BinOp::Add => numeric_binop(lv, rv, |a, b| a + b),
        BinOp::Sub => numeric_binop(lv, rv, |a, b| a - b),
        BinOp::Mul => numeric_binop(lv, rv, |a, b| a * b),
        BinOp::Div => match (lv.as_f64(), rv.as_f64()) {
            (Some(a), Some(b)) => {
                if b == 0.0 {
                    Value::Error(FormulaError::DivZero)
                } else {
                    Value::Number(a / b)
                }
            }
            _ => Value::Error(FormulaError::Value),
        },
        BinOp::Pow => numeric_binop(lv, rv, |a, b| a.powf(b)),
        BinOp::Concat => Value::Text(format!("{}{}", lv.as_str(), rv.as_str())),
        BinOp::Eq => comparison_binop(lv, rv, |a, b| (a - b).abs() < f64::EPSILON),
        BinOp::Ne => comparison_binop(lv, rv, |a, b| (a - b).abs() >= f64::EPSILON),
        BinOp::Lt => comparison_binop(lv, rv, |a, b| a < b),
        BinOp::Le => comparison_binop(lv, rv, |a, b| a <= b),
        BinOp::Gt => comparison_binop(lv, rv, |a, b| a > b),
        BinOp::Ge => comparison_binop(lv, rv, |a, b| a >= b),
    }
}

fn numeric_binop(lv: Value, rv: Value, f: impl Fn(f64, f64) -> f64) -> Value {
    match (lv.as_f64(), rv.as_f64()) {
        (Some(a), Some(b)) => Value::Number(f(a, b)),
        _ => Value::Error(FormulaError::Value),
    }
}

fn comparison_binop(lv: Value, rv: Value, f: impl Fn(f64, f64) -> bool) -> Value {
    match (lv.as_f64(), rv.as_f64()) {
        (Some(a), Some(b)) => Value::Bool(f(a, b)),
        _ => {
            // Fallback to string comparison
            let ls = lv.as_str();
            let rs = rv.as_str();
            Value::Bool(ls == rs) // simplified
        }
    }
}

// ---------------------------------------------------------------------------
// Built-in functions (50+)
// ---------------------------------------------------------------------------

fn collect_numbers(args: &[Expr], provider: &dyn CellProvider) -> Vec<f64> {
    let mut nums = Vec::new();
    for arg in args {
        let val = evaluate(arg, provider);
        match val {
            Value::Number(n) => nums.push(n),
            Value::Array(rows) => {
                for row in &rows {
                    for cell in row {
                        if let Some(n) = cell.as_f64() {
                            nums.push(n);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    nums
}

fn eval_function(name: &str, args: &[Expr], provider: &dyn CellProvider) -> Value {
    match name {
        // --- Math ---
        "SUM" => {
            let nums = collect_numbers(args, provider);
            Value::Number(nums.iter().sum())
        }
        "AVERAGE" | "AVG" => {
            let nums = collect_numbers(args, provider);
            if nums.is_empty() {
                Value::Error(FormulaError::DivZero)
            } else {
                Value::Number(nums.iter().sum::<f64>() / nums.len() as f64)
            }
        }
        "COUNT" => {
            let nums = collect_numbers(args, provider);
            Value::Number(nums.len() as f64)
        }
        "COUNTA" => {
            let mut count = 0usize;
            for arg in args {
                let val = evaluate(arg, provider);
                match val {
                    Value::Number(_) | Value::Text(_) | Value::Bool(_) => count += 1,
                    Value::Array(rows) => {
                        for row in &rows {
                            for cell in row {
                                match cell {
                                    Value::Number(_) | Value::Text(_) | Value::Bool(_) => {
                                        count += 1
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Value::Number(count as f64)
        }
        "COUNTBLANK" => {
            let mut count = 0usize;
            for arg in args {
                let val = evaluate(arg, provider);
                match val {
                    Value::Number(0.0) => count += 1,
                    Value::Text(s) if s.is_empty() => count += 1,
                    Value::Array(rows) => {
                        for row in &rows {
                            for cell in row {
                                match cell {
                                    Value::Number(0.0) => count += 1,
                                    Value::Text(s) if s.is_empty() => count += 1,
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Value::Number(count as f64)
        }
        "MIN" => {
            let nums = collect_numbers(args, provider);
            Value::Number(nums.iter().copied().fold(f64::INFINITY, f64::min))
        }
        "MAX" => {
            let nums = collect_numbers(args, provider);
            Value::Number(nums.iter().copied().fold(f64::NEG_INFINITY, f64::max))
        }
        "ABS" => single_number(args, provider, |n| n.abs()),
        "ROUND" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let n = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
            let digits = evaluate(&args[1], provider).as_f64().unwrap_or(0.0);
            let factor = 10f64.powi(digits as i32);
            Value::Number((n * factor).round() / factor)
        }
        "ROUNDUP" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let n = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
            let digits = evaluate(&args[1], provider).as_f64().unwrap_or(0.0);
            let factor = 10f64.powi(digits as i32);
            Value::Number((n * factor).ceil() / factor)
        }
        "ROUNDDOWN" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let n = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
            let digits = evaluate(&args[1], provider).as_f64().unwrap_or(0.0);
            let factor = 10f64.powi(digits as i32);
            Value::Number((n * factor).floor() / factor)
        }
        "INT" => single_number(args, provider, |n| n.floor()),
        "MOD" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let a = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
            let b = evaluate(&args[1], provider).as_f64().unwrap_or(1.0);
            if b == 0.0 {
                Value::Error(FormulaError::DivZero)
            } else {
                Value::Number(a % b)
            }
        }
        "POWER" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let base = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
            let exp = evaluate(&args[1], provider).as_f64().unwrap_or(0.0);
            Value::Number(base.powf(exp))
        }
        "SQRT" => single_number(
            args,
            provider,
            |n| if n < 0.0 { f64::NAN } else { n.sqrt() },
        ),
        "LOG" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let n = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
            let base = if args.len() > 1 {
                evaluate(&args[1], provider).as_f64().unwrap_or(10.0)
            } else {
                10.0
            };
            Value::Number(n.log(base))
        }
        "LN" => single_number(args, provider, f64::ln),
        "LOG10" => single_number(args, provider, f64::log10),
        "EXP" => single_number(args, provider, f64::exp),
        "PI" => Value::Number(std::f64::consts::PI),
        "RAND" => Value::Number(rand_f64()),
        "RANDBETWEEN" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let lo = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
            let hi = evaluate(&args[1], provider).as_f64().unwrap_or(1.0);
            Value::Number(lo + (hi - lo) * rand_f64())
        }
        "CEILING" => single_number(args, provider, |n| n.ceil()),
        "FLOOR" => single_number(args, provider, |n| n.floor()),
        "TRUNC" => {
            if args.len() < 2 {
                single_number(args, provider, |n| n.trunc())
            } else {
                let n = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
                let digits = evaluate(&args[1], provider).as_f64().unwrap_or(0.0);
                let factor = 10f64.powi(digits as i32);
                Value::Number((n * factor).trunc() / factor)
            }
        }
        "SIGN" => single_number(args, provider, |n| {
            if n > 0.0 {
                1.0
            } else if n < 0.0 {
                -1.0
            } else {
                0.0
            }
        }),

        // --- Trigonometric ---
        "SIN" => single_number(args, provider, f64::sin),
        "COS" => single_number(args, provider, f64::cos),
        "TAN" => single_number(args, provider, f64::tan),
        "ASIN" => single_number(args, provider, f64::asin),
        "ACOS" => single_number(args, provider, f64::acos),
        "ATAN" => single_number(args, provider, f64::atan),
        "ATAN2" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let y = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
            let x = evaluate(&args[1], provider).as_f64().unwrap_or(0.0);
            Value::Number(y.atan2(x))
        }
        "RADIANS" => single_number(args, provider, |n| n * std::f64::consts::PI / 180.0),
        "DEGREES" => single_number(args, provider, |n| n * 180.0 / std::f64::consts::PI),

        // --- Statistical ---
        "MEDIAN" => {
            let mut nums = collect_numbers(args, provider);
            if nums.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = nums.len() / 2;
            let median = if nums.len().is_multiple_of(2) {
                (nums[mid - 1] + nums[mid]) / 2.0
            } else {
                nums[mid]
            };
            Value::Number(median)
        }
        "STDEV" | "STDEV.S" => {
            let nums = collect_numbers(args, provider);
            if nums.len() < 2 {
                return Value::Error(FormulaError::DivZero);
            }
            let mean = nums.iter().sum::<f64>() / nums.len() as f64;
            let variance =
                nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (nums.len() - 1) as f64;
            Value::Number(variance.sqrt())
        }
        "STDEV.P" | "STDEVP" => {
            let nums = collect_numbers(args, provider);
            if nums.is_empty() {
                return Value::Error(FormulaError::DivZero);
            }
            let mean = nums.iter().sum::<f64>() / nums.len() as f64;
            let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nums.len() as f64;
            Value::Number(variance.sqrt())
        }
        "VAR" | "VAR.S" => {
            let nums = collect_numbers(args, provider);
            if nums.len() < 2 {
                return Value::Error(FormulaError::DivZero);
            }
            let mean = nums.iter().sum::<f64>() / nums.len() as f64;
            let variance =
                nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (nums.len() - 1) as f64;
            Value::Number(variance)
        }
        "VAR.P" | "VARP" => {
            let nums = collect_numbers(args, provider);
            if nums.is_empty() {
                return Value::Error(FormulaError::DivZero);
            }
            let mean = nums.iter().sum::<f64>() / nums.len() as f64;
            let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nums.len() as f64;
            Value::Number(variance)
        }
        "PERCENTILE" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let mut nums = collect_numbers(&args[0..1], provider);
            let k = evaluate(&args[1], provider).as_f64().unwrap_or(0.0);
            if !(0.0..=1.0).contains(&k) {
                return Value::Error(FormulaError::Value);
            }
            nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            if nums.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let idx = k * (nums.len() - 1) as f64;
            let lo = idx.floor() as usize;
            let hi = idx.ceil() as usize;
            if lo == hi {
                Value::Number(nums[lo])
            } else {
                let frac = idx - lo as f64;
                Value::Number(nums[lo] * (1.0 - frac) + nums[hi] * frac)
            }
        }
        "LARGE" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let mut nums = collect_numbers(&args[0..1], provider);
            let k = evaluate(&args[1], provider).as_f64().unwrap_or(1.0) as usize;
            if k == 0 || k > nums.len() {
                return Value::Error(FormulaError::NA);
            }
            nums.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            Value::Number(nums[k - 1])
        }
        "SMALL" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let mut nums = collect_numbers(&args[0..1], provider);
            let k = evaluate(&args[1], provider).as_f64().unwrap_or(1.0) as usize;
            if k == 0 || k > nums.len() {
                return Value::Error(FormulaError::NA);
            }
            nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            Value::Number(nums[k - 1])
        }
        "RANK" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let target = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
            let nums = collect_numbers(&args[1..2], provider);
            let mut sorted: Vec<(usize, f64)> = nums.iter().copied().enumerate().collect();
            sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            for (rank, (idx, val)) in sorted.iter().enumerate() {
                if (*val - target).abs() < f64::EPSILON {
                    return Value::Number((rank + 1) as f64);
                }
                let _ = idx; // suppress warning
            }
            Value::Error(FormulaError::NA)
        }

        // --- Logical ---
        "IF" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let cond = evaluate(&args[0], provider);
            if cond.as_bool() {
                if args.len() > 1 {
                    evaluate(&args[1], provider)
                } else {
                    Value::Bool(true)
                }
            } else if args.len() > 2 {
                evaluate(&args[2], provider)
            } else {
                Value::Bool(false)
            }
        }
        "AND" => {
            for arg in args {
                if !evaluate(arg, provider).as_bool() {
                    return Value::Bool(false);
                }
            }
            Value::Bool(true)
        }
        "OR" => {
            for arg in args {
                if evaluate(arg, provider).as_bool() {
                    return Value::Bool(true);
                }
            }
            Value::Bool(false)
        }
        "NOT" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            Value::Bool(!evaluate(&args[0], provider).as_bool())
        }
        "XOR" => {
            let mut count = 0;
            for arg in args {
                if evaluate(arg, provider).as_bool() {
                    count += 1;
                }
            }
            Value::Bool(count % 2 == 1)
        }
        "IFERROR" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let val = evaluate(&args[0], provider);
            match val {
                Value::Error(_) => {
                    if args.len() > 1 {
                        evaluate(&args[1], provider)
                    } else {
                        Value::Text(String::new())
                    }
                }
                _ => val,
            }
        }
        "IFNA" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let val = evaluate(&args[0], provider);
            match val {
                Value::Error(FormulaError::NA) => {
                    if args.len() > 1 {
                        evaluate(&args[1], provider)
                    } else {
                        Value::Text(String::new())
                    }
                }
                _ => val,
            }
        }

        // --- Text ---
        "LEN" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let s = evaluate(&args[0], provider).as_str();
            Value::Number(s.len() as f64)
        }
        "LEFT" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let s = evaluate(&args[0], provider).as_str();
            let n = if args.len() > 1 {
                evaluate(&args[1], provider).as_f64().unwrap_or(1.0) as usize
            } else {
                1
            };
            Value::Text(s.chars().take(n).collect())
        }
        "RIGHT" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let s = evaluate(&args[0], provider).as_str();
            let n = if args.len() > 1 {
                evaluate(&args[1], provider).as_f64().unwrap_or(1.0) as usize
            } else {
                1
            };
            let chars: Vec<char> = s.chars().collect();
            let start = chars.len().saturating_sub(n);
            Value::Text(chars[start..].iter().collect())
        }
        "MID" => {
            if args.len() < 3 {
                return Value::Error(FormulaError::NA);
            }
            let s = evaluate(&args[0], provider).as_str();
            let start = evaluate(&args[1], provider).as_f64().unwrap_or(1.0) as usize;
            let len = evaluate(&args[2], provider).as_f64().unwrap_or(0.0) as usize;
            let chars: Vec<char> = s.chars().collect();
            let start_idx = start.saturating_sub(1).min(chars.len());
            let end_idx = (start_idx + len).min(chars.len());
            Value::Text(chars[start_idx..end_idx].iter().collect())
        }
        "UPPER" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            Value::Text(evaluate(&args[0], provider).as_str().to_uppercase())
        }
        "LOWER" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            Value::Text(evaluate(&args[0], provider).as_str().to_lowercase())
        }
        "TRIM" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            Value::Text(evaluate(&args[0], provider).as_str().trim().into())
        }
        "CONCATENATE" => {
            let mut result = String::new();
            for arg in args {
                result.push_str(&evaluate(arg, provider).as_str());
            }
            Value::Text(result)
        }
        "SUBSTITUTE" => {
            if args.len() < 3 {
                return Value::Error(FormulaError::NA);
            }
            let text = evaluate(&args[0], provider).as_str();
            let old = evaluate(&args[1], provider).as_str();
            let new = evaluate(&args[2], provider).as_str();
            let instance = if args.len() > 3 {
                evaluate(&args[3], provider).as_f64().unwrap_or(0.0) as usize
            } else {
                0 // replace all
            };
            if old.is_empty() {
                Value::Text(text)
            } else if instance == 0 {
                Value::Text(text.replace(&old, &new))
            } else {
                let mut count = 0;
                let mut result = String::new();
                let mut remaining: &str = &text;
                while let Some(pos) = remaining.find(&old) {
                    count += 1;
                    result.push_str(&remaining[..pos]);
                    if count == instance {
                        result.push_str(&new);
                    } else {
                        result.push_str(&old);
                    }
                    remaining = &remaining[pos + old.len()..];
                }
                result.push_str(remaining);
                Value::Text(result)
            }
        }
        "FIND" | "SEARCH" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let needle = evaluate(&args[0], provider).as_str();
            let haystack = evaluate(&args[1], provider).as_str();
            let start = if args.len() > 2 {
                evaluate(&args[2], provider).as_f64().unwrap_or(1.0) as usize
            } else {
                1
            };
            let search_haystack = if name == "SEARCH" {
                haystack.to_lowercase()
            } else {
                haystack.clone()
            };
            let search_needle = if name == "SEARCH" {
                needle.to_lowercase()
            } else {
                needle.clone()
            };
            let start_byte = search_haystack
                .char_indices()
                .nth(start.saturating_sub(1))
                .map(|(i, _)| i)
                .unwrap_or(search_haystack.len());
            match search_haystack[start_byte..].find(&*search_needle) {
                Some(pos) => {
                    let byte_pos = start_byte + pos;
                    let char_pos = haystack[..byte_pos].chars().count() + 1;
                    Value::Number(char_pos as f64)
                }
                None => Value::Error(FormulaError::Value),
            }
        }
        "REPLACE" => {
            if args.len() < 4 {
                return Value::Error(FormulaError::NA);
            }
            let text = evaluate(&args[0], provider).as_str();
            let start = evaluate(&args[1], provider).as_f64().unwrap_or(1.0) as usize;
            let len = evaluate(&args[2], provider).as_f64().unwrap_or(0.0) as usize;
            let new = evaluate(&args[3], provider).as_str();
            let chars: Vec<char> = text.chars().collect();
            let start_idx = start.saturating_sub(1).min(chars.len());
            let end_idx = (start_idx + len).min(chars.len());
            let mut result: String = chars[..start_idx].iter().collect();
            result.push_str(&new);
            result.extend(chars[end_idx..].iter());
            Value::Text(result)
        }
        "REPT" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let s = evaluate(&args[0], provider).as_str();
            let n = evaluate(&args[1], provider).as_f64().unwrap_or(0.0) as usize;
            Value::Text(s.repeat(n))
        }
        "TEXT" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            // Simplified: just convert to string
            Value::Text(evaluate(&args[0], provider).as_str())
        }
        "VALUE" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let s = evaluate(&args[0], provider).as_str();
            match s.parse::<f64>() {
                Ok(n) => Value::Number(n),
                Err(_) => Value::Error(FormulaError::Value),
            }
        }

        // --- Lookup / Reference ---
        "VLOOKUP" => {
            if args.len() < 3 {
                return Value::Error(FormulaError::NA);
            }
            let lookup_val = evaluate(&args[0], provider).as_str();
            // args[1] should be a range — evaluate it to get array
            let range_vals = evaluate(&args[1], provider);
            let col_idx = evaluate(&args[2], provider).as_f64().unwrap_or(1.0) as usize;
            if col_idx == 0 {
                return Value::Error(FormulaError::Value);
            }
            match range_vals {
                Value::Array(rows) if !rows.is_empty() => {
                    // Each row is a flat list of values from the range
                    // For VLOOKUP, we need to reinterpret: the range spans multiple columns
                    // Since our Array is flat per row of the range, we need the original dimensions
                    // Simplified: search first element of each "row" for match
                    let _ncols = if !rows[0].is_empty() {
                        rows[0].len()
                    } else {
                        1
                    };
                    let nrows = rows.len();
                    // Reconstruct as 2D: rows × cols
                    // Our range already gives us a flat array per range-row
                    for row in rows.iter().take(nrows) {
                        if let Some(v) = row.first() {
                            if v.as_str() == lookup_val {
                                if col_idx - 1 < row.len() {
                                    return row[col_idx - 1].clone();
                                } else {
                                    return Value::Error(FormulaError::Ref);
                                }
                            }
                        }
                    }
                    Value::Error(FormulaError::NA)
                }
                _ => Value::Error(FormulaError::NA),
            }
        }
        "HLOOKUP" => {
            // Similar to VLOOKUP but horizontal
            if args.len() < 3 {
                return Value::Error(FormulaError::NA);
            }
            let lookup_val = evaluate(&args[0], provider).as_str();
            let range_vals = evaluate(&args[1], provider);
            let row_idx = evaluate(&args[2], provider).as_f64().unwrap_or(1.0) as usize;
            if row_idx == 0 {
                return Value::Error(FormulaError::Value);
            }
            match range_vals {
                Value::Array(rows) if !rows.is_empty() => {
                    // Search first element of each column (i.e., first row)
                    if rows[0].is_empty() {
                        return Value::Error(FormulaError::NA);
                    }
                    for c in 0..rows[0].len() {
                        if let Some(v) = rows[0].get(c) {
                            if v.as_str() == lookup_val {
                                if row_idx - 1 < rows.len() {
                                    return rows[row_idx - 1]
                                        .get(c)
                                        .cloned()
                                        .unwrap_or(Value::Error(FormulaError::Ref));
                                } else {
                                    return Value::Error(FormulaError::Ref);
                                }
                            }
                        }
                    }
                    Value::Error(FormulaError::NA)
                }
                _ => Value::Error(FormulaError::NA),
            }
        }
        "INDEX" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let range_vals = evaluate(&args[0], provider);
            let row_num = evaluate(&args[1], provider).as_f64().unwrap_or(1.0) as usize;
            let col_num = if args.len() > 2 {
                evaluate(&args[2], provider).as_f64().unwrap_or(1.0) as usize
            } else {
                1
            };
            match range_vals {
                Value::Array(rows) => {
                    if row_num == 0 || row_num > rows.len() {
                        return Value::Error(FormulaError::Ref);
                    }
                    let row = &rows[row_num - 1];
                    if col_num == 0 || col_num > row.len() {
                        return Value::Error(FormulaError::Ref);
                    }
                    row[col_num - 1].clone()
                }
                v => {
                    if row_num == 1 && col_num == 1 {
                        v
                    } else {
                        Value::Error(FormulaError::Ref)
                    }
                }
            }
        }
        "MATCH" => {
            if args.len() < 2 {
                return Value::Error(FormulaError::NA);
            }
            let lookup_val = evaluate(&args[0], provider).as_str();
            let range_vals = evaluate(&args[1], provider);
            match range_vals {
                Value::Array(rows) => {
                    for (i, row) in rows.iter().enumerate() {
                        for cell in row {
                            if cell.as_str() == lookup_val {
                                return Value::Number((i + 1) as f64);
                            }
                        }
                    }
                    Value::Error(FormulaError::NA)
                }
                _ => Value::Error(FormulaError::NA),
            }
        }
        "ROW" => {
            // Returns row number of a reference — simplified
            if args.is_empty() {
                Value::Number(1.0) // current row (not tracked here)
            } else {
                match &args[0] {
                    Expr::CellRef { row, .. } => Value::Number((*row + 1) as f64),
                    _ => Value::Error(FormulaError::NA),
                }
            }
        }
        "COLUMN" => {
            if args.is_empty() {
                Value::Number(1.0)
            } else {
                match &args[0] {
                    Expr::CellRef { col, .. } => Value::Number((*col + 1) as f64),
                    _ => Value::Error(FormulaError::NA),
                }
            }
        }
        "ROWS" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            match &args[0] {
                Expr::Range {
                    start_row, end_row, ..
                } => Value::Number((*end_row - *start_row + 1) as f64),
                _ => Value::Number(1.0),
            }
        }
        "COLUMNS" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            match &args[0] {
                Expr::Range {
                    start_col, end_col, ..
                } => Value::Number((*end_col - *start_col + 1) as f64),
                _ => Value::Number(1.0),
            }
        }

        // --- Date/Time (simplified) ---
        "TODAY" => Value::Number(today_serial()),
        "NOW" => Value::Number(now_serial()),
        "YEAR" | "MONTH" | "DAY" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let serial = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
            let date = serial_to_date(serial);
            match name {
                "YEAR" => Value::Number(date.0 as f64),
                "MONTH" => Value::Number(date.1 as f64),
                "DAY" => Value::Number(date.2 as f64),
                _ => unreachable!(),
            }
        }
        "HOUR" | "MINUTE" | "SECOND" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let serial = evaluate(&args[0], provider).as_f64().unwrap_or(0.0);
            let time_frac = serial.fract();
            let total_secs = (time_frac * 86400.0).round() as u32;
            let (h, m, s) = (total_secs / 3600, (total_secs % 3600) / 60, total_secs % 60);
            match name {
                "HOUR" => Value::Number(h as f64),
                "MINUTE" => Value::Number(m as f64),
                "SECOND" => Value::Number(s as f64),
                _ => unreachable!(),
            }
        }

        // --- Info ---
        "ISBLANK" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let val = evaluate(&args[0], provider);
            Value::Bool(match &val {
                Value::Text(s) => s.is_empty(),
                Value::Number(0.0) => true,
                _ => false,
            })
        }
        "ISERROR" | "ISERR" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let val = evaluate(&args[0], provider);
            Value::Bool(matches!(val, Value::Error(_)))
        }
        "ISNUMBER" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            Value::Bool(matches!(evaluate(&args[0], provider), Value::Number(_)))
        }
        "ISTEXT" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            Value::Bool(matches!(evaluate(&args[0], provider), Value::Text(_)))
        }
        "ISNA" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            Value::Bool(matches!(
                evaluate(&args[0], provider),
                Value::Error(FormulaError::NA)
            ))
        }
        "TYPE" => {
            if args.is_empty() {
                return Value::Error(FormulaError::NA);
            }
            let val = evaluate(&args[0], provider);
            Value::Number(match val {
                Value::Number(_) => 1.0,
                Value::Text(_) => 2.0,
                Value::Bool(_) => 4.0,
                Value::Error(_) => 16.0,
                Value::Array(_) => 64.0,
            })
        }
        "N" => {
            if args.is_empty() {
                return Value::Number(0.0);
            }
            let val = evaluate(&args[0], provider);
            Value::Number(val.as_f64().unwrap_or(0.0))
        }

        _ => Value::Error(FormulaError::Name),
    }
}

fn single_number(args: &[Expr], provider: &dyn CellProvider, f: impl Fn(f64) -> f64) -> Value {
    if args.is_empty() {
        return Value::Error(FormulaError::NA);
    }
    match evaluate(&args[0], provider).as_f64() {
        Some(n) => Value::Number(f(n)),
        None => Value::Error(FormulaError::Value),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn rand_f64() -> f64 {
    // Simple pseudo-random using system time
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // xorshift32
    let mut state = nanos.wrapping_add(1);
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    (state as f64) / (u32::MAX as f64)
}

/// Excel serial date number for today.
fn today_serial() -> f64 {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let days_since_epoch = duration.as_secs() / 86400;
    // Excel epoch is 1900-01-01 = serial 1, Unix epoch is 1970-01-01
    // Days between 1900-01-01 and 1970-01-01 = 25569
    (days_since_epoch + 25569) as f64
}

/// Excel serial date number for now (with time fraction).
fn now_serial() -> f64 {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let days = duration.as_secs() as f64 / 86400.0;
    days + 25569.0
}

/// Convert Excel serial number to (year, month, day).
fn serial_to_date(serial: f64) -> (u32, u32, u32) {
    // Simplified: convert from Excel serial to a date
    let days = serial as i64 - 25569; // days since Unix epoch
    let _secs = days * 86400;
    // Use chrono-free approach
    let mut year = 1970u32;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }
    let leap = is_leap_year(year);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u32;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        month += 1;
    }
    let day = remaining as u32 + 1;
    (year, month, day)
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}
