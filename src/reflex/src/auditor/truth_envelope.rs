use serde::{Deserialize, Serialize};

/// Directive-87: The Truth Envelope
/// Captures "Hard Telemetry" to ground LLM reasoning.
/// This struct is injected into the System Prompt and used for post-inference validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthEnvelope {
    pub timestamp: f64,
    
    // Simons Physics Vectors
    pub velocity: f64,
    pub acceleration: f64,
    pub jerk: f64,
    
    // Hypatia Sentiment
    pub sentiment_score: f64, // -1.0 to 1.0
    
    // Market Data (The Anchor)
    pub mid_price: f64,
    pub bid_ask_spread: f64,
    
    // Governance / State
    pub regime_id: u8, // Deterministic Regime ID
    pub sequence_id: u64,
}

impl Default for TruthEnvelope {
    fn default() -> Self {
        Self {
            timestamp: 0.0,
            velocity: 0.0,
            acceleration: 0.0,
            jerk: 0.0,
            sentiment_score: 0.0,
            mid_price: 0.0,
            bid_ask_spread: 0.0,
            regime_id: 0,
            sequence_id: 0,
        }
    }
}
