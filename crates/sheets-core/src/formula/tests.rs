use super::*;
use std::collections::{HashMap, HashSet};

struct TestProvider {
    values: HashMap<(usize, usize), String>,
    names: HashMap<String, (usize, usize, usize, usize)>,
}

impl TestProvider {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
            names: HashMap::new(),
        }
    }
    fn set(&mut self, row: usize, col: usize, val: &str) {
        self.values.insert((row, col), val.into());
    }
}

impl CellProvider for TestProvider {
    fn get_cell_value(&self, row: usize, col: usize) -> Option<&str> {
        self.values.get(&(row, col)).map(|s| s.as_str())
    }
    fn get_cell_display(&self, row: usize, col: usize) -> Option<&str> {
        self.values.get(&(row, col)).map(|s| s.as_str())
    }
    fn resolve_name(&self, name: &str) -> Option<(usize, usize, usize, usize)> {
        self.names.get(name).copied()
    }
}

fn eval_str(formula: &str, provider: &dyn CellProvider) -> String {
    let expr = parse_formula(formula).unwrap();
    evaluate(&expr, provider).as_str()
}

#[test]
fn parse_number() {
    let expr = parse_formula("42.5").unwrap();
    assert_eq!(expr, Expr::Number(42.5));
}

#[test]
fn parse_text() {
    let expr = parse_formula("\"hello\"").unwrap();
    assert_eq!(expr, Expr::Text("hello".into()));
}

#[test]
fn parse_single_cell_ref() {
    let expr = parse_formula("A1").unwrap();
    assert_eq!(expr, Expr::CellRef { col: 0, row: 0 });
}

#[test]
fn parse_range() {
    let expr = parse_formula("A1:B3").unwrap();
    assert_eq!(
        expr,
        Expr::Range {
            start_col: 0,
            start_row: 0,
            end_col: 1,
            end_row: 2,
        }
    );
}

#[test]
fn parse_function() {
    let expr = parse_formula("SUM(A1:A3)").unwrap();
    assert!(matches!(expr, Expr::Function { name, .. } if name == "SUM"));
}

#[test]
fn parse_binary_ops() {
    let expr = parse_formula("1+2*3").unwrap();
    // Should be 1 + (2*3) due to precedence
    match expr {
        Expr::BinOp { op: BinOp::Add, .. } => {}
        _ => panic!("Expected Add, got {:?}", expr),
    }
}

#[test]
fn eval_addition() {
    let p = TestProvider::new();
    assert_eq!(eval_str("1+2", &p), "3");
}

#[test]
fn eval_subtraction() {
    let p = TestProvider::new();
    assert_eq!(eval_str("10-3", &p), "7");
}

#[test]
fn eval_multiplication() {
    let p = TestProvider::new();
    assert_eq!(eval_str("4*5", &p), "20");
}

#[test]
fn eval_division() {
    let p = TestProvider::new();
    assert_eq!(eval_str("10/4", &p), "2.5");
}

#[test]
fn eval_div_by_zero() {
    let p = TestProvider::new();
    let expr = parse_formula("1/0").unwrap();
    let val = evaluate(&expr, &p);
    assert!(matches!(val, Value::Error(FormulaError::DivZero)));
}

#[test]
fn eval_power() {
    let p = TestProvider::new();
    assert_eq!(eval_str("2^10", &p), "1024");
}

#[test]
fn eval_concat() {
    let p = TestProvider::new();
    assert_eq!(eval_str("\"hello\"&\" \"&\"world\"", &p), "hello world");
}

#[test]
fn eval_comparison() {
    let p = TestProvider::new();
    assert_eq!(eval_str("1<2", &p), "TRUE");
    assert_eq!(eval_str("2<1", &p), "FALSE");
    assert_eq!(eval_str("1=1", &p), "TRUE");
    assert_eq!(eval_str("1<>2", &p), "TRUE");
}

#[test]
fn eval_cell_ref() {
    let mut p = TestProvider::new();
    p.set(0, 0, "42");
    assert_eq!(eval_str("A1", &p), "42");
}

