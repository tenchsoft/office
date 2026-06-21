use super::*;

// ---------------------------------------------------------------------------
// Comments
// ---------------------------------------------------------------------------

impl DocumentEngine {
    pub fn add_comment(&mut self, text: &str, range: CommentRange) -> Comment {
        let id = format!("cmt_{}", self.comments.len());
        let comment = Comment {
            id,
            author: "Current User".to_string(),
            text: text.to_string(),
            resolved: false,
            range,
        };
        self.comments.push(comment.clone());
        self.dirty = true;
        comment
    }

    pub fn edit_comment(&mut self, id: &str, text: &str) {
        if let Some(c) = self.comments.iter_mut().find(|c| c.id == id) {
            c.text = text.to_string();
            self.dirty = true;
        }
    }

    pub fn delete_comment(&mut self, id: &str) {
        self.comments.retain(|c| c.id != id);
        self.dirty = true;
    }

    pub fn resolve_comment(&mut self, id: &str) {
        if let Some(c) = self.comments.iter_mut().find(|c| c.id == id) {
            c.resolved = true;
            self.dirty = true;
        }
    }

    pub fn get_comments(&self) -> &[Comment] {
        &self.comments
    }
}
