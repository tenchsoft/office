//! Session manager for open document editing sessions.
//!
//! Each session holds a [`DocumentEngine`] and the associated
//! [`OfficeArtifact`] metadata. The manager maps string IDs to sessions and
//! supports concurrent lookups from Tauri commands.

use std::collections::HashMap;

use tench_document_core::{DocumentEngine, OfficeArtifact, TenchDocument};

/// A single open document session.
#[allow(dead_code)]
pub struct DocumentSession {
    pub engine: DocumentEngine,
    pub artifact: OfficeArtifact,
}

/// Manages all open document sessions.
#[allow(dead_code)]
pub struct DocumentSessionManager {
    sessions: HashMap<String, DocumentSession>,
    active_doc_id: Option<String>,
}

impl DocumentSessionManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        DocumentSessionManager {
            sessions: HashMap::new(),
            active_doc_id: None,
        }
    }

    /// Create a new session from an artifact and a TDM document.
    /// Returns the generated session ID.
    #[allow(dead_code)]
    pub fn create_session(&mut self, artifact: OfficeArtifact, document: TenchDocument) -> String {
        let doc_id = artifact.id.clone();
        let engine = DocumentEngine::new(document);
        self.sessions
            .insert(doc_id.clone(), DocumentSession { engine, artifact });
        self.active_doc_id = Some(doc_id.clone());
        doc_id
    }

    /// Get a mutable reference to the engine for the given session.
    pub fn get_engine(&mut self, doc_id: &str) -> Option<&mut DocumentEngine> {
        self.sessions.get_mut(doc_id).map(|s| &mut s.engine)
    }

    /// Get a reference to the artifact for the given session.
    #[allow(dead_code)]
    pub fn get_artifact(&self, doc_id: &str) -> Option<&OfficeArtifact> {
        self.sessions.get(doc_id).map(|s| &s.artifact)
    }

    /// Update the artifact for a session (e.g. after save).
    #[allow(dead_code)]
    pub fn update_artifact(&mut self, doc_id: &str, artifact: OfficeArtifact) {
        if let Some(session) = self.sessions.get_mut(doc_id) {
            session.artifact = artifact;
        }
    }

    /// Remove a session.
    #[allow(dead_code)]
    pub fn remove_session(&mut self, doc_id: &str) {
        self.sessions.remove(doc_id);
        if self.active_doc_id.as_deref() == Some(doc_id) {
            self.active_doc_id = None;
        }
    }

    /// Get the active document ID.
    #[allow(dead_code)]
    pub fn active_doc_id(&self) -> Option<&str> {
        self.active_doc_id.as_deref()
    }

    /// Set the active document ID.
    #[allow(dead_code)]
    pub fn set_active(&mut self, doc_id: &str) {
        if self.sessions.contains_key(doc_id) {
            self.active_doc_id = Some(doc_id.to_string());
        }
    }
}

impl Default for DocumentSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use tench_document_core::{OfficeFileFormat, OfficeProductKind};

    use super::*;

    fn test_artifact(id: &str) -> OfficeArtifact {
        OfficeArtifact {
            id: id.to_string(),
            title: "Test".to_string(),
            product: OfficeProductKind::Docs,
            format: OfficeFileFormat::Docx,
            path: None,
            schema_version: "1".to_string(),
            created_at: None,
            updated_at: None,
            dirty: false,
            tags: vec![],
            assets: vec![],
        }
    }

    #[test]
    fn create_and_lookup_session() {
        let mut mgr = DocumentSessionManager::new();
        let artifact = test_artifact("doc_1");
        let doc = TenchDocument::plain_text("Hello");

        let id = mgr.create_session(artifact, doc);
        assert_eq!(id, "doc_1");
        assert_eq!(mgr.active_doc_id(), Some("doc_1"));

        let engine = mgr.get_engine("doc_1").expect("engine");
        assert_eq!(engine.get_document().to_plain_text(), "Hello");
    }

    #[test]
    fn remove_session_clears_active() {
        let mut mgr = DocumentSessionManager::new();
        let artifact = test_artifact("doc_1");
        mgr.create_session(artifact, TenchDocument::new("Test"));

        mgr.remove_session("doc_1");
        assert!(mgr.get_engine("doc_1").is_none());
        assert_eq!(mgr.active_doc_id(), None);
    }

    #[test]
    fn update_artifact() {
        let mut mgr = DocumentSessionManager::new();
        let artifact = test_artifact("doc_1");
        mgr.create_session(artifact, TenchDocument::new("Test"));

        let mut updated = test_artifact("doc_1");
        updated.title = "Updated".to_string();
        mgr.update_artifact("doc_1", updated);

        assert_eq!(mgr.get_artifact("doc_1").unwrap().title, "Updated");
    }
}
