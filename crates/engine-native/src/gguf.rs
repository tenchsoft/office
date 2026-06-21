use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

use crate::error::invalid_model;
use crate::{
    GgufMetadata, GgufTensorInfo, GgufValue, NativeEngineError, NativeModelFormat,
    NativeModelMetadata,
};

const GGUF_MAGIC: &[u8; 4] = b"GGUF";

pub fn parse_gguf_file(path: &Path) -> Result<GgufMetadata, NativeEngineError> {
    let bytes = fs::read(path).map_err(|error| NativeEngineError::Io(error.to_string()))?;
    parse_gguf_bytes(&bytes)
}

pub fn parse_gguf_bytes(bytes: &[u8]) -> Result<GgufMetadata, NativeEngineError> {
    let mut reader = Cursor::new(bytes);
    let mut magic = [0_u8; 4];
    reader
        .read_exact(&mut magic)
        .map_err(|_| invalid_model("GGUF file is missing the magic header"))?;
    if &magic != GGUF_MAGIC {
        return Err(invalid_model("GGUF file has an invalid magic header"));
    }

    let version = read_u32(&mut reader)?;
    if !(2..=3).contains(&version) {
        return Err(NativeEngineError::UnsupportedFormat(format!(
            "Unsupported GGUF version: {version}"
        )));
    }

    let tensor_count = read_u64(&mut reader)?;
    let metadata_count = read_u64(&mut reader)?;
    let mut fields = BTreeMap::new();

    for _ in 0..metadata_count {
        let key = read_gguf_string(&mut reader)?;
        let value_type = read_u32(&mut reader)?;
        let value = read_gguf_value(&mut reader, value_type)?;
        fields.insert(key, value);
    }

    let mut tensors = Vec::new();
    for _ in 0..tensor_count {
        let name = read_gguf_string(&mut reader)?;
        let dimension_count = read_u32(&mut reader)?;
        let mut dimensions = Vec::new();
        for _ in 0..dimension_count {
            dimensions.push(read_u64(&mut reader)?);
        }
        let tensor_type = read_u32(&mut reader)?;
        let offset = read_u64(&mut reader)?;
        tensors.push(GgufTensorInfo {
            name,
            dimensions,
            tensor_type,
            offset,
        });
    }

    Ok(GgufMetadata {
        version,
        tensor_count,
        metadata_count,
        fields,
        tensors,
    })
}

pub(crate) fn native_metadata_from_gguf(metadata: GgufMetadata) -> NativeModelMetadata {
    let architecture = metadata
        .fields
        .get("general.architecture")
        .and_then(GgufValue::as_str)
        .map(str::to_string);
    let context_key = architecture
        .as_ref()
        .map(|architecture| format!("{architecture}.context_length"));
    let embedding_key = architecture
        .as_ref()
        .map(|architecture| format!("{architecture}.embedding_length"));
    let block_key = architecture
        .as_ref()
        .map(|architecture| format!("{architecture}.block_count"));

    NativeModelMetadata {
        format: NativeModelFormat::Gguf,
        display_name: metadata
            .fields
            .get("general.name")
            .and_then(GgufValue::as_str)
            .map(str::to_string),
        architecture,
        tokenizer: metadata
            .fields
            .get("tokenizer.ggml.model")
            .and_then(GgufValue::as_str)
            .map(str::to_string),
        context_length: context_key
            .as_deref()
            .and_then(|key| metadata.fields.get(key))
            .and_then(GgufValue::as_u64),
        embedding_length: embedding_key
            .as_deref()
            .and_then(|key| metadata.fields.get(key))
            .and_then(GgufValue::as_u64),
        block_count: block_key
            .as_deref()
            .and_then(|key| metadata.fields.get(key))
            .and_then(GgufValue::as_u64),
        tensor_count: Some(metadata.tensor_count),
        metadata_count: Some(metadata.metadata_count),
        embedding_only: false,
        parse_error: None,
    }
}

