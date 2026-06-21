//! Model Health Monitoring (#467)
//!
//! Tracks provider/model availability, response times, and error rates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Health status of a provider or model endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// A single health check measurement.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub timestamp_secs: u64,
    pub latency_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// Aggregated health metrics for an endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub id: String,
    pub status: HealthStatus,
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub success_rate: f64,
    pub total_checks: u64,
    pub recent_failures: u64,
    pub last_check: Option<HealthCheck>,
}

/// Internal tracking state.
struct EndpointState {
    checks: Vec<HealthCheck>,
    max_history: usize,
}

pub struct HealthMonitor {
    endpoints: RwLock<HashMap<String, EndpointState>>,
    max_history: usize,
    degraded_threshold: f64,
    unhealthy_threshold: f64,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            endpoints: RwLock::new(HashMap::new()),
            max_history: 100,
            degraded_threshold: 0.95,
            unhealthy_threshold: 0.80,
        }
    }

    /// Register an endpoint for monitoring.
    pub fn register(&self, id: String) {
        let mut endpoints = self
            .endpoints
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        endpoints.insert(
            id,
            EndpointState {
                checks: Vec::new(),
                max_history: self.max_history,
            },
        );
    }

    /// Record a health check result.
    pub fn record(&self, id: &str, latency_ms: u64, success: bool, error: Option<String>) {
        let mut endpoints = self
            .endpoints
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(state) = endpoints.get_mut(id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            state.checks.push(HealthCheck {
                timestamp_secs: now,
                latency_ms,
                success,
                error,
            });
            if state.checks.len() > state.max_history {
                state.checks.remove(0);
            }
        }
    }

    /// Perform a simple ping check (records the result).
    pub fn ping(&self, id: &str) -> HealthStatus {
        // In a real implementation, this would make an HTTP request.
        // For now, we just compute status from existing data.
        self.get_metrics(id)
            .map(|m| m.status)
            .unwrap_or(HealthStatus::Unknown)
    }

    /// Get aggregated health metrics for an endpoint.
    pub fn get_metrics(&self, id: &str) -> Option<HealthMetrics> {
        let endpoints = self
            .endpoints
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let state = endpoints.get(id)?;
        if state.checks.is_empty() {
            return Some(HealthMetrics {
                id: id.to_string(),
                status: HealthStatus::Unknown,
                avg_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                success_rate: 1.0,
                total_checks: 0,
                recent_failures: 0,
                last_check: None,
            });
        }

        let total = state.checks.len() as u64;
        let successes = state.checks.iter().filter(|c| c.success).count() as f64;
        let success_rate = successes / total as f64;

        let mut latencies: Vec<u64> = state.checks.iter().map(|c| c.latency_ms).collect();
        latencies.sort_unstable();
        let avg_latency_ms = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
        let p99_idx = ((latencies.len() as f64) * 0.99) as usize;
        let p99_latency_ms = latencies
            .get(p99_idx.min(latencies.len() - 1))
            .copied()
            .unwrap_or(0) as f64;

        let recent_failures = state
            .checks
            .iter()
            .rev()
            .take(5)
            .filter(|c| !c.success)
            .count() as u64;

        let status = if success_rate >= self.degraded_threshold {
            HealthStatus::Healthy
        } else if success_rate >= self.unhealthy_threshold {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        Some(HealthMetrics {
            id: id.to_string(),
            status,
            avg_latency_ms,
            p99_latency_ms,
            success_rate,
            total_checks: total,
            recent_failures,
            last_check: state.checks.last().cloned(),
        })
    }

    /// List all monitored endpoint IDs.
    pub fn list_endpoints(&self) -> Vec<String> {
        self.endpoints
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .keys()
            .cloned()
            .collect()
    }

    /// Get metrics for all endpoints.
    pub fn all_metrics(&self) -> Vec<HealthMetrics> {
        let ids: Vec<String> = self
            .endpoints
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .keys()
            .cloned()
            .collect();
        ids.iter().filter_map(|id| self.get_metrics(id)).collect()
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_after_successes() {
        let monitor = HealthMonitor::new();
        monitor.register("openai".into());
        for _ in 0..10 {
            monitor.record("openai", 100, true, None);
        }
        let metrics = monitor.get_metrics("openai").unwrap();
        assert_eq!(metrics.status, HealthStatus::Healthy);
        assert!((metrics.success_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn unhealthy_after_failures() {
        let monitor = HealthMonitor::new();
        monitor.register("bad-provider".into());
        for _ in 0..10 {
            monitor.record("bad-provider", 5000, false, Some("timeout".into()));
        }
        let metrics = monitor.get_metrics("bad-provider").unwrap();
        assert_eq!(metrics.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn unknown_when_no_data() {
        let monitor = HealthMonitor::new();
        monitor.register("new-provider".into());
        let metrics = monitor.get_metrics("new-provider").unwrap();
        assert_eq!(metrics.status, HealthStatus::Unknown);
    }
}
