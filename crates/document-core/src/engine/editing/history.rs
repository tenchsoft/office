use super::*;

impl DocumentEngine {
    pub fn undo(&mut self) -> EditResult {
        let Some(snapshot) = self.undo_stack.pop() else {
            return self.make_result();
        };
        let current = UndoSnapshot {
            content: self.document.content.clone(),
            cursor: self.cursor.clone(),
            selection: self.selection.clone(),
        };
        self.redo_stack.push(current);
        self.document.content = snapshot.content;
        self.cursor = snapshot.cursor;
        self.selection = snapshot.selection;
        self.ensure_block();
        self.dirty = true;
        self.make_result()
    }

    pub fn redo(&mut self) -> EditResult {
        let Some(snapshot) = self.redo_stack.pop() else {
            return self.make_result();
        };
        let current = UndoSnapshot {
            content: self.document.content.clone(),
            cursor: self.cursor.clone(),
            selection: self.selection.clone(),
        };
        self.undo_stack.push(current);
        self.document.content = snapshot.content;
        self.cursor = snapshot.cursor;
        self.selection = snapshot.selection;
        self.ensure_block();
        self.dirty = true;
        self.make_result()
    }
}
