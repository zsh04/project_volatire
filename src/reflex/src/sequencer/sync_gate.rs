use std::time::{Duration, Instant};
use tracing::warn;

#[derive(Debug, Clone)]
pub enum SyncError {
    LatencyExceeded { duration_ms: f64, limit: f64 },
    PriceDrift { delta_bps: f64, limit: f64 },
    Staleness { lag_ticks: u64, limit: u64 },
}

pub struct SyncGate {
    max_latency_ms: f64,
    max_drift_bps: f64,
    max_staleness_ticks_low_entropy: u64,
    max_staleness_ticks_high_entropy: u64,
}

impl SyncGate {
    pub fn new() -> Self {
        Self {
            max_latency_ms: 500.0, // D-91 Atomic Clock
            max_drift_bps: 10.0,   // 0.1% Drift Detector
            max_staleness_ticks_low_entropy: 20,
            max_staleness_ticks_high_entropy: 5,
        }
    }

    /// Atomic Clock Check: Validates inference latency
    pub fn measure_latency(&self, start_time: Instant) -> Result<(), SyncError> {
        let duration = start_time.elapsed().as_secs_f64() * 1000.0;
        if duration > self.max_latency_ms {
            return Err(SyncError::LatencyExceeded {
                duration_ms: duration,
                limit: self.max_latency_ms,
            });
        }
        Ok(())
    }

    /// D-94: Late-Check Veto
    /// Checks L1 Order Book one last time before wire transmit.
    pub fn check_late_l1(&self, _price: f64) -> bool {
        // Mock L1 Check: Check if price has moved > 0.01% in last microsecond.
        // In simulation, we assume stability.
        true 
    }

    /// Drift Detector: Validates price movement during inference
    pub fn check_drift(&self, start_price: f64, current_price: f64) -> Result<(), SyncError> {
        let delta = (current_price - start_price).abs();
        let delta_bps = (delta / start_price) * 10000.0;
        
        if delta_bps > self.max_drift_bps {
            return Err(SyncError::PriceDrift {
                delta_bps,
                limit: self.max_drift_bps,
            });
        }
        Ok(())
    }

    /// Staleness Threshold: Validates sequence lag based on Regime
    pub fn check_staleness(&self, start_seq: u64, current_seq: u64, regime_id: u8) -> Result<(), SyncError> {
        if current_seq < start_seq {
            // Should be impossible if GSID is monotonic, but safety check
            return Ok(()); 
        }

        let lag = current_seq - start_seq;
        
        let limit = match regime_id {
            2 | 3 => self.max_staleness_ticks_high_entropy, // Turbulent/Violent (5 ticks)
            _ => self.max_staleness_ticks_low_entropy,      // Laminar/Unknown (20 ticks)
        };

        if lag > limit {
            return Err(SyncError::Staleness {
                lag_ticks: lag,
                limit,
            });
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_check() {
        let gate = SyncGate::new();
        let start = Instant::now();
        assert!(gate.measure_latency(start).is_ok());
    }

    #[test]
    fn test_drift_check() {
        let gate = SyncGate::new();
        let start_price = 100.0;
        
        // No drift
        assert!(gate.check_drift(start_price, 100.0).is_ok());
        
        // Small drift (0.05%) -> OK (Limit is 0.1% = 10 bps)
        assert!(gate.check_drift(start_price, 100.05).is_ok());
        
        // Large drift (0.2%) -> Err
        assert!(matches!(gate.check_drift(start_price, 100.20), Err(SyncError::PriceDrift{..})));
    }

    #[test]
    fn test_staleness_check() {
        let gate = SyncGate::new();
        let start_seq = 100;
        
        // Laminar (Regime 0) -> Limit 20
        assert!(gate.check_staleness(start_seq, 110, 0).is_ok());
        assert!(matches!(gate.check_staleness(start_seq, 125, 0), Err(SyncError::Staleness{..}))); // > 20

        // Turbulent (Regime 2) -> Limit 5
        assert!(gate.check_staleness(start_seq, 103, 2).is_ok());
        assert!(matches!(gate.check_staleness(start_seq, 106, 2), Err(SyncError::Staleness{..}))); // > 5
    }
}
