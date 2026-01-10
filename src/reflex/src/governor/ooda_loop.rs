use std::time::{Duration, Instant};

// --- Data Structures ---

pub use crate::feynman::PhysicsState;

use crate::telemetry::forensics::DecisionPacket;
use tokio::sync::mpsc;
use opentelemetry::trace::TraceContextExt;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[derive(Debug, Clone)]
pub struct OODAState {
    pub physics: PhysicsState,
    pub sentiment_score: Option<f64>, // Narrative (Hypatia)
    pub nearest_regime: Option<String>, // Memory (LanceDB)
    pub vector_distance: Option<f64>, // Similarity Score
    pub oriented_at: Instant,
    pub trace_id: String, // Traceability link
    pub brain_latency: Option<f64>, // ms
}

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    Buy(f64),  // Size
    Sell(f64), // Size
    Hold,
    Reduce(f64), // Risk Off
    Halt, // Nuclear option
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub action: Action,
    pub reason: String,
    pub confidence: f64,
}

impl Decision {
    pub fn default_hold() -> Self {
        Self {
            action: Action::Hold,
            reason: "Default/Fallback Hold".to_string(),
            confidence: 1.0,
        }
    }
}

impl Default for OODAState {
    fn default() -> Self {
        Self {
            physics: PhysicsState::default(),
            sentiment_score: None,
            nearest_regime: None,
            vector_distance: None,
            oriented_at: Instant::now(),
            trace_id: String::new(),
            brain_latency: None,
        }
    }
}

// --- The Governor ---

use crate::governor::provisional::ProvisionalExecutive;
use crate::brain::veto_gate::VetoGate;
use crate::auditor::firewall::{Firewall, LlmInferenceResponse, FirewallError};
use crate::auditor::truth_envelope::TruthEnvelope;
use crate::auditor::nullifier::Nullifier; // D-88
use crate::auditor::red_team::RedTeam; // D-93
use crate::governor::ensemble_manager::EnsembleManager; // D-95
use crate::governor::health::PhoenixMonitor; // D-96

pub use crate::sequencer::sync_gate::SyncGate;
use crate::sequencer::shadow_gate::ShadowGate; // D-91
use crate::gateway::binary_packer::BinaryPacker; // D-94

pub struct OODACore {
    // Mock clients for now. In prod, these would be Redis/LanceDB clients.
    pub jitter_threshold: Duration,
    pub provisional: ProvisionalExecutive,
    pub veto_gate: VetoGate,
    pub firewall: Firewall, // D-87
    pub nullifier: Nullifier, // D-88
    pub red_team: RedTeam, // D-93
    pub sync_gate: SyncGate, // D-91
    pub shadow_gate: ShadowGate, // D-92
    pub binary_packer: BinaryPacker, // D-94
    pub ensemble_manager: EnsembleManager, // D-95
    pub phoenix_monitor: PhoenixMonitor, // D-96
    pub symbol: String,
    pub forensic_tx: Option<mpsc::Sender<DecisionPacket>>,
    pub mirror_tx: Option<mpsc::Sender<DecisionPacket>>,
    pub decay_tx: Option<mpsc::Sender<DecisionPacket>>,
}

use crate::client::BrainClient;

impl OODACore {
    pub fn new(
        symbol: String,
        forensic_tx: Option<mpsc::Sender<DecisionPacket>>,
        mirror_tx: Option<mpsc::Sender<DecisionPacket>>,
        decay_tx: Option<mpsc::Sender<DecisionPacket>>
    ) -> Self {
        Self {
            jitter_threshold: Duration::from_millis(20),
            provisional: ProvisionalExecutive::new(),
            veto_gate: VetoGate::new(),
            firewall: Firewall::new(), // D-87
            nullifier: Nullifier::new(), // D-88
            red_team: RedTeam::new(), // D-93
            sync_gate: SyncGate::new(), // D-91
            shadow_gate: ShadowGate::new(symbol.clone()), // D-92
            binary_packer: BinaryPacker::new(), // D-94
            ensemble_manager: EnsembleManager::new(), // D-95
            phoenix_monitor: PhoenixMonitor::new(), // D-96
            forensic_tx,
            mirror_tx,
            decay_tx,
            symbol,
        }
    }



