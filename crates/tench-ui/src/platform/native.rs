//! Native backend — connects tench-ui to a winit window via wgpu.
//!
//! Uses winit for window management and input events, wgpu for the rendering
//! surface, and vello for GPU-accelerated vector rendering.
//!
//! The backend maintains a shared `GlobalState` and uses hit testing to route
//! pointer events to the correct widget in the tree. Enter/Leave events are
//! synthesised when the hovered widget changes. Text events are dispatched to
//! the focused widget.

use std::sync::Arc;

use kurbo::Point;
use vello::wgpu;
use vello::{AaSupport, RenderParams, Renderer, RendererOptions};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};

use crate::core::events::{
    ImeEvent, KeyboardEvent, LogicalKey, Modifiers, NamedKey, PointerButton, PointerButtonEvent,
    PointerButtons, PointerEvent, PointerMoveEvent, PointerScrollEvent, TextEvent,
    WindowEvent as TenchWindowEvent,
};
use crate::core::types::WidgetId;
use crate::core::widget::{EventCtx, GlobalState, WidgetPod};
use crate::layout::LayoutPass;
use crate::render::RenderPass;
use crate::theme::Theme;

/// The texture format Vello requires for its render target.
const VELLO_RENDER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Returns a [`wgpu::TextureDescriptor`] suitable for use as a Vello render target.
///
/// The format is always [`Rgba8Unorm`] and the usage flags include everything
/// Vello needs (`STORAGE_BINDING`, `TEXTURE_BINDING`, `COPY_SRC`) plus
/// `RENDER_ATTACHMENT` for potential future use.
///
/// [`Rgba8Unorm`]: wgpu::TextureFormat::Rgba8Unorm
pub fn vello_target_texture_desc(width: u32, height: u32) -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        label: Some("tench-ui render target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: VELLO_RENDER_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    }
}

/// Configuration for creating a [`NativeBackend`].
pub struct NativeConfig {
    /// Window title.
    pub title: String,
    /// Initial window width in logical pixels.
    pub width: f64,
    /// Initial window height in logical pixels.
    pub height: f64,
    /// Whether the window is resizable.
    pub resizable: bool,
}

impl Default for NativeConfig {
    fn default() -> Self {
        Self {
            title: "Tench App".into(),
            width: 1280.0,
            height: 720.0,
            resizable: true,
        }
    }
}

/// Manages the tench-ui rendering within a native winit window.
///
/// Owns the wgpu device/queue, vello renderer, and the widget tree.
/// Converts winit events into tench-ui events and drives layout + rendering
/// each frame.
pub struct NativeBackend {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    /// Blitter for converting the Rgba8Unorm intermediate texture to the
    /// surface format (which may be Bgra8Unorm on Windows).
    blitter: Option<wgpu::util::TextureBlitter>,
    renderer: Renderer,
    root: Option<WidgetPod>,
    theme: Theme,
    size: kurbo::Size,
    global: GlobalState,
    /// The widget currently under the pointer (for Enter/Leave tracking).
    prev_hovered: Option<WidgetId>,
    /// Whether an animation frame has been requested.
    anim_requested: bool,
    /// Last known pointer position in window coordinates.
    last_cursor_pos: Point,
    /// Currently pressed pointer buttons.
    pointer_buttons: PointerButtons,
    /// Current keyboard modifier state.
    modifiers: Modifiers,
}

impl NativeBackend {
    /// Create a new backend by initialising wgpu and creating a surface from
    /// the given winit window.
    ///
    /// This call blocks while it sets up the GPU adapter and device.
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("Failed to create wgpu surface from window");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find a suitable GPU adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("tench-ui device"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .expect("Failed to create GPU device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| {
                matches!(
                    f,
                    wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
                )
            })
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

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

        // Create a blitter to convert Rgba8Unorm (Vello output) to the surface
        // format (which may be Bgra8Unorm on Windows/DirectX).
        let blitter = if surface_format != VELLO_RENDER_FORMAT {
            Some(wgpu::util::TextureBlitter::new(
                device.as_ref(),
                surface_format,
            ))
        } else {
            None
        };

