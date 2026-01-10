use questdb::ingress::{Sender, Buffer, ProtocolVersion};
use deadpool_postgres::{Config, Pool, Runtime};
use tokio::sync::mpsc;
use tokio_postgres::NoTls;
use tracing::{info, error};
use crate::feynman::PhysicsState;


#[derive(Debug, Clone)]
pub struct FrictionLog {
    pub ts: Option<i64>, // Explicit Timestamp (nanos) for Simulation/Backfill
    pub symbol: String,
    pub order_id: String, // D-27
    pub side: String,
    pub intent_qty: f64,
    pub fill_price: f64,
    pub slippage_bps: f64,
    pub gas_usd: f64,
    pub realized_pnl: f64,
    pub fee_native: f64, // D-27
    pub tax_buffer: f64, // D-27
}

#[derive(Debug, Clone)]
pub struct TickLog {
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub ts: i64,
}

pub enum AuditLog {
    Friction(FrictionLog),
    Tick(TickLog),
}

#[derive(Debug, Clone)]
pub struct ForensicLog {
    pub timestamp: f64,
    pub trace_id: String,
    pub physics: PhysicsState,
    pub sentiment: f64,
    pub vector_distance: f64,
    pub quantile_score: i32,
    pub decision: String,
    pub operator_hash: String,
}

#[derive(Clone)]
pub struct QuestBridge {
    ilp_sender: mpsc::Sender<AuditLog>,
    forensic_sender: mpsc::Sender<ForensicLog>,
    sql_pool: Pool,
}