#[test]
fn eval_sum_function() {
    let mut p = TestProvider::new();
    p.set(0, 0, "10");
    p.set(1, 0, "20");
    p.set(2, 0, "30");
    assert_eq!(eval_str("SUM(A1:A3)", &p), "60");
}

#[test]
fn eval_average_function() {
    let mut p = TestProvider::new();
    p.set(0, 0, "10");
    p.set(1, 0, "20");
    p.set(2, 0, "30");
    assert_eq!(eval_str("AVERAGE(A1:A3)", &p), "20");
}

#[test]
fn eval_count_function() {
    let mut p = TestProvider::new();
    p.set(0, 0, "10");
    p.set(1, 0, "20");
    p.set(2, 0, "hello");
    assert_eq!(eval_str("COUNT(A1:A3)", &p), "2");
}

#[test]
fn eval_min_max() {
    let mut p = TestProvider::new();
    p.set(0, 0, "5");
    p.set(1, 0, "3");
    p.set(2, 0, "8");
    assert_eq!(eval_str("MIN(A1:A3)", &p), "3");
    assert_eq!(eval_str("MAX(A1:A3)", &p), "8");
}

#[test]
fn eval_if_function() {
    let p = TestProvider::new();
    assert_eq!(eval_str("IF(1>0,\"yes\",\"no\")", &p), "yes");
    assert_eq!(eval_str("IF(0>1,\"yes\",\"no\")", &p), "no");
}

#[test]
fn eval_abs_round() {
    let p = TestProvider::new();
    assert_eq!(eval_str("ABS(-5)", &p), "5");
    assert_eq!(eval_str("ROUND(3.14159,2)", &p), "3.14");
}

#[test]
fn eval_sqrt() {
    let p = TestProvider::new();
    assert_eq!(eval_str("SQRT(144)", &p), "12");
}

#[test]
fn eval_text_functions() {
    let p = TestProvider::new();
    assert_eq!(eval_str("LEN(\"hello\")", &p), "5");
    assert_eq!(eval_str("UPPER(\"hello\")", &p), "HELLO");
    assert_eq!(eval_str("LOWER(\"HELLO\")", &p), "hello");
    assert_eq!(eval_str("LEFT(\"hello\",3)", &p), "hel");
    assert_eq!(eval_str("RIGHT(\"hello\",2)", &p), "lo");
    assert_eq!(eval_str("MID(\"hello\",2,3)", &p), "ell");
}

#[test]
fn eval_concatenate() {
    let p = TestProvider::new();
    assert_eq!(eval_str("CONCATENATE(\"a\",\"b\",\"c\")", &p), "abc");
}

#[test]
fn eval_and_or_not() {
    let p = TestProvider::new();
    assert_eq!(eval_str("AND(1,1)", &p), "TRUE");
    assert_eq!(eval_str("AND(1,0)", &p), "FALSE");
    assert_eq!(eval_str("OR(0,1)", &p), "TRUE");
    assert_eq!(eval_str("OR(0,0)", &p), "FALSE");
    assert_eq!(eval_str("NOT(1)", &p), "FALSE");
}

#[test]
fn eval_pi() {
    let p = TestProvider::new();
    let result = eval_str("PI()", &p);
    assert!(result.starts_with("3.14"));
}

#[test]
fn eval_nested_formulas() {
    let mut p = TestProvider::new();
    p.set(0, 0, "10");
    p.set(1, 0, "20");
    assert_eq!(eval_str("SUM(A1:A2)*2", &p), "60");
}

#[test]
fn eval_mod_function() {
    let p = TestProvider::new();
    assert_eq!(eval_str("MOD(10,3)", &p), "1");
}

#[test]
fn eval_iferror() {
    let p = TestProvider::new();
    assert_eq!(eval_str("IFERROR(1/0,\"err\")", &p), "err");
    assert_eq!(eval_str("IFERROR(42,\"err\")", &p), "42");
}

#[test]
fn eval_percent() {
    let p = TestProvider::new();
    assert_eq!(eval_str("50%", &p), "0.5");
}

