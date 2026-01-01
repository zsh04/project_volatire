use std::time::Instant;
use std::sync::Mutex;

/// A simple Token Bucket Rate Limiter.
/// Ensures we do not exceed a certain number of requests per second.
pub struct TokenBucket {
    capacity: f64,
    tokens: Mutex<f64>,
    refill_rate_per_sec: f64,
    last_refill: Mutex<Instant>,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: Mutex::new(capacity), // Start full
            refill_rate_per_sec: refill_rate,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    /// Attempts to consume `amount` tokens. Returns true if successful.
    pub fn try_consume(&self, amount: f64) -> bool {
        let mut tokens = self.tokens.lock().unwrap();
        let mut last_refill = self.last_refill.lock().unwrap();
        
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        
        // Refill
        let new_tokens = elapsed * self.refill_rate_per_sec;
        if new_tokens > 0.0 {
            *tokens = (*tokens + new_tokens).min(self.capacity);
            *last_refill = now;
        }

        if *tokens >= amount {
            *tokens -= amount;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_token_bucket_throttle() {
        // Capacity 5, Refill 1 per sec
        let bucket = TokenBucket::new(5.0, 1.0);

        // Consume all initial tokens
        assert!(bucket.try_consume(5.0));
        
        // Should fail immediately after
        assert!(!bucket.try_consume(1.0));

        // Sleep 1.1s to regenerate 1 token
        thread::sleep(Duration::from_millis(1100));
        
        // Should succeed now
        assert!(bucket.try_consume(1.0));
    }
}
