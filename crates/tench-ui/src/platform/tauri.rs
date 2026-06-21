//! Tauri integration — connects tench-ui to a Tauri window via wgpu.
//!
//! This module uses vello's re-exported wgpu types to avoid version conflicts.
//!
//! The backend maintains a shared `GlobalState` and uses hit testing to route
//! pointer events to the correct widget in the tree. Enter/Leave events are
//! synthesised when the hovered widget changes. Text events are dispatched to
//! the focused widget.

use std::io::Cursor;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use kurbo::{Point, Vec2};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tench_ui_automation_core::{
    find_node, format_capture_report, node_inventory, UiAutomationAction, UiAutomationCapture,
    UiAutomationCaptureRequest, UiAutomationError, UiAutomationKey, UiAutomationModifiers,
    UiAutomationNode, UiAutomationNodeSummary, UiAutomationPoint, UiAutomationRect,
    UiAutomationSelector,
};
use vello::wgpu;
use vello::{AaSupport, RenderParams, Renderer, RendererOptions};

use crate::core::events::{
    KeyboardEvent, LogicalKey, Modifiers, NamedKey, PointerButton, PointerButtonEvent,
    PointerButtons, PointerEvent, PointerMoveEvent, PointerScrollEvent, TextEvent, WindowEvent,
};
use crate::core::types::WidgetId;
use crate::core::widget::AccessRole;
use crate::core::widget::{EventCtx, GlobalState, WidgetPod};
use crate::layout::LayoutPass;
use crate::render::RenderPass;
use crate::theme::Theme;

mod automation;

use automation::{
    automation_key_press_events, automation_node_from_pod, automation_type_text_events,
    register_ui_automation_plugin, widget_tree_contains_id,
};
#[cfg(debug_assertions)]
pub use automation::{ui_automation_plugin, ui_automation_runtime_enabled};
/// Configuration for wiring tench-ui into a Tauri webview window.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TauriUiOptions {
    /// Tauri window label that owns the native rendering surface.
    pub window_label: String,
    /// Render immediately after the backend is registered.
    pub render_first_frame: bool,
}

impl Default for TauriUiOptions {
    fn default() -> Self {
        Self {
            window_label: "main".to_string(),
            render_first_frame: true,
        }
    }
}

/// Thread-safe application state for a Tauri-backed tench-ui renderer.
pub struct TauriBackendState {
    pub backend: Mutex<TauriBackend>,
}

impl TauriBackendState {
    /// Create app state from an initialized backend.
    pub fn new(backend: TauriBackend) -> Self {
        Self {
            backend: Mutex::new(backend),
        }
    }

    /// Lock the backend, recovering from poison by keeping the owned backend.
    pub fn with_backend<R>(&self, f: impl FnOnce(&mut TauriBackend) -> R) -> R {
        let mut backend = self
            .backend
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        f(&mut backend)
    }
}

/// Initialize tench-ui rendering on a Tauri window and register resize handling.
pub fn init_tauri_ui(
    app: &mut ::tauri::App,
    options: TauriUiOptions,
    configure_backend: impl FnOnce(&mut TauriBackend, &mut ::tauri::App),
) {
    use ::tauri::Manager;

    let wvw = app
        .get_webview_window(&options.window_label)
        .expect("no Tauri window found for tench-ui");
    let size = wvw.inner_size().expect("failed to get window size");
    let window = wvw.as_ref().window();
    let window_handle = window.window_handle().expect("failed to get window handle");
    let display_handle = window
        .display_handle()
        .expect("failed to get display handle");

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = unsafe {
        instance
            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_window_handle: window_handle.as_raw(),
                raw_display_handle: display_handle.as_raw(),
            })
            .expect("failed to create wgpu surface")
    };

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .expect("failed to find suitable GPU adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("tench-ui device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        ..Default::default()
    }))
    .expect("failed to create wgpu device");

    let device = Arc::new(device);
    let queue = Arc::new(queue);
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .find(|f| matches!(f, wgpu::TextureFormat::Rgba8Unorm))
        .or_else(|| surface_caps.formats.first())
        .copied()
        .unwrap_or(wgpu::TextureFormat::Rgba8Unorm);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    let mut backend = TauriBackend::new(device, queue, surface, config);
    configure_backend(&mut backend, app);
    app.manage(TauriBackendState::new(backend));
    register_resize_listener(app, &options.window_label);
    register_drag_drop_listener(app, &options.window_label);
    register_ui_automation_plugin(app);

    if options.render_first_frame {
        render_managed_backend(app);
    }
}

