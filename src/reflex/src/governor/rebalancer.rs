use std::collections::VecDeque;
use tracing::{info, warn, error};

// D-90: Recursive Risk Re-balancer (The Governor)
pub struct Rebalancer {
    pub fidelity: f64,
    start_equity: f64,
    slippage_window: VecDeque<f64>, // Stores delta % (Actual - Expected) / Expected
    max_mdd_percent: f64,
}

impl Rebalancer {
    pub fn new(start_equity: f64) -> Self {
        Self {
            fidelity: 1.0,
            start_equity,
            slippage_window: VecDeque::with_capacity(10),
            max_mdd_percent: 0.15, // 15% Max Session Drawdown
        }
    }

    /// Update Fidelity based on Nullification (Punish)
    pub fn punish_nullification(&mut self) {
        self.fidelity = (self.fidelity - 0.05).max(0.0);
        warn!("📉 FIDELITY DROP: Nullification detected. F={:.4}", self.fidelity);
    }

    /// Update Fidelity based on Success (Reward)
    pub fn reward_success(&mut self) {
        self.fidelity = (self.fidelity + 0.01).min(1.0);
    }

    /// Calculate Adjusted Size
    pub fn get_safe_size(&self, standard_size: f64) -> f64 {
        if self.fidelity < 0.5 {
            warn!("🛑 FIDELITY CRITICAL (F={:.2} < 0.5). OBSERVATION MODE LOCKED.", self.fidelity);
            return 0.0;
        }
        standard_size * self.fidelity
    }

    /// Record a trade fill and check slippage anomalies
    /// Returns true if Hot-Swap is recommended (Slippage > 2 sigma)
    pub fn record_fill(&mut self, expected_price: f64, fill_price: f64) -> bool {
        let delta = (fill_price - expected_price).abs() / expected_price;
        
        if self.slippage_window.len() >= 10 {
            self.slippage_window.pop_front();
        }
        self.slippage_window.push_back(delta);

        // Analyze if window is full
        if self.slippage_window.len() == 10 {
            let mean: f64 = self.slippage_window.iter().sum::<f64>() / 10.0;
            let variance: f64 = self.slippage_window.iter().map(|value| {
                let diff = mean - *value;
                diff * diff
            }).sum::<f64>() / 10.0;
            let std_dev = variance.sqrt();

            if delta > mean + (2.0 * std_dev) {
                error!("⚠️ SLIPPAGE ANOMALY: Delta {:.4} > 2σ (Mean {:.4}, σ {:.4})", delta, mean, std_dev);
                return true; // Hot-Swap
            }
        }
        false
    }

    /// Check Omega Kill-Switch (Session Drawdown)
    /// Returns true if OMEGA Triggered (KILL)
    pub fn check_omega(&self, current_equity: f64) -> bool {
        let drawdown = (self.start_equity - current_equity) / self.start_equity;
        if drawdown > self.max_mdd_percent {
            error!("💀 OMEGA KILL-SWITCH TRIGGERED: Drawdown {:.2}% > Max {:.2}%", drawdown * 100.0, self.max_mdd_percent * 100.0);
            return true;
        }
        false
    }
}
