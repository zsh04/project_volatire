use crate::governor::wave_legislator::WaveLegislator;
use std::collections::VecDeque;

const HISTORY_SIZE: usize = 10;
const SLIPPAGE_TOLERANCE_BPS: f64 = 2.0; // Basis points
const DRIFT_SENSITIVITY: f64 = 0.05; // How much each bad trade impacts drift

#[derive(Debug, Clone)]
pub struct TradeResult {
    pub predicted_price: f64,
    pub filled_price: f64,
    pub side: TradeSide, 
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeSide {
    Buy,
    Sell,
}

pub struct AuditLoop {
    history: VecDeque<TradeResult>,
    pub drift_score: f64, // 0.0 (Perfect) -> 1.0 (Broken)
}

impl AuditLoop {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(HISTORY_SIZE),
            drift_score: 0.0,
        }
    }

    /// Ingest a new trade result and update model drift metrics.
    pub fn register_trade(&mut self, result: TradeResult) {
        if self.history.len() >= HISTORY_SIZE {
            self.history.pop_front();
        }
        self.history.push_back(result);
        
        self.recalculate_drift();
    }

    fn recalculate_drift(&mut self) {
        if self.history.is_empty() {
            self.drift_score = 0.0;
            return;
        }

        let mut total_drift_impact = 0.0;

        for trade in &self.history {
            let slippage = match trade.side {
                TradeSide::Buy => trade.filled_price - trade.predicted_price,
                TradeSide::Sell => trade.predicted_price - trade.filled_price,
            };
            
            // Convert to basis points approx (assuming price around 100 for simplicity or just raw diff if generic)
            // For this logic, let's treat the inputs as raw price diffs. 
            // If slippage is positive (bad fill), it adds to drift.
            
            if slippage > 0.0 {
                // Penalize strict slippage
                total_drift_impact += DRIFT_SENSITIVITY;
            }
        }

        // Normalize drift score 0.0 to 1.0
        self.drift_score = total_drift_impact.min(1.0);
    }

    /// Automatically adjusts valid wave thresholds if drift is too high.
    pub fn recalibrate_legislator(&self, legislator: &mut WaveLegislator) -> bool {
        if self.drift_score > 0.3 {
            // Significant drift detected. Tighten the requirements.
            let current = legislator.tunneling_threshold();
            // Increase threshold by 5% roughly, maxing at 0.99
            let new_threshold = (current + 0.02).min(0.999);
            
            if (new_threshold - current).abs() > 0.001 {
                legislator.set_tunneling_threshold(new_threshold);
                return true; // Action taken
            }
        }
        // If drift is low, we could relax, but safety first: only tighten for now.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_calculation() {
        let mut audit = AuditLoop::new();
        
        // Good trades (No slippage or positive slippage)
        audit.register_trade(TradeResult {
            predicted_price: 100.0,
            filled_price: 100.0,
            side: TradeSide::Buy,
        });
        assert_eq!(audit.drift_score, 0.0);

        // Bad trade (Slippage)
        audit.register_trade(TradeResult {
            predicted_price: 100.0,
            filled_price: 101.0, // Bought higher than predicted
            side: TradeSide::Buy,
        });
        assert!(audit.drift_score > 0.0);
    }

    #[test]
    fn test_recalibration() {
        let mut audit = AuditLoop::new();
        let mut legislator = WaveLegislator::new(0.90);

        // Fill history with bad trades
        for _ in 0..8 {
            audit.register_trade(TradeResult {
                predicted_price: 100.0,
                filled_price: 105.0,
                side: TradeSide::Buy,
            });
        }

        assert!(audit.drift_score > 0.3);
        
        let initial_threshold = legislator.tunneling_threshold();
        let recalibrated = audit.recalibrate_legislator(&mut legislator);
        
        assert!(recalibrated);
        assert!(legislator.tunneling_threshold() > initial_threshold);
    }
}
