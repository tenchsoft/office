use super::*;

// ---------------------------------------------------------------------------
// Track Changes
// ---------------------------------------------------------------------------

impl DocumentEngine {
    pub fn is_track_changes_enabled(&self) -> bool {
        self.track_changes_enabled
    }

    pub fn toggle_track_changes(&mut self) {
        self.track_changes_enabled = !self.track_changes_enabled;
    }

    pub fn get_tracked_changes(&self) -> &[TrackedChange] {
        &self.tracked_changes
    }

    pub fn accept_change(&mut self, change_id: &str) -> EditResult {
        if let Some(pos) = self.tracked_changes.iter().position(|c| c.id == change_id) {
            let change = self.tracked_changes.remove(pos);
            // For inserts: keep the text (it's already in the document)
            // For deletes: actually remove the text
            if change.change_type == ChangeType::Delete {
                self.cursor = CursorState {
                    block_idx: change.block_idx,
                    offset: change.start_offset,
                };
                self.selection = Some(SelectionRange {
                    start: CursorState {
                        block_idx: change.block_idx,
                        offset: change.start_offset,
                    },
                    end: CursorState {
                        block_idx: change.block_idx,
                        offset: change.end_offset,
                    },
                });
                self.delete_selection_inner();
            }
            self.dirty = true;
        }
        self.make_result()
    }

    pub fn reject_change(&mut self, change_id: &str) -> EditResult {
        if let Some(pos) = self.tracked_changes.iter().position(|c| c.id == change_id) {
            let change = self.tracked_changes.remove(pos);
            // For inserts: remove the inserted text
            // For deletes: keep the text (restore it)
            if change.change_type == ChangeType::Insert {
                self.cursor = CursorState {
                    block_idx: change.block_idx,
                    offset: change.start_offset,
                };
                self.selection = Some(SelectionRange {
                    start: CursorState {
                        block_idx: change.block_idx,
                        offset: change.start_offset,
                    },
                    end: CursorState {
                        block_idx: change.block_idx,
                        offset: change.end_offset,
                    },
                });
                self.delete_selection_inner();
            }
            self.dirty = true;
        }
        self.make_result()
    }

    pub fn accept_all_changes(&mut self) -> EditResult {
        let ids: Vec<String> = self.tracked_changes.iter().map(|c| c.id.clone()).collect();
        for id in ids {
            self.accept_change(&id);
        }
        self.make_result()
    }

    pub fn reject_all_changes(&mut self) -> EditResult {
        let ids: Vec<String> = self.tracked_changes.iter().map(|c| c.id.clone()).collect();
        for id in ids {
            self.reject_change(&id);
        }
        self.make_result()
    }

    /// Record a tracked change (called internally when track changes is enabled).
    /// To be wired into insert_text/backspace/delete once track-changes UX is finalized.
    #[allow(dead_code)]
    fn record_change(
        &mut self,
        change_type: ChangeType,
        text: String,
        block_idx: usize,
        start_offset: usize,
        end_offset: usize,
    ) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let id = format!("ch_{}", self.tracked_changes.len());
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.tracked_changes.push(TrackedChange {
            id,
            change_type,
            author: "Current User".to_string(),
            timestamp,
            text,
            block_idx,
            start_offset,
            end_offset,
        });
    }
}
