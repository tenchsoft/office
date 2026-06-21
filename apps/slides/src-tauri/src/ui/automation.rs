use super::*;
pub(crate) use super::automation_license::push_license_nodes;

pub(crate) fn slides_automation_nodes(
    slides: &SlidesState,
    size: Size,
    base_id: u64,
) -> Vec<UiAutomationNode> {
    let mut nodes = Vec::new();
    let mut next_id = base_id.saturating_mul(1000);
    let toolbar_rect = Rect::new(0.0, 0.0, size.width, TOOLBAR_H);
    for (label, x_start, x_end, action) in toolbar_layout(toolbar_rect) {
        push_slides_node(
            &mut nodes,
            &mut next_id,
            "button",
            label,
            slides_toolbar_debug_id(action),
            Rect::new(x_start, 6.0, x_end, TOOLBAR_H - 6.0),
        );
    }

    // Caption buttons (minimize / maximize-restore / close).
    for (control, debug_id, label) in [
        (
            WindowControl::Minimize,
            "slides.window.minimize",
            "Minimize",
        ),
        (
            WindowControl::MaximizeRestore,
            "slides.window.maximize",
            "Maximize",
        ),
        (WindowControl::Close, "slides.window.close", "Close"),
    ] {
        let rect = tench_ui::widgets::control_rect(size.width, TOOLBAR_H, control);
        push_slides_node(&mut nodes, &mut next_id, "button", label, debug_id, rect);
        if control == WindowControl::MaximizeRestore {
            if let Some(node) = nodes.last_mut() {
                node.value = Some(
                    if slides.window_maximized {
                        "maximized"
                    } else {
                        "restored"
                    }
                    .to_string(),
                );
            }
        }
    }

    for (index, slide) in slides.slides.iter().enumerate() {
        let y = TOOLBAR_H + 36.0 + index as f64 * (80.0 + 6.0);
        if y + 80.0 > size.height - NOTES_H {
            break;
        }
        push_slides_node(
            &mut nodes,
            &mut next_id,
            "button",
            if slide.title.is_empty() {
                format!("Slide {}", index + 1)
            } else {
                slide.title.clone()
            },
            format!("slides.filmstrip.slide.{index}"),
            Rect::new(20.0, y, 160.0, y + 80.0),
        );
    }

    let canvas_rect = Rect::new(
        THUMB_W,
        TOOLBAR_H,
        size.width - PROPS_W,
        size.height - NOTES_H,
    );
    push_slides_node(
        &mut nodes,
        &mut next_id,
        "canvas",
        "Slide canvas",
        "slides.canvas",
        canvas_rect,
    );
    let page = slide_page_rect(
        canvas_rect,
        slides.zoom.level,
        slides.zoom.pan_x,
        slides.zoom.pan_y,
    );
    push_slides_node(
        &mut nodes,
        &mut next_id,
        "page",
        "Current slide",
        "slides.canvas.page",
        page,
    );

    if let Some(slide) = slides.current_slide() {
        let scale_x = page.width() / 640.0;
        let scale_y = page.height() / 360.0;
        for (index, element) in slide.elements.iter().enumerate() {
            let rect = Rect::new(
                page.x0 + element.x * scale_x,
                page.y0 + element.y * scale_y,
                page.x0 + (element.x + element.w) * scale_x,
                page.y0 + (element.y + element.h) * scale_y,
            );
            push_slides_node(
                &mut nodes,
                &mut next_id,
                "slide_element",
                element.text.as_deref().unwrap_or(element.kind.as_str()),
                format!("slides.canvas.element.{index}"),
                rect,
            );
        }
    }

    let props_rect = Rect::new(
        size.width - PROPS_W,
        TOOLBAR_H,
        size.width,
        size.height - NOTES_H,
    );
    push_slides_node(
        &mut nodes,
        &mut next_id,
        "panel",
        "Properties",
        "slides.properties",
        props_rect,
    );
    for region in properties_layout(slides, props_rect) {
        let (label, debug_id, role) = slides_property_node_meta(region.action);
        push_slides_node(&mut nodes, &mut next_id, role, label, debug_id, region.rect);
    }

    push_slides_node(
        &mut nodes,
        &mut next_id,
        "text_input",
        "Speaker notes",
        "slides.notes",
        Rect::new(0.0, size.height - NOTES_H, size.width, size.height),
    );

    if let Some(modal) = &slides.active_modal {
        let modal_rect = slides_modal_rect(size, modal);
        push_slides_node(
            &mut nodes,
            &mut next_id,
            "dialog",
            slides_modal_label(modal),
            slides_modal_debug_id(modal),
            modal_rect,
        );
        push_slides_modal_child_nodes(&mut nodes, &mut next_id, size, modal);
    }

    if slides.presenting {
        push_slides_node(
            &mut nodes,
            &mut next_id,
            "dialog",
            "Presentation",
            "slides.presenter.overlay",
            Rect::new(0.0, 0.0, size.width, size.height),
        );
    }

    nodes
}

