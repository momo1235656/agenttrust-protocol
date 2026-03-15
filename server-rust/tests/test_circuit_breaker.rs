/// Unit tests for the CircuitBreaker.

#[cfg(test)]
mod tests {
    use agenttrust_server::middleware::circuit_breaker::CircuitBreaker;

    #[test]
    fn test_circuit_starts_closed() {
        let cb = CircuitBreaker::new(3, 60);
        assert!(cb.is_allowed());
        assert!(!cb.is_open());
    }

    #[test]
    fn test_circuit_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, 60);
        cb.record_failure();
        cb.record_failure();
        assert!(cb.is_allowed()); // Still closed after 2 failures
        cb.record_failure(); // Third failure -> opens
        assert!(cb.is_open());
        assert!(!cb.is_allowed());
    }

    #[test]
    fn test_success_resets_circuit() {
        let cb = CircuitBreaker::new(3, 60);
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert!(!cb.is_open());
        assert!(cb.is_allowed());
        assert_eq!(cb.failure_count(), 0);
    }

    #[tokio::test]
    async fn test_half_open_after_timeout() {
        // Threshold=1, reset=0 seconds → opens immediately on failure, then half-opens quickly
        let cb = CircuitBreaker::new(1, 0);
        cb.record_failure();
        assert!(cb.is_open());

        // After 0-second reset_timeout, half-open allows one request
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        assert!(cb.is_allowed());
    }
}
