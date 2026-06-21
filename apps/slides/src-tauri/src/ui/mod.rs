//! Slides UI: native presentation editor widget.

mod automation;
mod canvas;
mod filmstrip;
pub mod modal;
mod notes;
mod properties;
pub mod state;
mod toolbar;
mod widget;

use canvas::{paint_slide_canvas, slide_page_rect};
use filmstrip::{hit_test_slide, paint_filmstrip};
use notes::paint_notes;
use properties::{
    paint_properties, properties_layout, slider_value_from_click, PropertyAction, SliderKind,
};
use state::{DragMode, SlidesState};
use tench_ui::core::events::ImeEvent;
use tench_ui::prelude::*;
use toolbar::{paint_toolbar, toolbar_layout, ToolbarAction};

use crate::{dialog_sender, presentation_service, DialogResult};

use modal::{paint_modal, ActiveModal};

const TOOLBAR_H: f64 = 40.0;
const THUMB_W: f64 = 180.0;
const PROPS_W: f64 = 260.0;
const NOTES_H: f64 = 96.0;

/// Root widget for the Slides presentation editor.
pub struct SlidesApp {
    state: SlidesState,
    /// Tauri AppHandle for native file dialogs.
    app_handle: Option<tauri::AppHandle>,
    /// Receiver for async dialog results.
    dialog_rx: Option<std::sync::mpsc::Receiver<DialogResult>>,
}

impl Default for SlidesApp {
    fn default() -> Self {
        Self::new()
    }
}

