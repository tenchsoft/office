use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Token bucket rate limiter.
#[derive(Clone, Debug)]
pub struct RateLimiter {
    tokens: Arc<Mutex<f64>>,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(max_requests_per_minute: u32) -> Self {
        let max_tokens = max_requests_per_minute as f64;
        let refill_rate = max_tokens / 60.0;
        Self {
            tokens: Arc::new(Mutex::new(max_tokens)),
            max_tokens,
            refill_rate,
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn try_acquire(&self) -> bool {
        self.refill();
        let mut tokens = self
            .tokens
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }

    pub fn wait_and_acquire(&self) {
        while !self.try_acquire() {
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn refill(&self) {
        let mut last = self
            .last_refill
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let now = Instant::now();
        let elapsed = now.duration_since(*last).as_secs_f64();
        let mut tokens = self
            .tokens
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *tokens = (*tokens + elapsed * self.refill_rate).min(self.max_tokens);
        *last = now;
    }
}

/// Priority level for queued requests.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RequestPriority {
    Background = 0,
    Normal = 1,
    Urgent = 2,
}

/// A queued request with priority and insertion time.
#[derive(Clone, Debug)]
pub struct QueuedRequest {
    pub id: String,
    pub priority: RequestPriority,
    pub enqueued_at: Instant,
}

/// Request queue with priority ordering and rate limiting.
#[derive(Clone, Debug)]
pub struct RequestQueue {
    queue: Arc<Mutex<VecDeque<QueuedRequest>>>,
    limiter: RateLimiter,
    max_size: usize,
}

impl RequestQueue {
    pub fn new(max_size: usize, requests_per_minute: u32) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            limiter: RateLimiter::new(requests_per_minute),
            max_size,
        }
    }

    pub fn enqueue(&self, id: String, priority: RequestPriority) -> Result<(), String> {
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if queue.len() >= self.max_size {
            return Err(format!("Queue full (max {})", self.max_size));
        }
        let request = QueuedRequest {
            id,
            priority,
            enqueued_at: Instant::now(),
        };
        // Insert in priority order (higher priority first)
        let pos = queue
            .iter()
            .position(|r| r.priority < request.priority)
            .unwrap_or(queue.len());
        queue.insert(pos, request);
        Ok(())
    }

    pub fn dequeue(&self) -> Option<QueuedRequest> {
        self.limiter.wait_and_acquire();
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        queue.pop_front()
    }

    pub fn try_dequeue(&self) -> Option<QueuedRequest> {
        if !self.limiter.try_acquire() {
            return None;
        }
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_empty()
    }

    pub fn remove(&self, id: &str) -> bool {
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(pos) = queue.iter().position(|r| r.id == id) {
            queue.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_allows_within_budget() {
        let limiter = RateLimiter::new(100);
        assert!(limiter.try_acquire());
    }

    #[test]
    fn queue_priority_ordering() {
        let queue = RequestQueue::new(100, 1000);
        queue
            .enqueue("bg".into(), RequestPriority::Background)
            .unwrap();
        queue
            .enqueue("urgent".into(), RequestPriority::Urgent)
            .unwrap();
        queue
            .enqueue("normal".into(), RequestPriority::Normal)
            .unwrap();

        let first = queue.try_dequeue().unwrap();
        assert_eq!(first.id, "urgent");
        assert_eq!(first.priority, RequestPriority::Urgent);
    }

    #[test]
    fn queue_max_size() {
        let queue = RequestQueue::new(2, 1000);
        assert!(queue.enqueue("a".into(), RequestPriority::Normal).is_ok());
        assert!(queue.enqueue("b".into(), RequestPriority::Normal).is_ok());
        assert!(queue.enqueue("c".into(), RequestPriority::Normal).is_err());
    }
}