fn slides_toolbar_debug_id(action: ToolbarAction) -> &'static str {
    match action {
        ToolbarAction::NewSlide => "slides.toolbar.new",
        ToolbarAction::OpenFile => "slides.toolbar.open",
        ToolbarAction::Save => "slides.toolbar.save",
        ToolbarAction::Undo => "slides.toolbar.undo",
        ToolbarAction::Redo => "slides.toolbar.redo",
        ToolbarAction::InsertText => "slides.toolbar.text",
        ToolbarAction::InsertShape => "slides.toolbar.shape",
        ToolbarAction::InsertImage => "slides.toolbar.image",
        ToolbarAction::TogglePresentation => "slides.toolbar.play",
        ToolbarAction::License => "slides.toolbar.license",
    }
}

fn slides_property_node_meta(action: PropertyAction) -> (&'static str, &'static str, &'static str) {
    match action {
        PropertyAction::ActionButton(0) => {
            ("Bring forward", "slides.properties.bring_forward", "button")
        }
        PropertyAction::ActionButton(1) => {
            ("Send backward", "slides.properties.send_backward", "button")
        }
        PropertyAction::ActionButton(2) => ("Duplicate", "slides.properties.duplicate", "button"),
        PropertyAction::ActionButton(3) => ("Delete", "slides.properties.delete", "button"),
        PropertyAction::ActionButton(_) => ("Action", "slides.properties.action", "button"),
        PropertyAction::Slider(SliderKind::PositionX) => ("X", "slides.properties.x", "slider"),
        PropertyAction::Slider(SliderKind::PositionY) => ("Y", "slides.properties.y", "slider"),
        PropertyAction::Slider(SliderKind::Width) => ("Width", "slides.properties.width", "slider"),
        PropertyAction::Slider(SliderKind::Height) => {
            ("Height", "slides.properties.height", "slider")
        }
        PropertyAction::Slider(SliderKind::Rotation) => {
            ("Rotation", "slides.properties.rotation", "slider")
        }
        PropertyAction::Slider(SliderKind::Opacity) => {
            ("Opacity", "slides.properties.opacity", "slider")
        }
        PropertyAction::FillColorSwatch => ("Fill color", "slides.properties.fill_color", "button"),
        PropertyAction::BorderColorSwatch => {
            ("Border color", "slides.properties.border_color", "button")
        }
        PropertyAction::BackgroundButton => {
            ("Background", "slides.properties.background", "button")
        }
        PropertyAction::ThemeButton => ("Theme", "slides.properties.theme", "button"),
    }
}