fn read_gguf_value(
    reader: &mut Cursor<&[u8]>,
    value_type: u32,
) -> Result<GgufValue, NativeEngineError> {
    match value_type {
        0 => Ok(GgufValue::Uint8(read_u8(reader)?)),
        1 => Ok(GgufValue::Int8(read_u8(reader)? as i8)),
        2 => Ok(GgufValue::Uint16(read_u16(reader)?)),
        3 => Ok(GgufValue::Int16(read_i16(reader)?)),
        4 => Ok(GgufValue::Uint32(read_u32(reader)?)),
        5 => Ok(GgufValue::Int32(read_i32(reader)?)),
        6 => Ok(GgufValue::Float32(read_f32(reader)?)),
        7 => Ok(GgufValue::Bool(read_u8(reader)? != 0)),
        8 => Ok(GgufValue::String(read_gguf_string(reader)?)),
        9 => {
            let item_type = read_u32(reader)?;
            if item_type == 9 {
                return Err(NativeEngineError::UnsupportedFormat(
                    "Nested GGUF arrays are not supported".to_string(),
                ));
            }
            let len = read_u64(reader)?;
            let mut values = Vec::new();
            for _ in 0..len {
                values.push(read_gguf_value(reader, item_type)?);
            }
            Ok(GgufValue::Array { item_type, values })
        }
        10 => Ok(GgufValue::Uint64(read_u64(reader)?)),
        11 => Ok(GgufValue::Int64(read_i64(reader)?)),
        12 => Ok(GgufValue::Float64(read_f64(reader)?)),
        _ => Err(NativeEngineError::UnsupportedFormat(format!(
            "Unsupported GGUF metadata value type: {value_type}"
        ))),
    }
}

fn read_gguf_string(reader: &mut Cursor<&[u8]>) -> Result<String, NativeEngineError> {
    let len = read_u64(reader)?;
    let mut bytes = vec![0_u8; len as usize];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| invalid_model("Unexpected EOF while reading GGUF string"))?;
    String::from_utf8(bytes)
        .map_err(|error| invalid_model(format!("Invalid UTF-8 string: {error}")))
}

fn read_u8(reader: &mut Cursor<&[u8]>) -> Result<u8, NativeEngineError> {
    let mut bytes = [0_u8; 1];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| invalid_model("Unexpected EOF while reading u8"))?;
    Ok(bytes[0])
}

fn read_u16(reader: &mut Cursor<&[u8]>) -> Result<u16, NativeEngineError> {
    let mut bytes = [0_u8; 2];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| invalid_model("Unexpected EOF while reading u16"))?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_i16(reader: &mut Cursor<&[u8]>) -> Result<i16, NativeEngineError> {
    let mut bytes = [0_u8; 2];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| invalid_model("Unexpected EOF while reading i16"))?;
    Ok(i16::from_le_bytes(bytes))
}

fn read_u32(reader: &mut Cursor<&[u8]>) -> Result<u32, NativeEngineError> {
    let mut bytes = [0_u8; 4];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| invalid_model("Unexpected EOF while reading u32"))?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_i32(reader: &mut Cursor<&[u8]>) -> Result<i32, NativeEngineError> {
    let mut bytes = [0_u8; 4];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| invalid_model("Unexpected EOF while reading i32"))?;
    Ok(i32::from_le_bytes(bytes))
}

fn read_u64(reader: &mut Cursor<&[u8]>) -> Result<u64, NativeEngineError> {
    let mut bytes = [0_u8; 8];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| invalid_model("Unexpected EOF while reading u64"))?;
    Ok(u64::from_le_bytes(bytes))
}

fn read_i64(reader: &mut Cursor<&[u8]>) -> Result<i64, NativeEngineError> {
    let mut bytes = [0_u8; 8];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| invalid_model("Unexpected EOF while reading i64"))?;
    Ok(i64::from_le_bytes(bytes))
}

fn read_f32(reader: &mut Cursor<&[u8]>) -> Result<f32, NativeEngineError> {
    let mut bytes = [0_u8; 4];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| invalid_model("Unexpected EOF while reading f32"))?;
    Ok(f32::from_le_bytes(bytes))
}

fn read_f64(reader: &mut Cursor<&[u8]>) -> Result<f64, NativeEngineError> {
    let mut bytes = [0_u8; 8];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| invalid_model("Unexpected EOF while reading f64"))?;
    Ok(f64::from_le_bytes(bytes))
}

#[cfg(test)]
pub(crate) const TEST_GGUF_MAGIC: &[u8; 4] = GGUF_MAGIC;
