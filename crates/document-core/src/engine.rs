//! Document editing engine built on top of the Tench Document Model (TDM).
//!
//! The engine owns a [`TenchDocument`] and exposes high-level editing
//! operations (insert text, backspace, toggle marks, undo/redo, etc.).
//! Every mutation returns an [`EditResult`] snapshot that can be forwarded
//! to the UI layer.

use serde::{Deserialize, Serialize};

use crate::tdm::{
    Alignment, BlockNode, HeadersFooters, ImageSource, InlineNode, ListItem, Margins, Marks,
    Orientation, PageSetup, PaperSize, ParagraphAttrs, TaskItem, TenchDocument,
};

// ---------------------------------------------------------------------------
// Search types
// ---------------------------------------------------------------------------

/// A single search match within the document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchMatch {
    pub block_idx: usize,
    pub start_offset: usize,
    pub end_offset: usize,
}

// ---------------------------------------------------------------------------
// Clipboard types
// ---------------------------------------------------------------------------

/// Content that has been cut or copied, stored in multiple formats.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipboardContent {
    pub tdm_nodes: Vec<BlockNode>,
    pub html: String,
    pub plain_text: String,
}

// ---------------------------------------------------------------------------
// Track Changes types
// ---------------------------------------------------------------------------

/// The type of tracked change.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Insert,
    Delete,
}

/// A single tracked change annotation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackedChange {
    pub id: String,
    pub change_type: ChangeType,
    pub author: String,
    pub timestamp: u64,
    pub text: String,
    pub block_idx: usize,
    pub start_offset: usize,
    pub end_offset: usize,
}

// ---------------------------------------------------------------------------
// Comment types
// ---------------------------------------------------------------------------

/// Range within a document that a comment applies to.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommentRange {
    pub block_idx: usize,
    pub start_offset: usize,
    pub end_offset: usize,
}

/// A comment annotation on a document range.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author: String,
    pub text: String,
    pub resolved: bool,
    pub range: CommentRange,
}

const UNDO_LIMIT: usize = 200;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Cursor position within a document, expressed as block index + character
/// offset inside that block's text content.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CursorState {
    /// Index into [`TenchDocument::content`].
    pub block_idx: usize,
    /// Byte offset within the flattened text of the block.
    pub offset: usize,
}

/// A contiguous selection range between two cursor positions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelectionRange {
    pub start: CursorState,
    pub end: CursorState,
}

/// Snapshot used for undo/redo.
#[derive(Clone, Debug)]
struct UndoSnapshot {
    content: Vec<BlockNode>,
    cursor: CursorState,
    selection: Option<SelectionRange>,
}

/// Kinds of inline formatting marks that can be toggled.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkType {
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Superscript,
    Subscript,
    Code,
}

/// Block-level types that can be applied via `set_block_type`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Paragraph,
    Heading(u8),
    BlockQuote,
    CodeBlock,
    BulletList,
    OrderedList,
    TaskList,
    Footnote,
}

/// Cursor movement directions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveDirection {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    WordLeft,
    WordRight,
    DocStart,
    DocEnd,
}

/// Placeholder for layout information needed for click-to-cursor conversion.
///
/// Will be populated when Parley integration is complete. For now it is an
/// empty struct so the engine compiles and the API is ready.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LayoutInfo {}

/// Result returned by every engine mutation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditResult {
    pub document: TenchDocument,
    pub cursor: CursorState,
    pub selection: Option<SelectionRange>,
    pub dirty: bool,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// The core editing engine for a single document session.
pub struct DocumentEngine {
    document: TenchDocument,
    cursor: CursorState,
    selection: Option<SelectionRange>,
    undo_stack: Vec<UndoSnapshot>,
    redo_stack: Vec<UndoSnapshot>,
    active_marks: Marks,
    dirty: bool,
    // Search state
    search_matches: Vec<SearchMatch>,
    current_match_idx: Option<usize>,
    // Track changes
    track_changes_enabled: bool,
    tracked_changes: Vec<TrackedChange>,
    // Comments
    comments: Vec<Comment>,
    // Clipboard (last copied content for internal use)
    clipboard: Option<ClipboardContent>,
}

impl DocumentEngine {
    // ----- construction -----

    pub fn new(document: TenchDocument) -> Self {
        DocumentEngine {
            document,
            cursor: CursorState::default(),
            selection: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            active_marks: Marks::default(),
            dirty: false,
            search_matches: Vec::new(),
            current_match_idx: None,
            track_changes_enabled: false,
            tracked_changes: Vec::new(),
            comments: Vec::new(),
            clipboard: None,
        }
    }

    // ----- read access -----

    pub fn get_document(&self) -> &TenchDocument {
        &self.document
    }

    pub fn get_cursor(&self) -> &CursorState {
        &self.cursor
    }

    pub fn get_selection(&self) -> &Option<SelectionRange> {
        &self.selection
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Returns `true` if there are snapshots on the undo stack.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if there are snapshots on the redo stack.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }

    // ----- helpers -----

    fn push_undo(&mut self) {
        let snapshot = UndoSnapshot {
            content: self.document.content.clone(),
            cursor: self.cursor.clone(),
            selection: self.selection.clone(),
        };
        if self.undo_stack.len() >= UNDO_LIMIT {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(snapshot);
        self.redo_stack.clear();
    }

    fn make_result(&self) -> EditResult {
        EditResult {
            document: self.document.clone(),
            cursor: self.cursor.clone(),
            selection: self.selection.clone(),
            dirty: self.dirty,
        }
    }

    /// Ensure the document has at least one block. If empty, push an empty
    /// paragraph and clamp the cursor.
    fn ensure_block(&mut self) {
        if self.document.content.is_empty() {
            self.document.content.push(BlockNode::Paragraph {
                content: Vec::new(),
                attrs: ParagraphAttrs::default(),
            });
            self.cursor.block_idx = 0;
            self.cursor.offset = 0;
        }
        self.cursor.block_idx = self
            .cursor
            .block_idx
            .min(self.document.content.len().saturating_sub(1));
    }

    /// Return the flattened text of the block at `block_idx`.
    fn block_text(&self, block_idx: usize) -> String {
        helpers::block_text_of(&self.document.content, block_idx)
    }
}

mod clipboard;
mod comments;
mod editing;
mod helpers;
mod search;
mod tracking;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;
