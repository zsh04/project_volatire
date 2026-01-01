use questdb::ingress::{Sender, Buffer, ProtocolVersion};
use deadpool_postgres::{Config, Pool, Runtime};
use tokio::sync::mpsc;
use tokio_postgres::NoTls;
use tracing::{info, error};


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

#[derive(Clone)]
pub struct QuestBridge {
    ilp_sender: mpsc::Sender<FrictionLog>,
    sql_pool: Pool,
}

impl QuestBridge {
    pub async fn new(ilp_host: &str, sql_host: &str, user: &str, pass: &str, db: &str) -> Self {
        // 1. ILP Channel Setup
        let (tx, mut rx) = mpsc::channel::<FrictionLog>(4096);
        let ilp_host_owned = ilp_host.to_string();

        // 2. Spawn ILP Worker
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

            while let Some(log) = rx.recv().await {
                // Serialize into Buffer
                let serialization_result = (|| -> Result<(), questdb::Error> {
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
            sql_pool: pool,
        }
    }
    
    /// Fire-and-forget logging to the ILP worker.
    pub fn log(&self, log: FrictionLog) {
        let sender = self.ilp_sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send(log).await {
                error!("Failed to queue audit log: {}", e);
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
