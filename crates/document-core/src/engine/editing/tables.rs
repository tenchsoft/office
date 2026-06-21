use super::*;

impl DocumentEngine {
    pub fn insert_table(&mut self, rows: usize, cols: usize) -> EditResult {
        self.push_undo();
        self.ensure_block();
        use crate::tdm::{TableCell, TableRow};

        let row = TableRow {
            cells: (0..cols).map(|_| TableCell::default()).collect(),
        };
        let table = BlockNode::Table {
            rows: (0..rows).map(|_| row.clone()).collect(),
        };
        self.document
            .content
            .insert(self.cursor.block_idx + 1, table);
        self.cursor.block_idx += 1;
        self.cursor.offset = 0;
        self.dirty = true;
        self.make_result()
    }

    /// Insert a row into the table at the cursor's block.
    /// If `above` is true, insert before the current row; otherwise after.
    pub fn insert_table_row(&mut self, above: bool) -> EditResult {
        self.push_undo();
        use crate::tdm::{TableCell, TableRow};

        let block_idx = self.cursor.block_idx;
        if let Some(BlockNode::Table { rows }) = self.document.content.get_mut(block_idx) {
            let col_count = rows.first().map_or(1, |r| r.cells.len());
            let new_row = TableRow {
                cells: (0..col_count).map(|_| TableCell::default()).collect(),
            };
            // Determine current row from cursor offset (rough heuristic).
            let current_row = self.cursor.offset.min(rows.len().saturating_sub(1));
            let insert_at = if above {
                current_row
            } else {
                (current_row + 1).min(rows.len())
            };
            rows.insert(insert_at, new_row);
            self.dirty = true;
        }
        self.make_result()
    }

    /// Insert a column into the table at the cursor's block.
    /// If `left` is true, insert before the current column; otherwise after.
    pub fn insert_table_column(&mut self, left: bool) -> EditResult {
        self.push_undo();
        use crate::tdm::TableCell;

        let block_idx = self.cursor.block_idx;
        if let Some(BlockNode::Table { rows }) = self.document.content.get_mut(block_idx) {
            let col_count = rows.first().map_or(1, |r| r.cells.len());
            // Determine current column from cursor offset (rough heuristic).
            let current_col = if col_count > 0 {
                self.cursor.offset % col_count
            } else {
                0
            };
            let insert_at = if left {
                current_col
            } else {
                (current_col + 1).min(col_count)
            };
            for row in rows.iter_mut() {
                row.cells.insert(insert_at, TableCell::default());
            }
            self.dirty = true;
        }
        self.make_result()
    }

    /// Delete the current row from the table at the cursor's block.
    pub fn delete_table_row(&mut self) -> EditResult {
        self.push_undo();
        let block_idx = self.cursor.block_idx;
        if let Some(BlockNode::Table { rows }) = self.document.content.get_mut(block_idx) {
            if rows.len() > 1 {
                let current_row = self.cursor.offset.min(rows.len().saturating_sub(1));
                rows.remove(current_row);
            }
            self.dirty = true;
        }
        self.make_result()
    }

    /// Delete the current column from the table at the cursor's block.
    pub fn delete_table_column(&mut self) -> EditResult {
        self.push_undo();
        let block_idx = self.cursor.block_idx;
        if let Some(BlockNode::Table { rows }) = self.document.content.get_mut(block_idx) {
            let col_count = rows.first().map_or(0, |r| r.cells.len());
            if col_count > 1 {
                let current_col = self.cursor.offset % col_count;
                for row in rows.iter_mut() {
                    if current_col < row.cells.len() {
                        row.cells.remove(current_col);
                    }
                }
            }
            self.dirty = true;
        }
        self.make_result()
    }

    /// Delete the entire table at the cursor's block.
    pub fn delete_table(&mut self) -> EditResult {
        self.push_undo();
        let block_idx = self.cursor.block_idx;
        if block_idx < self.document.content.len()
            && matches!(self.document.content[block_idx], BlockNode::Table { .. })
        {
            self.document.content.remove(block_idx);
            if self.cursor.block_idx >= self.document.content.len() {
                self.cursor.block_idx = self.document.content.len().saturating_sub(1);
            }
            self.cursor.offset = 0;
            self.ensure_block();
        }
        self.dirty = true;
        self.make_result()
    }
}
