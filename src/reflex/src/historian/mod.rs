pub mod events;
pub mod shm_buffer;
pub mod logger;
pub mod biopsy;

use serde::Deserialize;
use reqwest::Client;
use crate::reflex_proto::PhysicsResponse;
use tokio::sync::mpsc;
use error_chain::error_chain;

error_chain! {
    foreign_links {
        Reqwest(reqwest::Error);
        Tokio(tokio::task::JoinError);
    }
}

#[derive(Debug, Deserialize)]
struct QuestDBRow {
    price: f64,
    velocity: f64,
    acceleration: f64,
    jerk: f64,
    entropy: f64,
    efficiency_index: f64,
    timestamp: f64,
    sequence_id: i64,
    // Add other fields as needed if we widen the query
}

pub struct TickReader {
    client: Client,
    questdb_url: String,
}

impl TickReader {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            questdb_url: "http://localhost:9000".to_string(), // Default QuestDB HTTP
        }
    }

    pub async fn fetch_ticks(
        &self,
        _symbol: &str,
        start_time_ms: f64,
        end_time_ms: f64,
        tx: mpsc::Sender<std::result::Result<PhysicsResponse, tonic::Status>>,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // QuestDB SQL query
        // Note: Timestamp in QuestDB is typically micros if standard, but we stored as double in physics.
        // Assuming 'physics' table has 'timestamp' column as double (ms) based on logger implementation.
        // D-82 logger used 'timestamp' f64.
        
        let query = format!(
            "SELECT price, velocity, acceleration, jerk, entropy, efficiency_index, timestamp, sequence_id \
             FROM physics \
             WHERE timestamp >= {} AND timestamp <= {} \
             ORDER BY timestamp ASC",
             start_time_ms, end_time_ms
        );

        let url = format!("{}/exec?query={}", self.questdb_url, urlencoding::encode(&query));
        
        // Streaming response is harder with simple HTTP, so we'll fetch JSON and iterate.
        // For production "Time Machine", we'd want chunked responses, but for trade replay (short window),
        // a single fetch is acceptable.
        
        let resp = self.client.get(&url).send().await?.json::<serde_json::Value>().await?;
        
        if let Some(dataset) = resp.get("dataset") {
            if let Some(rows) = dataset.as_array() {
                for row in rows {
                    // QuestDB returns arrays for rows in JSON exec
                    // [price, velocity, acceleration, jerk, entropy, efficiency_index, timestamp, sequence_id]
                    if let (
                        Some(price), 
                        Some(velocity), 
                        Some(acceleration), 
                        Some(jerk), 
                        Some(entropy), 
                        Some(efficiency), 
                        Some(ts), 
                        Some(seq)
                    ) = (
                        row[0].as_f64(),
                        row[1].as_f64(),
                        row[2].as_f64(),
                        row[3].as_f64(),
                        row[4].as_f64(),
                        row[5].as_f64(),
                        row[6].as_f64(),
                        row[7].as_i64()
                    ) {
                        let physics = PhysicsResponse {
                            price,
                            velocity,
                            acceleration,
                            jerk,
                            entropy,
                            efficiency_index: efficiency,
                            timestamp: ts,
                            sequence_id: seq,
                            // Fill rest with defaults (historical context only cares about microstructure)
                            unrealized_pnl: 0.0,
                            equity: 0.0,
                            balance: 0.0,
                            realized_pnl: 0.0,
                            btc_position: 0.0,
                            gemma_tokens_per_sec: 0.0,
                            gemma_latency_ms: 0.0,
                            staircase_tier: 0,
                            staircase_progress: 0.0,
                            audit_drift: 0.0,
                            system_latency_us: 0.0,
                            system_jitter_us: 0.0,
                            vitality_status: "REPLAY".to_string(),
                            reasoning_trace: vec![],
                            ignition_status: "HISTORICAL".to_string(),
                            system_sanity_score: 1.0,
                            positions: vec![],
                            orders: vec![],
                        };
                        
                        if let Err(_) = tx.send(Ok(physics)).await {
                            break; // Client disconnected
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