        Self {
            device,
            queue,
            surface: Some(surface),
            config: Some(config),
            blitter,
            renderer,
            root: None,
            theme: Theme::default(),
            size: kurbo::Size::new(size.width as f64, size.height as f64),
            global: GlobalState::new(),
            prev_hovered: None,
            anim_requested: false,
            last_cursor_pos: Point::ZERO,
            pointer_buttons: PointerButtons::new(),
            modifiers: Modifiers::default(),
        }
    }

    /// Set the root widget.
    pub fn set_root(&mut self, root: impl crate::core::widget::Widget + 'static) {
        self.root = Some(WidgetPod::new(root));
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
        if let Some(config) = &mut self.config {
            config.width = width;
            config.height = height;
            self.size = kurbo::Size::new(width as f64, height as f64);
            if let Some(surface) = &self.surface {
                surface.configure(&self.device, config);
            }
        }
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
    pub fn on_pointer_event(&mut self, event: PointerEvent) -> bool {
        let Some(root) = &mut self.root else {
            return false;
        };

        let pos = Self::pointer_pos(&event);
        let mut needs_redraw = match &event {
            PointerEvent::Move(move_event) => !move_event.buttons.is_empty(),
            _ => true,
        };

        if let Some(pos) = pos {
            let new_hovered = root.hit_test_recursive(pos);

            if new_hovered != self.prev_hovered {
                if let Some(prev_id) = self.prev_hovered {
                    dispatch_pointer_event_recursive(
                        root,
                        &mut self.global,
                        prev_id,
                        &PointerEvent::Leave,
                    );
                }
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
                needs_redraw = true;
            }

            let target = new_hovered.unwrap_or(root.id());
            dispatch_pointer_event_recursive(root, &mut self.global, target, &event);
        } else {
            let _ = (event, root);
        }

        needs_redraw
    }

    /// Process a text event.
    ///
    /// Dispatches to the focused widget if one exists.
    pub fn on_text_event(&mut self, event: TextEvent) -> bool {
        let Some(root) = &mut self.root else {
            return false;
        };

        let target = self.global.focused_widget.unwrap_or(root.id());
        dispatch_text_event_recursive(root, &mut self.global, target, &event);
        true
    }

    /// Process a window event.
    ///
    /// Broadcasts to the root widget (which propagates to children internally).
    pub fn on_window_event(&mut self, event: TenchWindowEvent) -> bool {
        let Some(root) = &mut self.root else {
            return false;
        };

        let mut ctx = EventCtx {
            state: &mut root.state,
            global: &mut self.global,
            anim_requested: false,
        };
        root.widget.on_window_event(&mut ctx, &event);
        if ctx.anim_requested {
            self.anim_requested = true;
        }
        true
    }

    /// Returns `true` if an animation frame has been requested since the last
    /// render.
    pub fn should_anim_frame(&self) -> bool {
        self.anim_requested
    }

    /// Deliver an animation frame event to the widget tree.
    pub fn on_anim_frame(&mut self, timestamp: u64) {
        self.anim_requested = false;
        self.on_window_event(TenchWindowEvent::AnimFrame(timestamp));
    }

    /// Run layout and render a frame.
    pub fn render(&mut self) {
        let surface = match &self.surface {
            Some(s) => s,
            None => return,
        };
        let config = match &self.config {
            Some(c) => c,
            None => return,
        };

        let surface_texture = match surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };

        if let Some(root) = &mut self.root {
            LayoutPass::run(root, self.size, &mut self.global);

            let scene = RenderPass::run(root, self.size, &mut self.global, &self.theme);

            let texture = self
                .device
                .create_texture(&vello_target_texture_desc(config.width, config.height));
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let render_params = RenderParams {
                base_color: crate::core::types::Color::BLACK.into(),
                width: config.width,
                height: config.height,
                antialiasing_method: vello::AaConfig::Area,
            };

            self.renderer
                .render_to_texture(&self.device, &self.queue, &scene, &view, &render_params)
                .expect("Vello render failed");

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("tench-ui blit"),
                });

            let surface_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            if let Some(blitter) = &self.blitter {
                // Surface format differs from Vello's Rgba8Unorm — use
                // the blitter which renders a fullscreen quad to convert.
                blitter.copy(&self.device, &mut encoder, &view, &surface_view);
            } else {
                // Same format — a simple copy is sufficient.
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
                        width: config.width,
                        height: config.height,
                        depth_or_array_layers: 1,
                    },
                );
            }

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        surface_texture.present();
    }

    /// Convert a winit `WindowEvent` into tench-ui events and process them.
    fn handle_winit_window_event(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                self.resize(width, height);
                self.on_window_event(TenchWindowEvent::Resize { width, height });
                true
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.on_window_event(TenchWindowEvent::ScaleFactor(scale_factor));
                true
            }
            WindowEvent::Focused(focused) => {
                self.on_window_event(TenchWindowEvent::Focused(focused));
                true
            }
            WindowEvent::Destroyed => {
                self.on_window_event(TenchWindowEvent::Destroyed);
                false
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos = Point::new(position.x, position.y);
                self.last_cursor_pos = pos;
                self.on_pointer_event(PointerEvent::Move(PointerMoveEvent {
                    pos,
                    delta: kurbo::Vec2::ZERO, // winit doesn't provide delta directly
                    buttons: self.pointer_buttons.clone(),
                }))
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let btn = convert_mouse_button(button);
                self.pointer_buttons
                    .set(btn, state == ElementState::Pressed);
                let event = match state {
                    ElementState::Pressed => PointerEvent::Down(PointerButtonEvent {
                        button: btn,
                        pos: self.last_cursor_pos,
                        buttons: self.pointer_buttons.clone(),
                    }),
                    ElementState::Released => PointerEvent::Up(PointerButtonEvent {
                        button: btn,
                        pos: self.last_cursor_pos,
                        buttons: self.pointer_buttons.clone(),
                    }),
                };
                self.on_pointer_event(event)
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        kurbo::Vec2::new((x * 40.0) as f64, (y * 40.0) as f64)
                    }
                    MouseScrollDelta::PixelDelta(pos) => kurbo::Vec2::new(pos.x, pos.y),
                };
                self.on_pointer_event(PointerEvent::Scroll(PointerScrollEvent {
                    pos: self.last_cursor_pos,
                    delta: d,
                    modifiers: self.modifiers,
                }))
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let logical_key = convert_logical_key(&event.logical_key);
                let physical_key = match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(code) => code as u32,
                    winit::keyboard::PhysicalKey::Unidentified(_) => 0,
                };
                self.on_text_event(TextEvent::Keyboard(KeyboardEvent {
                    physical_key,
                    logical_key,
                    is_pressed: event.state == ElementState::Pressed,
                    is_repeat: event.repeat,
                    modifiers: self.modifiers,
                }))
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = convert_modifiers(&mods);
                false
            }
            WindowEvent::Ime(ime) => {
                let event = match ime {
                    winit::event::Ime::Enabled => ImeEvent::Enabled,
                    winit::event::Ime::Disabled => ImeEvent::Disabled,
                    winit::event::Ime::Commit(text) => ImeEvent::Commit(text),
                    winit::event::Ime::Preedit(text, cursor) => ImeEvent::Preedit { text, cursor },
                };
                self.on_text_event(TextEvent::Ime(event))
            }
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Winit event dispatch helpers
// ---------------------------------------------------------------------------

