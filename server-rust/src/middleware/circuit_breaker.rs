use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use tokio::time::{Duration, Instant};

/// Circuit breaker with three states: Closed (normal), Open (rejecting), Half-Open (testing).
pub struct CircuitBreaker {
    failure_count: AtomicU64,
    is_open: AtomicBool,
    last_failure: Mutex<Option<Instant>>,
    threshold: u64,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(threshold: u64, reset_timeout_secs: u64) -> Self {
        Self {
            failure_count: AtomicU64::new(0),
            is_open: AtomicBool::new(false),
            last_failure: Mutex::new(None),
            threshold,
            reset_timeout: Duration::from_secs(reset_timeout_secs),
        }
    }

    /// Returns true if a request should be allowed through.
    pub fn is_allowed(&self) -> bool {
        if !self.is_open.load(Ordering::Acquire) {
            return true;
        }
        // Half-open: allow one request through after reset_timeout
        let last = self.last_failure.lock().unwrap();
        if let Some(last_time) = *last {
            if last_time.elapsed() > self.reset_timeout {
                return true;
            }
        }
        false
    }

    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::Release);
        self.is_open.store(false, Ordering::Release);
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
        if count >= self.threshold {
            self.is_open.store(true, Ordering::Release);
            *self.last_failure.lock().unwrap() = Some(Instant::now());
            tracing::warn!("Circuit breaker OPEN after {} consecutive failures", count);
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Acquire)
    }

    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Acquire)
    }
}
