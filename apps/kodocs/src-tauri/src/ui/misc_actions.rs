// ---------------------------------------------------------------------------
// Miscellaneous action helpers
// ---------------------------------------------------------------------------

use tench_document_core::MarkType;
use tench_ui::prelude::*;

use super::state::{self, LinkModalState};
use super::KodocsApp;

impl KodocsApp {
    pub(super) fn open_file(&mut self, path: &str) {
        self.open_file_from_path(path);
    }

    pub(super) fn hit_test_image_resize_handle(
        &self,
        _x: f64,
        _y: f64,
    ) -> Option<state::ResizeHandle> {
        None
    }

    pub(super) fn handle_context_menu_item(
        &mut self,
        item: &str,
        _menu_x: f64,
        _menu_y: f64,
        ctx: &mut EventCtx,
    ) {
        match item {
            "자르기" => {
                self.cut_selection();
            }
            "복사" => {
                self.copy_selection();
            }
            "붙여넣기" => {
                self.paste_clipboard();
            }
            "코멘트 추가" => {
                self.state.toast = Some(("코멘트 기능 준비 중".into(), 0.0));
            }
            "링크 삽입" => {
                self.state.link_modal = Some(LinkModalState {
                    url: String::new(),
                    display_text: String::new(),
                    cursor_pos: 0,
                });
            }
            "서식 지우기" => {
                let _ = self.engine().toggle_mark(MarkType::Bold);
                let _ = self.engine().toggle_mark(MarkType::Italic);
                let _ = self.engine().toggle_mark(MarkType::Underline);
            }
            "이미지 크기 조정" => {
                self.state.toast = Some(("이미지 크기 조절: 드래그하세요".into(), 0.0));
            }
            "행 삽입" | "열 삽입" | "셋 병합" | "셋 분할" => {
                self.state.toast = Some(("표 기능 준비 중".into(), 0.0));
            }
            "탭 닫기" if self.state.open_tabs.len() > 1 => {
                let idx = self.state.active_tab_idx;
                self.close_tab(idx);
            }
            _ => {}
        }
        let _ = ctx;
    }

    pub(super) fn compute_toolbar_tooltip(&self, x: f64, _y: f64) -> Option<String> {
        let tooltips = [
            (8.0, 36.0, "저장"),
            (44.0, 72.0, "실행 취소"),
            (80.0, 108.0, "다시 실행"),
            (116.0, 144.0, "굵게 (Ctrl+B)"),
            (152.0, 180.0, "기울임 (Ctrl+I)"),
            (188.0, 216.0, "밑줄 (Ctrl+U)"),
            (224.0, 252.0, "취소선"),
            (260.0, 288.0, "글꼴"),
            (296.0, 324.0, "글꼴 크기"),
            (332.0, 360.0, "글꼴 색상"),
            (368.0, 396.0, "배경색"),
            (404.0, 432.0, "왼쪽 정렬"),
            (440.0, 468.0, "가운데 정렬"),
            (476.0, 504.0, "오른쪽 정렬"),
            (512.0, 540.0, "양쪽 정렬"),
            (548.0, 576.0, "글머리 기호"),
            (584.0, 612.0, "번호 매기기"),
            (620.0, 648.0, "들여쓰기 감소"),
            (656.0, 684.0, "들여쓰기 증가"),
            (692.0, 720.0, "그림 삽입"),
            (728.0, 756.0, "표 삽입"),
            (764.0, 792.0, "링크 (Ctrl+K)"),
            (800.0, 828.0, "수식"),
        ];
        for (x0, x1, tip) in tooltips {
            if x >= x0 && x < x1 {
                return Some(tip.to_string());
            }
        }
        None
    }
}
