use std::fs::{self};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Log level for engine logging.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// A single log entry.
#[derive(Clone, Debug, Serialize)]
pub struct LogEntry {
    pub timestamp_ms: u64,
    pub level: LogLevel,
    pub message: String,
    pub request_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub duration_ms: Option<u64>,
    pub token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
}

/// Structured logger with file rotation and sensitive data masking.
#[derive(Clone, Debug)]
pub struct EngineLogger {
    level: LogLevel,
    log_dir: Option<PathBuf>,
    mask_sensitive: bool,
    buffer: Arc<Mutex<Vec<LogEntry>>>,
    max_buffer_size: usize,
    #[allow(dead_code)]
    flush_interval: Duration,
    last_flush: Arc<Mutex<Instant>>,
}

impl EngineLogger {
    pub fn new(level: LogLevel, log_dir: Option<PathBuf>, mask_sensitive: bool) -> Self {
        Self {
            level,
            log_dir,
            mask_sensitive,
            buffer: Arc::new(Mutex::new(Vec::new())),
            max_buffer_size: 100,
            flush_interval: Duration::from_secs(5),
            last_flush: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(LogLevel::Info, None, true)
    }

    pub fn log(&self, entry: LogEntry) {
        if entry.level < self.level {
            return;
        }
        let mut buffer = self
            .buffer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        buffer.push(entry);
        if buffer.len() >= self.max_buffer_size {
            drop(buffer);
            let _ = self.flush();
        }
    }

    pub fn log_request(
        &self,
        level: LogLevel,
        message: &str,
        request_id: &str,
        provider: Option<&str>,
        model: Option<&str>,
    ) {
        self.log(LogEntry {
            timestamp_ms: now_ms(),
            level,
            message: message.to_string(),
            request_id: Some(request_id.to_string()),
            provider: provider.map(|s| s.to_string()),
            model: model.map(|s| s.to_string()),
            duration_ms: None,
            token_count: None,
            extra: None,
        });
    }

    pub fn log_response(
        &self,
        request_id: &str,
        provider: &str,
        model: &str,
        duration_ms: u64,
        token_count: u32,
    ) {
        self.log(LogEntry {
            timestamp_ms: now_ms(),
            level: LogLevel::Info,
            message: "Response received".to_string(),
            request_id: Some(request_id.to_string()),
            provider: Some(provider.to_string()),
            model: Some(model.to_string()),
            duration_ms: Some(duration_ms),
            token_count: Some(token_count),
            extra: None,
        });
    }

    pub fn log_error(&self, request_id: &str, message: &str) {
        self.log(LogEntry {
            timestamp_ms: now_ms(),
            level: LogLevel::Error,
            message: message.to_string(),
            request_id: Some(request_id.to_string()),
            provider: None,
            model: None,
            duration_ms: None,
            token_count: None,
            extra: None,
        });
    }

    pub fn flush(&self) -> Result<usize, String> {
        let mut buffer = self
            .buffer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let count = buffer.len();
        if count == 0 {
            return Ok(0);
        }

        if let Some(ref dir) = self.log_dir {
            if !dir.exists() {
                fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            let filename = format!("engine_{}.jsonl", today_string());
            let path = dir.join(&filename);
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| e.to_string())?;
            for entry in buffer.drain(..) {
                let mut line = serde_json::to_string(&entry).unwrap_or_default();
                if self.mask_sensitive {
                    line = mask_api_keys(&line);
                }
                writeln!(file, "{line}").map_err(|e| e.to_string())?;
            }
        } else {
            for entry in buffer.drain(..) {
                eprintln!("[{}] {}", entry.level, entry.message);
            }
        }

        *self
            .last_flush
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Instant::now();
        Ok(count)
    }

    pub fn mask(text: &str) -> String {
        mask_api_keys(text)
    }
}

use std::fs::OpenOptions;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn today_string() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    format!("{days}")
}

fn mask_api_keys(text: &str) -> String {
    let mut result = text.to_string();
    for prefix in &["sk_live_", "sk_test_", "sk-", "key-", "pk_"] {
        // Search from longest prefix first to avoid partial overlap
        let mut offset = 0;
        while let Some(start) = result[offset..].find(prefix) {
            let abs_start = offset + start;
            let abs_end = (abs_start + prefix.len() + 8).min(result.len());
            let masked = format!(
                "{}****",
                &result[abs_start..abs_start + prefix.len() + 4.min(result.len() - abs_start)]
            );
            let masked_len = masked.len();
            result.replace_range(abs_start..abs_end, &masked);
            offset = abs_start + masked_len;
            if offset >= result.len() {
                break;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_hides_api_keys() {
        let masked = mask_api_keys("key=sk-1234567890abcdef");
        assert!(!masked.contains("1234567890"));
        assert!(masked.contains("****"));
    }

    #[test]
    fn logger_respects_level() {
        let logger = EngineLogger::new(LogLevel::Warn, None, false);
        logger.log(LogEntry {
            timestamp_ms: 0,
            level: LogLevel::Debug,
            message: "debug".into(),
            request_id: None,
            provider: None,
            model: None,
            duration_ms: None,
            token_count: None,
            extra: None,
        });
        assert!(logger
            .buffer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_empty());
    }
}
