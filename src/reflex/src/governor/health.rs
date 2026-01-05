use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Critical(String),
}

#[derive(Debug, Clone)]
pub struct PhoenixMonitor {
    jitter_history: VecDeque<Duration>,
    window_size: usize,
    latency_threshold_degraded: Duration,
    latency_threshold_critical: Duration,
}

impl PhoenixMonitor {
    pub fn new() -> Self {
        Self {
            jitter_history: VecDeque::with_capacity(100),
            window_size: 100,
            // D-96 Thresholds: 
            // Nominal is ~10-15us. 
            // 25us is Degraded.
            // 50us is Critical (suggests GC pause or contention).
            latency_threshold_degraded: Duration::from_micros(25),
            latency_threshold_critical: Duration::from_micros(50),
        }
    }

    /// Records a loop latency sample and returns current Health Status.
    pub fn check_vitals(&mut self, latency_sample: Duration) -> HealthStatus {
        if self.jitter_history.len() >= self.window_size {
            self.jitter_history.pop_front();
        }
        self.jitter_history.push_back(latency_sample);

        // Simple Average (could use Percentile in future)
        let sum: Duration = self.jitter_history.iter().sum();
        let count = self.jitter_history.len() as u32;
        if count == 0 { return HealthStatus::Healthy; }
        
        let avg = sum / count;

        if avg > self.latency_threshold_critical {
            // "The Phoenix": Signal likely Re-Zeroing needed
            HealthStatus::Critical(format!("Avg Latency {:?} > Critical {:?}", avg, self.latency_threshold_critical))
        } else if avg > self.latency_threshold_degraded {
            HealthStatus::Degraded(format!("Avg Latency {:?} > Degraded {:?}", avg, self.latency_threshold_degraded))
        } else {
            HealthStatus::Healthy
        }
    }

    /// Signals that the specific process should initiate state transfer.
    pub fn initiate_handoff(&self) {
        tracing::warn!("🔥 PHOENIX PROTOCOL: INITIATING HANDOFF SEQUENCE");
        // In prod: Write to /dev/shm/handoff_signal or Redis key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metabolic_thresholds() {
        let mut monitor = PhoenixMonitor::new();
        
        // 1. Healthy (10us)
        for i in 0..100 {
            let status = monitor.check_vitals(Duration::from_micros(10));
            if i == 99 {
                assert_eq!(status, HealthStatus::Healthy);
            }
        }

        // 2. Degraded (30us) -> Avg will drift up
        for _ in 0..150 {
            monitor.check_vitals(Duration::from_micros(30)); 
        }
        // Current avg should be 30us, which is > 25us (Degraded)
        let status = monitor.check_vitals(Duration::from_micros(30));
        assert!(matches!(status, HealthStatus::Degraded(_)));

        // 3. Critical (60us) -> Avg drifts higher
        for _ in 0..150 {
            monitor.check_vitals(Duration::from_micros(60));
        }
        let status_crit = monitor.check_vitals(Duration::from_micros(60));
        assert!(matches!(status_crit, HealthStatus::Critical(_)), "Should be Critical, got {:?}", status_crit);
    }
}
