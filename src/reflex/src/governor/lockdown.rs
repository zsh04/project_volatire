// Directive-85: The State-of-Nature Lockdown (Forensic Seal)
// Manages the cryptographic seal state and validates readiness certificates

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessCertificate {
    pub timestamp: u64,
    pub directive: String,
    pub status: SealStatus,
    pub codebase_hash: String,
    pub test_results_hash: String,
    pub signature: String,
    pub acceptance_gates: AcceptanceGates,
    pub genesis_baseline: GenesisBaseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SealStatus {
    #[serde(rename = "SEALED")]
    Sealed,
    #[serde(rename = "REJECTED")]
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceGates {
    pub jitter_stability: GateResult,
    pub cognitive_alignment: GateResult,
    pub hud_fidelity: GateResult,
}

impl AcceptanceGates {
    pub fn all_passed(&self) -> bool {
        self.jitter_stability.passed
            && self.cognitive_alignment.passed
            && self.hud_fidelity.passed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub passed: bool,
    #[serde(flatten)]
    pub metrics: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBaseline {
    pub ooda_jitter_us: f64,
    pub gemma_latency_ms: f64,
    pub hud_fps: f64,
    pub worker_latency_ms: f64,
}

pub struct LockdownGovernor {
    certificate: Option<ReadinessCertificate>,
    sealed: bool,
}

impl LockdownGovernor {
    pub fn new() -> Self {
        Self {
            certificate: None,
            sealed: false,
        }
    }

    /// Load certificate from file
    pub fn load_certificate<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let cert: ReadinessCertificate = serde_json::from_str(&contents)?;
        
        // Validate certificate
        if cert.status == SealStatus::Sealed && cert.acceptance_gates.all_passed() {
            self.certificate = Some(cert);
            self.sealed = true;
            tracing::info!("✅ Readiness certificate loaded and validated");
            Ok(())
        } else {
            tracing::error!("❌ Certificate rejected: Gates did not pass");
            Err("Certificate validation failed".into())
        }
    }

    /// Check if system is sealed and ready
    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    /// Get certificate if loaded
    pub fn certificate(&self) -> Option<&ReadinessCertificate> {
        self.certificate.as_ref()
    }

    /// Check acceptance gates
    pub fn check_acceptance_gates(&self) -> bool {
        if let Some(ref cert) = self.certificate {
            cert.acceptance_gates.all_passed()
        } else {
            false
        }
    }

    /// Seal the system (called after successful validation)
    pub fn seal(&mut self) {
        if self.certificate.is_some() && self.check_acceptance_gates() {
            self.sealed = true;
            tracing::info!("🔒 SYSTEM SEALED: Ready for Phase 7 Ignition");
        } else {
            tracing::error!("⛔ SEAL REJECTED: Acceptance gates not satisfied");
        }
    }

    /// Force unseal (for testing/emergency only)
    #[allow(dead_code)]
    pub fn unseal(&mut self) {
        self.sealed = false;
        tracing::warn!("🔓 SYSTEM UNSEALED: Ignition disabled");
    }

    /// Get genesis baseline
    pub fn genesis_baseline(&self) -> Option<&GenesisBaseline> {
        self.certificate.as_ref().map(|c| &c.genesis_baseline)
    }

    /// Validate against current performance
    /// Returns true if current metrics are within acceptable range of baseline
    pub fn validate_against_baseline(
        &self,
        current_jitter_us: f64,
        current_latency_ms: f64,
    ) -> bool {
        if let Some(baseline) = self.genesis_baseline() {
            // Allow 20% degradation from baseline
            let jitter_ok = current_jitter_us <= baseline.ooda_jitter_us * 1.2;
            let latency_ok = current_latency_ms <= baseline.gemma_latency_ms * 1.2;
            
            if !jitter_ok {
                tracing::warn!(
                    "⚠️ Performance degradation: Jitter {:.1}μs vs baseline {:.1}μs",
                    current_jitter_us,
                    baseline.ooda_jitter_us
                );
            }
            
            if !latency_ok {
                tracing::warn!(
                    "⚠️ Performance degradation: Latency {:.1}ms vs baseline {:.1}ms",
                    current_latency_ms,
                    baseline.gemma_latency_ms
                );
            }
            
            jitter_ok && latency_ok
        } else {
            // No baseline - assume OK
            true
        }
    }
}

impl Default for LockdownGovernor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceptance_gates_all_passed() {
        let gates = AcceptanceGates {
            jitter_stability: GateResult {
                passed: true,
                metrics: serde_json::json!({"jitter_us": 42.0}),
            },
            cognitive_alignment: GateResult {
                passed: true,
                metrics: serde_json::json!({"gsid_alignment": 100.0}),
            },
            hud_fidelity: GateResult {
                passed: true,
                metrics: serde_json::json!({"fps": 61.2}),
            },
        };
        
        assert!(gates.all_passed());
    }

    #[test]
    fn test_acceptance_gates_one_failed() {
        let gates = AcceptanceGates {
            jitter_stability: GateResult {
                passed: false,
                metrics: serde_json::json!({"jitter_us": 55.0}),
            },
            cognitive_alignment: GateResult {
                passed: true,
                metrics: serde_json::json!({"gsid_alignment": 100.0}),
            },
            hud_fidelity: GateResult {
                passed: true,
                metrics: serde_json::json!({"fps": 61.2}),
            },
        };
        
        assert!(!gates.all_passed());
    }

    #[test]
    fn test_lockdown_governor_default() {
        let governor = LockdownGovernor::new();
        assert!(!governor.is_sealed());
        assert!(governor.certificate().is_none());
    }
}
