use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EngineRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: EngineMethod,
    #[serde(default)]
    pub params: Value,
}

impl EngineRequest {
    pub fn new(id: impl Into<String>, method: EngineMethod, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method,
            params,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineResponse {
    pub jsonrpc: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<EngineError>,
}

impl EngineResponse {
    pub fn success(id: impl Into<String>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            result: Some(result),
            error: None,
        }
    }

    pub fn failure(id: impl Into<String>, error: EngineError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            result: None,
            error: Some(error),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EngineMethod {
    #[serde(rename = "models.list")]
    ModelsList,
    #[serde(rename = "chat.completions.create")]
    ChatCompletionsCreate,
    #[serde(rename = "completions.create")]
    CompletionsCreate,
    #[serde(rename = "embeddings.create")]
    EmbeddingsCreate,
    #[serde(rename = "providers.list")]
    ProvidersList,
    #[serde(rename = "tasks.get")]
    TasksGet,
    #[serde(rename = "tasks.cancel")]
    TasksCancel,
    #[serde(rename = "usage.stats")]
    UsageStats,
}

impl EngineMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            EngineMethod::ModelsList => "models.list",
            EngineMethod::ChatCompletionsCreate => "chat.completions.create",
            EngineMethod::CompletionsCreate => "completions.create",
            EngineMethod::EmbeddingsCreate => "embeddings.create",
            EngineMethod::ProvidersList => "providers.list",
            EngineMethod::TasksGet => "tasks.get",
            EngineMethod::TasksCancel => "tasks.cancel",
            EngineMethod::UsageStats => "usage.stats",
        }
    }
}