    /// OBSERVE -> ORIENT
    /// Fuses real-time Physics with asynchronous Semantic checks.
    /// Implements "Jitter" Fallback: If Semantics take too long, we proceed with Safety.
    /// Implements "Cognitive Firewall" (D-87): Validates Brain response against Hard Telemetry.
    /// Implements "Semantic Nullification" (D-88): Purges corrupted reasoning.
    /// Implements "Semantic Nullification" (D-88): Purges corrupted reasoning.
    #[tracing::instrument(skip(self, client))]
    pub async fn orient(&mut self, physics: PhysicsState, regime_id: u8, client: Option<&mut BrainClient>, legislative_bias: String) -> OODAState {
        let _start = Instant::now();
        // D-92: Shadow Gate Reality Check
        // Check for fills on pending virtual orders against current physics price
        self.shadow_gate.check_fills(physics.price);
        
        // Capture TraceID from current span
        let span = tracing::Span::current();
        let cx = span.context();
        let trace_id = cx.span().span_context().trace_id().to_string();

        // 1. Asynchronous Fetch Logic
        let (sentiment, regime, latency, distance) = if let Some(c) = client {
            // LIVE PATH (D-54)
            // D-87: COGNITIVE FIREWALL - Construct Truth Envelope
            let mut truth = TruthEnvelope {
                timestamp: physics.timestamp,
                velocity: physics.velocity,
                acceleration: physics.acceleration,
                jerk: physics.jerk,
                sentiment_score: 0.0, // Initial seed
                mid_price: physics.price,
                bid_ask_spread: physics.bid_ask_spread,
                regime_id,
                sequence_id: 0,      // TODO: Pass sequence_id
            };
            
            // D-93: ADVERSARIAL STRESS INJECTION (The Red-Teamer)
            // We mutate the Envelope BEFORE sending to Brain or verifying.
            self.red_team.inject_chaos(&mut truth);

            // D-95: THE CHAMELEON (Multi-Regime Ensemble)
            // 1. Identify Target Adapter from PREVIOUS Regime (or best guess)
            // Note: In a real loop, we'd use the regime from the LAST cycle to pick the adapter for THIS cycle,
            // or use a "Fast" regime classifier here.
            // For now, we update based on the passed `regime_id` (assuming it came from heavy DB lookup or cache).
            let current_regime_name = match regime_id {
                0 => "Laminar",
                4 => "Turbulent",
                5 => "Violent",
                _ => "Unknown",
            };
            self.ensemble_manager.update_regime(current_regime_name);
            let _active_adapter = self.ensemble_manager.get_active_adapter();

            // TODO: Pass `active_adapter` to client.get_context()
            // For now, we just log it in the trace context or debug 
            // tracing::debug!("Using Adapter: {}", active_adapter);
            
            // Enforce Jitter Budget (e.g., 20ms) via Timeout
            match tokio::time::timeout(
                self.jitter_threshold,
                c.get_context(&truth, &legislative_bias) // D-107: Pass Bias
            ).await {
                Ok(Ok(ctx)) => {
                    // D-91: TEMPORAL SYNC-GATE
                    // 1. Latency Check (Atomic Clock)
                    if let Err(e) = self.sync_gate.measure_latency(_start) {
                        tracing::warn!("BTC-91 SyncGate Violation (Latency): {:?}", e);
                        (None, None, None, None)
                    } else {
                        // Map Proto ContextResponse to LlmInferenceResponse for validation
                        // We treat context info as "inference" for validation purposes
                        let llm_resp = LlmInferenceResponse {
                            reasoning: ctx.reasoning.clone(), 
                            decision: "CONTEXT".to_string(),
                            confidence: 1.0,
                            referenced_price: if ctx.referenced_price > 0.0 { Some(ctx.referenced_price) } else { None },
                            regime_classification: Some(ctx.nearest_regime.clone()),
                        };

                    match self.firewall.validate(&llm_resp, &truth) {
                        Ok(_) => {
                            self.nullifier.reset_continuity(); // D-88: Success resets counter
                            let lat = ctx.computation_time_ns as f64 / 1_000_000.0;
                            (Some(ctx.sentiment_score), Some(ctx.nearest_regime), Some(lat), Some(ctx.regime_distance))
                        },
                        Err(e) => {
                            // D-88: NULLIFICATION "THE ERASER"
                            let triggered_amr = self.nullifier.nullify(e, ctx.reasoning.clone());
                            if triggered_amr {
                                tracing::warn!("⚡ AMR: BRAIN RESET REQUESTED");
                                // TODO: Actually trigger reset callback or signal if needed here
                            }
                            
                            // Return BLIND STATE (Nullified)
                            (None, None, None, None)
                        }
                    }
                } // End SyncGate Else
                },
                Ok(Err(e)) => {
                    tracing::warn!("Brain Error: {}", e);
                    (None, None, None, None) // Error -> Blind
                },
                Err(_) => {
                    tracing::warn!("Brain Timeout (Jitter Violated)");
                    (None, None, None, None) // Timeout -> Blind
                }
            }
        } else {
            // SIM PATH (Mock)
            self.fetch_semantics_simulated()
        };

        // 2. Final Jitter Check (Redundant if timeout works, but good for local processing tracking)
        let loop_latency = _start.elapsed();
        
        // D-96: METABOLIC CHECK (Phoenix Monitor)
        use crate::governor::health::HealthStatus;
        match self.phoenix_monitor.check_vitals(loop_latency) {
            HealthStatus::Critical(msg) => {
                tracing::error!("🔥 PHOENIX CRITICAL: {}", msg);
                self.phoenix_monitor.initiate_handoff();
            },
            HealthStatus::Degraded(msg) => {
                tracing::warn!("⚠️ PHOENIX WARNING: {}", msg);
            },
            HealthStatus::Healthy => {}
        }
        
        OODAState {
            physics,
            sentiment_score: sentiment,
            nearest_regime: regime,
            vector_distance: distance,
            oriented_at: Instant::now(),
            trace_id,
            brain_latency: latency,
        }
    }

