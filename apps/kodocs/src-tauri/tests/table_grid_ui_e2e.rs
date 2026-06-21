use tench_kodocs_lib::ui::KodocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(
        KodocsApp::new(),
        HarnessConfig::with_viewport(1440.0, 900.0),
    )
}

fn open_table_grid(h: &mut TestHarness) {
    click(h, "kodocs.toolbar.insert_table");
}

#[test]
fn table_grid_1x1_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.1x1");
}

#[test]
fn table_grid_1x2_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.1x2");
}

#[test]
fn table_grid_1x3_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.1x3");
}

#[test]
fn table_grid_1x4_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.1x4");
}

#[test]
fn table_grid_1x5_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.1x5");
}

#[test]
fn table_grid_2x1_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.2x1");
}

#[test]
fn table_grid_2x2_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.2x2");
}

#[test]
fn table_grid_2x3_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.2x3");
}

#[test]
fn table_grid_2x4_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.2x4");
}

#[test]
fn table_grid_2x5_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.2x5");
}

#[test]
fn table_grid_3x1_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.3x1");
}

#[test]
fn table_grid_3x2_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.3x2");
}

#[test]
fn table_grid_3x3_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.3x3");
}

#[test]
fn table_grid_3x4_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.3x4");
}

#[test]
fn table_grid_3x5_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.3x5");
}

#[test]
fn table_grid_4x1_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.4x1");
}

#[test]
fn table_grid_4x2_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.4x2");
}

#[test]
fn table_grid_4x3_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.4x3");
}

#[test]
fn table_grid_4x4_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.4x4");
}

#[test]
fn table_grid_4x5_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.4x5");
}

#[test]
fn table_grid_5x1_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.5x1");
}

#[test]
fn table_grid_5x2_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.5x2");
}

#[test]
fn table_grid_5x3_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.5x3");
}

#[test]
fn table_grid_5x4_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.5x4");
}

#[test]
fn table_grid_5x5_present() {
    let mut h = make_harness();
    open_table_grid(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.table_grid.cell.5x5");
}
