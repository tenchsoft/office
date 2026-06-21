use std::path::PathBuf;

use serde_json::{json, Value};
use tench_engine_native::{NativeEngine, NativeModelArtifact};
use tench_shared_types::{
    ChatCompletionParams, ChatCompletionResult, EngineError, EngineErrorType, EngineEvent,
    LocalRuntimeDescriptor, LocalRuntimeKind, LocalRuntimeStatus, ModelDescriptor, ModelLocation,
    ProviderDescriptor, ProviderStatus,
};

use crate::{detect_hardware_profile, EngineEventStream, EngineProvider};

#[derive(Clone, Debug)]
pub struct NativeProvider {
    engine: NativeEngine,
    hardware: tench_shared_types::HardwareProfile,
}

impl Default for NativeProvider {
    fn default() -> Self {
        Self::new(tench_engine_native::default_model_dirs())
    }
}

impl NativeProvider {
    pub fn new(model_dirs: Vec<PathBuf>) -> Self {
        Self {
            engine: NativeEngine::new(model_dirs),
            hardware: detect_hardware_profile(),
        }
    }

    pub fn runtime(&self) -> LocalRuntimeDescriptor {
        LocalRuntimeDescriptor {
            kind: LocalRuntimeKind::Native,
            status: LocalRuntimeStatus::NotReady,
            endpoint: None,
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            hardware: Some(self.hardware.clone()),
        }
    }

    pub fn model_dirs(&self) -> &[PathBuf] {
        self.engine.model_dirs()
    }

    fn resolve_model(
        &self,
        request_id: &str,
        requested_model: &str,
        capability: &str,
    ) -> Result<NativeModelArtifact, EngineError> {
        let models = self.engine.discover_models();

        if requested_model.starts_with("tench/") {
            return models
                .iter()
                .find(|model| model.capability() == capability)
                .or_else(|| models.first())
                .cloned()
                .ok_or_else(|| native_model_not_found(request_id, requested_model));
        }

        models
            .into_iter()
            .find(|model| model.id == requested_model)
            .ok_or_else(|| native_model_not_found(request_id, requested_model))
    }
}

impl EngineProvider for NativeProvider {
    fn provider(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: "native".to_string(),
            display_name: "Tench Native Runtime".to_string(),
            status: ProviderStatus::NotReady,
            location: ModelLocation::Local,
        }
    }

    fn list_models(&self) -> Vec<ModelDescriptor> {
        self.engine
            .discover_models()
            .into_iter()
            .map(native_model_descriptor)
            .collect()
    }

    fn chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> Result<ChatCompletionResult, EngineError> {
        let model = self.resolve_model(request_id, &params.model, "chat")?;
        Err(native_runtime_not_ready_for_model(
            request_id,
            &model,
            "Native runtime is unavailable until a local executor is configured",
        ))
    }

    fn stream_chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> EngineEventStream {
        let model = match self.resolve_model(request_id, &params.model, "chat") {
            Ok(model) => model,
            Err(error) => return vec![EngineEvent::Error { error }],
        };

        vec![EngineEvent::Error {
            error: native_runtime_not_ready_for_model(
                request_id,
                &model,
                "Native runtime is unavailable until a local executor is configured",
            ),
        }]
    }

    fn text_completion(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
        let model = params
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("local/native/default");
        let model = self.resolve_model(request_id, model, "chat")?;
        Err(native_runtime_not_ready_for_model(
            request_id,
            &model,
            "Native text generation is unavailable until a local executor is configured",
        ))
    }

    fn embeddings(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
        let model = params
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("local/native/default");
        let model = self.resolve_model(request_id, model, "embedding")?;
        Err(native_runtime_not_ready_for_model(
            request_id,
            &model,
            "Native embeddings are unavailable until a local executor is configured",
        ))
    }

    fn usage_stats(&self) -> Value {
        let models = self.engine.discover_models();
        json!({
            "provider": "native",
            "runtime": self.runtime(),
            "model_dirs": self.model_dirs(),
            "models_found": models.len(),
            "models": models,
        })
    }
}

fn native_model_descriptor(artifact: NativeModelArtifact) -> ModelDescriptor {
    let display_name = artifact.display_name();
    let capability = artifact.capability().to_string();

    ModelDescriptor {
        id: artifact.id,
        display_name,
        provider: "native".to_string(),
        capability,
        location: ModelLocation::Local,
    }
}

fn native_model_not_found(request_id: &str, model: &str) -> EngineError {
    EngineError::new(
        "model_not_found",
        format!("Native model not found: {model}"),
        EngineErrorType::InvalidRequest,
        request_id,
        false,
    )
    .with_details(json!({
        "model": model,
        "provider": "native",
        "expected_namespace": "local/native/{model_name}",
    }))
}

fn native_runtime_not_ready_for_model(
    request_id: &str,
    model: &NativeModelArtifact,
    message: impl Into<String>,
) -> EngineError {
    EngineError::new(
        "native_runtime_not_ready",
        message.into(),
        EngineErrorType::RuntimeError,
        request_id,
        false,
    )
    .with_details(json!({
        "model": &model.id,
        "path": &model.path,
        "format": &model.format,
        "metadata": &model.metadata,
        "provider": "native",
        "next": "wire native tensor kernels, tokenizer, model loader, and executor",
    }))
}
