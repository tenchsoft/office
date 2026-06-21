use std::io::Write;
use std::path::Path;

use tench_kodocs_lib::ui::KodocsApp;
use tench_ui_automation_core::UiAutomationCaptureRequest;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::TestHarness;

fn save_png(harness: &mut TestHarness, name: &str) {
    let capture = harness.automation_capture(UiAutomationCaptureRequest::default());
    let dir = Path::new("../../plans/ui");
    std::fs::create_dir_all(dir).unwrap();
    let path = dir.join(format!("kodocs_{name}.png"));
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(&capture.png_bytes).unwrap();
    eprintln!("saved {}", path.display());
}

#[test]
#[ignore]
fn capture_kodocs_screenshots() {
    let mut harness = TestHarness::with_config(
        KodocsApp::new(),
        HarnessConfig::with_viewport(1280.0, 820.0),
    );

    // Default state
    save_png(&mut harness, "default");

    // Open file menu (Korean)
    let _ = harness.automation_click_debug_id("kodocs.menu.file");
    save_png(&mut harness, "file_menu");

    // Open edit menu
    let _ = harness.automation_click_debug_id("kodocs.menu.edit");
    save_png(&mut harness, "edit_menu");

    // Open insert menu
    let _ = harness.automation_click_debug_id("kodocs.menu.insert");
    save_png(&mut harness, "insert_menu");

    // Open format menu
    let _ = harness.automation_click_debug_id("kodocs.menu.format");
    save_png(&mut harness, "format_menu");
}