    /// Mocks the external fetch to LanceDB / DistilBERT
    fn fetch_semantics_simulated(&self) -> (Option<f64>, Option<String>, Option<f64>, Option<f64>) {
        // Simulate variability. 
        // Most of the time it's fast (cache), sometimes it lags.
        // For deterministic logic testing (not verify), we return instant mock.
        // For stress testing, we'd inject sleep.
        // Hardcoding a "Fast Path" scenario for default logic flow.
        (Some(-0.8), Some("Liquidity Crisis 2020".to_string()), Some(12.5), Some(0.12))
    }

    /// ORIENT -> DECIDE
    /// Weighted Voting: Simons (Physics), Kepler (MeanRev), Hypatia (Sentiment)
    /// Now includes Directive-43: Provisional Risk Sizing
    /// Now includes Directive-43: Provisional Risk Sizing
    /// Now includes Directive-45: Nuclear Veto (Double-Key)
    #[tracing::instrument(skip(self))]
    pub fn decide(&mut self, state: &OODAState, legislation: &crate::governor::legislator::LegislativeState) -> Decision {
        let physics = &state.physics;
        
        // 1. Update Sentinel Components
        // Feed real sentiment to VetoGate if available
        if let Some(s) = state.sentiment_score {
            self.veto_gate.update_sentiment(s);
        }
        
        // 2. Check Nuclear Veto (Double-Key)
        // Need Omega Ratio. Calculating simplistic Omega or passing it.
        // For now, assume Omega > 1.0 (Safe) unless we calculate it.
        // If we want to test D-45, we need to pass a mock Omega.
        // Let's assume passed in State or computed.
        // Mocking Omega = 1.2 normally.
        let mock_omega = 1.2; 
        if self.veto_gate.check_hard_stop(physics, mock_omega) {
             let d = Decision {
                action: Action::Halt,
                reason: "NUCLEAR VETO: Sentiment + Physics Collapse".to_string(),
                confidence: 1.0,
            };
            self.log_forensics(state, &d, 0.0); // 0.0 risk
            return d;
        }

        // 3. Update Provisional Executive
        // Use real physics metrics for risk sizing
        let entropy = physics.entropy;
        let efficiency = physics.efficiency_index;
        
        let _promoted = self.provisional.update(physics, entropy, efficiency);
        let max_risk = self.provisional.get_current_max_risk();


        // 4. Initial Signal from Simons (Pattern/Physics)
        // Simple heuristic: Positive Acceleration = Buy Signal
        let mut base_signal: f64 = if physics.acceleration > 0.0 { 1.0 } else { -1.0 };
        
        // 5. Apply Soft Veto (Qualitative Filter)
        // If we have a semantic score
        if let Some(sentiment) = state.sentiment_score {
            // VETO: Physics says Buy (1.0), but Sentiment is Negative (< -0.5)
            // Note: This is separate from Nuclear Veto. This is just "Don't Buy".
            if base_signal > 0.0 && sentiment < -0.5 {
                 let d = Decision {
                    action: Action::Hold, // Or Reduce
                    reason: format!("VETO: Hypatia Sentiment ({}) overruled Physics.", sentiment),
                    confidence: 1.0, 
                };
                self.log_forensics(state, &d, max_risk);
                return d;
            }
        } else {
            // MODE: Jitter Fallback (Blind Physics)
            // Apply "Risk Floor" -> Reduce conviction
            base_signal *= 0.5; // Reduce sizing by half if flying blind
        }
        
        // 6. Final Decision Construction w/ Provisional Sizing
        let mut decision = if base_signal >= 0.5 {
            Decision {
                action: Action::Buy(max_risk * base_signal.abs()), // Apply Provisional Limit
                reason: format!("Physics & Sentiment Aligned. Risk Tier: {}", self.provisional.current_tier_index),
                confidence: 0.9,
            }
        } else if base_signal <= -0.5 {
            Decision {
                action: Action::Sell(max_risk * base_signal.abs()), // Apply Provisional Limit
                reason: "Physics Bearish".to_string(),
                confidence: 0.9,
            }
        } else {
            Decision {
                action: Action::Hold,
                reason: "Uncertain / Risk Floor".to_string(),
                confidence: 0.5,
            }
        };

        // D-107: LEGISLATIVE VETO (Rust Layer)
        use crate::governor::legislator::StrategicBias;
        match legislation.bias {
            StrategicBias::LongOnly => {
                 if let Action::Sell(_) = decision.action {
                     // Override to Hold
                     tracing::warn!("🚫 RUST VETO: Sell Blocked by LongOnly Legislation");
                     decision = Decision {
                         action: Action::Hold,
                         reason: "Legislative Veto: Long Only".to_string(),
                         confidence: 1.0,
                     };
                 }
            },
            StrategicBias::ShortOnly => {
                 if let Action::Buy(_) = decision.action {
                     tracing::warn!("🚫 RUST VETO: Buy Blocked by ShortOnly Legislation");
                     decision = Decision {
                         action: Action::Hold,
                         reason: "Legislative Veto: Short Only".to_string(),
                         confidence: 1.0,
                     };
                 }
            },
            StrategicBias::Neutral => {}
        }


        self.log_forensics(state, &decision, max_risk);
        decision
    }

