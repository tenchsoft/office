use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EngineConfig {
    pub server: ServerConfig,
    pub providers: ProvidersConfig,
    pub cache: CacheConfig,
    pub retry: RetryConfig,
    pub logging: LoggingConfig,
    pub sessions: SessionsConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub allow_browser_origins: bool,
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: default_bind_addr(),
            port: default_port(),
            allow_browser_origins: false,
            request_timeout_secs: default_request_timeout_secs(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvidersConfig {
    #[serde(default)]
    pub default_provider: String,
    #[serde(default = "default_model")]
    pub default_model: String,
    #[serde(default)]
    pub api_keys_dir: Option<PathBuf>,
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            default_provider: "mock".to_string(),
            default_model: default_model(),
            api_keys_dir: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_cache_ttl_secs")]
    pub default_ttl_secs: u64,
    #[serde(default = "default_cache_max_entries")]
    pub max_entries: usize,
    #[serde(default = "default_cache_max_entry_size")]
    pub max_entry_size_bytes: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl_secs: default_cache_ttl_secs(),
            max_entries: default_cache_max_entries(),
            max_entry_size_bytes: default_cache_max_entry_size(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetryConfig {
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_base_delay_ms")]
    pub base_delay_ms: u64,
    #[serde(default = "default_retry_max_delay_ms")]
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            base_delay_ms: default_retry_base_delay_ms(),
            max_delay_ms: default_retry_max_delay_ms(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub log_dir: Option<PathBuf>,
    #[serde(default = "default_true")]
    pub mask_sensitive: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            log_dir: None,
            mask_sensitive: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionsConfig {
    #[serde(default = "default_session_ttl_secs")]
    pub ttl_secs: u64,
    #[serde(default = "default_session_max_context_tokens")]
    pub max_context_tokens: usize,
}

impl Default for SessionsConfig {
    fn default() -> Self {
        Self {
            ttl_secs: default_session_ttl_secs(),
            max_context_tokens: default_session_max_context_tokens(),
        }
    }
}

impl EngineConfig {
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file {}: {e}", path.display()))?;
        let config: Self =
            toml::from_str(&content).map_err(|e| format!("Failed to parse config file: {e}"))?;
        Ok(config.apply_env_overrides())
    }

    pub fn load_or_default() -> Self {
        let config_path = Self::default_config_path();
        if config_path.exists() {
            Self::load_from_file(&config_path).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn default_config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".tench")
            .join("engine")
            .join("config.toml")
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.server.request_timeout_secs)
    }

    fn apply_env_overrides(mut self) -> Self {
        if let Ok(addr) = std::env::var("TENCH_ENGINE_BIND_ADDR") {
            self.server.bind_addr = addr;
        }
        if let Ok(port) = std::env::var("TENCH_ENGINE_PORT") {
            if let Ok(p) = port.parse() {
                self.server.port = p;
            }
        }
        if let Ok(provider) = std::env::var("TENCH_ENGINE_DEFAULT_PROVIDER") {
            self.providers.default_provider = provider;
        }
        if let Ok(model) = std::env::var("TENCH_ENGINE_DEFAULT_MODEL") {
            self.providers.default_model = model;
        }
        if let Ok(level) = std::env::var("TENCH_ENGINE_LOG_LEVEL") {
            self.logging.level = level;
        }
        self
    }
}

fn default_bind_addr() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    1873
}
fn default_request_timeout_secs() -> u64 {
    30
}
fn default_model() -> String {
    "tench/chat".to_string()
}
fn default_cache_ttl_secs() -> u64 {
    3600
}
fn default_cache_max_entries() -> usize {
    1000
}
fn default_cache_max_entry_size() -> usize {
    1024 * 1024 // 1 MB
}
fn default_max_retries() -> u32 {
    3
}
fn default_retry_base_delay_ms() -> u64 {
    1000
}
fn default_retry_max_delay_ms() -> u64 {
    30000
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_session_ttl_secs() -> u64 {
    3600
}
fn default_session_max_context_tokens() -> usize {
    128_000
}
fn default_true() -> bool {
    true
}
