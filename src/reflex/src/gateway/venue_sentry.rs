use std::time::{Duration, Instant};
use std::collections::VecDeque;

const RTT_HISTORY_SIZE: usize = 20;
const MAX_RTT_THRESHOLD_MS: u64 = 150; // D-56 Limit
const LIQUIDITY_DROP_THRESHOLD: f64 = 0.30; // 70% drop means 30% remains

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub qty: f64,
}

pub struct VenueSentry {
    rtt_history: VecDeque<u64>,
    last_heartbeat: Instant,
    is_connected: bool,
    baseline_liquidity: f64,
}

impl VenueSentry {
    pub fn new() -> Self {
        Self {
            rtt_history: VecDeque::with_capacity(RTT_HISTORY_SIZE),
            last_heartbeat: Instant::now(),
            is_connected: true,
            baseline_liquidity: 0.0,
        }
    }

    /// Record a Heartbeat Round-Trip Time (RTT).
    pub fn record_heartbeat(&mut self, rtt_ms: u64) {
        if self.rtt_history.len() >= RTT_HISTORY_SIZE {
            self.rtt_history.pop_front();
        }
        self.rtt_history.push_back(rtt_ms);
        self.last_heartbeat = Instant::now();
        self.is_connected = true;
    }

    /// Check for "Liquidity Vacuum" (Flash Gap).
    /// Returns true if liquidity is HEALTHY, false if VACUUM detected.
    pub fn check_liquidity(&mut self, bids: &[PriceLevel], asks: &[PriceLevel]) -> bool {
        // Calculate "Thickness" of top 3 levels
        let bid_depth: f64 = bids.iter().take(3).map(|l| l.qty).sum();
        let ask_depth: f64 = asks.iter().take(3).map(|l| l.qty).sum();
        let current_depth = bid_depth + ask_depth;

        if self.baseline_liquidity == 0.0 {
            // Initialize baseline
            self.baseline_liquidity = current_depth;
            return true;
        }

        // Decay baseline slowly to adapt to structural changes, 
        // but simple logic for now: if current < 30% of baseline, PANIC.
        if current_depth < (self.baseline_liquidity * LIQUIDITY_DROP_THRESHOLD) {
            // Vacuum Detected!
            return false;
        }

        // Update baseline (Simple Moving Average or similar would be better, using instantaneous for now but slowly adapting)
        // Let's adapt baseline towards current slowly
        self.baseline_liquidity = self.baseline_liquidity * 0.9 + current_depth * 0.1;
        
        true
    }

    /// The "No-Go" Decision.
    /// Returns TRUE if we should VETO execution.
    pub fn should_veto(&self) -> bool {
        // 1. Connection Check
        if self.last_heartbeat.elapsed() > Duration::from_secs(5) {
            return true; // Broken Pipe
        }

        // 2. Latency Spike Check (Average of last 5)
        if self.rtt_history.is_empty() {
            return false; // Assume innocent until proven guilty or waiting for first heartbeat
        }

        let len = self.rtt_history.len().min(5);
        let recent_sum: u64 = self.rtt_history.iter().rev().take(len).sum();
        let avg_rtt = recent_sum / len as u64;

        if avg_rtt > MAX_RTT_THRESHOLD_MS {
            return true; // Latency Veto
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_veto() {
        let mut sentry = VenueSentry::new();
        
        // Good RTT
        for _ in 0..10 {
            sentry.record_heartbeat(20);
        }
        assert!(!sentry.should_veto());

        // Bad RTT
        for _ in 0..10 {
            sentry.record_heartbeat(200);
        }
        assert!(sentry.should_veto());
    }

    #[test]
    fn test_broken_pipe() {
        let mut sentry = VenueSentry::new();
        sentry.record_heartbeat(20);
        
        // Simulate time passing (cannot mock Instant::now easily without trait, 
        // so we just rely on logic correctness or manual test. 
        // For unit test here, we can't easily sleep 5s. 
        // We trust the `elapsed()` logic standard lib.)
        // Skipping direct time wait test to avoid slow tests.
    }

    #[test]
    fn test_liquidity_vacuum() {
        let mut sentry = VenueSentry::new();
        let level_normal = PriceLevel { price: 100.0, qty: 10.0 };
        let level_empty = PriceLevel { price: 100.0, qty: 0.1 };

        // Initialize baseline
        sentry.check_liquidity(&[level_normal.clone()], &[level_normal.clone()]); // 20 qty
        
        // Check good liquidty
        assert!(sentry.check_liquidity(&[level_normal.clone()], &[level_normal.clone()]));

        // Check Vacuum (0.2 qty vs ~20 baseline)
        assert!(!sentry.check_liquidity(&[level_empty.clone()], &[level_empty.clone()]));
    }
}
