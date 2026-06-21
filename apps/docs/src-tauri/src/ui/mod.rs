//! Docs UI: native document editor widget.

mod automation;
mod automation_document;
mod automation_dropdown;
mod automation_helpers;
mod automation_sidebar;
mod automation_status;
mod automation_version_history;
mod chrome;
mod clipboard;
mod dispatch;
mod document;
mod document_text;
mod events;
mod file_actions;
mod find_replace;
mod hit_testing;
mod layout;
mod menu_bar;
mod modals;
mod page_setup;
mod panels;
mod popovers;
mod ruler;
mod state;
mod tabs;
mod toolbar;
mod toolbar_actions;
mod version_history;
mod widget;

use chrome::{
    info_modal_close_rect, info_modal_rect, is_info_modal, menu_at, menu_items_for,
    paint_docs_modal, paint_docs_toast, paint_find_replace_modal, paint_link_modal,
    paint_title_row,
};
use document_text::extract_block_text;
use modals::{paint_comment_modal, paint_context_menu};
use page_setup::paint_page_setup_dialog;
use popovers::{
    color_picker_hit, font_family_dropdown_hit, font_size_dropdown_hit, paint_tab_bar,
    paragraph_style_dropdown_hit, table_grid_hit,
};
use toolbar_actions::{
    toolbar_action_at, toolbar_tooltip_at, ToolbarAction, BTN_GAP, FONT_FAMILY_SELECT_W,
    FONT_SIZE_SELECT_W, PARAGRAPH_SELECT_W, SEPARATOR_W, TOOLBAR_LAYOUT, TOOLBAR_LEFT_PAD,
};

use layout::modal::{
    compute_find_replace, compute_goto, compute_info_modal, compute_print_preview,
};
use layout::sidebar::compute_sidebar;

use document::paint_document_area;
use menu_bar::paint_menu_bar;
use panels::{
    paint_comment_panel, paint_goto_modal, paint_print_preview, paint_special_char_modal,
    paint_status_bar, paint_style_panel, paint_thumbnails, paint_word_count_modal,
};
use ruler::{
    extract_indents_for_automation, paint_ruler, ruler_drag_to_indent, ruler_drag_to_margin,
    ruler_hit_test,
};
use state::{
    c_canvas_bg, c_separator, c_text_dim, c_text_light, extract_tdm, CommentModalState,
    ContextMenuState, ContextMenuType, DocsState, DocumentSession, FindReplaceState,
    GotoModalState, LinkModalState, PageSetupDialogState, ParagraphStyle, RulerDragTarget,
    SidebarTab, SpecialCharModalState, TabInfo, TableGridState, ToolbarDropdown, COLOR_PALETTE,
    FONT_FAMILIES, FONT_SIZES, MENU_BAR_H, RULER_H, STATUS_BAR_H, STYLE_PANEL_W, THUMB_PANEL_W,
    TITLE_ROW_H, TOOLBAR_H,
};
use tench_document_core::{
    Alignment, BlockNode, BlockType, CommentRange, CursorState, DocumentEngine, MarkType,
    MoveDirection, Orientation, PaperSize, TenchDocument,
};
use tench_ui::anim::AnimInterval;
use tench_ui::core::events::{ImeEvent, KeyboardEvent, PointerEvent};
use tench_ui::prelude::*;
use tench_ui::render::TextCache;
use tench_ui::{UiAutomationNode, UiAutomationRect};
use toolbar::{paint_toolbar, paint_toolbar_dropdowns, paint_toolbar_tooltip};

use crate::DialogResult;
use automation_document::push_document_state_nodes;
use automation_dropdown::push_dropdown_nodes;
use automation_helpers::{push_docs_node, simple_docs_id, toolbar_action_debug_id};
use automation_sidebar::push_sidebar_nodes;
use automation_status::push_status_nodes;

/// Top-level document editor widget.
pub struct DocsApp {
    state: DocsState,
    /// Per-tab sessions, each owning its own engine and document state.
    sessions: Vec<DocumentSession>,
    /// Index into `sessions` for the currently active tab.
    active_session_idx: usize,
    #[allow(dead_code)] // Will be used for cached text rendering
    text_cache: TextCache,
    cursor_timer: AnimInterval,
    drag_start: Option<CursorState>,
    /// Tauri AppHandle for native file dialogs.
    app_handle: Option<tauri::AppHandle>,
    /// Receiver for async dialog results.
    dialog_rx: Option<std::sync::mpsc::Receiver<DialogResult>>,
}

impl Default for DocsApp {
    fn default() -> Self {
        Self::new()
    }
}

impl DocsApp {
    pub fn new() -> Self {
        let state = DocsState::new();
        let engine = DocumentEngine::new(state.current_document().clone());
        let session = DocumentSession {
            engine,
            document: state.current_document().clone(),
            cursor: CursorState::default(),
            scroll_y: 0.0,
            dirty: false,
            title: "Untitled Document".into(),
        };
        Self {
            state,
            sessions: vec![session],
            active_session_idx: 0,
            text_cache: TextCache::new(),
            cursor_timer: AnimInterval::new(500.0),
            drag_start: None,
            app_handle: None,
            dialog_rx: None,
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

    #[doc(hidden)]
    pub fn automation_tab_count(&self) -> usize {
        self.state.open_tabs.len()
    }

    #[doc(hidden)]
    pub fn automation_active_tab_idx(&self) -> usize {
        self.state.active_tab_idx
    }

    /// Loads a document from a plain-text string for test automation.
    ///
    /// Each line becomes a separate paragraph. The current document is
    /// replaced and the engine/cursor are reset.
    #[doc(hidden)]
    pub fn load_plain_text(&mut self, text: &str) {
        let doc = TenchDocument::plain_text(text);
        let title = if doc.metadata.title.is_empty() {
            "Test Document".into()
        } else {
            doc.metadata.title.clone()
        };
        let engine = DocumentEngine::new(doc.clone());
        self.state.load_document(doc);
        let session = &mut self.sessions[self.active_session_idx];
        session.engine = engine;
        session.document = self.state.current_document().clone();
        session.cursor = CursorState::default();
        session.dirty = false;
        session.title = title;
    }

    #[cfg(debug_assertions)]
    pub fn debug_open_tab_count(&self) -> usize {
        self.state.open_tabs.len()
    }

    /// Get a mutable reference to the active session's engine.
    fn engine(&mut self) -> &mut DocumentEngine {
        &mut self.sessions[self.active_session_idx].engine
    }
}

// ---------------------------------------------------------------------------
// Remaining DocsApp methods: save, hit-test, cursor, clipboard, find/replace,
// version history, tooltips
// ---------------------------------------------------------------------------