/// Convert a winit `MouseButton` to a tench-ui `PointerButton`.
fn convert_mouse_button(button: MouseButton) -> PointerButton {
    match button {
        MouseButton::Left => PointerButton::Primary,
        MouseButton::Right => PointerButton::Secondary,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Other(id) => PointerButton::Other(id),
        MouseButton::Back => PointerButton::Other(4),
        MouseButton::Forward => PointerButton::Other(5),
    }
}

/// Convert a winit `Key` to a tench-ui `LogicalKey`.
fn convert_logical_key(key: &winit::keyboard::Key) -> LogicalKey {
    match key {
        winit::keyboard::Key::Character(s) => LogicalKey::Character(s.to_string()),
        winit::keyboard::Key::Named(named) => LogicalKey::Named(convert_named_key(*named)),
        winit::keyboard::Key::Unidentified(_) | winit::keyboard::Key::Dead(_) => {
            LogicalKey::Unidentified
        }
    }
}

/// Convert a winit `NamedKey` to a tench-ui `NamedKey`.
fn convert_named_key(key: winit::keyboard::NamedKey) -> NamedKey {
    match key {
        winit::keyboard::NamedKey::Enter => NamedKey::Enter,
        winit::keyboard::NamedKey::Tab => NamedKey::Tab,
        winit::keyboard::NamedKey::Backspace => NamedKey::Backspace,
        winit::keyboard::NamedKey::Delete => NamedKey::Delete,
        winit::keyboard::NamedKey::Escape => NamedKey::Escape,
        winit::keyboard::NamedKey::Space => NamedKey::Space,
        winit::keyboard::NamedKey::ArrowUp => NamedKey::ArrowUp,
        winit::keyboard::NamedKey::ArrowDown => NamedKey::ArrowDown,
        winit::keyboard::NamedKey::ArrowLeft => NamedKey::ArrowLeft,
        winit::keyboard::NamedKey::ArrowRight => NamedKey::ArrowRight,
        winit::keyboard::NamedKey::Home => NamedKey::Home,
        winit::keyboard::NamedKey::End => NamedKey::End,
        winit::keyboard::NamedKey::PageUp => NamedKey::PageUp,
        winit::keyboard::NamedKey::PageDown => NamedKey::PageDown,
        winit::keyboard::NamedKey::Shift => NamedKey::Shift,
        winit::keyboard::NamedKey::Control => NamedKey::Control,
        winit::keyboard::NamedKey::Alt => NamedKey::Alt,
        winit::keyboard::NamedKey::Super => NamedKey::Super,
        winit::keyboard::NamedKey::CapsLock => NamedKey::CapsLock,
        winit::keyboard::NamedKey::Fn => NamedKey::Escape, // Fn key (not F1-F24)
        k => {
            // Map F1–F24 individually
            let f_num = match k {
                winit::keyboard::NamedKey::F1 => Some(1),
                winit::keyboard::NamedKey::F2 => Some(2),
                winit::keyboard::NamedKey::F3 => Some(3),
                winit::keyboard::NamedKey::F4 => Some(4),
                winit::keyboard::NamedKey::F5 => Some(5),
                winit::keyboard::NamedKey::F6 => Some(6),
                winit::keyboard::NamedKey::F7 => Some(7),
                winit::keyboard::NamedKey::F8 => Some(8),
                winit::keyboard::NamedKey::F9 => Some(9),
                winit::keyboard::NamedKey::F10 => Some(10),
                winit::keyboard::NamedKey::F11 => Some(11),
                winit::keyboard::NamedKey::F12 => Some(12),
                winit::keyboard::NamedKey::F13 => Some(13),
                winit::keyboard::NamedKey::F14 => Some(14),
                winit::keyboard::NamedKey::F15 => Some(15),
                winit::keyboard::NamedKey::F16 => Some(16),
                winit::keyboard::NamedKey::F17 => Some(17),
                winit::keyboard::NamedKey::F18 => Some(18),
                winit::keyboard::NamedKey::F19 => Some(19),
                winit::keyboard::NamedKey::F20 => Some(20),
                winit::keyboard::NamedKey::F21 => Some(21),
                winit::keyboard::NamedKey::F22 => Some(22),
                winit::keyboard::NamedKey::F23 => Some(23),
                winit::keyboard::NamedKey::F24 => Some(24),
                _ => None,
            };
            match f_num {
                Some(n) => NamedKey::F(n),
                None => NamedKey::Escape, // Fallback for unmapped keys
            }
        }
    }
}