    fn log_forensics(&self, state: &OODAState, decision: &Decision, _max_risk: f64) {
        let mut packet = DecisionPacket {
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64(),
            trace_id: state.trace_id.clone(),
            physics: state.physics.clone(),
            sentiment: state.sentiment_score.unwrap_or(0.0),
            vector_distance: state.vector_distance.unwrap_or(0.0),
            quantile_score: self.provisional.current_tier_index as i32,
            decision: format!("{:?}", decision.action),
            operator_hash: String::new(),
        };
        packet.seal();
        
        // 1. Send to The Scribe (Forensics) - Fire & Forget
        if let Some(tx) = &self.forensic_tx {
             if let Err(e) = tx.try_send(packet.clone()) {
                tracing::warn!("⚠️ Forensic Log Dropped (Channel Full): {}", e);
            }
        }

        // 2. Send to The Mirror (Audit Core) - Fire & Forget
        if let Some(tx) = &self.mirror_tx {
             if let Err(e) = tx.try_send(packet.clone()) {
                tracing::warn!("⚠️ Mirror Packet Dropped (Channel Full): {}", e);
            }
        }

        // 3. Send to The Decay Monitor (Directive-52) - Fire & Forget
        if let Some(tx) = &self.decay_tx {
             if let Err(e) = tx.try_send(packet) {
                tracing::warn!("⚠️ Decay Packet Dropped (Channel Full): {}", e);
            }
        }
    }