fn register_resize_listener(app: &mut ::tauri::App, window_label: &str) {
    use ::tauri::{Listener, Manager};

    let app_handle = app.handle().clone();
    let window_label = window_label.to_string();

    app.listen("tauri://resize", move |_event| {
        let Some(wvw) = app_handle.get_webview_window(&window_label) else {
            return;
        };
        let Ok(size) = wvw.inner_size() else {
            return;
        };
        let Some(state) = app_handle.try_state::<TauriBackendState>() else {
            return;
        };
        state.with_backend(|backend| {
            backend.resize(size.width, size.height);
            backend.render();
        });
    });
}

fn register_drag_drop_listener(app: &mut ::tauri::App, _window_label: &str) {
    use ::tauri::{Listener, Manager};

    let app_handle = app.handle().clone();

    // Listen for Tauri drag-drop events and forward as WindowEvent::FileDrop.
    // The payload is a JSON string like: {"paths":["/path/to/file"]}
    // We parse it without serde_json to avoid adding that dependency.
    app.listen("tauri://drag-drop", move |event| {
        let payload = event.payload();
        let paths = parse_drag_drop_paths(payload);

        if paths.is_empty() {
            return;
        }

        let Some(state) = app_handle.try_state::<TauriBackendState>() else {
            return;
        };
        state.with_backend(|backend| {
            backend.on_window_event(WindowEvent::FileDrop { paths });
            backend.render();
        });
    });
}

