use std::collections::VecDeque;
use tokio::sync::mpsc;
use opentelemetry::{global, metrics::Histogram};
use tracing::{info, warn, instrument};
use crate::telemetry::forensics::DecisionPacket;

/// Represents the reality of a trade execution (Fill).
#[derive(Debug, Clone)]
pub struct FillPacket {
    pub trace_id: String,
    pub fill_price: f64,
    pub quantity: f64,
    pub timestamp: f64,
}

/// Record for a single matched trade (Decision + Fill).
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TradeRecord {
    trace_id: String,
    expected_price: f64,
    realized_price: f64,
    slippage: f64,
    decay_pct: f64,
    jerk_at_decision: f64,
}

/// The Decay Monitor (Alpha Validation Engine).
/// Compares Intent (Decision) vs Reality (Fill) to calculate Alpha Decay.
pub struct DecayMonitor {
    // Input Streams
    decision_rx: mpsc::Receiver<DecisionPacket>,
    fill_rx: mpsc::Receiver<FillPacket>,
    
    // Internal State
    pending_decisions: std::collections::HashMap<String, DecisionPacket>,
    trade_window: VecDeque<TradeRecord>,
    rolling_decay: f64,
    
    // Metrics
    alpha_decay_histogram: Histogram<f64>,
}

impl DecayMonitor {
    pub fn new(decision_rx: mpsc::Receiver<DecisionPacket>, fill_rx: mpsc::Receiver<FillPacket>) -> Self {
        let meter = global::meter("reflex_decay");
        let alpha_decay_histogram = meter
            .f64_histogram("alpha_decay_percent")
            .with_description("Rolling Alpha Decay Distribution (Expected vs Realized)")
            .init();
            
        Self {
            decision_rx,
            fill_rx,
            pending_decisions: std::collections::HashMap::new(),
            trade_window: VecDeque::with_capacity(100),
            rolling_decay: 0.0,
            alpha_decay_histogram,
        }
    }

    pub async fn run(mut self) {
        info!("📉 Decay Monitor Online");

        loop {
            tokio::select! {
                // Handle new Decision (Intent)
                Some(decision) = self.decision_rx.recv() => {
                    // Only track decisions that result in trades (BUY/SELL)
                    if decision.decision == "BUY" || decision.decision == "SELL" {
                        self.pending_decisions.insert(decision.trace_id.clone(), decision);
                    }
                }
                
                // Handle new Fill (Reality)
                Some(fill) = self.fill_rx.recv() => {
                    self.process_fill(fill);
                }
            }
        }
    }

    #[instrument(skip(self))]
    fn process_fill(&mut self, fill: FillPacket) {
        if let Some(decision) = self.pending_decisions.remove(&fill.trace_id) {
            
            // 1. Calculate Decay
            let expected = decision.physics.price; // Price at decision time
            let realized = fill.fill_price;
            
            // Decay is bad if it moves against us.
            // BUY: Expected 100, Fill 101 -> Decay positive (bad)
            // SELL: Expected 100, Fill 99 -> Decay positive (bad)
            
            let decay_raw = match decision.decision.as_str() {
                "BUY" => realized - expected,
                "SELL" => expected - realized,
                _ => 0.0,
            };
            
            let decay_pct = if expected != 0.0 {
                decay_raw / expected
            } else {
                0.0
            };

            // 2. Jerk Filter (Zero-False-Positive Safety)
            // If Jerk was high at decision time, high slippage is expected.
            // We discount the decay impact in these volatile moments.
            let adjusted_decay = if decision.physics.jerk.abs() > 50.0 {
                0.0 // Ignore decay during extreme jerk
            } else {
                decay_pct
            };

            // 3. Update Window
            let record = TradeRecord {
                trace_id: fill.trace_id.clone(),
                expected_price: expected,
                realized_price: realized,
                slippage: decay_raw,
                decay_pct: adjusted_decay,
                jerk_at_decision: decision.physics.jerk,
            };

            if self.trade_window.len() >= 100 {
                self.trade_window.pop_front();
            }
            self.trade_window.push_back(record);

            // 4. Calculate Rolling Decay (Avg of window)
            let sum_decay: f64 = self.trade_window.iter().map(|r| r.decay_pct).sum();
            let count = self.trade_window.len();
            if count > 0 {
                self.rolling_decay = sum_decay / count as f64;
            }

            // 5. Emit Telemetry
            self.alpha_decay_histogram.record(self.rolling_decay, &[]);

            // 6. Check Fail-Safe Trigger
            if self.rolling_decay > 0.15 {
                warn!(
                    decay = self.rolling_decay, 
                    "🚨 ALPHA DECAY CRITICAL (>15%). REQUIRING DEMOTION."
                );
                // In a full implementation, this sends a command to OODA/SharedState.
                // For now, we just log the requirement as per prompt acceptance criteria.
            }
        } else {
            warn!("Orphaned Fill received: {}", fill.trace_id);
        }
    }
}
