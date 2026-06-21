use super::*;

#[cfg(debug_assertions)]
pub(super) fn register_ui_automation_plugin(app: &mut ::tauri::App) {
    if !ui_automation_runtime_enabled() {
        return;
    }
    if let Err(error) = app.handle().plugin(ui_automation_plugin()) {
        log::warn!("failed to register tench-ui automation plugin: {error}");
    }
}

#[cfg(not(debug_assertions))]
pub(super) fn register_ui_automation_plugin(_app: &mut ::tauri::App) {}

#[cfg(debug_assertions)]
const UI_AUTOMATION_ENV: &str = "TENCH_UI_AUTOMATION";

#[cfg(debug_assertions)]
pub fn ui_automation_runtime_enabled() -> bool {
    std::env::var(UI_AUTOMATION_ENV).as_deref() == Ok("1")
}

#[cfg(debug_assertions)]
pub fn ui_automation_plugin<R: ::tauri::Runtime>() -> ::tauri::plugin::TauriPlugin<R> {
    ::tauri::plugin::Builder::new("tench-ui-automation")
        .invoke_handler(::tauri::generate_handler![
            tench_ui_automation_capture,
            tench_ui_automation_tree,
            tench_ui_automation_node,
            tench_ui_automation_inventory,
            tench_ui_automation_report,
            tench_ui_automation_action,
            tench_ui_automation_wait_for,
        ])
        .build()
}

#[cfg(debug_assertions)]
#[::tauri::command]
fn tench_ui_automation_capture(
    state: ::tauri::State<'_, TauriBackendState>,
    request: UiAutomationCaptureRequest,
) -> Result<UiAutomationCapture, UiAutomationError> {
    guard_ui_automation_enabled()?;
    state.with_backend(|backend| backend.automation_capture(request))
}

#[cfg(debug_assertions)]
#[::tauri::command]
fn tench_ui_automation_tree(
    state: ::tauri::State<'_, TauriBackendState>,
) -> Result<UiAutomationNode, UiAutomationError> {
    guard_ui_automation_enabled()?;
    state.with_backend(TauriBackend::automation_tree)
}

#[cfg(debug_assertions)]
#[::tauri::command]
fn tench_ui_automation_node(
    state: ::tauri::State<'_, TauriBackendState>,
    selector: UiAutomationSelector,
) -> Result<UiAutomationNode, UiAutomationError> {
    guard_ui_automation_enabled()?;
    state.with_backend(|backend| backend.automation_node(&selector))
}

#[cfg(debug_assertions)]
#[::tauri::command]
fn tench_ui_automation_inventory(
    state: ::tauri::State<'_, TauriBackendState>,
) -> Result<Vec<UiAutomationNodeSummary>, UiAutomationError> {
    guard_ui_automation_enabled()?;
    state.with_backend(TauriBackend::automation_inventory)
}

#[cfg(debug_assertions)]
#[::tauri::command]
fn tench_ui_automation_report(
    state: ::tauri::State<'_, TauriBackendState>,
) -> Result<String, UiAutomationError> {
    guard_ui_automation_enabled()?;
    state.with_backend(TauriBackend::automation_report)
}

#[cfg(debug_assertions)]
#[::tauri::command]
fn tench_ui_automation_action(
    state: ::tauri::State<'_, TauriBackendState>,
    action: UiAutomationAction,
) -> Result<UiAutomationCapture, UiAutomationError> {
    guard_ui_automation_enabled()?;
    state.with_backend(|backend| backend.automation_action(action))
}

#[cfg(debug_assertions)]
#[::tauri::command]
fn tench_ui_automation_wait_for(
    state: ::tauri::State<'_, TauriBackendState>,
    selector: UiAutomationSelector,
    timeout_ms: u64,
) -> Result<UiAutomationNode, UiAutomationError> {
    guard_ui_automation_enabled()?;
    state.with_backend(|backend| backend.automation_wait_for(&selector, timeout_ms))
}

