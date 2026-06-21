use std::io::Write;
use std::path::Path;

use tench_docs_lib::ui::DocsApp;
use tench_ui_automation_core::UiAutomationCaptureRequest;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::TestHarness;

fn save_png(harness: &mut TestHarness, name: &str) {
    let capture = harness.automation_capture(UiAutomationCaptureRequest::default());
    let dir = Path::new("../../plans/ui");
    std::fs::create_dir_all(dir).unwrap();
    let path = dir.join(format!("docs_{name}.png"));
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(&capture.png_bytes).unwrap();
    eprintln!("saved {}", path.display());
}

#[test]
#[ignore]
fn capture_docs_screenshots() {
    let mut harness =
        TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0));

    // Default state
    save_png(&mut harness, "default");

    // Open File menu
    let _ = harness.automation_click_debug_id("docs.menu.file");
    save_png(&mut harness, "file_menu");

    // Open Edit menu
    let _ = harness.automation_click_debug_id("docs.menu.edit");
    save_png(&mut harness, "edit_menu");

    // Open View menu
    let _ = harness.automation_click_debug_id("docs.menu.view");
    save_png(&mut harness, "view_menu");

    // Open Insert menu
    let _ = harness.automation_click_debug_id("docs.menu.insert");
    save_png(&mut harness, "insert_menu");

    // Open Format menu
    let _ = harness.automation_click_debug_id("docs.menu.format");
    save_png(&mut harness, "format_menu");
}
