use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeModelFormat {
    Gguf,
    SafeTensors,
    Onnx,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NativeModelArtifact {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub format: NativeModelFormat,
    pub size_bytes: u64,
    pub metadata: NativeModelMetadata,
}

impl NativeModelArtifact {
    pub fn display_name(&self) -> String {
        self.metadata
            .display_name
            .clone()
            .unwrap_or_else(|| self.name.clone())
    }

    pub fn capability(&self) -> &'static str {
        if self.metadata.embedding_only {
            "embedding"
        } else {
            "chat"
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NativeModelMetadata {
    pub format: NativeModelFormat,
    pub display_name: Option<String>,
    pub architecture: Option<String>,
    pub tokenizer: Option<String>,
    pub context_length: Option<u64>,
    pub embedding_length: Option<u64>,
    pub block_count: Option<u64>,
    pub tensor_count: Option<u64>,
    pub metadata_count: Option<u64>,
    pub embedding_only: bool,
    pub parse_error: Option<String>,
}

impl NativeModelMetadata {
    pub(crate) fn unparsed(format: NativeModelFormat, error: impl Into<String>) -> Self {
        Self {
            format,
            display_name: None,
            architecture: None,
            tokenizer: None,
            context_length: None,
            embedding_length: None,
            block_count: None,
            tensor_count: None,
            metadata_count: None,
            embedding_only: false,
            parse_error: Some(error.into()),
        }
    }

    pub(crate) fn minimal(format: NativeModelFormat) -> Self {
        Self {
            format,
            display_name: None,
            architecture: None,
            tokenizer: None,
            context_length: None,
            embedding_length: None,
            block_count: None,
            tensor_count: None,
            metadata_count: None,
            embedding_only: false,
            parse_error: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GgufMetadata {
    pub version: u32,
    pub tensor_count: u64,
    pub metadata_count: u64,
    pub fields: BTreeMap<String, GgufValue>,
    pub tensors: Vec<GgufTensorInfo>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GgufTensorInfo {
    pub name: String,
    pub dimensions: Vec<u64>,
    pub tensor_type: u32,
    pub offset: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum GgufValue {
    Uint8(u8),
    Int8(i8),
    Uint16(u16),
    Int16(i16),
    Uint32(u32),
    Int32(i32),
    Float32(f32),
    Bool(bool),
    String(String),
    Array {
        item_type: u32,
        values: Vec<GgufValue>,
    },
    Uint64(u64),
    Int64(i64),
    Float64(f64),
}

impl GgufValue {
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Uint8(value) => Some(u64::from(*value)),
            Self::Uint16(value) => Some(u64::from(*value)),
            Self::Uint32(value) => Some(u64::from(*value)),
            Self::Uint64(value) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SafeTensorsMetadata {
    pub tensor_count: u64,
    pub metadata: BTreeMap<String, String>,
}
