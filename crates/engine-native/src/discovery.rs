use std::fs;
use std::path::{Path, PathBuf};

use crate::gguf::{native_metadata_from_gguf, parse_gguf_file};
use crate::safetensors::parse_safetensors_file;
use crate::{NativeModelArtifact, NativeModelFormat, NativeModelMetadata};

pub fn discover_model_artifacts(model_dirs: &[PathBuf]) -> Vec<NativeModelArtifact> {
    let mut artifacts = Vec::new();
    for dir in model_dirs {
        scan_model_dir(dir, &mut artifacts);
    }
    artifacts.sort_by(|left, right| left.id.cmp(&right.id));
    artifacts.dedup_by(|left, right| left.id == right.id);
    artifacts
}

fn scan_model_dir(dir: &Path, artifacts: &mut Vec<NativeModelArtifact>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_model_dir(&path, artifacts);
            continue;
        }

        if let Some(artifact) = model_artifact_from_path(&path) {
            artifacts.push(artifact);
        }
    }
}

fn model_artifact_from_path(path: &Path) -> Option<NativeModelArtifact> {
    let format = model_format(path)?;
    let size_bytes = fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("model")
        .to_string();
    let metadata = model_metadata(path, &format);

    Some(NativeModelArtifact {
        id: format!("local/native/{}", model_id_component(&name)),
        name,
        path: path.to_path_buf(),
        format,
        size_bytes,
        metadata,
    })
}

fn model_format(path: &Path) -> Option<NativeModelFormat> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "gguf" => Some(NativeModelFormat::Gguf),
        "safetensors" => Some(NativeModelFormat::SafeTensors),
        "onnx" => Some(NativeModelFormat::Onnx),
        _ => None,
    }
}

fn model_metadata(path: &Path, format: &NativeModelFormat) -> NativeModelMetadata {
    match format {
        NativeModelFormat::Gguf => match parse_gguf_file(path) {
            Ok(metadata) => native_metadata_from_gguf(metadata),
            Err(error) => NativeModelMetadata::unparsed(format.clone(), error.to_string()),
        },
        NativeModelFormat::SafeTensors => match parse_safetensors_file(path) {
            Ok(metadata) => NativeModelMetadata {
                format: format.clone(),
                display_name: metadata.metadata.get("name").cloned(),
                architecture: metadata.metadata.get("architecture").cloned(),
                tokenizer: metadata.metadata.get("tokenizer").cloned(),
                context_length: metadata
                    .metadata
                    .get("context_length")
                    .and_then(|value| value.parse().ok()),
                embedding_length: metadata
                    .metadata
                    .get("embedding_length")
                    .and_then(|value| value.parse().ok()),
                block_count: metadata
                    .metadata
                    .get("block_count")
                    .and_then(|value| value.parse().ok()),
                tensor_count: Some(metadata.tensor_count),
                metadata_count: Some(metadata.metadata.len() as u64),
                embedding_only: metadata
                    .metadata
                    .get("capability")
                    .map(|value| value == "embedding")
                    .unwrap_or(false),
                parse_error: None,
            },
            Err(error) => NativeModelMetadata::unparsed(format.clone(), error.to_string()),
        },
        NativeModelFormat::Onnx => NativeModelMetadata::minimal(format.clone()),
    }
}

fn model_id_component(name: &str) -> String {
    let value = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric()
                || character == '-'
                || character == '_'
                || character == '.'
            {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();

    if value.is_empty() {
        "model".to_string()
    } else {
        value
    }
}
