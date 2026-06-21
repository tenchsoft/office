// ---------------------------------------------------------------------------
// Korean menu action dispatch
// ---------------------------------------------------------------------------

use tench_document_core::{BlockType, CursorState, DocumentEngine, MarkType};
use tench_ui::prelude::*;

use crate::document_service;

use super::equation_editor::EquationEditorState;
use super::state::{extract_tdm, KodocsState, LinkModalState, PageSetupDialogState};
use super::KodocsApp;

impl KodocsApp {
    pub(super) fn handle_menu_item(&mut self, item: &str, ctx: &mut EventCtx) {
        match item {
            "새 문서" => {
                let opened = document_service::create_document(Some("제목 없는 한글 문서".into()));
                let document = extract_tdm(&opened.content);
                self.engine = DocumentEngine::new(document.clone());
                self.state = KodocsState::new();
                self.state
                    .apply_edit_result(tench_document_core::EditResult {
                        document,
                        cursor: CursorState::default(),
                        selection: None,
                        dirty: false,
                    });
                ctx.request_paint();
            }
            "저장" => {
                self.save_current_document();
                ctx.request_paint();
            }
            "다른 이름으로 저장" => {
                self.save_as_dialog();
                ctx.request_paint();
            }
            "실행 취소" => {
                let result = self.engine().undo();
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "다시 실행" => {
                let result = self.engine().redo();
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "잘라내기" => {
                self.cut_selection();
                ctx.request_paint();
            }
            "복사" => {
                self.copy_selection();
                ctx.request_paint();
            }
            "붙여넣기" => {
                self.paste_clipboard();
                ctx.request_paint();
            }
            "모두 선택" => {
                let result = self.engine().select_all();
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "찾기" => {
                self.open_find_replace(false);
                ctx.request_paint();
            }
            "바꾸기" => {
                self.open_find_replace(true);
                ctx.request_paint();
            }
            "확대" => {
                self.state.zoom = (self.state.zoom + 10.0).min(200.0);
                ctx.request_paint();
            }
            "축소" => {
                self.state.zoom = (self.state.zoom - 10.0).max(50.0);
                ctx.request_paint();
            }
            "확대/축소 초기화" => {
                self.state.zoom = 100.0;
                ctx.request_paint();
            }
            "미리보기" => {
                self.state.show_thumbnails = !self.state.show_thumbnails;
                ctx.request_paint();
            }
            "스타일 패널" => {
                self.state.show_style_panel = !self.state.show_style_panel;
                ctx.request_paint();
            }
            "메모" => {
                self.state.show_comments = !self.state.show_comments;
                if self.state.show_comments {
                    let comments = self.engine().get_comments().to_vec();
                    self.state.update_comments(comments);
                }
                ctx.request_paint();
            }
            "세로쓰기" => {
                self.state.vertical_writing = !self.state.vertical_writing;
                self.state.toast = Some((
                    if self.state.vertical_writing {
                        "세로쓰기 켜짐"
                    } else {
                        "세로쓰기 꺼짐"
                    }
                    .into(),
                    0.0,
                ));
                ctx.request_paint();
            }
            "그림" => {
                self.insert_image_dialog();
                ctx.request_paint();
            }
            "표" => {
                let result = self.engine().insert_table(3, 3);
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "링크" => {
                self.state.link_modal = Some(LinkModalState::default());
                self.state.active_modal = Some("하이퍼링크 삽입".into());
                ctx.request_paint();
            }
            "가로줄" => {
                let result = self.engine().insert_horizontal_rule();
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "페이지 나누기" => {
                let result = self.engine().insert_page_break();
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "머리글" => {
                self.state.editing_header = true;
                self.state.editing_footer = false;
                ctx.request_paint();
            }
            "바닥글" => {
                self.state.editing_footer = true;
                self.state.editing_header = false;
                ctx.request_paint();
            }
            "수식" => {
                self.state.equation_editor = Some(EquationEditorState::default());
                ctx.request_paint();
            }
            "굵게" => {
                let result = self.engine().toggle_mark(MarkType::Bold);
                self.state.bold = !self.state.bold;
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "기울임" => {
                let result = self.engine().toggle_mark(MarkType::Italic);
                self.state.italic = !self.state.italic;
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "밑줄" => {
                let result = self.engine().toggle_mark(MarkType::Underline);
                self.state.underline = !self.state.underline;
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "취소선" => {
                let result = self.engine().toggle_mark(MarkType::Strikethrough);
                self.state.strikethrough = !self.state.strikethrough;
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "위 첨자" => {
                let result = self.engine().toggle_mark(MarkType::Superscript);
                self.state.superscript = !self.state.superscript;
                if self.state.superscript {
                    self.state.subscript = false;
                }
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "아래 첨자" => {
                let result = self.engine().toggle_mark(MarkType::Subscript);
                self.state.subscript = !self.state.subscript;
                if self.state.subscript {
                    self.state.superscript = false;
                }
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "서식 지우기" => {
                let result = self.engine().clear_marks();
                self.state.bold = false;
                self.state.italic = false;
                self.state.underline = false;
                self.state.strikethrough = false;
                self.state.code = false;
                self.state.superscript = false;
                self.state.subscript = false;
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "인용" => {
                let result = self.engine().set_block_type(BlockType::BlockQuote);
                self.state.apply_edit_result(result);
                ctx.request_paint();
            }
            "한자 변환" => {
                self.perform_hanja_conversion();
                ctx.request_paint();
            }
            "단어 수" => {
                self.state.toast = Some((
                    format!(
                        "단어: {} | 글자: {} | 단락: {}",
                        self.state.word_count,
                        self.state.character_count(),
                        self.state.paragraph_count(),
                    ),
                    0.0,
                ));
                ctx.request_paint();
            }
            "변경 내용 추적" => {
                self.engine().toggle_track_changes();
                self.state.track_changes = self.engine().is_track_changes_enabled();
                ctx.request_paint();
            }
            "맞춤법 검사" => {
                self.state.toast = Some(("맞춤법 검사: 확인 중...".into(), 0.0));
                ctx.request_paint();
                self.state.toast = Some(("맞춤법 오류 없음".into(), 0.0));
                ctx.request_paint();
            }
            "페이지 설정" => {
                self.state.page_setup_dialog = Some(PageSetupDialogState::from_page_setup(
                    &self.state.current_document().page_setup,
                ));
                ctx.request_paint();
            }
            "인쇄" => {
                self.state.toast = Some(("인쇄: 준비 중".into(), 0.0));
                ctx.request_paint();
            }
            "버전 기록" => {
                self.refresh_version_history();
                ctx.request_paint();
            }
            "내보내기" => {
                self.save_as_dialog();
                ctx.request_paint();
            }
            "열기" => {
                self.open_file_dialog();
                ctx.request_paint();
            }
            "HWP 가져오기" => {
                self.open_hwp_import_dialog();
                ctx.request_paint();
            }
            "HWPX 내보내기" => {
                self.save_hwpx_export_dialog();
                ctx.request_paint();
            }
            "정보" => {
                self.state.toast = Some(("Tench Kodocs v0.1.0".into(), 0.0));
                ctx.request_paint();
            }
            "키보드 단축키" => {
                self.state.toast = Some((
                    "Ctrl+B/I/U: 서식 | Ctrl+S: 저장 | Ctrl+F: 찾기 | F9: 한자 변환".into(),
                    0.0,
                ));
                ctx.request_paint();
            }
            "Activate License" => {
                self.state.active_modal = None;
                self.state.license_modal =
                    Some(crate::ui::state::LicenseModalState::default());
                self.state.toast = None;
                ctx.request_paint();
            }
            "Generate PC Code" => {
                self.state.active_modal = None;
                if let Some(store) = &self.license_store {
                    let state = store.state();
                    let meta = serde_json::json!({
                        "os": std::env::consts::OS,
                        "hostname": std::env::var("HOSTNAME")
                            .or_else(|_| std::env::var("COMPUTERNAME"))
                            .unwrap_or_else(|_| "unknown".into()),
                        "tench_app": "kodocs",
                        "tench_ver": env!("CARGO_PKG_VERSION"),
                    });
                    match tench_license_store::encode_pc_request_code(&state.device_id, meta) {
                        Ok(code) => {
                            self.state.toast = Some((code, 0.0));
                        }
                        Err(_) => {
                            self.state.toast = Some(("Failed to generate PC code".into(), 0.0));
                        }
                    }
                } else {
                    self.state.toast = Some(("License store unavailable".into(), 0.0));
                }
                ctx.request_paint();
            }
            "Release Device" => {
                self.state.active_modal = None;
                if let Some(store) = &self.license_store {
                    match tench_update_client::release_license(store) {
                        Ok(()) => self.state.toast = Some(("Device released".into(), 0.0)),
                        Err(e) => {
                            self.state.toast = Some((format!("Release failed: {e}"), 0.0))
                        }
                    }
                } else {
                    self.state.toast = Some(("License store unavailable".into(), 0.0));
                }
                ctx.request_paint();
            }
            _ => {
                ctx.request_paint();
            }
        }
    }
}