/// Parse paths from a tauri://drag-drop payload string.
///
/// Expected format: `{"paths":["/path/one","/path/two"]}`
fn parse_drag_drop_paths(payload: &str) -> Vec<String> {
    // Find the "paths" array in the JSON payload
    let Some(paths_start) = payload.find("\"paths\"") else {
        return Vec::new();
    };
    let remainder = &payload[paths_start + 7..]; // skip "paths"
    let Some(arr_start) = remainder.find('[') else {
        return Vec::new();
    };
    let arr_content = &remainder[arr_start + 1..];
    let Some(arr_end) = arr_content.find(']') else {
        return Vec::new();
    };
    let arr_content = &arr_content[..arr_end];

    // Extract quoted strings from the array
    let mut paths = Vec::new();
    let mut i = 0;
    let bytes = arr_content.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() {
                if bytes[end] == b'"' && bytes[end - 1] != b'\\' {
                    break;
                }
                end += 1;
            }
            if end < bytes.len() {
                let path_str = &arr_content[start..end];
                // Unescape simple \\\" sequences
                let path = path_str.replace("\\\"", "\"").replace("\\\\", "\\");
                paths.push(path);
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    paths
}

fn render_managed_backend(app: &mut ::tauri::App) {
    use ::tauri::Manager;

    if let Some(state) = app.try_state::<TauriBackendState>() {
        state.with_backend(TauriBackend::render);
    }
}

/// Manages the tench-ui rendering within a Tauri window.
pub struct TauriBackend {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
    root: Option<WidgetPod>,
    theme: Theme,
    size: kurbo::Size,
    global: GlobalState,
    /// The widget currently under the pointer (for Enter/Leave tracking).
    prev_hovered: Option<WidgetId>,
    /// Whether an animation frame has been requested.
    anim_requested: bool,
}

impl TauriBackend {
    /// Create a new backend bound to a wgpu surface.
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        let renderer = Renderer::new(
            device.as_ref(),
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .expect("Failed to create Vello renderer");

        Self {
            device,
            queue,
            surface,
            size: kurbo::Size::new(config.width as f64, config.height as f64),
            config,
            renderer,
            root: None,
            theme: Theme::default(),
            global: GlobalState::new(),
            prev_hovered: None,
            anim_requested: false,
        }
    }

    /// Set the root widget.
    pub fn set_root(&mut self, root: impl crate::core::widget::Widget + 'static) {
        self.root = Some(WidgetPod::new(root));
    }

    /// Access the root widget and downcast it to a concrete type.
    ///
    /// Returns `None` if the root is not set or the downcast fails.
    pub fn root_widget<W: crate::core::widget::Widget>(&mut self) -> Option<&mut W> {
        self.root.as_mut()?.widget.downcast_mut()
    }

    /// Set the theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Handle window resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.size = kurbo::Size::new(width as f64, height as f64);
        self.surface.configure(&self.device, &self.config);
    }

    /// Extract the pointer position from a pointer event (window coordinates).
    fn pointer_pos(event: &PointerEvent) -> Option<Point> {
        match event {
            PointerEvent::Move(e) => Some(e.pos),
            PointerEvent::Down(e) => Some(e.pos),
            PointerEvent::Up(e) => Some(e.pos),
            PointerEvent::Scroll(e) => Some(e.pos),
            PointerEvent::Enter | PointerEvent::Leave => None,
        }
    }

    /// Process a pointer event.
    ///
    /// Uses hit testing to find the deepest widget under the pointer, then
    /// dispatches the event to that widget. Enter/Leave events are synthesised
    /// when the hovered widget changes between moves.
    pub fn on_pointer_event(&mut self, event: PointerEvent) {
        let Some(root) = &mut self.root else { return };

        // Determine the position for hit testing (Move/Down/Up/Scroll have positions).
        let pos = Self::pointer_pos(&event);

        // For Move events, perform hit testing and manage Enter/Leave.
        if let Some(pos) = pos {
            let new_hovered = root.hit_test_recursive(pos);

            // Synthesise Enter/Leave if the hovered widget changed.
            if new_hovered != self.prev_hovered {
                // Send Leave to the previously hovered widget.
                if let Some(prev_id) = self.prev_hovered {
                    dispatch_pointer_event_recursive(
                        root,
                        &mut self.global,
                        prev_id,
                        &PointerEvent::Leave,
                    );
                }
                // Send Enter to the newly hovered widget.
                if let Some(new_id) = new_hovered {
                    dispatch_pointer_event_recursive(
                        root,
                        &mut self.global,
                        new_id,
                        &PointerEvent::Enter,
                    );
                }
                self.prev_hovered = new_hovered;
                self.global.hovered_widget = new_hovered;
            }

            // Dispatch the actual event to the hit widget (or root as fallback).
            let target = new_hovered.unwrap_or(root.id());
            dispatch_pointer_event_recursive(root, &mut self.global, target, &event);
        } else {
            // Enter/Leave events without a position — dispatch directly.
            let _ = (event, root);
        }
    }

    /// Process a text event.
    ///
    /// Dispatches to the focused widget if one exists.
    pub fn on_text_event(&mut self, event: TextEvent) {
        let Some(root) = &mut self.root else { return };

        let target = self.global.focused_widget.unwrap_or(root.id());
        dispatch_text_event_recursive(root, &mut self.global, target, &event);
    }

    /// Process a window event.
    ///
    /// Broadcasts to the root widget (which propagates to children internally).
    pub fn on_window_event(&mut self, event: WindowEvent) {
        let Some(root) = &mut self.root else { return };

        let mut ctx = EventCtx {
            state: &mut root.state,
            global: &mut self.global,
            anim_requested: false,
        };
        root.widget.on_window_event(&mut ctx, &event);
        if ctx.anim_requested {
            self.anim_requested = true;
        }
    }

    /// Returns `true` if an animation frame has been requested since the last
    /// render. The embedder should call this after event processing and, if it
    /// returns `true`, call `render()` (or schedule a render on the next vsync).
    pub fn should_anim_frame(&self) -> bool {
        self.anim_requested
    }

    /// Deliver an animation frame event to the widget tree.
    ///
    /// Call this from the embedder's vsync / requestAnimationFrame callback.
    /// `timestamp` is typically milliseconds since some epoch (e.g.
    /// `performance.now()` in JS-land or `Instant::now().as_millis()`).
    pub fn on_anim_frame(&mut self, timestamp: u64) {
        self.anim_requested = false;
        self.on_window_event(WindowEvent::AnimFrame(timestamp));
    }

    /// Capture the current UI and optionally include a semantic UI tree.
    pub fn automation_capture(
        &mut self,
        request: UiAutomationCaptureRequest,
    ) -> Result<UiAutomationCapture, UiAutomationError> {
        let png_bytes = if request.include_png {
            self.render_png()?
        } else {
            Vec::new()
        };
        let ui_tree = if request.include_tree {
            Some(self.automation_tree()?)
        } else {
            None
        };

        Ok(UiAutomationCapture {
            width: self.config.width,
            height: self.config.height,
            png_bytes,
            ui_tree,
            focused_node: self.global.focused_widget.map(WidgetId::to_raw),
            hovered_node: self.global.hovered_widget.map(WidgetId::to_raw),
        })
    }

    /// Return the semantic tree used by selector-based UI automation.
    pub fn automation_tree(&mut self) -> Result<UiAutomationNode, UiAutomationError> {
        let Some(root) = &mut self.root else {
            return Err(UiAutomationError::BackendUnavailable);
        };
        LayoutPass::run(root, self.size, &mut self.global);
        Ok(automation_node_from_pod(root, Point::ZERO, &self.global))
    }

    /// Return one automation node by selector, including bounds and center data.
    pub fn automation_node(
        &mut self,
        selector: &UiAutomationSelector,
    ) -> Result<UiAutomationNode, UiAutomationError> {
        let tree = self.automation_tree()?;
        find_node(&tree, selector)
            .cloned()
            .ok_or(UiAutomationError::SelectorNotFound)
    }

    /// Return a flat UI inventory for test design and agent debugging.
    pub fn automation_inventory(
        &mut self,
    ) -> Result<Vec<UiAutomationNodeSummary>, UiAutomationError> {
        let tree = self.automation_tree()?;
        Ok(node_inventory(&tree))
    }

    /// Return a human-readable UI report with selector hints and coordinates.
    pub fn automation_report(&mut self) -> Result<String, UiAutomationError> {
        let capture = self.automation_capture(UiAutomationCaptureRequest::tree_only())?;
        Ok(format_capture_report(&capture))
    }

    /// Apply an automation action and return a post-action capture.
    pub fn automation_action(
        &mut self,
        action: UiAutomationAction,
    ) -> Result<UiAutomationCapture, UiAutomationError> {
        match action {
            UiAutomationAction::Capture { request } => return self.automation_capture(request),
            UiAutomationAction::Click {
                selector,
                modifiers: _,
            } => {
                self.dispatch_automation_click(&selector, 1, PointerButton::Primary)?;
            }
            UiAutomationAction::RightClick {
                selector,
                modifiers: _,
            } => {
                self.dispatch_automation_click(&selector, 1, PointerButton::Secondary)?;
            }
            UiAutomationAction::DoubleClick {
                selector,
                modifiers: _,
            } => {
                self.dispatch_automation_click(&selector, 2, PointerButton::Primary)?;
            }
            UiAutomationAction::TypeText { selector, text } => {
                self.dispatch_automation_click(&selector, 1, PointerButton::Primary)?;
                if let Some(target) = self.resolve_widget_id(&selector)? {
                    self.dispatch_text_to_widget(target, &text);
                } else {
                    for event in automation_type_text_events(&text) {
                        self.on_text_event(event);
                    }
                }
            }
            UiAutomationAction::KeyPress { key, modifiers } => {
                for event in automation_key_press_events(key, modifiers) {
                    self.on_text_event(event);
                }
            }
            UiAutomationAction::Scroll {
                selector,
                delta_x,
                delta_y,
            } => {
                let point = self.resolve_point(&selector)?;
                self.on_pointer_event(PointerEvent::Scroll(PointerScrollEvent {
                    pos: point,
                    delta: Vec2::new(delta_x, delta_y),
                    modifiers: Modifiers::default(),
                }));
            }
            UiAutomationAction::Drag { start, end, steps } => {
                self.dispatch_automation_drag(start, end, steps);
            }
            UiAutomationAction::DragFromTo {
                from_selector,
                to_selector,
                steps,
            } => {
                let start = self.resolve_point(&from_selector)?;
                let end = self.resolve_point(&to_selector)?;
                let start_point = UiAutomationPoint {
                    x: start.x,
                    y: start.y,
                };
                let end_point = UiAutomationPoint { x: end.x, y: end.y };
                self.dispatch_automation_drag(start_point, end_point, steps);
            }
            UiAutomationAction::Hover { selector } => {
                let point = self.resolve_point(&selector)?;
                self.on_pointer_event(PointerEvent::Move(PointerMoveEvent {
                    pos: point,
                    delta: Vec2::ZERO,
                    buttons: PointerButtons::new(),
                }));
            }
            UiAutomationAction::WaitFor {
                selector,
                timeout_ms,
            } => {
                self.automation_wait_for(&selector, timeout_ms)?;
            }
            UiAutomationAction::AnimFrame { timestamp_ms } => {
                self.on_anim_frame(timestamp_ms);
            }
        }

        self.automation_capture(UiAutomationCaptureRequest::default())
    }

    pub fn automation_wait_for(
        &mut self,
        selector: &UiAutomationSelector,
        timeout_ms: u64,
    ) -> Result<UiAutomationNode, UiAutomationError> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            let tree = self.automation_tree()?;
            if let Some(node) = find_node(&tree, selector) {
                return Ok(node.clone());
            }
            if Instant::now() >= deadline {
                return Err(UiAutomationError::Timeout);
            }
            std::thread::sleep(Duration::from_millis(16));
        }
    }

    fn dispatch_automation_click(
        &mut self,
        selector: &UiAutomationSelector,
        count: usize,
        button: PointerButton,
    ) -> Result<(), UiAutomationError> {
        let point = self.resolve_point(selector)?;
        for _ in 0..count.max(1) {
            let mut buttons = PointerButtons::new();
            buttons.set(button, true);
            self.on_pointer_event(PointerEvent::Down(PointerButtonEvent {
                button,
                pos: point,
                buttons,
            }));
            self.on_pointer_event(PointerEvent::Up(PointerButtonEvent {
                button,
                pos: point,
                buttons: PointerButtons::new(),
            }));
        }
        Ok(())
    }

    fn dispatch_automation_drag(
        &mut self,
        start: UiAutomationPoint,
        end: UiAutomationPoint,
        steps: usize,
    ) {
        let start = Point::new(start.x, start.y);
        let end = Point::new(end.x, end.y);
        let steps = steps.max(1);
        let mut buttons = PointerButtons::new();
        buttons.set(PointerButton::Primary, true);
        self.on_pointer_event(PointerEvent::Down(PointerButtonEvent {
            button: PointerButton::Primary,
            pos: start,
            buttons: buttons.clone(),
        }));
        let delta = (end - start) / steps as f64;
        for step in 1..=steps {
            self.on_pointer_event(PointerEvent::Move(PointerMoveEvent {
                pos: start + delta * step as f64,
                delta,
                buttons: buttons.clone(),
            }));
        }
        self.on_pointer_event(PointerEvent::Up(PointerButtonEvent {
            button: PointerButton::Primary,
            pos: end,
            buttons: PointerButtons::new(),
        }));
    }

    fn dispatch_text_to_widget(&mut self, target: WidgetId, text: &str) {
        let Some(root) = &mut self.root else { return };
        self.global.focused_widget = Some(target);
        for event in automation_type_text_events(text) {
            dispatch_text_event_recursive(root, &mut self.global, target, &event);
        }
    }

    fn resolve_point(
        &mut self,
        selector: &UiAutomationSelector,
    ) -> Result<Point, UiAutomationError> {
        match selector {
            UiAutomationSelector::ByPoint { point } => Ok(Point::new(point.x, point.y)),
            _ => {
                let tree = self.automation_tree()?;
                let node = find_node(&tree, selector).ok_or(UiAutomationError::SelectorNotFound)?;
                let center = node.center();
                Ok(Point::new(center.x, center.y))
            }
        }
    }

    fn resolve_widget_id(
        &mut self,
        selector: &UiAutomationSelector,
    ) -> Result<Option<WidgetId>, UiAutomationError> {
        match selector {
            UiAutomationSelector::ByPoint { point } => {
                let Some(root) = &mut self.root else {
                    return Ok(None);
                };
                Ok(root.hit_test_recursive(Point::new(point.x, point.y)))
            }
            _ => {
                let tree = self.automation_tree()?;
                let Some(node) = find_node(&tree, selector) else {
                    return Err(UiAutomationError::SelectorNotFound);
                };
                let Some(widget_id) = WidgetId::from_raw(node.id) else {
                    return Ok(None);
                };
                let Some(root) = &mut self.root else {
                    return Ok(None);
                };
                if widget_tree_contains_id(root, widget_id) {
                    Ok(Some(widget_id))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn render_png(&mut self) -> Result<Vec<u8>, UiAutomationError> {
        let rgba = self.render_rgba()?;
        let image = image::RgbaImage::from_raw(self.config.width, self.config.height, rgba)
            .ok_or(UiAutomationError::CaptureUnavailable)?;
        let mut bytes = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .map_err(|error| UiAutomationError::Internal(error.to_string()))?;
        Ok(bytes.into_inner())
    }

    fn render_rgba(&mut self) -> Result<Vec<u8>, UiAutomationError> {
        let Some(root) = &mut self.root else {
            return Err(UiAutomationError::BackendUnavailable);
        };

        LayoutPass::run(root, self.size, &mut self.global);
        let scene = RenderPass::run(root, self.size, &mut self.global, &self.theme);
        let texture = self.render_scene_to_texture(&scene)?;
        self.read_texture_rgba(&texture)
    }

    fn render_scene_to_texture(
        &mut self,
        scene: &vello::Scene,
    ) -> Result<wgpu::Texture, UiAutomationError> {
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("tench-ui automation capture target"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        };
        let texture = self.device.create_texture(&texture_desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let render_params = RenderParams {
            base_color: crate::core::types::Color::BLACK.into(),
            width: self.config.width,
            height: self.config.height,
            antialiasing_method: vello::AaConfig::Area,
        };
        self.renderer
            .render_to_texture(&self.device, &self.queue, scene, &view, &render_params)
            .map_err(|error| UiAutomationError::Internal(error.to_string()))?;
        Ok(texture)
    }

    fn read_texture_rgba(&self, texture: &wgpu::Texture) -> Result<Vec<u8>, UiAutomationError> {
        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = self.config.width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let buffer_size = padded_bytes_per_row as u64 * self.config.height as u64;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tench-ui automation capture readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("tench-ui automation capture copy"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(self.config.height),
                },
            },
            wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = buffer.slice(..);
        let (sender, receiver) = mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| UiAutomationError::Internal(error.to_string()))?;
        receiver
            .recv()
            .map_err(|error| UiAutomationError::Internal(error.to_string()))?
            .map_err(|error| UiAutomationError::Internal(error.to_string()))?;

        let mapped = buffer_slice.get_mapped_range();
        let mut rgba = vec![0; (self.config.width * self.config.height * bytes_per_pixel) as usize];
        for row in 0..self.config.height as usize {
            let src_start = row * padded_bytes_per_row as usize;
            let dst_start = row * unpadded_bytes_per_row as usize;
            let src_end = src_start + unpadded_bytes_per_row as usize;
            let dst_end = dst_start + unpadded_bytes_per_row as usize;
            rgba[dst_start..dst_end].copy_from_slice(&mapped[src_start..src_end]);
        }
        drop(mapped);
        buffer.unmap();
        Ok(rgba)
    }

    /// Run layout and render a frame.
    pub fn render(&mut self) {
        self.anim_requested = false;

        let surface_texture = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };

        if let Some(root) = &mut self.root {
            // Layout pass — shared GlobalState
            LayoutPass::run(root, self.size, &mut self.global);

            // Build scene — shared GlobalState + Theme
            let scene = RenderPass::run(root, self.size, &mut self.global, &self.theme);

            // Render to intermediate texture
            let texture_desc = wgpu::TextureDescriptor {
                label: Some("tench-ui render target"),
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };
            let texture = self.device.create_texture(&texture_desc);
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let render_params = RenderParams {
                base_color: crate::core::types::Color::BLACK.into(),
                width: self.config.width,
                height: self.config.height,
                antialiasing_method: vello::AaConfig::Area,
            };

            self.renderer
                .render_to_texture(&self.device, &self.queue, &scene, &view, &render_params)
                .expect("Vello render failed");

            // Blit to surface
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("tench-ui blit"),
                });

            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &surface_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
            );

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        surface_texture.present();
    }
}

