use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use tokio::sync::mpsc;
use crate::feynman::PhysicsState;
use crate::audit::{QuestBridge, ForensicLog};

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
            let forensic_log = ForensicLog {
                timestamp: packet.timestamp,
                trace_id: packet.trace_id.clone(),
                physics: packet.physics,
                sentiment: packet.sentiment,
                vector_distance: packet.vector_distance,
                quantile_score: packet.quantile_score,
                decision: packet.decision.clone(),
                operator_hash: packet.operator_hash.clone(),
            };

            self._auditor.log_forensic(forensic_log);
            
            // We'll verify the flow by printing the Sovereign Hash
            if packet.quantile_score < 5 {
                tracing::warn!("⚠️ Low Stability Decision Recorded: Hash={}", packet.operator_hash);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_forensic_logging_flow() {
        // 1. Setup Auditor (QuestBridge)
        // Will fail to connect but should not panic
        let auditor = QuestBridge::new("localhost:9009", "localhost", "admin", "quest", "qdb").await;

        // 2. Setup Logger
        let (tx, rx) = mpsc::channel(10);
        let logger = ForensicLogger::new(rx, auditor.clone());

        // 3. Spawn Logger
        tokio::spawn(async move {
            logger.run().await;
        });

        // 4. Send Packet
        let mut packet = DecisionPacket {
            timestamp: 1234567890.0,
            trace_id: "test_trace".to_string(),
            physics: PhysicsState::default(),
            sentiment: 0.5,
            vector_distance: 0.1,
            quantile_score: 8,
            decision: "Hold".to_string(),
            operator_hash: String::new(),
        };
        packet.seal();

        tx.send(packet).await.expect("Failed to send packet");

        // 5. Wait briefly for processing (async fire & forget)
        tokio::time::sleep(Duration::from_millis(100)).await;

        // If we reached here without panic, success.
        // We cannot easily assert internal state of QuestBridge without adding inspection methods,
        // but this verifies the integration glue code.
    }
}
