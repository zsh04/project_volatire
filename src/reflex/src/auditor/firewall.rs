use crate::auditor::truth_envelope::TruthEnvelope;
use serde::{Deserialize, Serialize};

/// Standardized LLM Response Schema (must match what we expect from Python/Brain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmInferenceResponse {
    pub reasoning: String,
    pub decision: String, // "BUY", "SELL", "HOLD"
    pub confidence: f64,
    // Optional fields the model *might* halllucinate, or return if asked
    pub referenced_price: Option<f64>,
    pub regime_classification: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FirewallError {
    NumericHallucination { claimed: f64, truth: f64, delta: f64 },
    RegimeMismatch { claimed: String, truth_id: u8 },
    SchemaViolation(String),
}

pub struct Firewall {
    // Directive-87: Numeric Anchor Tolerance (0.5%)
    // If model quotes a price, it must be within +/- 0.5% of live mid_price
    tolerance: f64, 
}

impl Firewall {
    pub fn new() -> Self {
        Self {
            tolerance: 0.005, // 0.5%
        }
    }

    /// Directive-87: The Validation Gate
    /// Validates an LLM response against the Hard Telemetry Truth Envelope.
    pub fn validate(
        &self, 
        response: &LlmInferenceResponse, 
        truth: &TruthEnvelope
    ) -> Result<(), FirewallError> {
        
        // 1. Numeric Anchor Check (NAC)
        if let Some(price) = response.referenced_price {
            // Avoid div by zero
            if truth.mid_price > f64::EPSILON {
                let delta_pct = (price - truth.mid_price).abs() / truth.mid_price;
                if delta_pct > self.tolerance {
                    return Err(FirewallError::NumericHallucination { 
                        claimed: price, 
                        truth: truth.mid_price, 
                        delta: delta_pct 
                    });
                }
            }
        }

        // 2. Regime Continuity Guard
        if let Some(regime_str) = &response.regime_classification {
            // Map Truth ID to expected strings
            // Regime 0: Undefined, 1: Laminar, 2: Turbulent, 3: Violent/Decoherent
            let valid = match (truth.regime_id, regime_str.to_uppercase().as_str()) {
                (1, "LAMINAR") => true,
                (2, "TURBULENT") => true,
                (3, "VIOLENT") | (3, "DECOHERENT") => true,
                // Soft matching allows model some variance if it implies correct state? 
                // Strict requirement says: "If summary describes Laminar while Kernel is Regime 4... rejected"
                // We'll enforce strict matching for now.
                _ => false,
            };

            if !valid {
                return Err(FirewallError::RegimeMismatch { 
                    claimed: regime_str.clone(), 
                    truth_id: truth.regime_id 
                });
            }
        }
        
        // 3. Schema Strictness is handled by Serde deserialize before this fn is called.
        // If we parsed LlmInferenceResponse successfully, strict JSON schema is validated.

        Ok(())
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_numeric_anchor_pass() {
        let firewall = Firewall::new();
        let mut truth = TruthEnvelope::default();
        truth.mid_price = 100.0;
        
        let resp = LlmInferenceResponse {
            reasoning: "ok".into(),
            decision: "HOLD".into(),
            confidence: 1.0,
            referenced_price: Some(100.4), // +0.4% (Pass)
            regime_classification: None,
        };
        
        assert!(firewall.validate(&resp, &truth).is_ok());
    }

    #[test]
    fn test_numeric_anchor_fail() {
        let firewall = Firewall::new();
        let mut truth = TruthEnvelope::default();
        truth.mid_price = 100.0;
        
        let resp = LlmInferenceResponse {
            reasoning: "bad".into(),
            decision: "HOLD".into(),
            confidence: 1.0,
            referenced_price: Some(100.6), // +0.6% (Fail > 0.5%)
            regime_classification: None,
        };
        
        match firewall.validate(&resp, &truth) {
            Err(FirewallError::NumericHallucination { delta, .. }) => {
                assert!(delta > 0.005);
            },
            _ => panic!("Should fail NAC"),
        }
    }
}
