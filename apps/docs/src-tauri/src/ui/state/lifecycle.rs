use tench_document_core::{
    Alignment, Comment, CursorState, EditResult, OfficeArtifact, TenchDocument,
};

use super::*;
use crate::document_service;

impl DocsState {
    pub fn new() -> Self {
        let opened = document_service::create_document(Some("Untitled Document".into()));
        let document = extract_tdm(&opened.content);
        let document_text = document.to_plain_text();
        let word_count = count_words(&document_text);
        Self {
            artifact: opened.artifact,
            cursor: CursorState::default(),
            dirty: false,
            last_saved_text: document_text,
            document,
            status: "Unsaved local document".into(),
            show_thumbnails: false,
            show_style_panel: true,
            show_comments: false,
            track_changes: false,
            active_modal: None,
            toast: None,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            code: false,
            superscript: false,
            subscript: false,
            current_alignment: Alignment::Left,
            selection: None,
            selection_anchor: None,
            zoom: 100.0,
            hovered_btn: None,
            hovered_menu_item: None,
            cursor_visible: true,
            page_count: 1,
            current_page: 1,
            word_count,
            language: "English (US)".into(),
            open_dropdown: None,
            current_font_size: 16.0,
            current_paragraph_style: ParagraphStyle::Paragraph,
            link_modal: None,
            table_grid: TableGridState::default(),
            selected_text_color: None,
            selected_bg_color: None,
            scroll_y: 0.0,
            page_setup_dialog: None,
            editing_header: false,
            editing_footer: false,
            ruler_drag: None,
            find_replace: None,
            comments: Vec::new(),
            last_autosave_ts: 0.0,
            autosave_interval_ms: 30_000.0, // 30 seconds
            ctrl_pressed: false,
            version_history: Vec::new(),
            tracked_changes: Vec::new(),
            open_tabs: vec![TabInfo {
                title: "Untitled Document".into(),
                dirty: false,
            }],
            active_tab_idx: 0,
            layout_cache: DocumentLayoutCache::new(),
            last_window_size: (800.0, 600.0),
            hovered_dropdown_item: None,
            pending_file_action: None,
            context_menu: None,
            comment_modal: None,
            header_text: String::new(),
            footer_text: String::new(),
            image_resize_drag: None,
            targeted_image_block: None,
            print_preview: None,
            current_font_family: "Default".to_string(),
            custom_color_input: String::new(),
            sidebar_tab: SidebarTab::default(),
            goto_modal: None,
            word_count_modal: false,
            special_char_modal: None,
            comments_collapsed: false,
            version_history_collapsed: false,
            hovered_tooltip: None,
            hovered_tooltip_x: 0.0,
            undo_available: false,
            redo_available: false,
            clipboard_text: String::new(),
            clipboard_node_count: 0,
            window_maximized: false,
            window_control_hovered: None,
        }
    }

    pub fn title(&self) -> &str {
        &self.artifact.title
    }

    #[allow(dead_code)]
    pub fn document_text(&self) -> String {
        self.document.to_plain_text()
    }

    pub fn current_artifact(&self) -> &OfficeArtifact {
        &self.artifact
    }

    pub fn current_document(&self) -> &TenchDocument {
        &self.document
    }

    pub fn cursor(&self) -> &CursorState {
        &self.cursor
    }