#[cfg(debug_assertions)]
fn guard_ui_automation_enabled() -> Result<(), UiAutomationError> {
    if ui_automation_runtime_enabled() {
        Ok(())
    } else {
        Err(UiAutomationError::Disabled)
    }
}

pub(super) fn widget_tree_contains_id(pod: &mut WidgetPod, target_id: WidgetId) -> bool {
    if pod.state.id == target_id {
        return true;
    }
    let child_ids = pod.widget.children();
    for child_id in child_ids {
        if let Some(child) = pod.widget.child_mut(child_id) {
            if widget_tree_contains_id(child, target_id) {
                return true;
            }
        }
    }
    false
}

pub(super) fn automation_node_from_pod(
    pod: &mut WidgetPod,
    parent_origin: Point,
    global: &GlobalState,
) -> UiAutomationNode {
    let origin = parent_origin + pod.state.position.to_vec2();
    let semantics = pod.widget.accessibility_tree(&pod.state);
    let mut children = pod
        .widget
        .automation_children(&pod.state)
        .into_iter()
        .map(|node| offset_automation_node(node, origin))
        .collect::<Vec<_>>();

    let child_ids = pod.widget.children();
    for child_id in child_ids {
        if let Some(child) = pod.widget.child_mut(child_id) {
            children.push(automation_node_from_pod(child, origin, global));
        }
    }

    UiAutomationNode {
        id: pod.state.id.to_raw(),
        debug_id: pod.widget.debug_id().map(str::to_string),
        role: access_role_name(semantics.role).to_string(),
        label: semantics.label,
        value: semantics.value,
        bounds: UiAutomationRect {
            x: origin.x,
            y: origin.y,
            width: pod.state.size.width,
            height: pod.state.size.height,
        },
        enabled: !pod.state.is_disabled,
        focused: pod.state.is_focused || global.focused_widget == Some(pod.state.id),
        hovered: pod.state.is_hovered || global.hovered_widget == Some(pod.state.id),
        children,
    }
}

fn offset_automation_node(mut node: UiAutomationNode, origin: Point) -> UiAutomationNode {
    node.bounds.x += origin.x;
    node.bounds.y += origin.y;
    node.children = node
        .children
        .into_iter()
        .map(|child| offset_automation_node(child, origin))
        .collect();
    node
}

fn access_role_name(role: AccessRole) -> &'static str {
    match role {
        AccessRole::Unknown => "unknown",
        AccessRole::Button => "button",
        AccessRole::CheckBox => "checkbox",
        AccessRole::RadioButton => "radio",
        AccessRole::TextInput => "text_input",
        AccessRole::MultilineTextInput => "text_area",
        AccessRole::Heading => "heading",
        AccessRole::List => "list",
        AccessRole::ListItem => "list_item",
        AccessRole::Table => "table",
        AccessRole::TableRow => "row",
        AccessRole::TableCell => "cell",
        AccessRole::Paragraph => "paragraph",
        AccessRole::Label => "label",
        AccessRole::Image => "image",
        AccessRole::Link => "link",
        AccessRole::Menu => "menu",
        AccessRole::MenuItem => "menu_item",
        AccessRole::TabList => "tab_list",
        AccessRole::Tab => "tab",
        AccessRole::ScrollArea => "scroll_area",
        AccessRole::ProgressBar => "progress",
        AccessRole::Slider => "slider",
        AccessRole::Dialog => "dialog",
        AccessRole::GenericContainer => "container",
        AccessRole::Switch => "switch",
        AccessRole::Tree => "tree",
        AccessRole::TreeItem => "tree_item",
        AccessRole::Window => "window",
    }
}

pub(super) fn automation_type_text_events(text: &str) -> Vec<TextEvent> {
    text.chars()
        .flat_map(|ch| {
            let key = LogicalKey::Character(ch.to_string());
            [
                automation_key_event(key.clone(), true, UiAutomationModifiers::default()),
                automation_key_event(key, false, UiAutomationModifiers::default()),
            ]
        })
        .collect()
}

