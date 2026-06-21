use super::*;
use std::time::Instant;

use tench_document_core::OfficeArtifact;

impl SheetsState {
    pub fn apply_saved_artifact(&mut self, artifact: OfficeArtifact) {
        self.artifact = artifact;
        self.artifact.dirty = false;
        self.last_saved_csv = grid_to_csv(&self.grid);
        self.status = if let Some(path) = &self.artifact.path {
            format!("Saved {path}")
        } else {
            format!("Saved {}", self.artifact.title)
        };
        self.toast = Some(("Workbook saved".into(), Instant::now()));
        self.sync_content_from_grid();
    }
}
