//! Platform abstraction — bridges tench-ui to the windowing system.
//!
//! Multiple backends are available behind feature flags:
//!
//! - **`native`** (default) — Uses winit directly for window management and
//!   input events, with wgpu for the rendering surface.
//! - **`tauri`** — Uses Tauri's webview window with wgpu rendering.
//! - **`tray`** — System tray icon support via `tray-icon`.
//! - **`updater`** — Auto-update support via `reqwest`.

#[cfg(feature = "tauri")]
pub mod tauri;

#[cfg(feature = "native")]
pub mod native;

#[cfg(feature = "tray")]
pub mod tray;

#[cfg(feature = "updater")]
pub mod updater;

// Re-export primary backend types
#[cfg(feature = "tauri")]
pub use tauri::{init_tauri_ui, TauriBackend, TauriBackendState, TauriUiOptions};

#[cfg(feature = "native")]
pub use native::{run_native, run_native_with_config, NativeApp, NativeBackend, NativeConfig};