pub(super) fn automation_key_press_events(
    key: UiAutomationKey,
    modifiers: UiAutomationModifiers,
) -> [TextEvent; 2] {
    let logical = automation_logical_key(key);
    [
        automation_key_event(logical.clone(), true, modifiers),
        automation_key_event(logical, false, modifiers),
    ]
}

fn automation_key_event(
    logical_key: LogicalKey,
    is_pressed: bool,
    modifiers: UiAutomationModifiers,
) -> TextEvent {
    TextEvent::Keyboard(KeyboardEvent {
        physical_key: 0,
        logical_key,
        is_pressed,
        is_repeat: false,
        modifiers: Modifiers {
            shift: modifiers.shift,
            control: modifiers.control,
            alt: modifiers.alt,
            super_key: modifiers.meta,
        },
    })
}

fn automation_logical_key(key: UiAutomationKey) -> LogicalKey {
    match key {
        UiAutomationKey::Character(value) => LogicalKey::Character(value),
        UiAutomationKey::Enter => LogicalKey::Named(NamedKey::Enter),
        UiAutomationKey::Tab => LogicalKey::Named(NamedKey::Tab),
        UiAutomationKey::Escape => LogicalKey::Named(NamedKey::Escape),
        UiAutomationKey::Backspace => LogicalKey::Named(NamedKey::Backspace),
        UiAutomationKey::Delete => LogicalKey::Named(NamedKey::Delete),
        UiAutomationKey::ArrowLeft => LogicalKey::Named(NamedKey::ArrowLeft),
        UiAutomationKey::ArrowRight => LogicalKey::Named(NamedKey::ArrowRight),
        UiAutomationKey::ArrowUp => LogicalKey::Named(NamedKey::ArrowUp),
        UiAutomationKey::ArrowDown => LogicalKey::Named(NamedKey::ArrowDown),
        UiAutomationKey::Home => LogicalKey::Named(NamedKey::Home),
        UiAutomationKey::End => LogicalKey::Named(NamedKey::End),
        UiAutomationKey::PageUp => LogicalKey::Named(NamedKey::PageUp),
        UiAutomationKey::PageDown => LogicalKey::Named(NamedKey::PageDown),
        UiAutomationKey::F2 => LogicalKey::Named(NamedKey::F(2)),
        UiAutomationKey::F12 => LogicalKey::Named(NamedKey::F(12)),
    }
}

#[cfg(test)]
mod tests {
    use super::super::TauriUiOptions;
    #[cfg(debug_assertions)]
    use super::ui_automation_runtime_enabled;
    use super::{access_role_name, automation_key_press_events};
    use crate::core::events::{LogicalKey, NamedKey, TextEvent};
    use crate::core::widget::AccessRole;
    use tench_ui_automation_core::{UiAutomationKey, UiAutomationModifiers};

    #[test]
    fn default_options_target_main_window_and_render_first_frame() {
        let options = TauriUiOptions::default();

        assert_eq!(options.window_label, "main");
        assert!(options.render_first_frame);
    }

    #[test]
    fn automation_role_names_are_selector_stable_ui_automation() {
        assert_eq!(access_role_name(AccessRole::Button), "button");
        assert_eq!(access_role_name(AccessRole::TextInput), "text_input");
    }

    #[test]
    fn automation_key_press_maps_named_keys_ui_automation() {
        let events =
            automation_key_press_events(UiAutomationKey::Enter, UiAutomationModifiers::default());
        let TextEvent::Keyboard(event) = &events[0] else {
            panic!("expected keyboard event");
        };
        assert!(event.is_pressed);
        assert_eq!(event.logical_key, LogicalKey::Named(NamedKey::Enter));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn automation_runtime_requires_env_gate_ui_automation() {
        std::env::remove_var("TENCH_UI_AUTOMATION");
        assert!(!ui_automation_runtime_enabled());
        std::env::set_var("TENCH_UI_AUTOMATION", "1");
        assert!(ui_automation_runtime_enabled());
        std::env::remove_var("TENCH_UI_AUTOMATION");
    }
}
