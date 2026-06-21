use std::io::Write;
use std::path::Path;

use tench_slides_lib::ui::SlidesApp;
use tench_ui_automation_core::UiAutomationCaptureRequest;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::TestHarness;

fn save_png(harness: &mut TestHarness, name: &str) {
    let capture = harness.automation_capture(UiAutomationCaptureRequest::default());
    let dir = Path::new("../../plans/ui");
    std::fs::create_dir_all(dir).unwrap();
    let path = dir.join(format!("slides_{name}.png"));
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(&capture.png_bytes).unwrap();
    eprintln!("saved {}", path.display());
}

#[test]
#[ignore]
fn capture_slides_screenshots() {
    let mut harness = TestHarness::with_config(
        SlidesApp::new(),
        HarnessConfig::with_viewport(1280.0, 720.0),
    );

    // Default state
    save_png(&mut harness, "default");

    // After adding a text element
    let _ = harness.automation_click_debug_id("slides.toolbar.text");
    save_png(&mut harness, "text_added");
}