    /// DECIDE -> ACT
    /// Atomic execution (mocked)
    pub fn act(&mut self, decision: Decision, current_price: f64) {
         // D-92: Shadow Mode Hook
         // We submit every decision to the Shadow Gate for virtual execution
         self.shadow_gate.submit_order(&decision, current_price);
         
         // D-94: ADAPTIVE LATENCY HARVEST (The Shortcut)
         // Hot-Path Zero-Copy Serialization
         match decision.action {
             Action::Buy(qty) => {
                 // D-94 Part C: Late-Check Veto
                 if self.sync_gate.check_late_l1(current_price) {
                     let _packet = self.binary_packer.pack_buy(current_price, qty);
                     // In prod: unsafe { socket.send(_packet) };
                     // tracing::info!("⚡ SENT BINARY BUY: {} bytes", _packet.len());
                 } else {
                     tracing::warn!("⛔ D-94 PRE-FLIGHT ABORT: Price Moved");
                 }
             },
             Action::Sell(qty) => {
                 if self.sync_gate.check_late_l1(current_price) {
                     let _packet = self.binary_packer.pack_sell(current_price, qty);
                     // In prod: unsafe { socket.send(_packet) };
                     // tracing::info!("⚡ SENT BINARY SELL: {} bytes", _packet.len());
                 } else {
                     tracing::warn!("⛔ D-94 PRE-FLIGHT ABORT: Price Moved");
                 }
             },
             _ => {}
         }

        if let Action::Halt = decision.action {
            // In prod: Panic / Kill Switch
            println!("!!! SYSTEM SUPER-HALT !!!");
        }
        // In prod: send to Order Gateway
        // println!("ACT: {:?}", decision); 
    }
}

// --- Benchmarks & Verification ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governor::legislator::LegislativeState;

    #[tokio::test]
    async fn test_veto_logic() {
        let mut core = OODACore::new("BTC-USDT".to_string(), None, None, None);
        
        // Case: Bullish Physics
        let physics = PhysicsState {
            price: 50000.0,
            velocity: 10.0,
            acceleration: 5.0, // Bullish
            jerk: 0.1,
            ..Default::default()
        };

        // Standard Orient (Simulated)
        let state = core.orient(physics, 0, None, "NEUTRAL".to_string()).await;
        
        // Decide
        let decision = core.decide(&state, &LegislativeState::default());
        
        // EXPECTATION: HOLD/VETO because Sentiment is Negative (-0.8)
        match decision.action {
            Action::Hold => assert!(decision.reason.contains("VETO"), "Should be vetoed by Hypatia"),
            _ => panic!("Failed to veto bullish physics with negative sentiment! Got: {:?}", decision),
        }
    }

    #[tokio::test]
    async fn test_jitter_fallback_logic() {
        let mut core = OODACore::new("BTC-USDT".to_string(), None, None, None);
        let physics = PhysicsState {
            price: 50000.0,
            velocity: 10.0,
            acceleration: 5.0,
            jerk: 0.1,
            ..Default::default()
        };

        // Manually construct a "Blind" State (Jitter Fallback)
        let blind_state = OODAState {
            physics: physics.clone(),
            sentiment_score: None,
            nearest_regime: None,
            vector_distance: None,
            oriented_at: Instant::now(),
            trace_id: "test_trace".to_string(),
            brain_latency: None,
        };

        let decision = core.decide(&blind_state, &LegislativeState::default());
        
        // Expectation: Buy, but with Reduced Size/Confidence (0.5 multiplier)
        if let Action::Buy(pct) = decision.action {
            // Current Max Risk starts at 0.01.
            // Jitter reduces base_signal by half (0.5).
            // Result size = 0.01 * 0.5 = 0.005.
            assert!(pct <= 0.0051, "Risk Floor not applied! Expected <= 0.005, got {}", pct);
        } else {
            panic!("Should still buy on logic, just smaller.");
        }
    }

    #[tokio::test]
    async fn test_cycle_latency() {
        let mut core = OODACore::new("BTC-USDT".to_string(), None, None, None);
        let physics = PhysicsState {
            price: 50000.0,
            velocity: 0.0,
            acceleration: 0.0,
            jerk: 0.0,
            ..Default::default()
        };

        let start = Instant::now();
        for _ in 0..10_000 {
            // Using logic internal simulation for speed test
            let state = core.orient(physics.clone(), 0, None, "NEUTRAL".to_string()).await;
            let dec = core.decide(&state, &LegislativeState::default());
            core.act(dec, physics.price);
        }
        let total = start.elapsed();
        let per_op = total / 10_000;
        
        println!("Mean Cycle Time: {:?}", per_op);
        
        // Enforce < 150ms per cycle
        assert!(per_op < Duration::from_millis(150), "Cycle too slow!");
    }
}
