#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tench_ui::run_native_with_config(
        tench_ui::NativeConfig {
            title: "Tench Sheets".into(),
            width: 1440.0,
            height: 900.0,
            resizable: true,
            decorations: false,
        },
        |backend| backend.set_root(tench_sheets_lib::ui::SheetsApp::new()),
    );
}
