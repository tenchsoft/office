#[test]
fn formula_api_remains_public() {
    let expr = tench_sheets_core::formula::parse_formula("A1+B2").expect("formula");
    let deps = tench_sheets_core::formula::collect_dependencies(&expr);

    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&(0, 0)));
    assert!(deps.contains(&(1, 1)));
}
