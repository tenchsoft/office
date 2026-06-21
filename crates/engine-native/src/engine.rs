use std::path::PathBuf;

use crate::dirs::default_model_dirs;
use crate::discovery::discover_model_artifacts;
use crate::{NativeEngineError, NativeModelArtifact};

#[derive(Clone, Debug)]
pub struct NativeEngine {
    model_dirs: Vec<PathBuf>,
}

impl Default for NativeEngine {
    fn default() -> Self {
        Self::new(default_model_dirs())
    }
}

impl NativeEngine {
    pub fn new(model_dirs: Vec<PathBuf>) -> Self {
        Self { model_dirs }
    }

    pub fn model_dirs(&self) -> &[PathBuf] {
        &self.model_dirs
    }

    pub fn discover_models(&self) -> Vec<NativeModelArtifact> {
        discover_model_artifacts(&self.model_dirs)
    }

    pub fn load_metadata(&self, model_id: &str) -> Result<NativeModelArtifact, NativeEngineError> {
        self.discover_models()
            .into_iter()
            .find(|artifact| artifact.id == model_id)
            .ok_or_else(|| NativeEngineError::InvalidModel(format!("Model not found: {model_id}")))
    }
}
