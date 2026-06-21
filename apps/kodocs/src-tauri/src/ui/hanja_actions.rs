// ---------------------------------------------------------------------------
// Hanja conversion actions
// ---------------------------------------------------------------------------

use super::document_text::extract_block_text;
use super::hanja::lookup_hanja;
use super::hanja_popup::extract_korean_word_at;
use super::{state, KodocsApp};

impl KodocsApp {
    pub(super) fn perform_hanja_conversion(&mut self) {
        let doc = self.state.current_document();
        let cursor = self.state.cursor();
        let block_idx = cursor.block_idx;

        if block_idx >= doc.content.len() {
            self.state.toast = Some(("한자 변환: 커서 위치에 텍스트가 없습니다".into(), 0.0));
            return;
        }

        let text = extract_block_text(&doc.content[block_idx]);
        if text.is_empty() {
            self.state.toast = Some(("한자 변환: 텍스트를 입력하세요".into(), 0.0));
            return;
        }

        let offset = cursor.offset.min(text.len());
        let word = extract_korean_word_at(&text, offset);

        if word.is_empty() {
            self.state.toast = Some(("한자 변환: 한글 단어 위에 커서를 두세요".into(), 0.0));
            return;
        }

        let candidates = lookup_hanja(&word);
        if candidates.is_empty() {
            self.state.toast = Some((
                format!("한자 변환: '{}'에 대한 한자를 찾을 수 없습니다", word),
                0.0,
            ));
            return;
        }

        self.state.hanja_popup = Some(state::HanjaPopupState {
            source_word: word,
            candidates: candidates
                .iter()
                .map(|e| format!("{} ({})", e.hanja, e.meaning))
                .collect(),
            selected_idx: 0,
        });
    }
}
