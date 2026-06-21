use tench_slides_lib::ui::SlidesApp;
use tench_ui_automation_core::{
    find_node, UiAutomationAction, UiAutomationCapture, UiAutomationCaptureRequest,
    UiAutomationKey, UiAutomationModifiers, UiAutomationNode, UiAutomationRect,
    UiAutomationSelector,
};
use tench_ui_test::{harness::HarnessConfig, snapshot::is_nonblank, TestHarness};

fn harness() -> TestHarness {
    TestHarness::with_config(
        SlidesApp::new(),
        HarnessConfig::with_viewport(1280.0, 720.0),
    )
}

fn decode_png(capture: &UiAutomationCapture) -> image::RgbaImage {
    assert!(capture.png_bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    let image = image::load_from_memory(&capture.png_bytes)
        .expect("valid automation png")
        .to_rgba8();
    assert_eq!(image.width(), capture.width);
    assert_eq!(image.height(), capture.height);
    assert!(
        is_nonblank(&image, 64),
        "capture should be visually non-blank"
    );
    image
}

fn tree(capture: &UiAutomationCapture) -> &UiAutomationNode {
    capture.ui_tree.as_ref().expect("automation tree")
}

fn selector(debug_id: &str) -> UiAutomationSelector {
    UiAutomationSelector::ByDebugId {
        debug_id: debug_id.to_string(),
    }
}

fn assert_selector(capture: &UiAutomationCapture, debug_id: &str) {
    assert!(
        find_node(tree(capture), &selector(debug_id)).is_some(),
        "missing selector {debug_id}"
    );
}

fn assert_absent(capture: &UiAutomationCapture, debug_id: &str) {
    assert!(
        find_node(tree(capture), &selector(debug_id)).is_none(),
        "unexpected selector {debug_id}"
    );
}

fn assert_bounds_inside(capture: &UiAutomationCapture, debug_id: &str) {
    let node = find_node(tree(capture), &selector(debug_id)).expect("selector node");
    assert_rect_inside(
        &node.bounds,
        capture.width as f64,
        capture.height as f64,
        debug_id,
    );
}

fn assert_rect_inside(rect: &UiAutomationRect, width: f64, height: f64, label: &str) {
    assert!(
        rect.width > 0.0,
        "{label} width should be positive: {rect:?}"
    );
    assert!(
        rect.height > 0.0,
        "{label} height should be positive: {rect:?}"
    );
    assert!(rect.x >= 0.0, "{label} x should be on-screen: {rect:?}");
    assert!(rect.y >= 0.0, "{label} y should be on-screen: {rect:?}");
    assert!(
        rect.x + rect.width <= width,
        "{label} should fit viewport width: {rect:?} / {width}"
    );
    assert!(
        rect.y + rect.height <= height,
        "{label} should fit viewport height: {rect:?} / {height}"
    );
}

fn click(harness: &mut TestHarness, debug_id: &str) -> UiAutomationCapture {
    harness
        .automation_action(UiAutomationAction::Click {
            selector: selector(debug_id),
            modifiers: Default::default(),
        })
        .unwrap_or_else(|err| panic!("click {debug_id}: {err:?}"))
}

fn key(
    harness: &mut TestHarness,
    key: UiAutomationKey,
    modifiers: UiAutomationModifiers,
) -> UiAutomationCapture {
    harness
        .automation_action(UiAutomationAction::KeyPress { key, modifiers })
        .expect("key press")
}

#[test]
fn slides_plan_toolbar_controls_are_visible_and_clickable_ui_e2e() {
    let mut harness = harness();
    let capture = harness.automation_capture(UiAutomationCaptureRequest::default());
    decode_png(&capture);

    for debug_id in [
        "slides.toolbar.new",
        "slides.toolbar.open",
        "slides.toolbar.save",
        "slides.toolbar.undo",
        "slides.toolbar.redo",
        "slides.toolbar.text",
        "slides.toolbar.shape",
        "slides.toolbar.image",
        "slides.toolbar.play",
        "slides.canvas",
        "slides.canvas.page",
        "slides.properties",
        "slides.notes",
    ] {
        assert_selector(&capture, debug_id);
        assert_bounds_inside(&capture, debug_id);
    }

    let after_new = click(&mut harness, "slides.toolbar.new");
    decode_png(&after_new);
    assert_selector(&after_new, "slides.filmstrip.slide.1");
    assert_selector(&after_new, "slides.canvas.element.0");
    assert_selector(&after_new, "slides.properties.delete");
    assert_bounds_inside(&after_new, "slides.canvas.element.0");

    let after_shape = click(&mut harness, "slides.toolbar.shape");
    decode_png(&after_shape);
    assert_selector(&after_shape, "slides.modal.shape_selector");
    assert_selector(&after_shape, "slides.shape.rectangle");
    assert_bounds_inside(&after_shape, "slides.shape.rectangle");

    let rectangle = click(&mut harness, "slides.shape.rectangle");
    decode_png(&rectangle);
    assert_absent(&rectangle, "slides.modal.shape_selector");
    assert_selector(&rectangle, "slides.canvas.element.1");
}

#[test]
fn slides_plan_automatic_rendering_and_keyboard_flow_use_real_events_ui_e2e() {
    let mut harness = harness();

    let before = harness.automation_capture(UiAutomationCaptureRequest::default());
    let before_image = decode_png(&before);

    let after_text = click(&mut harness, "slides.toolbar.text");
    let text_image = decode_png(&after_text);
    assert!(
        before_image.as_raw() != text_image.as_raw(),
        "text insertion should change canvas/properties rendering"
    );
    assert_selector(&after_text, "slides.canvas.element.0");

    let after_play = click(&mut harness, "slides.toolbar.play");
    let play_image = decode_png(&after_play);
    assert!(
        text_image.as_raw() != play_image.as_raw(),
        "presentation overlay should produce a visible automatic UI change"
    );
    assert_selector(&after_play, "slides.presenter.overlay");

    let after_escape = key(
        &mut harness,
        UiAutomationKey::Escape,
        UiAutomationModifiers::default(),
    );
    decode_png(&after_escape);
    assert_absent(&after_escape, "slides.presenter.overlay");

    let after_find_shortcut = key(
        &mut harness,
        UiAutomationKey::Character("f".to_string()),
        UiAutomationModifiers {
            control: true,
            ..UiAutomationModifiers::default()
        },
    );
    decode_png(&after_find_shortcut);
    assert_selector(&after_find_shortcut, "slides.modal.find_replace");
    assert_selector(&after_find_shortcut, "slides.find.find");
}

#[test]
fn slides_plan_modals_properties_and_shortcuts_use_real_events_ui_e2e() {
    let mut harness = harness();

    let open_modal = click(&mut harness, "slides.toolbar.open");
    decode_png(&open_modal);
    assert_selector(&open_modal, "slides.modal.open_file");
    assert_selector(&open_modal, "slides.modal.ok");
    assert_selector(&open_modal, "slides.modal.cancel");
    let closed = click(&mut harness, "slides.modal.cancel");
    decode_png(&closed);
    assert_absent(&closed, "slides.modal.open_file");

    let image_modal = click(&mut harness, "slides.toolbar.image");
    decode_png(&image_modal);
    assert_selector(&image_modal, "slides.modal.insert_image");
    assert_selector(&image_modal, "slides.modal.cancel");
    let image_closed = click(&mut harness, "slides.modal.cancel");
    decode_png(&image_closed);
    assert_absent(&image_closed, "slides.modal.insert_image");

    let text_added = click(&mut harness, "slides.toolbar.text");
    decode_png(&text_added);
    assert_selector(&text_added, "slides.canvas.element.0");

    let shape_modal = click(&mut harness, "slides.toolbar.shape");
    decode_png(&shape_modal);
    assert_selector(&shape_modal, "slides.shape.rectangle");
    let shape_added = click(&mut harness, "slides.shape.rectangle");
    decode_png(&shape_added);
    for debug_id in [
        "slides.properties.bring_forward",
        "slides.properties.send_backward",
        "slides.properties.duplicate",
        "slides.properties.delete",
        "slides.properties.x",
        "slides.properties.y",
        "slides.properties.width",
        "slides.properties.height",
        "slides.properties.rotation",
        "slides.properties.opacity",
        "slides.properties.fill_color",
        "slides.properties.background",
        "slides.properties.theme",
    ] {
        assert_selector(&shape_added, debug_id);
        assert_bounds_inside(&shape_added, debug_id);
    }
    let line_modal = click(&mut harness, "slides.toolbar.shape");
    decode_png(&line_modal);
    let line_added = click(&mut harness, "slides.shape.line");
    decode_png(&line_added);
    assert_selector(&line_added, "slides.properties.border_color");
    assert_bounds_inside(&line_added, "slides.properties.border_color");

    let background = click(&mut harness, "slides.properties.background");
    let background_image = decode_png(&background);
    assert_selector(&background, "slides.modal.background");
    assert_selector(&background, "slides.background.navy");
    let navy = click(&mut harness, "slides.background.navy");
    let navy_image = decode_png(&navy);
    assert!(
        background_image.as_raw() != navy_image.as_raw(),
        "background swatch click should visibly update the slide preview"
    );
    let closed_background = key(
        &mut harness,
        UiAutomationKey::Escape,
        UiAutomationModifiers::default(),
    );
    decode_png(&closed_background);
    assert_absent(&closed_background, "slides.modal.background");

    let theme = click(&mut harness, "slides.properties.theme");
    decode_png(&theme);
    assert_selector(&theme, "slides.modal.theme");
    assert_selector(&theme, "slides.theme.dark");
    let dark = click(&mut harness, "slides.theme.dark");
    decode_png(&dark);
    assert_selector(&dark, "slides.modal.theme");

    let export = key(
        &mut harness,
        UiAutomationKey::Character("e".to_string()),
        UiAutomationModifiers {
            control: true,
            ..UiAutomationModifiers::default()
        },
    );
    decode_png(&export);
    assert_selector(&export, "slides.modal.export");
    assert_selector(&export, "slides.export.png");

    let png = click(&mut harness, "slides.export.png");
    decode_png(&png);
    assert_selector(&png, "slides.modal.export");

    let save_as = key(
        &mut harness,
        UiAutomationKey::Character("s".to_string()),
        UiAutomationModifiers {
            control: true,
            shift: true,
            ..UiAutomationModifiers::default()
        },
    );
    decode_png(&save_as);
    assert_selector(&save_as, "slides.modal.save_as");

    let shortcuts = key(
        &mut harness,
        UiAutomationKey::Character("?".to_string()),
        UiAutomationModifiers::default(),
    );
    decode_png(&shortcuts);
    assert_selector(&shortcuts, "slides.modal.keyboard_shortcuts");
}
