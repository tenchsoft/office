use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tench_shared_types::{EngineError, EngineErrorType};

/// Circuit breaker states.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker to prevent cascading failures.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<u32>>,
    success_count: Arc<Mutex<u32>>,
    threshold: u32,
    reset_timeout: Duration,
    last_failure: Arc<Mutex<Option<Instant>>>,
}

#[allow(dead_code)]
impl CircuitBreaker {
    pub fn new(threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            success_count: Arc::new(Mutex::new(0)),
            threshold,
            reset_timeout,
            last_failure: Arc::new(Mutex::new(None)),
        }
    }

    pub fn allow_request(&self) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let last = self
                    .last_failure
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if let Some(t) = *last {
                    if t.elapsed() >= self.reset_timeout {
                        *state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *state == CircuitState::HalfOpen {
            *state = CircuitState::Closed;
            *self
                .failure_count
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = 0;
        }
        *self
            .success_count
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) += 1;
    }

    pub fn record_failure(&self) {
        let mut count = self
            .failure_count
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *count += 1;
        *self
            .last_failure
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Instant::now());

        if *count >= self.threshold {
            *self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = CircuitState::Open;
        }
    }

    pub fn state(&self) -> CircuitState {
        *self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Retry policy with exponential backoff.
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
        }
    }
}

impl RetryPolicy {
    pub fn new(max_retries: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
        }
    }

    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let multiplier = 2u64.saturating_pow(attempt);
        let delay = self.base_delay * multiplier as u32;
        delay.min(self.max_delay)
    }

    pub fn should_retry(&self, error: &EngineError) -> bool {
        error.retryable
    }

    /// Execute a fallible operation with retry logic.
    pub fn retry<F, T>(&self, mut operation: F) -> Result<T, EngineError>
    where
        F: FnMut() -> Result<T, EngineError>,
    {
        let mut last_error = None;
        for attempt in 0..=self.max_retries {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    let retryable = error.retryable;
                    last_error = Some(error);
                    if !retryable || attempt >= self.max_retries {
                        break;
                    }
                    let delay = self.delay_for_attempt(attempt);
                    std::thread::sleep(delay);
                }
            }
        }
        Err(last_error.unwrap_or_else(|| {
            EngineError::new(
                "retry_exhausted",
                "All retry attempts exhausted",
                EngineErrorType::RuntimeError,
                "retry",
                false,
            )
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn retry_policy_delay_increases() {
        let policy = RetryPolicy::new(3, Duration::from_millis(100), Duration::from_secs(10));
        assert_eq!(policy.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(400));
    }

    #[test]
    fn retry_policy_succeeds_on_second_try() {
        let policy = RetryPolicy::new(3, Duration::from_millis(1), Duration::from_millis(10));
        let mut attempts = 0;
        let result = policy.retry(|| {
            attempts += 1;
            if attempts == 1 {
                Err(EngineError::new(
                    "test",
                    "fail",
                    EngineErrorType::ProviderError,
                    "r",
                    true,
                ))
            } else {
                Ok(42)
            }
        });
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 2);
    }
}