    pub fn selection(&self) -> &Option<tench_document_core::SelectionRange> {
        // Selection is stored in the engine; the state mirrors it
        // via apply_edit_result. For now, return None until the
        // selection field is added to DocsState.
        &self.selection
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Set the dirty flag.
    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    /// Returns `true` when any modal dialog is currently open.
    ///
    /// Used by keyboard and pointer routing to prevent document input from
    /// leaking through an active modal layer.
    pub fn any_modal_open(&self) -> bool {
        self.active_modal.is_some()
            || self.link_modal.is_some()
            || self.page_setup_dialog.is_some()
            || self.print_preview.is_some()
            || self.word_count_modal
            || self.goto_modal.is_some()
            || self.special_char_modal.is_some()
            || self.find_replace.is_some()
            || self.comment_modal.is_some()
    }

    /// Replace the current document with a new one, resetting cursor and
    /// selection state. Used by test automation to load fixture content.
    #[doc(hidden)]
    pub fn load_document(&mut self, doc: TenchDocument) {
        self.document = doc;
        self.dirty = false;
        self.cursor = CursorState::default();
        self.selection = None;
    }

    pub fn status_line(&self) -> &str {
        &self.status
    }

    pub fn document_lines(&self) -> Vec<String> {
        self.document
            .to_plain_text()
            .lines()
            .map(String::from)
            .collect()
    }

    pub fn character_count(&self) -> usize {
        self.document.to_plain_text().chars().count()
    }

    pub fn paragraph_count(&self) -> usize {
        self.document.content.len().max(1)
    }

    pub fn read_time_minutes(&self) -> usize {
        self.word_count.max(1).div_ceil(200)
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status = message.into();
    }

    /// Apply an EditResult received from the backend DocumentEngine.
    /// Reset editing state for a new document tab, preserving open tabs.
    pub fn reset_for_new_document(&mut self, document: TenchDocument) {
        let document_text = document.to_plain_text();
        self.document = document;
        self.cursor = CursorState::default();
        self.dirty = false;
        self.last_saved_text = document_text.clone();
        self.status = "Unsaved local document".into();
        self.selection = None;
        self.selection_anchor = None;
        self.bold = false;
        self.italic = false;
        self.underline = false;
        self.strikethrough = false;
        self.code = false;
        self.superscript = false;
        self.subscript = false;
        self.current_alignment = Alignment::Left;
        self.active_modal = None;
        self.open_dropdown = None;
        self.hovered_btn = None;
        self.hovered_menu_item = None;
        self.ruler_drag = None;
        self.scroll_y = 0.0;
        self.header_text = String::new();
        self.footer_text = String::new();
        self.table_grid = TableGridState::default();
        self.image_resize_drag = None;
        self.targeted_image_block = None;
        self.context_menu = None;
        self.word_count = count_words(&document_text);
        self.layout_cache.invalidate();
        self.ensure_layout_cache();
        self.page_count = self.layout_cache.num_pages();
        self.current_page = 1;
    }

    pub fn apply_edit_result(&mut self, result: EditResult) {
        self.document = result.document;
        self.cursor = result.cursor;
        self.selection = result.selection;
        // Clear selection anchor when the engine clears the selection
        // (e.g. after backspace, delete, click, or plain arrow key).
        if self.selection.is_none() {
            self.selection_anchor = None;
        }
        self.dirty = result.dirty;
        self.layout_cache.invalidate();
        self.ensure_layout_cache();
        let text = self.document.to_plain_text();
        self.word_count = count_words(&text);
        self.page_count = self.layout_cache.num_pages();
        self.current_page = self.current_page.clamp(1, self.page_count);
        if self.dirty {
            self.status = "Unsaved changes".into();
        }
        // Sync dirty state to active tab
        if let Some(tab) = self.open_tabs.get_mut(self.active_tab_idx) {
            tab.dirty = self.dirty;
        }
    }

    /// Update the comments list from the engine.
    pub fn update_comments(&mut self, comments: Vec<Comment>) {
        self.comments = comments;
    }

    pub fn apply_saved_artifact(&mut self, artifact: OfficeArtifact) {
        self.artifact = artifact;
        self.artifact.dirty = false;
        self.dirty = false;
        self.last_saved_text = self.document.to_plain_text();
        self.status = if let Some(path) = &self.artifact.path {
            format!("Saved {path}")
        } else {
            format!("Saved {}", self.artifact.title)
        };
        self.show_toast("Document saved");
        // Update active tab title and dirty state
        if let Some(tab) = self.open_tabs.get_mut(self.active_tab_idx) {
            tab.title = self.artifact.title.clone();
            tab.dirty = false;
        }
    }

    /// Clear stale overlays (dropdown, context menu, menu hover) before
    /// opening a modal dialog.  Prevents stale overlay state from appearing
    /// above the modal backdrop or receiving pointer events.
    pub fn prepare_modal_open(&mut self) {
        self.open_dropdown = None;
        self.hovered_dropdown_item = None;
        self.context_menu = None;
        self.hovered_menu_item = None;
        self.hovered_tooltip = None;
    }

    /// Show a toast notification. The expiry timestamp (0.0) will be set to
    /// `current_ts + 3000.0` on the next animation frame.
    pub fn show_toast(&mut self, message: impl Into<String>) {
        self.toast = Some((message.into(), 0.0));
    }

    /// Check if autosave is due and return true if so.
    pub fn should_autosave(&self, now_ts: f64) -> bool {
        self.dirty && (now_ts - self.last_autosave_ts) >= self.autosave_interval_ms
    }

    /// Mark that autosave just happened.
    pub fn mark_autosave_done(&mut self, now_ts: f64) {
        self.last_autosave_ts = now_ts;
    }
}

fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

pub fn extract_tdm(content: &tench_document_core::OfficeContent) -> TenchDocument {
    match content {
        tench_document_core::OfficeContent::Docs(rich) => rich
            .document
            .clone()
            .unwrap_or_else(|| TenchDocument::new("")),
        _ => TenchDocument::new(""),
    }
}
