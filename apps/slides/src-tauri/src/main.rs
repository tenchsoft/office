fn main() {
    tench_ui::run_native_with_config(
        tench_ui::NativeConfig {
            title: "Tench Slides".into(),
            width: 1440.0,
            height: 900.0,
            resizable: true,
        },
        |backend| backend.set_root(tench_slides_lib::ui::SlidesApp::new()),
    );
}
