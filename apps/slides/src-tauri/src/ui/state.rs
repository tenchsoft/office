use serde_json::json;
use tench_document_core::{OfficeArtifact, OfficeContent};
use tench_ui::prelude::{Color, Point, Rect};

use super::modal::ActiveModal;
use crate::presentation_service;

mod content;
mod element_ops;
mod elements;
mod interaction_ops;
mod presentation_ops;
mod search_clipboard_ops;
mod slide_ops;
mod text_ops;
mod theme;
mod types;
mod view_ops;
mod z_order_ops;

#[cfg(test)]
mod tests;

pub use content::content_to_slides;
use content::{slides_signature, slides_to_content};
pub use elements::*;
pub use theme::SlideTheme;
pub use types::*;

// ── Main application state ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SlidesState {
    artifact: OfficeArtifact,
    content: OfficeContent,
    last_saved_signature: String,
    status: String,
    pub slides: Vec<Slide>,
    pub current_slide: usize,
    pub selected_element: Option<usize>,
    pub selected_elements: Vec<usize>,
    pub active_modal: Option<ActiveModal>,
    pub toast: Option<String>,
    /// Phase 11: Frames remaining for toast auto-dismiss.
    pub toast_frames: u32,
    pub presenting: bool,
    pub show_animation_panel: bool,
    pub transition_name: String,
    // Phase 1: canvas interaction
    pub interaction: CanvasInteraction,
    // Phase 1.7: grouping
    pub next_group_id: u32,
    // Phase 1.8: undo/redo
    pub history: UndoRedoStack,
    // Phase 2: text editing
    pub text_edit: TextEditState,
    // Phase 6: presentation mode
    pub presentation: PresentationState,
    // Phase 8.1: find/replace
    pub find_replace: FindReplaceState,
    // Phase 8.2: auto-save
    pub auto_save: AutoSaveState,
    // Phase 8.3: clipboard
    pub clipboard: Option<ClipboardData>,
    // Phase 9.6: zoom
    pub zoom: ZoomState,
    // Phase 11: print
    pub print_settings: PrintSettings,
    // Phase 8.6: version history
    pub version_history: Vec<VersionEntry>,
    // Phase 3: double-click tracker
    pub click_tracker: ClickTracker,
    // Phase 7: theme
    pub slide_theme: SlideTheme,
    // Phase 8: filmstrip state
    pub selected_slides: Vec<usize>,
    pub filmstrip_drag: Option<usize>,
    // Phase 9: canvas helpers
    pub show_grid: bool,
    pub grid_size: f64,
    pub snap_to_grid: bool,
    // Phase 9: space-held for hand-pan
    pub space_held: bool,
    /// Whether the platform window is currently maximized (caption glyph).
    pub window_maximized: bool,
    /// Caption button currently under the pointer, if any (hover feedback).
    pub window_control_hovered: Option<tench_ui::WindowControl>,
    /// License: true when the local store has a valid, non-expired
    /// device_token. The toolbar's notification label hides when this is
    /// true. Synced from LicenseStore on each frame.
    pub license_active: bool,
    /// License: true when the update scheduler has seen a newer manifest
    /// version than the running binary. The notification label cycles two
    /// messages when both `license_active == false` and this is true.
    pub update_available: bool,
    /// License modal state — opens from the License toolbar button, hosts
    /// the license_key input field and shows status text.
    pub license_modal: Option<LicenseModalState>,
}

#[derive(Debug, Clone)]
pub struct VersionEntry {
    pub timestamp: String,
    pub slide_count: usize,
    pub signature: String,
}

impl Default for SlidesState {
    fn default() -> Self {
        Self::new()
    }
}