impl QuestBridge {
    pub async fn new(ilp_host: &str, sql_host: &str, user: &str, pass: &str, db: &str) -> Self {
        // 1. ILP Channel Setup
        let (tx, mut rx) = mpsc::channel::<AuditLog>(4096);
        let (tx_forensic, mut rx_forensic) = mpsc::channel::<ForensicLog>(4096);
        let ilp_host_owned = ilp_host.to_string();
        let ilp_host_forensic = ilp_host.to_string();

        // 2. Spawn ILP Worker (FrictionLog)
        tokio::spawn(async move {
            use questdb::ingress::TimestampNanos; // Ensure this is available

            info!("QuestDB ILP Worker: Connecting to {}", ilp_host_owned);
            let mut sender = match Sender::from_conf(&format!("tcp::addr={};", ilp_host_owned)) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to create ILP Sender: {}", e);
                    return;
                }
            };
            
            // QuestDB requires a separate Buffer for serialization
            let mut buffer = Buffer::new(ProtocolVersion::V3);

            while let Some(msg) = rx.recv().await {
                // Serialize into Buffer
                let serialization_result = (|| -> Result<(), questdb::Error> {
                    match msg {
                        AuditLog::Friction(log) => {
                            let row = buffer.table("friction_ledger")?
                                .symbol("symbol", &log.symbol)?
                                .symbol("order_id", &log.order_id)?
                                .symbol("side", &log.side)?
                                .column_f64("intent_qty", log.intent_qty)?
                                .column_f64("fill_price", log.fill_price)?
                                .column_f64("slippage_bps", log.slippage_bps)?
                                .column_f64("gas_usd", log.gas_usd)?
                                .column_f64("realized_pnl", log.realized_pnl)?
                                .column_f64("fee_native", log.fee_native)?
                                .column_f64("tax_buffer", log.tax_buffer)?;

                            if let Some(ts) = log.ts {
                                row.at(TimestampNanos::new(ts))?;
                            } else {
                                row.at_now()?;
                            }
                        },
                        AuditLog::Tick(log) => {
                            buffer.table("live_ticks")?
                                .symbol("symbol", &log.symbol)?
                                .column_f64("price", log.price)?
                                .column_f64("qty", log.quantity)?
                                .at(TimestampNanos::new(log.ts))?;
                        }
                    }
                    Ok(())
                })();

                if let Err(e) = serialization_result {
                     error!("QuestDB Serialization Failed: {}", e);
                     buffer.clear(); 
                     continue;
                }
                
                // Flush Buffer to Network
                if let Err(e) = sender.flush(&mut buffer) {
                    error!("QuestDB ILP Flush Failed: {}", e);
                    buffer.clear();
                }
            }
        });

        // 2b. Spawn ILP Worker (ForensicLog)
        tokio::spawn(async move {
            use questdb::ingress::TimestampNanos;

            info!("QuestDB Forensic Worker: Connecting to {}", ilp_host_forensic);
            let mut sender = match Sender::from_conf(&format!("tcp::addr={};", ilp_host_forensic)) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to create ILP Sender for Forensic: {}", e);
                    return;
                }
            };

            let mut buffer = Buffer::new(ProtocolVersion::V3);

            while let Some(log) = rx_forensic.recv().await {
                 let serialization_result = (|| -> Result<(), questdb::Error> {
                    let ts_nanos = (log.timestamp * 1_000_000.0) as i64;

                    buffer.table("forensic_events")?
                        .symbol("trace_id", &log.trace_id)?
                        .symbol("decision", &log.decision)?
                        .symbol("operator_hash", &log.operator_hash)?
                        .column_f64("sentiment", log.sentiment)?
                        .column_f64("vector_distance", log.vector_distance)?
                        .column_i64("quantile_score", log.quantile_score as i64)?
                        // Physics Flattening
                        .column_f64("physics_price", log.physics.price)?
                        .column_f64("physics_velocity", log.physics.velocity)?
                        .column_f64("physics_acceleration", log.physics.acceleration)?
                        .column_f64("physics_jerk", log.physics.jerk)?
                        .column_f64("physics_volatility", log.physics.volatility)?
                        .column_f64("physics_entropy", log.physics.entropy)?
                        .column_f64("physics_efficiency", log.physics.efficiency_index)?
                        .column_f64("physics_basis", log.physics.basis)?
                        .column_i64("physics_seq", log.physics.sequence_id as i64)?
                        .at(TimestampNanos::new(ts_nanos))?;
                    Ok(())
                 })();

                 if let Err(e) = serialization_result {
                     error!("QuestDB Forensic Serialization Failed: {}", e);
                     buffer.clear();
                     continue;
                 }

                if let Err(e) = sender.flush(&mut buffer) {
                    error!("QuestDB Forensic ILP Flush Failed: {}", e);
                    buffer.clear();
                }
            }
        });

        // 3. SQL Pool Setup
        let mut cfg = Config::new();
        cfg.host = Some(sql_host.to_string());
        cfg.user = Some(user.to_string());
        cfg.password = Some(pass.to_string());
        cfg.dbname = Some(db.to_string());
        cfg.port = Some(8812); // Default PG port for QuestDB
        
        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).expect("Failed to create Postgres pool");

        QuestBridge {
            ilp_sender: tx,
            forensic_sender: tx_forensic,
            sql_pool: pool,
        }
    }
    
    /// Fire-and-forget logging to the ILP worker (FrictionLog).
    pub fn log(&self, log: FrictionLog) {
        let sender = self.ilp_sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send(AuditLog::Friction(log)).await {
                error!("Failed to queue audit log: {}", e);
            }
        });
    }

    /// Fire-and-forget logging of Ticks.
    pub fn log_tick(&self, symbol: &str, price: f64, quantity: f64, ts_nanos: u64) {
        let sender = self.ilp_sender.clone();
        let log = TickLog {
            symbol: symbol.to_string(),
            price,
            quantity,
            ts: ts_nanos as i64,
        };
        tokio::spawn(async move {
            if let Err(e) = sender.send(AuditLog::Tick(log)).await {
                error!("Failed to queue tick log: {}", e);
            }
        });
    }

    /// Fire-and-forget logging of Forensics.
    pub fn log_forensic(&self, log: ForensicLog) {
        let sender = self.forensic_sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send(log).await {
                error!("Failed to queue forensic log: {}", e);
            }
        });
    }

    /// Helper to verify SQL connection (Handshake)
    pub async fn check_connection(&self) -> bool {
        match self.sql_pool.get().await {
            Ok(client) => {
                 match client.query_one("SELECT 1;", &[]).await {
                     Ok(_) => true,
                     Err(e) => {
                         error!("QuestDB SQL Handshake Failed: {}", e);
                         false
                     }
                 }
            },
            Err(e) => {
                error!("QuestDB Pool Error: {}", e);
                false
            }
        }
    }
}