fn slides_modal_label(modal: &ActiveModal) -> &'static str {
    match modal {
        ActiveModal::OpenFile => "Open file",
        ActiveModal::SaveAs => "Save as",
        ActiveModal::Export { .. } => "Export",
        ActiveModal::FindReplace { .. } => "Find and replace",
        ActiveModal::KeyboardShortcuts => "Keyboard shortcuts",
        ActiveModal::InsertImage => "Insert image",
        ActiveModal::ShapeSelector => "Shape selector",
        ActiveModal::LayoutSelector => "Layout selector",
        ActiveModal::BackgroundSettings { .. } => "Background",
        ActiveModal::ThemeSelector { .. } => "Theme",
        ActiveModal::SlideTransition => "Slide transition",
        ActiveModal::TableWizard { .. } => "Table",
        ActiveModal::ChartWizard { .. } => "Chart",
        ActiveModal::SaveError(_) => "Save error",
        ActiveModal::AnimationPanel => "Animation",
    }
}

fn slides_modal_debug_id(modal: &ActiveModal) -> &'static str {
    match modal {
        ActiveModal::OpenFile => "slides.modal.open_file",
        ActiveModal::SaveAs => "slides.modal.save_as",
        ActiveModal::Export { .. } => "slides.modal.export",
        ActiveModal::FindReplace { .. } => "slides.modal.find_replace",
        ActiveModal::KeyboardShortcuts => "slides.modal.keyboard_shortcuts",
        ActiveModal::InsertImage => "slides.modal.insert_image",
        ActiveModal::ShapeSelector => "slides.modal.shape_selector",
        ActiveModal::LayoutSelector => "slides.modal.layout_selector",
        ActiveModal::BackgroundSettings { .. } => "slides.modal.background",
        ActiveModal::ThemeSelector { .. } => "slides.modal.theme",
        ActiveModal::SlideTransition => "slides.modal.slide_transition",
        ActiveModal::TableWizard { .. } => "slides.modal.table_wizard",
        ActiveModal::ChartWizard { .. } => "slides.modal.chart_wizard",
        ActiveModal::SaveError(_) => "slides.modal.save_error",
        ActiveModal::AnimationPanel => "slides.modal.animation",
    }
}

fn slides_modal_rect(size: Size, modal: &ActiveModal) -> Rect {
    let modal_w = 360.0;
    let modal_h = match modal {
        ActiveModal::ShapeSelector => 200.0,
        ActiveModal::FindReplace { .. } => 220.0,
        ActiveModal::TableWizard { .. } => 200.0,
        ActiveModal::ChartWizard { .. } => 220.0,
        ActiveModal::LayoutSelector => 280.0,
        ActiveModal::KeyboardShortcuts => 400.0,
        ActiveModal::BackgroundSettings { .. } => 340.0,
        ActiveModal::ThemeSelector { .. } => 240.0,
        ActiveModal::Export { .. } => 240.0,
        ActiveModal::SaveError(_) => 180.0,
        _ => 180.0,
    };
    Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    )
}