impl SlidesState {
    pub fn new() -> Self {
        let opened =
            presentation_service::create_presentation(Some("Untitled Presentation".into()));
        // Phase 0.3: start with empty presentation (no mock data)
        let slides = content_to_slides(&opened.content);
        let signature = slides_signature(&slides);
        let content = slides_to_content(&opened.artifact.title, &slides);
        Self {
            artifact: opened.artifact,
            content,
            last_saved_signature: signature,
            status: "Presentation loaded locally".into(),
            slides,
            current_slide: 0,
            selected_element: None,
            selected_elements: Vec::new(),
            active_modal: None,
            toast: Some("Presentation loaded locally".into()),
            toast_frames: 180, // ~3 seconds at 60fps
            presenting: false,
            show_animation_panel: false,
            transition_name: "Fade".into(),
            interaction: CanvasInteraction::default(),
            next_group_id: 0,
            history: UndoRedoStack::new(),
            text_edit: TextEditState::default(),
            presentation: PresentationState::default(),
            find_replace: FindReplaceState::default(),
            auto_save: AutoSaveState::default(),
            clipboard: None,
            zoom: ZoomState::default(),
            print_settings: PrintSettings::default(),
            version_history: Vec::new(),
            click_tracker: ClickTracker::default(),
            slide_theme: SlideTheme::default(),
            selected_slides: Vec::new(),
            filmstrip_drag: None,
            show_grid: false,
            grid_size: 20.0,
            snap_to_grid: false,
            space_held: false,
            window_maximized: false,
            window_control_hovered: None,
            license_active: false,
            update_available: false,
            license_modal: None,
        }
    }

    // ── Accessors ──────────────────────────────────────────────────

    pub fn current_artifact(&self) -> &OfficeArtifact {
        &self.artifact
    }

    pub fn current_content(&self) -> &OfficeContent {
        &self.content
    }

    pub fn is_dirty(&self) -> bool {
        self.artifact.dirty
    }

    pub fn status_line(&self) -> &str {
        &self.status
    }

    pub fn apply_saved_artifact(&mut self, artifact: OfficeArtifact) {
        self.artifact = artifact;
        self.artifact.dirty = false;
        self.last_saved_signature = slides_signature(&self.slides);
        self.status = if let Some(path) = &self.artifact.path {
            format!("Saved {path}")
        } else {
            format!("Saved {}", self.artifact.title)
        };
        self.show_toast("Presentation saved");
        self.sync_content_from_slides();
    }

    /// Load a completely new presentation from artifact + content.
    pub fn load_presentation(&mut self, artifact: OfficeArtifact, content: OfficeContent) {
        self.artifact = artifact;
        self.content = content;
        self.sync_slides_from_content();
        self.current_slide = 0;
        self.selected_element = None;
        self.selected_elements.clear();
        self.selected_slides.clear();
        self.last_saved_signature = slides_signature(&self.slides);
        self.status = format!("Opened {}", self.artifact.title);
    }

    // ── Slide navigation ───────────────────────────────────────────

    pub fn current_slide(&self) -> Option<&Slide> {
        self.slides.get(self.current_slide)
    }

    pub fn current_slide_mut(&mut self) -> Option<&mut Slide> {
        self.slides.get_mut(self.current_slide)
    }

    // ── Undo/Redo support ──────────────────────────────────────────

    fn snapshot(&self) -> HistoryEntry {
        HistoryEntry {
            slides: self.slides.clone(),
            current_slide: self.current_slide,
            selected_element: self.selected_element,
        }
    }

    fn push_undo(&mut self) {
        self.history.push(self.snapshot());
    }

    pub fn undo(&mut self) -> bool {
        if let Some(entry) = self.history.undo(&self.snapshot()) {
            self.slides = entry.slides;
            self.current_slide = entry.current_slide.min(self.slides.len().saturating_sub(1));
            self.selected_element = entry.selected_element;
            self.sync_content_from_slides();
            return true;
        }
        false
    }

    pub fn redo(&mut self) -> bool {
        if let Some(entry) = self.history.redo(&self.snapshot()) {
            self.slides = entry.slides;
            self.current_slide = entry.current_slide.min(self.slides.len().saturating_sub(1));
            self.selected_element = entry.selected_element;
            self.sync_content_from_slides();
            return true;
        }
        false
    }
}

impl SlidesState {
    fn sync_content_from_slides(&mut self) {
        self.content = slides_to_content(&self.artifact.title, &self.slides);
        self.artifact.dirty = slides_signature(&self.slides) != self.last_saved_signature;
        if self.artifact.dirty {
            self.status = "Unsaved changes".into();
        }
    }

    fn sync_slides_from_content(&mut self) {
        self.slides = content_to_slides(&self.content);
    }
}
