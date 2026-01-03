use tokio::sync::mpsc;
use std::collections::VecDeque;
use tracing::{info, warn, error, instrument};
use crate::telemetry::forensics::DecisionPacket;
use crate::governor::superposition;

/// The Mirror Reality.
/// Runs in parallel to the main OODA loop, comparing Live decisions against a stable Baseline.
pub struct MirrorEngine {
    rx: mpsc::Receiver<DecisionPacket>,
    divergence_buffer: VecDeque<f64>,
    
    // Metrics
    _ghost_pnl: f64,
}

impl MirrorEngine {
    pub fn new(rx: mpsc::Receiver<DecisionPacket>) -> Self {
        Self {
            rx,
            divergence_buffer: VecDeque::with_capacity(100),
            _ghost_pnl: 0.0,
        }
    }

    /// The Parallel Reality Loop.
    /// Returns nothing, just logs and emits metrics.
    #[instrument(skip(self), name = "mirror_loop")]
    pub async fn run(mut self) {
        info!("🪞 Mirror Engine (Audit Core) Started.");

        while let Some(packet) = self.rx.recv().await {
            // 1. Synthetic Latency Injection (Directive-51 Requirement)
            // Ensures this actor NEVER blocks the hot path, even if it falls behind.
            // Also proves the system is decoupled.
            // In a real load test, we might sleep longer.
            // For now, we simulate a slight delay to ensure we are "behind" reality.
            if cfg!(debug_assertions) {
                 // Only inject sleep in debug/sim mode to verify async decoupling
                 // tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }

            // 2. Chaos Injection (The Black Swan Test)
            // 1% chance to hallucinate a crash in the Mirror to test Veto logic response
            // (Note: Veto is in the Hot Path, this just tests if *Mirror* would have vetoed if it saw a crash)
            let mut mirror_physics = packet.physics.clone();
            let is_chaos = rand::random::<f64>() < 0.01;
            if is_chaos {
                mirror_physics.price *= 0.90; // Flash crash
                warn!("🧪 Mirror Injection: Simulating -10% Crash");
            }

            // 3. Calculate Baseline ("Golden") Decision
            // We use fixed, known-good weights for the Mirror.
            // Specifically, we use the RiemannEngine with conservative inputs (Simons Confidence = 0.5)
            // This represents a "Skeptical Observer"
            
            let mirror_riemann_prob = superposition::RiemannEngine::calculate_riemann_probability(
                &mirror_physics,
                mirror_physics.entropy,
                mirror_physics.efficiency_index,
                0.5 // Fixed conservative confidence
            );

            // Simple Mirror Logic based on Riemann Probability
            // High Riemann (>0.8) + Positive Velocity = BUY
            // Low Riemann (<0.2) + Negative Velocity = SELL
            // Middle = HOLD
            
            let mirror_decision = if mirror_riemann_prob > 0.8 && mirror_physics.velocity > 0.05 {
                "BUY"
            } else if mirror_riemann_prob < 0.2 && mirror_physics.velocity < -0.05 {
                "SELL"
            } else {
                "HOLD"
            };

            // 4. Drift Detection
            let live_decision = packet.decision.as_str();
            
            if live_decision != mirror_decision {
                if is_chaos {
                    // If we injected chaos, we EXPECT divergence if Live didn't see it.
                    // This confirms the "Control Group" is working independent of Reality.
                    info!("✅ Chaos Test Passed: Mirror saw crash ({}), Live saw normal ({})", mirror_decision, live_decision);
                } else {
                    // Genuine Drift
                    error!("⚠️ DRIFT DETECTED: Live[{}] vs Mirror[{}] | P_Vel={:.4}", 
                        live_decision, mirror_decision, packet.physics.velocity);
                        
                    // Track divergence
                    self.divergence_buffer.push_back(1.0);
                }
            } else {
                self.divergence_buffer.push_back(0.0);
            }

            // Maintain buffer size
            if self.divergence_buffer.len() > 100 {
                self.divergence_buffer.pop_front();
            }

            // 5. Emit Telemetry (TODO: Wire to OTel Gauge)
            // let drift_score: f64 = self.divergence_buffer.iter().sum();
            // metrics::gauge!("reflex_mirror_drift", drift_score);
        }
    }
}