impl SlidesApp {
    pub fn new() -> Self {
        Self {
            state: SlidesState::new(),
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

    pub fn state(&self) -> &SlidesState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut SlidesState {
        &mut self.state
    }
}

impl SlidesApp {
    fn handle_pointer_down(
        &mut self,
        ctx: &mut EventCtx,
        e: &tench_ui::core::events::PointerButtonEvent,
    ) {
        let size = ctx.state.size;

        // Toolbar area
        if e.pos.y < TOOLBAR_H {
            // Caption buttons (top-right) take priority.
            if let Some(ctrl) = window_control_at(e.pos.x, e.pos.y, size.width, TOOLBAR_H) {
                match ctrl {
                    WindowControl::Close => ctx.submit_window_action(WindowAction::Close),
                    WindowControl::Minimize => ctx.submit_window_action(WindowAction::Minimize),
                    WindowControl::MaximizeRestore => {
                        ctx.submit_window_action(WindowAction::ToggleMaximize)
                    }
                }
                return;
            }
            let toolbar_rect = Rect::new(0.0, 0.0, size.width, TOOLBAR_H);
            let layout = toolbar_layout(toolbar_rect);
            let mut handled = false;
            for (_label, x_start, x_end, action) in &layout {
                if e.pos.x >= *x_start && e.pos.x < *x_end {
                    match action {
                        ToolbarAction::NewSlide => self.state.add_slide(),
                        ToolbarAction::OpenFile => {
                            self.state.active_modal = Some(ActiveModal::OpenFile);
                        }
                        ToolbarAction::Save => self.save_current_presentation(),
                        ToolbarAction::Undo => {
                            self.state.undo();
                        }
                        ToolbarAction::Redo => {
                            self.state.redo();
                        }
                        ToolbarAction::InsertText => {
                            self.state.insert_text_element("Text box");
                        }
                        ToolbarAction::InsertShape => {
                            self.state.active_modal = Some(ActiveModal::ShapeSelector);
                        }
                        ToolbarAction::InsertImage => {
                            self.state.active_modal = Some(ActiveModal::InsertImage);
                        }
                        ToolbarAction::TogglePresentation => {
                            if self.state.presenting {
                                self.state.stop_presentation();
                            } else {
                                self.state.start_presentation();
                            }
                            self.state.selected_element = None;
                        }
                    }
                    handled = true;
                    break;
                }
            }
            if handled {
                ctx.request_paint();
            } else {
                // Empty toolbar space: begin a window drag-move.
                ctx.submit_window_action(WindowAction::StartDrag);
            }
            return;
        }

        // Modal dismiss
        if self.state.active_modal.is_some() {
            let modal_size = size;
            let should_dismiss = self.handle_modal_click(ctx, e, modal_size);
            if should_dismiss {
                ctx.request_paint();
            }
            return;
        }

        // Filmstrip area
        let filmstrip_rect = Rect::new(0.0, TOOLBAR_H, THUMB_W + 20.0, size.height - NOTES_H);
        if filmstrip_rect.contains(e.pos) {
            if let Some(idx) = hit_test_slide(filmstrip_rect, e.pos, self.state.slides.len()) {
                self.state.selected_slides.clear();
                self.state.select_slide(idx);
                // Phase 8.2: start drag
                self.state.filmstrip_drag = Some(idx);
                ctx.request_paint();
            }
            return;
        }

        // Properties panel area
        let props_rect = Rect::new(
            size.width - PROPS_W,
            TOOLBAR_H,
            size.width,
            size.height - NOTES_H,
        );
        if props_rect.contains(e.pos) {
            let layout = properties_layout(&self.state, props_rect);
            let mut handled = false;
            for region in &layout {
                if region.rect.contains(e.pos) {
                    match region.action {
                        PropertyAction::ActionButton(i) => {
                            match i {
                                0 => self.state.bring_forward(),
                                1 => self.state.send_backward(),
                                2 => {
                                    self.state.duplicate_selected_element();
                                }
                                3 => {
                                    self.state.delete_selected_element();
                                }
                                _ => {}
                            }
                            handled = true;
                        }
                        PropertyAction::Slider(kind) => {
                            let val = slider_value_from_click(kind, region.rect, e.pos.x);
                            match kind {
                                SliderKind::PositionX => self.state.set_element_x(val),
                                SliderKind::PositionY => self.state.set_element_y(val),
                                SliderKind::Width => self.state.set_element_w(val),
                                SliderKind::Height => self.state.set_element_h(val),
                                SliderKind::Rotation => self.state.set_element_rotation(val),
                                SliderKind::Opacity => self.state.set_element_opacity(val),
                            }
                            handled = true;
                        }
                        PropertyAction::FillColorSwatch => {
                            self.state.cycle_element_fill_color();
                            handled = true;
                        }
                        PropertyAction::BorderColorSwatch => {
                            self.state.cycle_element_border_color();
                            handled = true;
                        }
                        PropertyAction::BackgroundButton => {
                            self.state.active_modal =
                                Some(modal::ActiveModal::BackgroundSettings { preset_index: 0 });
                            handled = true;
                        }
                        PropertyAction::ThemeButton => {
                            self.state.active_modal =
                                Some(modal::ActiveModal::ThemeSelector { selected_index: 0 });
                            handled = true;
                        }
                    }
                    break;
                }
            }
            if handled {
                ctx.request_paint();
            }
            return;
        }

        // Canvas area
        let canvas_rect = Rect::new(
            THUMB_W,
            TOOLBAR_H,
            size.width - PROPS_W,
            size.height - NOTES_H,
        );
        if canvas_rect.contains(e.pos) {
            // Phase 9: Space+drag for pan
            if self.state.space_held {
                self.state.interaction.mode = DragMode::Pan;
                self.state.interaction.start_pos = e.pos;
                ctx.request_paint();
                return;
            }

            let page = slide_page_rect(
                canvas_rect,
                self.state.zoom.level,
                self.state.zoom.pan_x,
                self.state.zoom.pan_y,
            );

            // Phase 1.2: Check resize handles first
            if let Some(handle) = self.state.hit_test_resize_handle(e.pos, page) {
                let Some(idx) = self.state.selected_element else {
                    return;
                };
                self.state.begin_resize(idx, handle, e.pos);
                ctx.request_paint();
                return;
            }

            // Phase 1.3: Check rotation handle
            if self.state.hit_test_rotate_handle(e.pos, page) {
                let Some(idx) = self.state.selected_element else {
                    return;
                };
                self.state.begin_rotate(idx, e.pos);
                ctx.request_paint();
                return;
            }

            // Check element hit
            if let Some(hit_idx) = self.state.hit_test_element(e.pos, page) {
                // Phase 3: Double-click detection for text editing
                let click_count = self.state.click_tracker.record_click(e.pos);
                let elem_kind = self
                    .state
                    .current_slide()
                    .and_then(|s| s.elements.get(hit_idx))
                    .map(|e| e.kind.as_str())
                    .unwrap_or("");
                let is_text_like = matches!(elem_kind, "text" | "title" | "subtitle");
                if click_count >= 2 && is_text_like {
                    self.state.begin_text_edit(hit_idx);
                    ctx.request_paint();
                    return;
                }

                // Phase 1.4: Shift+click for multi-select
                // TODO: check shift modifier when available
                #[allow(clippy::overly_complex_bool_expr)]
                if false {
                    self.state.toggle_element_selection(hit_idx);
                } else {
                    self.state.selected_element = Some(hit_idx);
                    // Phase 1.1: Begin drag
                    self.state.begin_drag(hit_idx, e.pos);
                }
            } else {
                // Clicked empty space
                self.state.selected_element = None;
                self.state.selected_elements.clear();
                // Phase 1.4: Begin box select
                self.state.begin_box_select(e.pos);
            }
            ctx.request_paint();
        }
    }

    fn handle_pointer_move(
        &mut self,
        ctx: &mut EventCtx,
        e: &tench_ui::core::events::PointerMoveEvent,
    ) {
        // Track caption button hover for visual feedback.
        let size = ctx.state.size;
        let new_hover = window_control_at(e.pos.x, e.pos.y, size.width, TOOLBAR_H);
        if new_hover != self.state.window_control_hovered {
            self.state.window_control_hovered = new_hover;
            ctx.request_paint();
        }
        match self.state.interaction.mode {
            DragMode::Move => {
                self.state.update_drag(e.pos);
                ctx.request_paint();
            }
            DragMode::Resize(_) => {
                self.state.update_resize(e.pos);
                ctx.request_paint();
            }
            DragMode::Rotate => {
                self.state.update_rotate(e.pos);
                ctx.request_paint();
            }
            DragMode::BoxSelect => {
                self.state.update_box_select(e.pos);
                ctx.request_paint();
            }
            DragMode::Pan => {
                let dx = e.pos.x - self.state.interaction.start_pos.x;
                let dy = e.pos.y - self.state.interaction.start_pos.y;
                self.state.zoom.pan_x += dx;
                self.state.zoom.pan_y += dy;
                self.state.interaction.start_pos = e.pos;
                ctx.request_paint();
            }
            DragMode::None => {
                // Phase 6: Laser pointer
                if self.state.presenting && self.state.presentation.laser_pointer {
                    self.state.set_laser_position(e.pos);
                    ctx.request_paint();
                }
            }
        }
    }

    fn handle_pointer_up(
        &mut self,
        ctx: &mut EventCtx,
        _e: &tench_ui::core::events::PointerButtonEvent,
    ) {
        if self.state.interaction.mode != DragMode::None {
            self.state.end_interaction();
            ctx.request_paint();
        }
    }

    fn handle_modal_click(
        &mut self,
        _ctx: &mut EventCtx,
        e: &tench_ui::core::events::PointerButtonEvent,
        size: Size,
    ) -> bool {
        use modal::{
            hit_test_background_gradients, hit_test_background_presets, hit_test_chart_wizard,
            hit_test_export_formats, hit_test_layout_selector, hit_test_modal_buttons,
            hit_test_shape_selector, hit_test_theme_selector, hit_test_transition_selector,
        };

        let Some(modal) = self.state.active_modal.clone() else {
            return false;
        };

        // Check for specific item clicks inside modals
        match &modal {
            ActiveModal::ShapeSelector => {
                if let Some(kind) = hit_test_shape_selector(size, e.pos) {
                    self.state.insert_shape(kind, 100.0, 100.0, 200.0, 120.0);
                    self.state.active_modal = None;
                    return true;
                }
            }
            ActiveModal::ChartWizard { .. } => {
                if let Some(ct) = hit_test_chart_wizard(size, e.pos) {
                    self.state.insert_chart(ct, 120.0, 80.0);
                    self.state.active_modal = None;
                    return true;
                }
            }
            ActiveModal::LayoutSelector => {
                if let Some(idx) = hit_test_layout_selector(size, e.pos) {
                    use state::SlideLayoutType;
                    let layout = match idx {
                        0 => SlideLayoutType::Blank,
                        1 => SlideLayoutType::Title,
                        2 => SlideLayoutType::TitleContent,
                        3 => SlideLayoutType::TwoColumn,
                        4 => SlideLayoutType::SectionHeader,
                        _ => SlideLayoutType::Blank,
                    };
                    self.state.apply_slide_layout(layout);
                    self.state.active_modal = None;
                    return true;
                }
            }
            ActiveModal::SlideTransition => {
                if let Some(name) = hit_test_transition_selector(size, e.pos) {
                    self.state.set_slide_transition(name, 500);
                    self.state.active_modal = None;
                    return true;
                }
            }
            ActiveModal::BackgroundSettings { preset_index } => {
                let current_preset = *preset_index;
                if let Some(idx) = hit_test_background_presets(size, e.pos) {
                    let colors = [
                        Color::WHITE,
                        Color::rgb8(0xF8, 0xF8, 0xF8),
                        Color::rgb8(0x1E, 0x1E, 0x2E),
                        Color::rgb8(0x0D, 0x1B, 0x2A),
                        Color::rgb8(0x2D, 0x1B, 0x0E),
                    ];
                    if let Some(&color) = colors.get(idx) {
                        self.state
                            .set_current_slide_background(state::SlideBackground {
                                color: Some(color),
                                gradient_start: None,
                                gradient_end: None,
                                image_path: None,
                            });
                    }
                    self.state.active_modal =
                        Some(ActiveModal::BackgroundSettings { preset_index: idx });
                    return true;
                }
                if let Some(idx) = hit_test_background_gradients(size, e.pos) {
                    let gradients = [
                        (Color::rgb8(0x60, 0xA5, 0xFA), Color::rgb8(0x93, 0x52, 0xF6)),
                        (Color::rgb8(0xEF, 0x44, 0x44), Color::rgb8(0xF5, 0x9E, 0x0B)),
                        (Color::rgb8(0x22, 0xC5, 0x5E), Color::rgb8(0x06, 0xB6, 0xD4)),
                    ];
                    if let Some(&(start, end)) = gradients.get(idx) {
                        self.state
                            .set_current_slide_background(state::SlideBackground {
                                color: None,
                                gradient_start: Some(start),
                                gradient_end: Some(end),
                                image_path: None,
                            });
                    }
                    self.state.active_modal = Some(ActiveModal::BackgroundSettings {
                        preset_index: current_preset,
                    });
                    return true;
                }
            }
            ActiveModal::ThemeSelector { .. } => {
                if let Some(idx) = hit_test_theme_selector(size, e.pos) {
                    let themes = state::SlideTheme::all_themes();
                    if let Some(theme) = themes.get(idx).cloned() {
                        self.state.set_slide_theme(theme);
                    }
                    self.state.active_modal = Some(ActiveModal::ThemeSelector {
                        selected_index: idx,
                    });
                    return true;
                }
            }
            ActiveModal::Export { format_index } => {
                if let Some(idx) = hit_test_export_formats(size, e.pos) {
                    self.state.active_modal = Some(ActiveModal::Export { format_index: idx });
                    return true;
                }
                let _ = format_index;
            }
            _ => {}
        }

        // Check OK/Cancel buttons
        let (handled, confirmed) = hit_test_modal_buttons(size, &modal, e.pos);
        if handled && confirmed {
            // OK clicked — perform the modal's action
            match &modal {
                ActiveModal::OpenFile => {
                    self.open_file_dialog();
                }
                ActiveModal::InsertImage => {
                    self.insert_image_dialog();
                }
                ActiveModal::SaveAs => {
                    self.save_as_dialog();
                }
                ActiveModal::Export { format_index } => {
                    let formats = ["PDF", "PNG", "PPTX"];
                    let fmt = formats.get(*format_index).unwrap_or(&"PDF");
                    self.state.show_toast(format!("Exporting as {}...", fmt));
                    // Phase 10: Export via presentation_service
                    let _ = fmt;
                }
                ActiveModal::TableWizard { rows, cols } => {
                    self.state.insert_table(*rows, *cols, 80.0, 60.0);
                }
                _ => {}
            }
            self.state.active_modal = None;
            return true;
        }
        if handled && !confirmed {
            // Cancel clicked
            self.state.active_modal = None;
            return true;
        }

        false
    }

    fn handle_scroll(
        &mut self,
        ctx: &mut EventCtx,
        e: &tench_ui::core::events::PointerScrollEvent,
    ) {
        // Mouse wheel zoom (Ctrl+scroll) or plain scroll
        if e.modifiers.control {
            if e.delta.y > 0.0 {
                self.state.zoom_in();
            } else if e.delta.y < 0.0 {
                self.state.zoom_out();
            }
            ctx.request_paint();
        } else {
            // Pan
            self.state.pan(-e.delta.x, -e.delta.y);
            ctx.request_paint();
        }
    }

    fn save_current_presentation(&mut self) {
        let artifact = self.state.current_artifact().clone();
        let content = self.state.current_content().clone();

        if artifact.path.is_some() {
            match presentation_service::save_presentation(artifact, content, None, None) {
                Ok(saved) => self.state.apply_saved_artifact(saved.artifact),
                Err(error) => {
                    self.state.show_toast(String::from("Save failed"));
                    self.state.active_modal = Some(ActiveModal::SaveError(error.to_string()));
                }
            }
            return;
        }

        match presentation_service::save_recovery_snapshot(artifact, content) {
            Ok(_) => self
                .state
                .show_toast(String::from("Recovery snapshot saved")),
            Err(error) => {
                self.state.show_toast(String::from("Save failed"));
                self.state.active_modal = Some(ActiveModal::SaveError(error.to_string()));
            }
        }
    }

    // ── Native file dialogs ──

    /// Open a native file dialog to pick a presentation to open.
    fn open_file_dialog(&self) {
        use tauri_plugin_dialog::DialogExt;

        let Some(ref handle) = self.app_handle else {
            return;
        };
        let Some(tx) = dialog_sender() else {
            return;
        };

        let tx = tx.clone();
        handle
            .dialog()
            .file()
            .add_filter("Presentations", &["json", "pptx", "odp"])
            .set_title("Open Presentation")
            .pick_file(move |path| {
                if let Some(p) = path {
                    let _ = tx.send(DialogResult::OpenFile(p.to_string()));
                }
            });
    }

    /// Open a native file dialog to pick a save location.
    fn save_as_dialog(&self) {
        use tauri_plugin_dialog::DialogExt;

        let Some(ref handle) = self.app_handle else {
            return;
        };
        let Some(tx) = dialog_sender() else {
            return;
        };

        let tx = tx.clone();
        handle
            .dialog()
            .file()
            .add_filter("Presentations", &["json"])
            .set_title("Save As")
            .set_file_name("Untitled.json")
            .save_file(move |path| {
                if let Some(p) = path {
                    let _ = tx.send(DialogResult::SaveAs(p.to_string()));
                }
            });
    }

    /// Open a native file dialog to pick an image to insert.
    fn insert_image_dialog(&self) {
        use tauri_plugin_dialog::DialogExt;

        let Some(ref handle) = self.app_handle else {
            return;
        };
        let Some(tx) = dialog_sender() else {
            return;
        };

        let tx = tx.clone();
        handle
            .dialog()
            .file()
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg"],
            )
            .set_title("Insert Image")
            .pick_file(move |path| {
                if let Some(p) = path {
                    let _ = tx.send(DialogResult::InsertImage(p.to_string()));
                }
            });
    }

    /// Process any pending dialog results from async dialogs.
    fn process_dialog_results(&mut self, ctx: &mut EventCtx) {
        let results: Vec<DialogResult> = {
            let Some(ref rx) = self.dialog_rx else {
                return;
            };
            rx.try_iter().collect()
        };
        for result in results {
            match result {
                DialogResult::OpenFile(path) => {
                    match presentation_service::open_presentation(path) {
                        Ok(resp) => {
                            self.state.load_presentation(resp.artifact, resp.content);
                            self.state.show_toast(String::from("Presentation opened"));
                        }
                        Err(e) => {
                            self.state.show_toast(format!("Open failed: {e}"));
                        }
                    }
                }
                DialogResult::SaveAs(path) => {
                    let artifact = self.state.current_artifact().clone();
                    let content = self.state.current_content().clone();
                    match presentation_service::save_presentation(
                        artifact,
                        content,
                        Some(path),
                        None,
                    ) {
                        Ok(saved) => {
                            self.state.apply_saved_artifact(saved.artifact);
                            self.state.show_toast(String::from("Saved"));
                        }
                        Err(e) => {
                            self.state.show_toast(format!("Save As failed: {e}"));
                        }
                    }
                }
                DialogResult::InsertImage(path) => {
                    let filename = std::path::Path::new(&path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "image".into());
                    self.state
                        .insert_image_element(&filename, 80.0, 80.0, 240.0, 180.0);
                    self.state.show_toast(format!("Inserted: {filename}"));
                }
            }
        }
        ctx.request_paint();
    }
}

fn paint_slides_toast(p: &mut Painter<'_>, theme: &Theme, size: Size, message: &str) {
    let rect = Rect::new(
        size.width - 260.0,
        TOOLBAR_H + 12.0,
        size.width - 16.0,
        TOOLBAR_H + 46.0,
    );
    p.fill_rounded_rect(rect, theme.surface, theme.border_radius);
    p.stroke_rounded_rect(rect, theme.border, 1.0, theme.border_radius);
    p.draw_text(
        message,
        rect.x0 + 12.0,
        rect.y0 + 22.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
}

fn paint_presenter_overlay(p: &mut Painter<'_>, theme: &Theme, size: Size, state: &SlidesState) {
    // Full-screen black background
    let full = Rect::new(0.0, 0.0, size.width, size.height);
    p.fill_rect(full, Color::BLACK);

    let pres_slide_idx = state.presentation.current_slide;
    let Some(slide) = state.slides.get(pres_slide_idx) else {
        return;
    };

    // Phase 6.1: Paint actual slide content in the center
    let slide_margin = 40.0;
    let slide_area_w = size.width - slide_margin * 2.0;
    let slide_area_h = size.height - slide_margin * 2.0 - 60.0; // reserve bottom for notes
                                                                // Maintain 16:9 aspect ratio
    let slide_aspect = 16.0 / 9.0;
    let (slide_w, slide_h) = if slide_area_w / slide_area_h > slide_aspect {
        (slide_area_h * slide_aspect, slide_area_h)
    } else {
        (slide_area_w, slide_area_w / slide_aspect)
    };
    let slide_x = (size.width - slide_w) / 2.0;
    let slide_y = slide_margin + 20.0;

    // Slide background
    let slide_rect = Rect::new(slide_x, slide_y, slide_x + slide_w, slide_y + slide_h);
    let bg = &slide.background;
    if let Some(start) = bg.gradient_start {
        let end = bg.gradient_end.unwrap_or(Color::BLACK);
        p.fill_rounded_rect(slide_rect, start, 4.0);
        let _ = end;
    } else if let Some(bg_color) = bg.color {
        p.fill_rounded_rect(slide_rect, bg_color, 4.0);
    } else {
        p.fill_rounded_rect(slide_rect, Color::WHITE, 4.0);
    }

    // Paint slide elements scaled to presentation size
    let scale_x = slide_w / 640.0;
    let scale_y = slide_h / 360.0;
    for elem in &slide.elements {
        let elem_rect = Rect::new(
            slide_x + elem.x * scale_x,
            slide_y + elem.y * scale_y,
            slide_x + (elem.x + elem.w) * scale_x,
            slide_y + (elem.y + elem.h) * scale_y,
        );

        // Fill
        if let Some(fill) = elem.fill {
            p.fill_rounded_rect(elem_rect, fill, 2.0);
        }

        // Border
        if let Some(border) = &elem.border {
            if border.width > 0.0 {
                p.stroke_rounded_rect(elem_rect, border.color, border.width, 2.0);
            }
        }

        // Text
        if let Some(text) = &elem.text {
            if !matches!(elem.kind.as_str(), "table" | "chart" | "image") {
                let title = elem.kind == "title";
                let font_size = if title {
                    24.0 * scale_x
                } else {
                    16.0 * scale_x
                };
                let text_color = if elem.fill.is_some() {
                    Color::WHITE
                } else {
                    Color::BLACK
                };
                p.draw_text(
                    text,
                    elem_rect.x0 + 8.0,
                    elem_rect.y0 + elem_rect.height() / 2.0 + font_size / 3.0,
                    text_color,
                    font_size as f32,
                    if title {
                        tench_ui::parley::FontWeight::BOLD
                    } else {
                        tench_ui::parley::FontWeight::NORMAL
                    },
                    false,
                );
            }
        }
    }

    // Phase 6.4: Timer with HH:MM:SS format
    let elapsed = state.presentation_elapsed_secs();
    let hours = (elapsed / 3600.0) as u32;
    let mins = ((elapsed % 3600.0) / 60.0) as u32;
    let secs = (elapsed % 60.0) as u32;
    let timer_text = format!("{}:{:02}:{:02}", hours, mins, secs);
    p.draw_text(
        &timer_text,
        size.width - 160.0,
        30.0,
        Color::rgb8(0x99, 0x99, 0x99),
        18.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );

    // Phase 6.4: Slide counter
    p.draw_text(
        &format!("{}/{}", pres_slide_idx + 1, state.slides.len()),
        size.width - 160.0,
        52.0,
        Color::rgb8(0x99, 0x99, 0x99),
        14.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );

    // Phase 6.4: Speaker notes at bottom
    if !slide.notes.is_empty() {
        let notes_y = size.height - 50.0;
        p.draw_text(
            &slide.notes,
            40.0,
            notes_y,
            Color::rgb8(0xAA, 0xAA, 0xAA),
            12.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
    }

    // Phase 6.5: Laser pointer
    if state.presentation.laser_pointer {
        p.fill_circle(
            state.presentation.laser_pos,
            4.0,
            Color::rgb8(0xFF, 0x00, 0x00),
        );
    }

    let _ = theme;
}
