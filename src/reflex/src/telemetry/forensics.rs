use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use tokio::sync::mpsc;
use crate::feynman::PhysicsState;
use crate::audit::QuestBridge;

/// The immutable record of a decision event.
/// Matches the schema required for "Combat Replay".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPacket {
    pub timestamp: f64,
    pub trace_id: String,
    pub physics: PhysicsState,
    pub sentiment: f64,
    pub vector_distance: f64, // Variance from historical regime
    pub quantile_score: i32,  // 1-10 Stability Score
    pub decision: String,     // Action taken
    pub operator_hash: String, // Cryptographic seal
}

impl DecisionPacket {
    /// Generates a sovereign hash of the packet content provided.
    /// This seals the record before it leaves the decision core.
    pub fn generate_hash(
        ts: f64, 
        trace_id: &str, 
        physics_digest: &str, 
        decision: &str
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(ts.to_be_bytes());
        hasher.update(trace_id.as_bytes());
        hasher.update(physics_digest.as_bytes());
        hasher.update(decision.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn seal(&mut self) {
        // Simple serialization of physics state for hashing
        let p_digest = format!("{}:{}:{}:{}", 
            self.physics.price, 
            self.physics.velocity, 
            self.physics.jerk, 
            self.physics.entropy
        );
        self.operator_hash = Self::generate_hash(
            self.timestamp, 
            &self.trace_id, 
            &p_digest, 
            &self.decision
        );
    }
}

/// The Scribe: Asynchronous Logger for Forensic Records.
/// Decouples high-latency I/O (Disk/DB) from the hot simulation loop.
pub struct ForensicLogger {
    rx: mpsc::Receiver<DecisionPacket>,
    _auditor: QuestBridge, // Reuse QuestBridge for ILP transport
}

impl ForensicLogger {
    pub fn new(rx: mpsc::Receiver<DecisionPacket>, auditor: QuestBridge) -> Self {
        Self { rx, _auditor: auditor }
    }

    pub async fn run(mut self) {
        tracing::info!("📜 Forensic Logger (The Scribe) Started.");

        while let Some(packet) = self.rx.recv().await {
            // 1. Ingest into QuestDB (Hot Storage)
            // We'll map this to a generic log structure or extend QuestBridge
            // For now, we assume QuestBridge has a method or we add one.
            // Since QuestBridge is strictly typed in current impl, we might need to extend it.
            // For this pass, we will just log to INFO and TODO: impl persistent write
            
            // In a real impl, this calls self.auditor.log_forensic(&packet);
            // We'll verify the flow by printing the Sovereign Hash
            if packet.quantile_score < 5 {
                tracing::warn!("⚠️ Low Stability Decision Recorded: Hash={}", packet.operator_hash);
            }
        }
    }
}