/// Convert winit modifier key state to tench-ui `Modifiers`.
fn convert_modifiers(mods: &winit::event::Modifiers) -> Modifiers {
    Modifiers {
        shift: mods.state().shift_key(),
        control: mods.state().control_key(),
        alt: mods.state().alt_key(),
        super_key: mods.state().super_key(),
    }
}

// ---------------------------------------------------------------------------
// Recursive event dispatch helpers (shared with TauriBackend)
// ---------------------------------------------------------------------------

/// Walk the widget tree to find `target_id` and dispatch a pointer event to it.
fn dispatch_pointer_event_recursive(
    pod: &mut WidgetPod,
    global: &mut GlobalState,
    target_id: WidgetId,
    event: &PointerEvent,
) {
    if pod.state.id == target_id {
        let mut ctx = EventCtx {
            state: &mut pod.state,
            global,
            anim_requested: false,
        };
        pod.widget.on_pointer_event(&mut ctx, event);
        return;
    }

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

// ---------------------------------------------------------------------------
// Application handler — bridges winit's event loop to NativeBackend
// ---------------------------------------------------------------------------

/// Winit `ApplicationHandler` that drives a [`NativeBackend`].
///
/// Owns the winit window and the native backend. On `Resumed` it creates the
/// window and initialises the backend. On `WindowEvent` it converts winit
/// events and forwards them. On `AboutToWait` it triggers a render.
pub struct NativeApp {
    /// Optional factory that produces the root widget once the window is ready.
    // clippy: type alias would not improve clarity for a single-use closure field
    #[allow(clippy::type_complexity)]
    root_factory: Option<Box<dyn FnOnce(&mut NativeBackend)>>,
    /// Window creation settings.
    config: NativeConfig,
    /// The theme to apply (stored until the backend is created).
    theme: Option<Theme>,
    /// The winit window, created on first resume.
    window: Option<Arc<Window>>,
    /// The native backend, created alongside the window.
    backend: Option<NativeBackend>,
}

impl NativeApp {
    /// Create a new `NativeApp` with the given root widget factory.
    ///
    /// The factory is called once when the window is first created, allowing
    /// the caller to set up the widget tree.
    pub fn new(factory: impl FnOnce(&mut NativeBackend) + 'static) -> Self {
        Self {
            root_factory: Some(Box::new(factory)),
            config: NativeConfig::default(),
            theme: None,
            window: None,
            backend: None,
        }
    }

    /// Set the native window configuration.
    pub fn with_config(mut self, config: NativeConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the theme to use.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl ApplicationHandler for NativeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title(self.config.title.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.width,
                self.config.height,
            ))
            .with_resizable(self.config.resizable);

        let window = event_loop
            .create_window(attrs)
            .expect("Failed to create window");
        let window = Arc::new(window);

        let mut backend = NativeBackend::new(Arc::clone(&window));

        if let Some(theme) = self.theme.take() {
            backend.set_theme(theme);
        }

        if let Some(factory) = self.root_factory.take() {
            factory(&mut backend);
        }

        window.request_redraw();
        self.window = Some(window);
        self.backend = Some(backend);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match &event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                return;
            }
            WindowEvent::RedrawRequested => {
                if let Some(backend) = &mut self.backend {
                    backend.render();
                }
                return;
            }
            _ => {}
        }

        let mut needs_redraw = false;
        if let Some(backend) = &mut self.backend {
            needs_redraw = backend.handle_winit_window_event(event);
        }
        if needs_redraw {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(backend) = &mut self.backend {
            if backend.should_anim_frame() {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                backend.on_anim_frame(timestamp);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        _event: DeviceEvent,
    ) {
        // Device events (e.g. raw mouse motion) can be handled here if needed.
    }
}

/// Run a tench-ui application with a native winit window.
///
/// This is the main entry point for native apps. It creates the event loop,
/// window, and GPU backend, then runs until the window is closed.
///
/// # Example
///
/// ```no_run
/// use tench_ui::platform::native::{run_native, NativeConfig};
/// use tench_ui::widgets::Button;
///
/// run_native(|backend| {
///     backend.set_root(Button::new("Click me"));
/// });
/// ```
pub fn run_native(factory: impl FnOnce(&mut NativeBackend) + 'static) {
    run_native_with_config(NativeConfig::default(), factory);
}

/// Run a tench-ui application with explicit native window settings.
pub fn run_native_with_config(
    config: NativeConfig,
    factory: impl FnOnce(&mut NativeBackend) + 'static,
) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = NativeApp::new(factory).with_config(config);
    event_loop.run_app(&mut app).expect("Event loop error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::TextureUsages;

    #[test]
    fn vello_target_texture_is_rgba8unorm() {
        let desc = vello_target_texture_desc(800, 600);
        assert_eq!(
            desc.format,
            wgpu::TextureFormat::Rgba8Unorm,
            "Vello render target must use Rgba8Unorm regardless of surface format"
        );
    }

    #[test]
    fn vello_target_texture_has_storage_binding() {
        let desc = vello_target_texture_desc(800, 600);
        assert!(
            desc.usage.contains(TextureUsages::STORAGE_BINDING),
            "Vello compute shaders require STORAGE_BINDING"
        );
    }

    #[test]
    fn vello_target_texture_has_copy_src() {
        let desc = vello_target_texture_desc(800, 600);
        assert!(
            desc.usage.contains(TextureUsages::COPY_SRC),
            "Blitter needs COPY_SRC to read from intermediate texture"
        );
    }

    #[test]
    fn vello_target_texture_has_render_attachment() {
        let desc = vello_target_texture_desc(800, 600);
        assert!(
            desc.usage.contains(TextureUsages::RENDER_ATTACHMENT),
            "Render attachment required for blit fallback"
        );
    }

    #[test]
    fn vello_target_texture_dimensions() {
        let desc = vello_target_texture_desc(1920, 1080);
        assert_eq!(desc.size.width, 1920);
        assert_eq!(desc.size.height, 1080);
        assert_eq!(desc.size.depth_or_array_layers, 1);
        assert_eq!(desc.mip_level_count, 1);
        assert_eq!(desc.sample_count, 1);
    }
}
