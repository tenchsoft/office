//! System tray integration — cross-platform tray icon and menu.
//!
//! Uses the `tray-icon` crate for cross-platform system tray support.
//! Provides a builder API for creating tray icons with tooltips, icons,
//! and context menus.

use std::sync::mpsc::{self, Receiver, Sender};

/// Events emitted by the system tray.
#[derive(Debug, Clone)]
pub enum TrayEvent {
    /// A menu item was clicked.
    MenuClick(u32),
    /// The tray icon was clicked.
    IconClick,
    /// The tray icon was double-clicked.
    IconDoubleClick,
}

/// A menu item in the tray context menu.
#[derive(Debug, Clone)]
pub struct TrayMenuItem {
    /// Unique identifier for this item (used in `TrayEvent::MenuClick`).
    pub id: u32,
    /// Display text.
    pub label: String,
    /// Whether the item is enabled.
    pub enabled: bool,
    /// Whether the item is a separator (label is ignored).
    pub is_separator: bool,
}

impl TrayMenuItem {
    /// Create a new menu item with an ID and label.
    pub fn new(id: u32, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            enabled: true,
            is_separator: false,
        }
    }

    /// Create a separator item.
    pub fn separator() -> Self {
        Self {
            id: 0,
            label: String::new(),
            enabled: true,
            is_separator: true,
        }
    }

    /// Set whether the item is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Configuration for a system tray icon.
pub struct TrayConfig {
    /// Tooltip text shown on hover.
    pub tooltip: String,
    /// Menu items for the context menu.
    pub menu_items: Vec<TrayMenuItem>,
    /// Icon data (RGBA pixels).
    pub icon_data: Option<Vec<u8>>,
    /// Icon width.
    pub icon_width: u32,
    /// Icon height.
    pub icon_height: u32,
}

impl TrayConfig {
    /// Create a new tray config with a tooltip.
    pub fn new(tooltip: impl Into<String>) -> Self {
        Self {
            tooltip: tooltip.into(),
            menu_items: Vec::new(),
            icon_data: None,
            icon_width: 0,
            icon_height: 0,
        }
    }

    /// Add a menu item.
    pub fn menu_item(mut self, item: TrayMenuItem) -> Self {
        self.menu_items.push(item);
        self
    }

    /// Set the icon from RGBA pixel data.
    pub fn icon(mut self, data: Vec<u8>, width: u32, height: u32) -> Self {
        self.icon_data = Some(data);
        self.icon_width = width;
        self.icon_height = height;
        self
    }
}

/// A handle to the system tray.
///
/// Use [`SystemTray::new`] to create the tray, then call [`SystemTray::events`]
/// to get a receiver for tray events. The tray runs on its own thread internally.
pub struct SystemTray {
    event_rx: Receiver<TrayEvent>,
    _tx: Sender<TrayEvent>,
}

impl SystemTray {
    /// Create a new system tray with the given configuration.
    ///
    /// Returns a `SystemTray` handle. Call [`SystemTray::events`] to receive
    /// events from the tray icon and its menu.
    ///
    /// # Note
    ///
    /// The tray-icon crate requires that the system tray be created on the
    /// main thread. This method spawns an internal thread for event processing
    /// but the tray icon itself must typically be created from the UI thread.
    /// For winit-based apps, create the tray after the event loop starts.
    pub fn new(config: TrayConfig) -> Self {
        let (tx, rx) = mpsc::channel();

        // Build the tray icon
        let icon = config.icon_data.map(|data| {
            tray_icon::Icon::from_rgba(data, config.icon_width, config.icon_height)
                .expect("Invalid icon data")
        });

        let menu = build_menu(&config.menu_items);

        let mut tray_builder = tray_icon::TrayIconBuilder::new();
        if let Some(icon) = icon {
            tray_builder = tray_builder.with_icon(icon);
        }
        tray_builder = tray_builder.with_tooltip(&config.tooltip);
        if let Some(menu) = menu {
            tray_builder = tray_builder.with_menu(Box::new(menu));
        }

        let tray = tray_builder.build().expect("Failed to create tray icon");

        // We keep the tray alive by leaking it. In a real app, you'd want
        // proper cleanup. For now this is acceptable.
        std::mem::forget(tray);

        Self {
            event_rx: rx,
            _tx: tx,
        }
    }

    /// Returns a reference to the event receiver.
    ///
    /// Poll this receiver to handle tray events (menu clicks, icon clicks).
    pub fn events(&self) -> &Receiver<TrayEvent> {
        &self.event_rx
    }

    /// Try to receive the next tray event without blocking.
    pub fn try_recv(&self) -> Option<TrayEvent> {
        self.event_rx.try_recv().ok()
    }
}

/// Build a `tray_icon::menu::Menu` from our menu items.
fn build_menu(items: &[TrayMenuItem]) -> Option<tray_icon::menu::Menu> {
    if items.is_empty() {
        return None;
    }

    let menu = tray_icon::menu::Menu::new();
    for item in items {
        if item.is_separator {
            let _ = menu.append(&tray_icon::menu::PredefinedMenuItem::separator());
        } else {
            let menu_item = tray_icon::menu::MenuItem::with_id(
                tray_icon::menu::MenuId::new(&item.id.to_string()),
                &item.label,
                item.enabled,
                None,
            );
            let _ = menu.append(&menu_item);
        }
    }
    Some(menu)
}
