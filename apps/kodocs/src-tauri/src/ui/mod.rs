//! Kodocs UI: native Korean document editor widget.

mod automation;
mod automation_dropdown;
mod automation_helpers;
mod automation_license;
mod automation_modals;
mod automation_popups;
mod chrome;
mod clipboard;
mod document;
mod document_text;
pub(crate) mod equation_editor;
mod file_actions;
mod find_replace;
pub(crate) mod hanja;
mod hanja_actions;
mod hanja_popup;
mod hit_testing;
mod keyboard_events;
mod keyboard_modal_handlers;
mod menu_bar;
mod menu_dispatch;
mod misc_actions;
mod page_setup;
mod panels;
mod pointer_events;
mod pointer_find_page;
mod pointer_modal_clicks;
mod pointer_page_setup;
mod popovers;
mod ruler;
mod selection_actions;
pub mod state;
mod toolbar;
mod toolbar_actions;
mod toolbar_dispatch;
mod version_history;
mod widget;
mod window_events;

use automation_dropdown::push_dropdown_nodes;
use automation_helpers::push_kodocs_node;
use automation_modals::push_modal_nodes;
use automation_popups::push_popup_nodes;
use chrome::{
    menu_at, menu_items_for, paint_context_menu, paint_docs_modal, paint_docs_toast,
    paint_find_replace_modal, paint_link_modal, paint_title_row,
};
use equation_editor::paint_equation_dialog;
use hanja_popup::paint_hanja_popup;
use page_setup::paint_page_setup_dialog;
use popovers::{
    color_picker_hit, font_family_dropdown_hit, font_size_dropdown_hit, paint_tab_bar,
    paragraph_style_dropdown_hit, table_grid_hit,
};
use toolbar_actions::{toolbar_action_at, ToolbarAction};

use document::paint_document_area;
use menu_bar::paint_menu_bar;
use panels::{paint_comment_panel, paint_status_bar, paint_style_panel, paint_thumbnails};
use ruler::{paint_ruler, ruler_drag_to_indent, ruler_drag_to_margin, ruler_hit_test};
use state::{
    c_canvas_bg, c_separator, c_text_dim, c_text_light, FindReplaceState, KodocsState,
    LinkModalState, ParagraphStyle, RulerDragTarget, ToolbarDropdown, COLOR_PALETTE, FONT_FAMILIES,
    FONT_SIZES, MENU_BAR_H, RULER_H, STATUS_BAR_H, STYLE_PANEL_W, THUMB_PANEL_W, TITLE_ROW_H,
    TOOLBAR_H,
};
use tench_document_core::{CursorState, DocumentEngine, MarkType, MoveDirection, Orientation};
use tench_ui::core::events::{ImeEvent, KeyboardEvent, PointerEvent};
use tench_ui::prelude::*;
use tench_ui::render::TextCache;
use tench_ui::UiAutomationNode;
use toolbar::paint_toolbar;
use toolbar::paint_toolbar_dropdowns;

use crate::DialogResult;

/// Top-level Korean document editor widget.
pub struct KodocsApp {
    state: KodocsState,
    engine: DocumentEngine,
    #[allow(dead_code)] // Will be used for cached text rendering
    text_cache: TextCache,
    cursor_timer: AnimInterval,
    drag_start: Option<CursorState>,
    /// Tauri AppHandle for native file dialogs.
    app_handle: Option<tauri::AppHandle>,
    /// Receiver for async dialog results.
    dialog_rx: Option<std::sync::mpsc::Receiver<DialogResult>>,
    /// Encrypted license credential store. Read by automation nodes so UI
    /// tests can verify activation state without going through the Tauri
    /// command layer.
    license_store: Option<std::sync::Arc<tench_license_store::LicenseStore>>,
}

impl Default for KodocsApp {
    fn default() -> Self {
        Self::new()
    }
}

impl KodocsApp {
    pub fn new() -> Self {
        let state = KodocsState::new();
        let engine = DocumentEngine::new(state.current_document().clone());
        Self {
            state,
            engine,
            text_cache: TextCache::new(),
            cursor_timer: AnimInterval::new(500.0),
            drag_start: None,
            app_handle: None,
            dialog_rx: None,
            license_store: None,
        }
    }

    /// Set the Tauri AppHandle for native file dialogs.
    pub fn set_app_handle(&mut self, handle: tauri::AppHandle) {
        self.app_handle = Some(handle);
    }

    /// Set the dialog result receiver.
    pub fn set_dialog_receiver(&mut self, rx: std::sync::mpsc::Receiver<DialogResult>) {
        self.dialog_rx = Some(rx);
    }

    /// Set the license credential store. After this is called, automation
    /// nodes `kodocs.license.*` will be emitted with the current state.
    pub fn set_license_store(
        &mut self,
        store: std::sync::Arc<tench_license_store::LicenseStore>,
    ) {
        self.license_store = Some(store);
    }

    /// Get a mutable reference to the engine.
    fn engine(&mut self) -> &mut DocumentEngine {
        &mut self.engine
    }
}
