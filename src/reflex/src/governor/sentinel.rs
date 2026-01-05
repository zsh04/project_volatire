use std::time::Instant;
use std::collections::VecDeque;

/// Vitality Status for the System
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VitalityStatus {
    Optimal,
    Degraded,
    Critical,
}

impl VitalityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            VitalityStatus::Optimal => "OPTIMAL",
            VitalityStatus::Degraded => "DEGRADED",
            VitalityStatus::Critical => "CRITICAL",
        }
    }
}

pub struct Sentinel {
    // Config
    jitter_threshold_us: f64,
    latency_threshold_us: f64,
    
    // State
    last_tick: Instant,
    history: VecDeque<f64>, // Cycle times in us
    last_instability: Instant, // Timestamp of last degraded/critical event
    
    // Current Metrics
    pub current_latency_us: f64,
    pub current_jitter_us: f64,
    pub status: VitalityStatus,
}

impl Sentinel {
    pub fn new() -> Self {
        Self {
            jitter_threshold_us: 50.0, // 50 microseconds (Directive-80)
            latency_threshold_us: 1000.0, // 1ms target for loop (Simulated/Real)
            last_tick: Instant::now(),
            history: VecDeque::with_capacity(100),
            last_instability: Instant::now(), // Assume unstable at boot
            current_latency_us: 0.0,
            current_jitter_us: 0.0,
            status: VitalityStatus::Optimal,
        }
    }

    /// Check if system has been stable (Optimal) for at least the given duration
    pub fn is_stable_for(&self, duration: std::time::Duration) -> bool {
        self.status == VitalityStatus::Optimal && self.last_instability.elapsed() >= duration
    }

    /// Call this at the start/end of every OODA loop cycle
    pub fn tick(&mut self) -> VitalityStatus {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick).as_micros() as f64;
        self.last_tick = now;

        // Update History
        if self.history.len() >= 100 {
            self.history.pop_front();
        }
        self.history.push_back(elapsed);
        
        // Calculate Metrics
        self.current_latency_us = elapsed;
        
        // Calculate Jitter (Standard Deviation of Cycle Time)
        let sum: f64 = self.history.iter().sum();
        let count = self.history.len() as f64;
        let mean = sum / count;
        
        // Only calculate significant jitter if we have enough samples
        if count > 10.0 {
            let variance_sum: f64 = self.history.iter().map(|&x| (x - mean).powi(2)).sum();
            self.current_jitter_us = (variance_sum / count).sqrt();
        } else {
            self.current_jitter_us = 0.0;
        }

        // Determine Status
        let new_status = if self.current_jitter_us > self.jitter_threshold_us * 2.0 {
            VitalityStatus::Critical
        } else if self.current_jitter_us > self.jitter_threshold_us {
            VitalityStatus::Degraded
        } else {
            VitalityStatus::Optimal
        };

        if new_status != VitalityStatus::Optimal {
             self.last_instability = Instant::now();
        }
        
        self.status = new_status;
        self.status
    }
}