// ---------------------------------------------------------------------------
// Recursive event dispatch helpers
// ---------------------------------------------------------------------------

/// Walk the widget tree to find `target_id` and dispatch a pointer event to it.
/// Container widgets that manage children will propagate events internally.
fn dispatch_pointer_event_recursive(
    pod: &mut WidgetPod,
    global: &mut GlobalState,
    target_id: WidgetId,
    event: &PointerEvent,
) {
    // If this pod is the target, dispatch directly.
    if pod.state.id == target_id {
        let mut ctx = EventCtx {
            state: &mut pod.state,
            global,
            anim_requested: false,
        };
        pod.widget.on_pointer_event(&mut ctx, event);
        return;
    }

    // Otherwise, recurse into children.
    let child_ids = pod.widget.children();
    for child_id in child_ids {
        if let Some(child) = pod.widget.child_mut(child_id) {
            dispatch_pointer_event_recursive(child, global, target_id, event);
        }
    }
}

/// Walk the widget tree to find `target_id` and dispatch a text event to it.
fn dispatch_text_event_recursive(
    pod: &mut WidgetPod,
    global: &mut GlobalState,
    target_id: WidgetId,
    event: &TextEvent,
) {
    if pod.state.id == target_id {
        let mut ctx = EventCtx {
            state: &mut pod.state,
            global,
            anim_requested: false,
        };
        pod.widget.on_text_event(&mut ctx, event);
        return;
    }

    let child_ids = pod.widget.children();
    for child_id in child_ids {
        if let Some(child) = pod.widget.child_mut(child_id) {
            dispatch_text_event_recursive(child, global, target_id, event);
        }
    }
}