impl fmt::Display for EngineMethod {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for EngineMethod {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "models.list" => Ok(EngineMethod::ModelsList),
            "chat.completions.create" => Ok(EngineMethod::ChatCompletionsCreate),
            "completions.create" => Ok(EngineMethod::CompletionsCreate),
            "embeddings.create" => Ok(EngineMethod::EmbeddingsCreate),
            "providers.list" => Ok(EngineMethod::ProvidersList),
            "tasks.get" => Ok(EngineMethod::TasksGet),
            "tasks.cancel" => Ok(EngineMethod::TasksCancel),
            "usage.stats" => Ok(EngineMethod::UsageStats),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineError {
    pub code: String,
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: EngineErrorType,
    pub request_id: String,
    pub retryable: bool,
    #[serde(default = "empty_object")]
    pub details: Value,
}

impl EngineError {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        error_type: EngineErrorType,
        request_id: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            error_type,
            request_id: request_id.into(),
            retryable,
            details: empty_object(),
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = details;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineErrorType {
    InvalidRequest,
    AuthenticationError,
    PermissionError,
    RateLimitError,
    ProviderError,
    RuntimeError,
    ResourceError,
    Cancelled,
    InternalError,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConnectionProfile {
    Mock {
        name: String,
    },
    LocalIpc {
        endpoint: String,
    },
    LanRpc {
        url: String,
    },
    RemotePrivate {
        url: String,
        network: PrivateNetwork,
    },
    RemotePublic {
        url: String,
    },
    HttpCompat {
        base_url: String,
    },
}

impl Default for ConnectionProfile {
    fn default() -> Self {
        Self::Mock {
            name: "mock-engine".to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateNetwork {
    Tailscale,
    WireGuard,
    Headscale,
    SshTunnel,
    Manual,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub id: String,
    pub display_name: String,
    pub provider: String,
    pub capability: String,
    pub location: ModelLocation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelLocation {
    Local,
    Lan,
    RemotePrivate,
    RemotePublic,
    Cloud,
    Mock,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub id: String,
    pub display_name: String,
    pub status: ProviderStatus,
    pub location: ModelLocation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Available,
    Unavailable,
    NotConfigured,
    NotReady,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskStatus {
    pub task_id: String,
    pub state: TaskState,
    pub progress: Option<f32>,
    pub message: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageStats {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

impl UsageStats {
    pub fn new(input_tokens: u64, output_tokens: u64) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub os: String,
    pub arch: String,
    pub accelerators: Vec<AcceleratorDescriptor>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AcceleratorDescriptor {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub backend: AcceleratorBackend,
    pub gfx_version: Option<String>,
    pub total_memory_bytes: Option<u64>,
    pub used_memory_bytes: Option<u64>,
    pub utilization_percent: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceleratorBackend {
    Rocm,
    OpenCl,
    Vulkan,
    Cuda,
    Metal,
    Cpu,
    Unknown,
}

impl AcceleratorBackend {
    /// Returns a static string label for the backend.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rocm => "ROCm",
            Self::OpenCl => "OpenCL",
            Self::Vulkan => "Vulkan",
            Self::Cuda => "CUDA",
            Self::Metal => "Metal",
            Self::Cpu => "CPU",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeKind {
    Native,
    Ollama,
    LlamaCpp,
    Mock,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeStatus {
    Available,
    Unavailable,
    NotInstalled,
    NotReady,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalRuntimeDescriptor {
    pub kind: LocalRuntimeKind,
    pub status: LocalRuntimeStatus,
    pub endpoint: Option<String>,
    pub version: Option<String>,
    pub hardware: Option<HardwareProfile>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum EngineEvent {
    Token {
        request_id: String,
        delta: String,
    },
    Thinking {
        request_id: String,
        delta: String,
    },
    ToolCall {
        request_id: String,
        call: Value,
    },
    Progress {
        task_id: String,
        request_id: String,
        stage: String,
        progress: f32,
        message: String,
        eta_ms: Option<u64>,
    },
    Metadata {
        request_id: String,
        metadata: Value,
    },
    Warning {
        request_id: String,
        message: String,
    },
    Error {
        error: EngineError,
    },
    Done {
        request_id: String,
        usage: Option<UsageStats>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionParams {
    pub model: String,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionResult {
    pub id: String,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: UsageStats,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

// ── Model ID namespace (#665) ──

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelNamespace {
    LocalNative,
    Runtime,
    Cloud,
    TenchAlias,
}

/// Parse a model ID string into its namespace and local name.
/// Returns `None` if the string does not match any known namespace.
///
/// Patterns:
/// - `local/native/{name}` → `LocalNative`
/// - `runtime/{adapter}/{name}` → `Runtime`
/// - `cloud/{provider}/{name}` → `Cloud`
/// - `tench/{capability}` → `TenchAlias`
pub fn parse_model_id(id: &str) -> Option<(ModelNamespace, &str)> {
    if let Some(name) = id.strip_prefix("local/native/") {
        if !name.is_empty() {
            return Some((ModelNamespace::LocalNative, name));
        }
    }
    if let Some(rest) = id.strip_prefix("runtime/") {
        if let Some(slash) = rest.find('/') {
            let name = &rest[slash + 1..];
            if !name.is_empty() {
                return Some((ModelNamespace::Runtime, name));
            }
        }
    }
    if let Some(rest) = id.strip_prefix("cloud/") {
        if let Some(slash) = rest.find('/') {
            let name = &rest[slash + 1..];
            if !name.is_empty() {
                return Some((ModelNamespace::Cloud, name));
            }
        }
    }
    if let Some(name) = id.strip_prefix("tench/") {
        if !name.is_empty() {
            return Some((ModelNamespace::TenchAlias, name));
        }
    }
    None
}

/// Format a model ID from a namespace and name.
///
/// For `Runtime` and `Cloud`, the second element of the tuple is the adapter/provider name.
pub fn format_model_id(namespace: ModelNamespace, parts: (&str, &str)) -> String {
    match namespace {
        ModelNamespace::LocalNative => format!("local/native/{}", parts.1),
        ModelNamespace::Runtime => format!("runtime/{}/{}", parts.0, parts.1),
        ModelNamespace::Cloud => format!("cloud/{}/{}", parts.0, parts.1),
        ModelNamespace::TenchAlias => format!("tench/{}", parts.1),
    }
}

// ── Retryable error rule (#666) ──

/// Returns `true` for error types that clients may retry with exponential backoff.
///
/// Non-retryable: `InvalidRequest`, `AuthenticationError`, `PermissionError`, `Cancelled`.
/// Retryable: `ProviderError`, `RateLimitError`, `InternalError`.
/// Context-dependent: `RuntimeError`, `ResourceError` (treated as non-retryable by default).
pub fn is_retryable_error_type(t: &EngineErrorType) -> bool {
    matches!(
        t,
        EngineErrorType::ProviderError
            | EngineErrorType::RateLimitError
            | EngineErrorType::InternalError
    )
}

pub fn empty_object() -> Value {
    json!({})
}