fn push_slides_modal_child_nodes(
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
    size: Size,
    modal: &ActiveModal,
) {
    let modal_rect = slides_modal_rect(size, modal);
    let x = modal_rect.x0 + 16.0;
    let mut y = modal_rect.y0 + 56.0;

    match modal {
        ActiveModal::ShapeSelector => {
            for (label, debug_id) in [
                ("Rectangle", "slides.shape.rectangle"),
                ("Ellipse", "slides.shape.ellipse"),
                ("Line", "slides.shape.line"),
                ("Arrow", "slides.shape.arrow"),
            ] {
                push_slides_node(
                    nodes,
                    next_id,
                    "button",
                    label,
                    debug_id,
                    Rect::new(x, y - 4.0, x + 160.0, y + 16.0),
                );
                y += 28.0;
            }
        }
        ActiveModal::FindReplace { .. } => {
            push_slides_node(
                nodes,
                next_id,
                "text_input",
                "Find",
                "slides.find.find",
                Rect::new(x + 50.0, y - 10.0, x + 300.0, y + 6.0),
            );
            y += 28.0;
            push_slides_node(
                nodes,
                next_id,
                "text_input",
                "Replace",
                "slides.find.replace",
                Rect::new(x + 50.0, y - 10.0, x + 300.0, y + 6.0),
            );
            y += 28.0;
            let mut btn_x = x;
            for (label, debug_id) in [
                ("Find Next", "slides.find.next"),
                ("Replace", "slides.find.replace_button"),
                ("Replace All", "slides.find.replace_all"),
            ] {
                push_slides_node(
                    nodes,
                    next_id,
                    "button",
                    label,
                    debug_id,
                    Rect::new(btn_x, y - 4.0, btn_x + 100.0, y + 16.0),
                );
                btn_x += 110.0;
            }
        }
        ActiveModal::BackgroundSettings { .. } => {
            for (idx, label) in ["white", "light_gray", "dark", "navy", "brown"]
                .iter()
                .enumerate()
            {
                push_slides_node(
                    nodes,
                    next_id,
                    "button",
                    *label,
                    format!("slides.background.{label}"),
                    Rect::new(x, y - 4.0, x + 24.0, y + 16.0),
                );
                let _ = idx;
                y += 24.0;
            }
            y += 24.0;
            for label in ["blue_purple", "red_orange", "green_cyan"] {
                push_slides_node(
                    nodes,
                    next_id,
                    "button",
                    label,
                    format!("slides.background.{label}"),
                    Rect::new(x, y - 4.0, x + 100.0, y + 16.0),
                );
                y += 24.0;
            }
        }
        ActiveModal::ThemeSelector { .. } => {
            for label in ["default", "dark", "blue_professional", "warm_earth"] {
                push_slides_node(
                    nodes,
                    next_id,
                    "button",
                    label,
                    format!("slides.theme.{label}"),
                    Rect::new(x, y - 4.0, x + 280.0, y + 20.0),
                );
                y += 28.0;
            }
        }
        ActiveModal::Export { .. } => {
            for label in ["pdf", "png", "pptx"] {
                push_slides_node(
                    nodes,
                    next_id,
                    "button",
                    label,
                    format!("slides.export.{label}"),
                    Rect::new(x, y - 4.0, x + 160.0, y + 16.0),
                );
                y += 28.0;
            }
        }
        ActiveModal::ChartWizard { .. } => {
            for label in ["bar", "line", "pie", "scatter"] {
                push_slides_node(
                    nodes,
                    next_id,
                    "button",
                    label,
                    format!("slides.chart.{label}"),
                    Rect::new(x, y - 4.0, x + 160.0, y + 16.0),
                );
                y += 28.0;
            }
        }
        ActiveModal::LayoutSelector => {
            for label in [
                "blank",
                "title",
                "title_content",
                "two_column",
                "section_header",
            ] {
                push_slides_node(
                    nodes,
                    next_id,
                    "button",
                    label,
                    format!("slides.layout.{label}"),
                    Rect::new(x, y - 4.0, x + 160.0, y + 16.0),
                );
                y += 24.0;
            }
        }
        _ => {}
    }

    let button_y = modal_rect.y1 - 40.0;
    push_slides_node(
        nodes,
        next_id,
        "button",
        "Cancel",
        "slides.modal.cancel",
        Rect::new(
            modal_rect.x1 - 160.0,
            button_y,
            modal_rect.x1 - 90.0,
            button_y + 28.0,
        ),
    );
    push_slides_node(
        nodes,
        next_id,
        "button",
        "OK",
        "slides.modal.ok",
        Rect::new(
            modal_rect.x1 - 80.0,
            button_y,
            modal_rect.x1 - 10.0,
            button_y + 28.0,
        ),
    );
}

fn push_slides_node(
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
    role: &str,
    label: impl Into<String>,
    debug_id: impl Into<String>,
    rect: Rect,
) {
    *next_id = next_id.saturating_add(1);
    nodes.push(UiAutomationNode {
        id: *next_id,
        debug_id: Some(debug_id.into()),
        role: role.to_string(),
        label: Some(label.into()),
        value: None,
        bounds: UiAutomationRect {
            x: rect.x0,
            y: rect.y0,
            width: rect.width(),
            height: rect.height(),
        },
        enabled: true,
        focused: false,
        hovered: false,
        children: Vec::new(),
    });
}
