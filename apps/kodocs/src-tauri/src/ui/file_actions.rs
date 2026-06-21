// ---------------------------------------------------------------------------
// File save/open actions
// ---------------------------------------------------------------------------

use tench_document_core::{CursorState, DocumentEngine, ImageSource, OfficeFileFormat};
use tench_office_io::docs::format as format_io;
use tench_ui::prelude::*;

use crate::document_service;
use crate::{dialog_sender, DialogResult};

use super::state::extract_tdm;
use super::KodocsApp;

impl KodocsApp {
    pub(super) fn save_current_document(&mut self) {
        let artifact = self.state.current_artifact().clone();
        let content = format_io::tdm_to_docs_content(self.state.current_document());

        if artifact.path.is_some() {
            match document_service::save_document(artifact, content, None, None) {
                Ok(saved) => self.state.apply_saved_artifact(saved.artifact),
                Err(error) => {
                    self.state.set_status(format!("저장 실패: {error}"));
                    self.state.toast = Some(("저장 실패".into(), 0.0));
                }
            }
            return;
        }

        match document_service::save_recovery_snapshot(artifact, content) {
            Ok(snapshot) => {
                self.state
                    .set_status(format!("복구 스냅샷 저장됨: {}", snapshot.recovery_path));
                self.state.toast = Some(("복구 스냅샷 저장됨".into(), 0.0));
            }
            Err(error) => {
                self.state.set_status(format!("복구 스냅샷 실패: {error}"));
                self.state.toast = Some(("저장 실패".into(), 0.0));
            }
        }
    }

    pub(super) fn open_file_dialog(&self) {
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
            .add_filter("문서", &["docx", "txt", "md", "html", "odt", "hwp", "hwpx"])
            .set_title("문서 열기")
            .pick_file(move |path| {
                if let Some(p) = path {
                    let _ = tx.send(DialogResult::OpenFile(p.to_string()));
                }
            });
    }

    pub(super) fn save_as_dialog(&self) {
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
            .add_filter("문서", &["hwpx", "hwp", "docx", "txt", "html", "odt"])
            .set_title("다른 이름으로 저장")
            .set_file_name("제목 없는 한글 문서.hwpx")
            .save_file(move |path| {
                if let Some(p) = path {
                    let _ = tx.send(DialogResult::SaveAs(p.to_string()));
                }
            });
    }

    pub(super) fn insert_image_dialog(&self) {
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
            .add_filter("그림", &["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg"])
            .set_title("그림 삽입")
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
                    let format = format_for_save_path(&path);
                    let content = format_io::tdm_to_docs_content(self.state.current_document());
                    match document_service::save_document(
                        artifact,
                        content,
                        Some(path),
                        Some(format),
                    ) {
                        Ok(saved) => self.state.apply_saved_artifact(saved.artifact),
                        Err(error) => {
                            self.state
                                .set_status(format!("다른 이름으로 저장 실패: {error}"));
                            self.state.toast = Some(("다른 이름으로 저장 실패".into(), 0.0));
                        }
                    }
                }
                DialogResult::InsertImage(path) => {
                    let result =
                        self.engine()
                            .insert_image(ImageSource::Referenced { path }, 400.0, 300.0);
                    self.state.apply_edit_result(result);
                }
            }
        }
        ctx.request_paint();
    }

    pub(super) fn open_file_from_path(&mut self, path: &str) {
        match document_service::open_document(path.to_string()) {
            Ok(opened) => {
                let document = extract_tdm(&opened.content);
                self.engine = DocumentEngine::new(document.clone());
                self.state
                    .apply_edit_result(tench_document_core::EditResult {
                        document,
                        cursor: CursorState::default(),
                        selection: None,
                        dirty: false,
                    });
                self.state.artifact = opened.artifact;
                self.state.dirty = false;
                self.state.last_saved_text = self.state.document.to_plain_text();
                self.state.status = format!("열림: {path}");
                let new_title = self.state.title().to_string();
                if let Some(tab) = self.state.open_tabs.get_mut(self.state.active_tab_idx) {
                    tab.title = new_title;
                    tab.dirty = false;
                }
            }
            Err(error) => {
                self.state.toast = Some((format!("열기 실패: {error}"), 0.0));
            }
        }
    }

    /// Open HWP import file dialog.
    pub(super) fn open_hwp_import_dialog(&self) {
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
            .add_filter("한글 문서", &["hwp", "hwpx"])
            .set_title("HWP 문서 열기")
            .pick_file(move |path| {
                if let Some(p) = path {
                    let _ = tx.send(DialogResult::OpenFile(p.to_string()));
                }
            });
    }

    /// Export to HWPX format.
    pub(super) fn save_hwpx_export_dialog(&self) {
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
            .add_filter("HWPX 문서", &["hwpx"])
            .set_title("HWPX로 내보내기")
            .set_file_name("문서.hwpx")
            .save_file(move |path| {
                if let Some(p) = path {
                    let _ = tx.send(DialogResult::SaveAs(p.to_string()));
                }
            });
    }
}

fn format_for_save_path(path: &str) -> OfficeFileFormat {
    if path.ends_with(".hwpx") {
        OfficeFileFormat::Hwpx
    } else if path.ends_with(".hwp") {
        OfficeFileFormat::Hwp
    } else if path.ends_with(".txt") {
        OfficeFileFormat::Txt
    } else if path.ends_with(".html") {
        OfficeFileFormat::Html
    } else if path.ends_with(".odt") {
        OfficeFileFormat::Odt
    } else {
        OfficeFileFormat::Docx
    }
}
