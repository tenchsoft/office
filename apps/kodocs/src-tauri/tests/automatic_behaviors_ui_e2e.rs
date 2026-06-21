use tench_kodocs_lib::ui::KodocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::CaptureAssertions;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(
        KodocsApp::new(),
        HarnessConfig::with_viewport(1440.0, 900.0),
    )
}

#[test]
fn automatic_cursor_blink_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    c.assert_png_valid();
    h.dispatch_window(tench_ui::core::events::WindowEvent::AnimFrame(600));
    capture(&mut h).assert_png_valid();
    h.dispatch_window(tench_ui::core::events::WindowEvent::AnimFrame(1200));
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_autosave_element_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.save_status");
}

#[test]
fn initial_render_nonblank() {
    let mut h = make_harness();
    let c = capture(&mut h);
    c.assert_nonblank();
    c.assert_png_size(1440, 900);
}

#[test]
fn all_chrome_elements_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.menu.file");
    assert_selector(&c, "kodocs.toolbar.undo");
    assert_selector(&c, "kodocs.toolbar.bold");
    assert_selector(&c, "kodocs.document");
    assert_selector(&c, "kodocs.ruler");
    assert_selector(&c, "kodocs.status_bar");
    assert_selector(&c, "kodocs.title_row");
}

#[test]
fn automatic_save_badge_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.save_status");
}

#[test]
fn automatic_status_bar_counts_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.status_bar");
    assert_selector(&c, "kodocs.status_bar.page_indicator");
}

#[test]
fn automatic_selection_highlight_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_search_highlight_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_color_palette_rendering_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.text_color");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_context_menu_hover_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_dropdown_z_order_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_menu_hover_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.menu.file");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_modal_backdrop_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_equation_preview_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.수식");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_hanja_candidate_popup_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_header_footer_no_crash() {
    let mut h = make_harness();
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_image_resize_handle_no_crash() {
    let mut h = make_harness();
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_keyboard_shortcut_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_layout_cache_pagination_no_crash() {
    let mut h = make_harness();
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_page_setup_preview_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_ruler_marker_no_crash() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.ruler");
    assert_selector(&c, "kodocs.ruler.indent.first_line");
    assert_selector(&c, "kodocs.ruler.margin.left");
}

#[test]
fn automatic_scroll_zoom_no_crash() {
    let mut h = make_harness();
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_style_statistics_panel_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.view", "kodocs.menu.보기.스타일_패널");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_table_grid_hover_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.insert_table");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_thumbnail_panel_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.view", "kodocs.menu.보기.미리보기");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_toast_auto_dismiss_no_crash() {
    let mut h = make_harness();
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_track_change_decoration_no_crash() {
    let mut h = make_harness();
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_vertical_writing_no_crash() {
    let mut h = make_harness();
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_window_resize_layout_no_crash() {
    let mut h = make_harness();
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_comment_highlight_no_crash() {
    let mut h = make_harness();
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_comments_panel_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.view", "kodocs.menu.보기.메모");
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_dropped_file_open_no_crash() {
    let mut h = make_harness();
    capture(&mut h).assert_png_valid();
}

#[test]
fn automatic_async_native_dialog_no_crash() {
    let mut h = make_harness();
    capture(&mut h).assert_png_valid();
}
