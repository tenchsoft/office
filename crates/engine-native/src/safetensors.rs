use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use serde_json::Value;

use crate::error::invalid_model;
use crate::{NativeEngineError, SafeTensorsMetadata};

const MAX_SAFE_TENSORS_HEADER_BYTES: u64 = 16 * 1024 * 1024;

pub fn parse_safetensors_file(path: &Path) -> Result<SafeTensorsMetadata, NativeEngineError> {
    let bytes = fs::read(path).map_err(|error| NativeEngineError::Io(error.to_string()))?;
    parse_safetensors_bytes(&bytes)
}

pub fn parse_safetensors_bytes(bytes: &[u8]) -> Result<SafeTensorsMetadata, NativeEngineError> {
    let mut reader = Cursor::new(bytes);
    let header_len = read_u64(&mut reader)?;
    if header_len > MAX_SAFE_TENSORS_HEADER_BYTES {
        return Err(invalid_model("safetensors header is too large"));
    }

    let mut header = vec![0_u8; header_len as usize];
    reader
        .read_exact(&mut header)
        .map_err(|_| invalid_model("safetensors file is missing the JSON header"))?;

    let value: Value = serde_json::from_slice(&header)
        .map_err(|error| invalid_model(format!("Invalid safetensors JSON header: {error}")))?;
    let Some(object) = value.as_object() else {
        return Err(invalid_model("safetensors header must be a JSON object"));
    };

    let mut metadata = BTreeMap::new();
    if let Some(meta) = object.get("__metadata__").and_then(Value::as_object) {
        for (key, value) in meta {
            if let Some(value) = value.as_str() {
                metadata.insert(key.clone(), value.to_string());
            }
        }
    }

    let tensor_count = object.keys().filter(|key| *key != "__metadata__").count() as u64;
    Ok(SafeTensorsMetadata {
        tensor_count,
        metadata,
    })
}

fn read_u64(reader: &mut Cursor<&[u8]>) -> Result<u64, NativeEngineError> {
    let mut bytes = [0_u8; 8];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| invalid_model("Unexpected EOF while reading u64"))?;
    Ok(u64::from_le_bytes(bytes))
}
