// ---------------------------------------------------------------------------
// File save/open actions
// ---------------------------------------------------------------------------

use tench_office_io::docs::format as format_io;
use tench_ui::prelude::*;

use crate::document_service;
use crate::{dialog_sender, DialogResult};

use super::DocsApp;

impl DocsApp {
    pub(super) fn save_current_document(&mut self) {
        let artifact = self.state.current_artifact().clone();
        let content = format_io::tdm_to_docs_content(self.state.current_document());

        if artifact.path.is_some() {
            match document_service::save_document(artifact, content, None, None) {
                Ok(saved) => self.state.apply_saved_artifact(saved.artifact),
                Err(error) => {
                    self.state.set_status(format!("Save failed: {error}"));
                    self.state.show_toast("Save failed");
                }
            }
            return;
        }

        match document_service::save_recovery_snapshot(artifact, content) {
            Ok(snapshot) => {
                self.state.set_status(format!(
                    "Recovery snapshot saved {}",
                    snapshot.recovery_path
                ));
                self.state.show_toast("Recovery snapshot saved");
            }
            Err(error) => {
                self.state
                    .set_status(format!("Recovery snapshot failed: {error}"));
                self.state.show_toast("Save failed");
            }
        }
    }

    /// Open a native file dialog to pick a document to open.
    pub(super) fn open_file_dialog(&mut self) {
        use tauri_plugin_dialog::DialogExt;

        let Some(ref handle) = self.app_handle else {
            self.state.pending_file_action = Some("open".into());
            self.state.show_toast("Open dialog is unavailable");
            return;
        };
        let Some(tx) = dialog_sender() else {
            self.state.pending_file_action = Some("open".into());
            self.state.show_toast("Open dialog is unavailable");
            return;
        };

        let tx = tx.clone();
        handle
            .dialog()
            .file()
            .add_filter("Documents", &["docx", "txt", "md", "html", "odt"])
            .set_title("Open Document")
            .pick_file(move |path| {
                if let Some(p) = path {
                    let _ = tx.send(DialogResult::OpenFile(p.to_string()));
                }
            });
    }

    /// Open a native file dialog to pick a save location.
    pub(super) fn save_as_dialog(&mut self) {
        use tauri_plugin_dialog::DialogExt;

        let Some(ref handle) = self.app_handle else {
            self.state.pending_file_action = Some("save_as".into());
            self.state.show_toast("Save As dialog is unavailable");
            return;
        };
        let Some(tx) = dialog_sender() else {
            self.state.pending_file_action = Some("save_as".into());
            self.state.show_toast("Save As dialog is unavailable");
            return;
        };

        let tx = tx.clone();
        handle
            .dialog()
            .file()
            .add_filter("Documents", &["docx"])
            .set_title("Save As")
            .set_file_name("Untitled.docx")
            .save_file(move |path| {
                if let Some(p) = path {
                    let _ = tx.send(DialogResult::SaveAs(p.to_string()));
                }
            });
    }

    /// Open a native file dialog to pick an image to insert.
    pub(super) fn insert_image_dialog(&mut self) {
        use tauri_plugin_dialog::DialogExt;

        let Some(ref handle) = self.app_handle else {
            self.state.pending_file_action = Some("insert_image".into());
            self.state.show_toast("Insert Image dialog is unavailable");
            return;
        };
        let Some(tx) = dialog_sender() else {
            self.state.pending_file_action = Some("insert_image".into());
            self.state.show_toast("Insert Image dialog is unavailable");
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
    pub(super) fn process_dialog_results(&mut self, ctx: &mut EventCtx) {
        let results: Vec<DialogResult> = {
            let Some(ref rx) = self.dialog_rx else {
                return;
            };
            rx.try_iter().collect()
        };
        for result in results {
            match result {
                DialogResult::OpenFile(path) => {
                    self.open_file_from_path(&path);
                }
                DialogResult::SaveAs(path) => {
                    let artifact = self.state.current_artifact().clone();
                    let content = format_io::tdm_to_docs_content(self.state.current_document());
                    match document_service::save_document(artifact, content, Some(path), None) {
                        Ok(saved) => self.state.apply_saved_artifact(saved.artifact),
                        Err(error) => {
                            self.state.set_status(format!("Save As failed: {error}"));
                            self.state.show_toast("Save As failed");
                        }
                    }
                }
                DialogResult::InsertImage(path) => {
                    let result = self.engine().insert_image(
                        tench_document_core::ImageSource::Referenced { path },
                        400.0,
                        300.0,
                    );
                    self.state.apply_edit_result(result);
                }
            }
        }
        ctx.request_paint();
    }
}