#[test]
fn eval_unary_negation() {
    let p = TestProvider::new();
    assert_eq!(eval_str("-5", &p), "-5");
    assert_eq!(eval_str("-(3+2)", &p), "-5");
}

#[test]
fn collect_deps_test() {
    let expr = parse_formula("A1+B2*SUM(C3:C5)").unwrap();
    let deps = collect_dependencies(&expr);
    assert!(deps.contains(&(0, 0))); // A1
    assert!(deps.contains(&(1, 1))); // B2
    assert!(deps.contains(&(2, 2))); // C3
    assert!(deps.contains(&(3, 2))); // C4
    assert!(deps.contains(&(4, 2))); // C5
    assert_eq!(deps.len(), 5);
}

#[test]
fn circular_detection() {
    let mut formula_cells = HashSet::new();
    formula_cells.insert((0, 0));
    formula_cells.insert((1, 1));
    let mut parsed = HashMap::new();
    parsed.insert((0, 0), parse_formula("B2").unwrap());
    parsed.insert((1, 1), parse_formula("A1").unwrap());
    let result = build_eval_order(&formula_cells, &parsed);
    assert!(matches!(result, Err(FormulaError::Circular)));
}

#[test]
fn topological_order() {
    let mut formula_cells = HashSet::new();
    formula_cells.insert((0, 0));
    formula_cells.insert((1, 0));
    let mut parsed = HashMap::new();
    parsed.insert((0, 0), parse_formula("5").unwrap());
    parsed.insert((1, 0), parse_formula("A1*2").unwrap());
    let order = build_eval_order(&formula_cells, &parsed).unwrap();
    assert_eq!(order[0], (0, 0));
    assert_eq!(order[1], (1, 0));
}

#[test]
fn evaluate_all_formulas_test() {
    let grid = vec![
        vec!["5".into(), "10".into()],
        vec!["=A1*2".into(), "=B1+A2".into()],
    ];
    let names = HashMap::new();
    let result = evaluate_all_formulas(&grid, &names);
    assert_eq!(result.get(&(1, 0)).unwrap(), "10");
    assert_eq!(result.get(&(1, 1)).unwrap(), "20");
}

#[test]
fn parse_cell_ref_multi_letter() {
    assert_eq!(parse_cell_ref("AA1"), Some((26, 0)));
    assert_eq!(parse_cell_ref("AB5"), Some((27, 4)));
    assert_eq!(parse_cell_ref("Z1"), Some((25, 0)));
}

#[test]
fn eval_boolean_literals() {
    let p = TestProvider::new();
    assert_eq!(eval_str("TRUE", &p), "TRUE");
    assert_eq!(eval_str("FALSE", &p), "FALSE");
}

#[test]
fn eval_substitute() {
    let p = TestProvider::new();
    assert_eq!(
        eval_str("SUBSTITUTE(\"hello world\",\"o\",\"0\")", &p),
        "hell0 w0rld"
    );
}

#[test]
fn eval_repeat() {
    let p = TestProvider::new();
    assert_eq!(eval_str("REPT(\"ab\",3)", &p), "ababab");
}

#[test]
fn eval_stdev() {
    let mut p = TestProvider::new();
    p.set(0, 0, "2");
    p.set(1, 0, "4");
    p.set(2, 0, "4");
    p.set(3, 0, "4");
    p.set(4, 0, "5");
    p.set(5, 0, "5");
    p.set(6, 0, "7");
    p.set(7, 0, "9");
    let result = eval_str("STDEV(A1:A8)", &p);
    let val: f64 = result.parse().unwrap();
    assert!((val - 2.0).abs() < 0.2);
}

#[test]
fn eval_median() {
    let mut p = TestProvider::new();
    p.set(0, 0, "1");
    p.set(1, 0, "2");
    p.set(2, 0, "3");
    p.set(3, 0, "4");
    p.set(4, 0, "5");
    assert_eq!(eval_str("MEDIAN(A1:A5)", &p), "3");
}
