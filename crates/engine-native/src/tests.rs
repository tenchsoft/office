use std::fs;

use super::*;
use crate::gguf::{native_metadata_from_gguf, TEST_GGUF_MAGIC};

#[test]
fn parse_minimal_gguf_metadata() {
    let bytes = minimal_gguf_bytes();
    let metadata = parse_gguf_bytes(&bytes).expect("gguf metadata");

    assert_eq!(metadata.version, 3);
    assert_eq!(metadata.tensor_count, 1);
    assert_eq!(
        metadata
            .fields
            .get("general.architecture")
            .and_then(GgufValue::as_str),
        Some("llama")
    );

    let native = native_metadata_from_gguf(metadata);
    assert_eq!(native.display_name.as_deref(), Some("Tiny Test"));
    assert_eq!(native.architecture.as_deref(), Some("llama"));
    assert_eq!(native.context_length, Some(2048));
    assert_eq!(native.embedding_length, Some(8));
    assert_eq!(native.block_count, Some(1));
}

#[test]
fn parse_safetensors_header_counts_tensors() {
    let mut bytes = Vec::new();
    let header = br#"{"__metadata__":{"name":"Tiny ST","capability":"embedding"},"model.embed":{"dtype":"F32","shape":[1,2],"data_offsets":[0,0]}}"#;
    bytes.extend_from_slice(&(header.len() as u64).to_le_bytes());
    bytes.extend_from_slice(header);

    let metadata = parse_safetensors_bytes(&bytes).expect("safetensors metadata");

    assert_eq!(metadata.tensor_count, 1);
    assert_eq!(
        metadata.metadata.get("name").map(String::as_str),
        Some("Tiny ST")
    );
}

#[test]
fn discover_models_keeps_parse_errors_for_placeholders() {
    let dir = std::env::temp_dir().join(format!("tench-native-discovery-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("tiny.gguf"), b"placeholder").unwrap();
    fs::write(dir.join("notes.txt"), b"ignore").unwrap();

    let artifacts = discover_model_artifacts(std::slice::from_ref(&dir));

    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].id, "local/native/tiny");
    assert_eq!(artifacts[0].format, NativeModelFormat::Gguf);
    assert!(artifacts[0].metadata.parse_error.is_some());

    let _ = fs::remove_dir_all(dir);
}

fn minimal_gguf_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(TEST_GGUF_MAGIC);
    push_u32(&mut bytes, 3);
    push_u64(&mut bytes, 1);
    push_u64(&mut bytes, 6);

    push_string_value(&mut bytes, "general.architecture", "llama");
    push_string_value(&mut bytes, "general.name", "Tiny Test");
    push_u32_value(&mut bytes, "llama.context_length", 2048);
    push_u32_value(&mut bytes, "llama.embedding_length", 8);
    push_u32_value(&mut bytes, "llama.block_count", 1);
    push_string_value(&mut bytes, "tokenizer.ggml.model", "llama");

    push_string(&mut bytes, "token_embd.weight");
    push_u32(&mut bytes, 2);
    push_u64(&mut bytes, 4);
    push_u64(&mut bytes, 8);
    push_u32(&mut bytes, 0);
    push_u64(&mut bytes, 0);

    bytes
}

fn push_string_value(bytes: &mut Vec<u8>, key: &str, value: &str) {
    push_string(bytes, key);
    push_u32(bytes, 8);
    push_string(bytes, value);
}

fn push_u32_value(bytes: &mut Vec<u8>, key: &str, value: u32) {
    push_string(bytes, key);
    push_u32(bytes, 4);
    push_u32(bytes, value);
}

fn push_string(bytes: &mut Vec<u8>, value: &str) {
    push_u64(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
