use tench_document_core::{
    Alignment, Comment, CursorState, EditResult, OfficeArtifact, TenchDocument,
};

use super::*;
use crate::document_service;

impl KodocsState {
    pub fn new() -> Self {
        let opened = document_service::create_document(Some("제목 없는 한글 문서".into()));
        let document = extract_tdm(&opened.content);
        let document_text = document.to_plain_text();
        let word_count = count_words(&document_text);
        Self {
            artifact: opened.artifact,
            cursor: CursorState::default(),
            dirty: false,
            last_saved_text: document_text,
            document,
            status: "저장 안 된 한글 문서".into(),
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
            zoom: 100.0,
            hovered_btn: None,
            hovered_menu_item: None,
            hovered_dropdown_item: None,
            cursor_visible: true,
            page_count: 1,
            current_page: 1,
            word_count,
            language: "한국어".into(),
            open_dropdown: None,
            current_font_family: FONT_FAMILIES[0].to_string(),
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
            open_tabs: vec![TabInfo {
                title: "제목 없는 한글 문서".into(),
                dirty: false,
            }],
            active_tab_idx: 0,
            layout_cache: DocumentLayoutCache::new(),
            hanja_popup: None,
            vertical_writing: false,
            last_window_size: (800.0, 600.0),
            equation_editor: None,
            tracked_changes: Vec::new(),
            context_menu: None,
            comment_modal: None,
            header_text: String::new(),
            footer_text: String::new(),
            image_resize_drag: None,
            print_preview: None,
            custom_color_input: String::new(),
            sidebar_tab: SidebarTab::default(),
            goto_modal: None,
            word_count_modal: false,
            special_char_modal: None,
            comments_collapsed: false,
            version_history_collapsed: false,
            hovered_tooltip: None,
            hovered_tooltip_x: 0.0,
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
        // selection field is added to KodocsState.
        &self.selection
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
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
    pub fn apply_edit_result(&mut self, result: EditResult) {
        self.document = result.document;
        self.cursor = result.cursor;
        self.selection = result.selection;
        self.dirty = result.dirty;
        self.layout_cache.invalidate();
        let text = self.document.to_plain_text();
        self.word_count = count_words(&text);
        self.page_count = compute_page_count(&self.document);
        self.current_page = self.current_page.clamp(1, self.page_count);
        if self.dirty {
            self.status = "저장 안 됨".into();
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
            format!("저장됨: {path}")
        } else {
            format!("저장됨: {}", self.artifact.title)
        };
        self.toast = Some(("문서가 저장되었습니다".into(), 0.0));
        // Update active tab title and dirty state
        if let Some(tab) = self.open_tabs.get_mut(self.active_tab_idx) {
            tab.title = self.artifact.title.clone();
            tab.dirty = false;
        }
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

/// Estimate page count based on content and page setup dimensions.
pub(super) fn compute_page_count(doc: &TenchDocument) -> usize {
    if doc.content.is_empty() {
        return 1;
    }
    let setup = &doc.page_setup;
    let content_h = setup.content_height_px();
    if content_h <= 0.0 {
        return 1;
    }
    // Rough estimate: each block takes about 32px of vertical space.
    let line_h = 20.0;
    let block_spacing = 12.0;
    let mut total_content_h = 0.0;
    for block in &doc.content {
        let text = extract_block_text_for_estimate(block);
        let content_w = setup.content_width_px();
        let chars_per_line = if content_w > 0.0 {
            (content_w / 7.0).max(1.0) as usize
        } else {
            80
        };
        let lines = if text.is_empty() {
            1
        } else {
            text.chars().count().div_ceil(chars_per_line)
        };
        total_content_h += lines as f64 * line_h + block_spacing;
    }
    let pages = (total_content_h / content_h).ceil() as usize;
    pages.max(1)
}

/// Extract text from a block for page count estimation.
pub(super) fn extract_block_text_for_estimate(block: &tench_document_core::BlockNode) -> String {
    match block {
        tench_document_core::BlockNode::Paragraph { content, .. }
        | tench_document_core::BlockNode::Heading { content, .. } => {
            let mut out = String::new();
            for node in content {
                match node {
                    tench_document_core::InlineNode::Text { text, .. } => out.push_str(text),
                    tench_document_core::InlineNode::Link { text, .. } => out.push_str(text),
                    _ => {}
                }
            }
            out
        }
        tench_document_core::BlockNode::CodeBlock { code, .. } => code.clone(),
        tench_document_core::BlockNode::BlockQuote { content } => content
            .iter()
            .map(extract_block_text_for_estimate)
            .collect::<Vec<_>>()
            .join("\n"),
        tench_document_core::BlockNode::BulletList { items }
        | tench_document_core::BlockNode::OrderedList { items, .. } => items
            .iter()
            .map(|i| {
                let mut out = String::new();
                for node in &i.content {
                    match node {
                        tench_document_core::InlineNode::Text { text, .. } => out.push_str(text),
                        tench_document_core::InlineNode::Link { text, .. } => out.push_str(text),
                        _ => {}
                    }
                }
                out
            })
            .collect::<Vec<_>>()
            .join("\n"),
        tench_document_core::BlockNode::TaskList { items } => items
            .iter()
            .map(|i| {
                let mut out = String::new();
                for node in &i.content {
                    match node {
                        tench_document_core::InlineNode::Text { text, .. } => out.push_str(text),
                        tench_document_core::InlineNode::Link { text, .. } => out.push_str(text),
                        _ => {}
                    }
                }
                out
            })
            .collect::<Vec<_>>()
            .join("\n"),
        tench_document_core::BlockNode::HorizontalRule
        | tench_document_core::BlockNode::PageBreak => String::new(),
        tench_document_core::BlockNode::Footnote { number, content } => {
            let mut out = format!("[{number}] ");
            for node in content {
                match node {
                    tench_document_core::InlineNode::Text { text, .. }
                    | tench_document_core::InlineNode::Link { text, .. } => out.push_str(text),
                    _ => {}
                }
            }
            out
        }
        tench_document_core::BlockNode::Image { alt, .. } => alt.clone().unwrap_or_default(),
        tench_document_core::BlockNode::Table { rows } => rows
            .iter()
            .flat_map(|r| r.cells.iter())
            .map(|c| {
                c.content
                    .iter()
                    .map(extract_block_text_for_estimate)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\t"),
    }
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
