mod client;
pub mod feynman;
pub mod market;
pub mod ingest;
pub mod ledger;
pub mod taleb;
pub mod audit;
pub mod simons;
pub mod execution;
// mod server; // Disable for Directive-11 Verification (Client focus)

use std::time::{Duration, Instant};
use tracing::{info, warn, error};


pub mod sim;
pub mod db; // Directive-27

// Import the generated code
pub mod reflex_proto {
    tonic::include_proto!("reflex");
}

pub mod brain_proto {
     tonic::include_proto!("brain");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match dotenvy::dotenv() {
        Ok(path) => println!("✅ Loaded .env from {:?}", path),
        Err(e) => println!("⚠️ Failed to load .env: {}", e),
    }

    // --- Configuration ---
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://admin:quest@localhost:8812/qdb".to_string());
    let parsed_url = url::Url::parse(&db_url).expect("Invalid DATABASE_URL");
    
    let sql_user = parsed_url.username();
    let sql_pass = parsed_url.password().unwrap_or("quest");
    let sql_host = parsed_url.host_str().unwrap_or("localhost");
    let sql_db = parsed_url.path().trim_start_matches('/').to_string();
    let _sql_port = parsed_url.port().unwrap_or(8812);

    let ilp_host = std::env::var("QUESTDB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let ilp_port = std::env::var("QUESTDB_ILP_PORT").unwrap_or_else(|_| "9009".to_string());
    let ilp_addr = format!("{}:{}", ilp_host, ilp_port);

    // --- Directive-21: Audit Bridge ---
    println!("Connecting to QuestDB ILP at {} and SQL at {}...", ilp_addr, sql_host);
    let _auditor = audit::QuestBridge::new(
        &ilp_addr, 
        sql_host, 
        sql_user, 
        sql_pass, 
        &if sql_db.is_empty() { "qdb".to_string() } else { sql_db }
    ).await; 

    if _auditor.check_connection().await {
         println!("✅ QuestDB SQL Connection: ESTABLISHED");
    } else {
         eprintln!("❌ QuestDB SQL Connection: FAILED - Check Docker Container");
         eprintln!("   URL: {}", db_url);
    }

    // --- Directive-27: Schema Migration ---
    println!("📦 QUESTDB: Running Migrations on {}...", db_url);
    let (client, connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    
    if let Err(e) = db::migration::run_migrations(&client).await {
        eprintln!("❌ MIGRATION FAILED: {}", e);
        // Continue? or Panic? For P0 schema, likely should work, but for now we warn.
    }
    drop(client); // Close migration connection

    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--mode".to_string()) && args.contains(&"sim".to_string()) {
       println!("🎰 REFLEX SIMULATION MODE");
       
       // Defaults
       let mut start_ts = 1577836800000; // 2020-01-01
       let mut end_ts = 1585699200000;   // 2020-04-01
       let mut speed = 100.0;
       
       // Parse Args
       let mut i = 0;
       while i < args.len() {
           match args[i].as_str() {
               "--speed" => {
                   if i + 1 < args.len() {
                       speed = args[i+1].parse().unwrap_or(100.0);
                   }
               },
               "--start" => {
                    if i + 1 < args.len() {
                        if let Ok(dt) = chrono::NaiveDate::parse_from_str(&args[i+1], "%Y-%m-%d") {
                            start_ts = dt.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis();
                        }
                    }
               },
               "--end" => {
                    if i + 1 < args.len() {
                        if let Ok(dt) = chrono::NaiveDate::parse_from_str(&args[i+1], "%Y-%m-%d") {
                            end_ts = dt.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis();
                        }
                    }
               },
               _ => {}
           }
           i += 1;
       }
       
       let sim = sim::engine::SimulationEngine::new(&db_url, _auditor.clone()).await?;
       sim.run(start_ts, end_ts, speed).await?;
       return Ok(());
    }

    if args.contains(&"--mode".to_string()) && args.contains(&"archive".to_string()) {
        println!("🗄️ REFLEX ARCHIVER MODE");
        
        let retention_days = 30; // Default

        // Connect to Postgres specifically for Archiver
        let (pg_client, connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        
        println!("🔌 Connected to QuestDB Metadata (SQL).");

        let archiver = db::archiver::Archiver::new(pg_client).await?;
        
        // Define tables to archive: (table_name, time_column)
        let tables_to_archive = vec![
            ("ohlcv_1min", "ts"),
            ("ohlcv_1d", "timestamp"),
            ("ohlcv_1min_backup", "timestamp"),
        ];

        for (table, time_col) in tables_to_archive {
             println!("--- Processing Table: {} ---", table);
             // 1. Find Cold Partitions
             let partitions = archiver.find_cold_partitions(table, retention_days).await?;
             
             if partitions.is_empty() {
                 println!("❄️ No cold partitions found for {} (> {} days).", table, retention_days);
             } else {
                 println!("❄️ Found {} cold partitions for {}. Beginning Archival...", partitions.len(), table);
                 for partition in partitions {
                     if let Err(e) = archiver.archive_partition(table, &partition, time_col).await {
                         eprintln!("❌ Failed to archive partition {}/{}: {}", table, partition, e);
                         // Continue to next partition despite error (Best Effort)
                     }
                 }
             }
        }
        
        println!("✅ Archival Process Complete.");
        return Ok(());
    }

    println!("🛡️ Reflex Service (The Body) starting...");

    // Connect to Brain (The Mind)
    let brain_url = "http://[::1]:50052".to_string(); 
    println!("🔌 Connecting to BrainD at {}...", brain_url);
    
    let brain_client_opt = match client::BrainClient::connect(brain_url).await {
        Ok(c) => {
            println!("✅ Connected to BrainD.");
            Some(c)
        },
        Err(e) => {
            eprintln!("⚠️ Failed to connect to BrainD: {}. Running in AUTONOMOUS mode.", e);
            None
        }
    };

    // Components
    let mut market = market::MarketData::new();
    let mut feynman = feynman::PhysicsEngine::new(2000);
    let _ledger = ledger::AccountState::new(50000.0, 0.0);
    let _taleb = taleb::RiskGuardian::new();
    let mut simons = simons::EchoStateNetwork::new(100);
    let _execution = execution::actor::ExecutionAdapter::new();
    println!("Components Initialized.");

    // Spawn Simulation Loop
    let mut client_clone = brain_client_opt; 

    println!("🔄 Simulation Loop Started.");
    let mut last_processed_yield = Instant::now();
    let tick_rate = Duration::from_micros(100); 
    let mut now_ms = 0.0;

    loop {
        now_ms += 100.0; // 100ms ticks
        
        // Simulate Sine Wave Market
        // Period = 50 ticks (5 seconds)
        let phase = (now_ms / 1000.0) * std::f64::consts::PI; 
        let signal = phase.sin() * 5.0;
        let noise = (rand::random::<f64>() - 0.5) * 2.0;
        
        // Spike Injection at tick 50 (~5s)
        let spike = if now_ms > 5000.0 && now_ms < 5500.0 { 10.0 } else { 0.0 };

        let price = 100.0 + signal + noise + spike;
        
        market.update_price(price);
        let state = feynman.update(market.price, now_ms);
        
        // Simons Prediction (Echo State Network)
        let simons_pred = simons.forward(state.velocity);
        let next_target = state.velocity * 0.99; // Damping target
        simons.train(next_target);

        // Talk to Brain
        if let Some(client) = &mut client_clone {
                // Throttle requests to 2Hz (every 500ms)
                if last_processed_yield.elapsed() > Duration::from_millis(500) {
                     match client.reason(
                         market.price,
                         state.velocity,
                         state.efficiency_index, 
                         state.entropy,
                         simons_pred
                     ).await {
                         Ok(intent) => {
                             // Map Protobuf Intent to Taleb TradeProposal
                             let proposal = taleb::TradeProposal {
                                 side: intent.action.clone(),
                                 price: market.price,
                                 qty: 1.0, 
                             };

                             // 1. Calculate Size (BES-Kelly)
                             let optimal_size = taleb::sizing::BESKelly::allocate(
                                 _ledger.available_balance(),
                                 market.price,
                                 intent.forecast_p90,
                                 intent.forecast_p10,
                                 intent.confidence
                             );
                             
                             // 2. Override Intent Qty
                             let mut final_proposal = proposal.clone();
                             final_proposal.qty = optimal_size / market.price; // Convert USD to Units

                             // 3. Risk Check (Omega Sieve - Entry)
                             let verdict = _taleb.check(
                                 &state, 
                                 &_ledger, 
                                 &final_proposal,
                                 intent.forecast_p10,
                                 intent.forecast_p50,
                                 intent.forecast_p90,
                                 intent.forecast_timestamp,
                                 intent.hurdle_rate
                             );
                             
                             // 4. Risk Shroud Check (BES - Exit)
                             // We check if the current price violates the BES boundary of the INTENT
                             let shroud_verdict = _taleb.check_shroud(
                                 market.price,
                                 &intent,
                                 state.entropy
                             );

                             if let taleb::shroud::ShroudVerdict::NuclearExit(reason) = shroud_verdict {
                                 error!("☢️ NUCLEAR EXIT TRIGGERED: {}", reason);
                                 // D-23: Nuclear Execution (IOC)
                                 let exit_proposal = taleb::TradeProposal {
                                     side: if intent.action == "LONG" { "SELL".to_string() } else { "BUY".to_string() },
                                     price: market.price, // Market Order
                                     qty: final_proposal.qty, // Close Position
                                 };
                                 _execution.execute_nuclear(&exit_proposal, &reason).await;
                                 
                                 // In a real system, we would also close existing positions.
                             }

                             match verdict {
                                 taleb::RiskVerdict::Allowed => {
                                     info!("RISK: ALLOWED. Executing Strategy: {:?}", final_proposal);
                                     
                                     // --- D-21: Friction Logging ---
                                     // In a real execution, we'd get fill price and slippage from the exchange.
                                     // Here we simulate the friction impact.
                                     let fill_price = state.price; // Slippage = 0 for now in sim
                                     let slippage_bps = 0.0;
                                     let gas_usd = 0.50; // Simulated Gas
                                     

                                     _auditor.log(audit::FrictionLog {
                                         ts: None, // Live execution uses server time
                                         symbol: "BTC-USDT".to_string(), 
                                         order_id: uuid::Uuid::new_v4().to_string(), // Generate UUID
                                         side: final_proposal.side.clone(),
                                         intent_qty: final_proposal.qty,
                                         fill_price,
                                         slippage_bps,
                                         gas_usd,
                                         realized_pnl: 0.0, 
                                         fee_native: 0.0,
                                         tax_buffer: 0.0,
                                     });
                                     
                                     // ... existing accounting logic ...
                                     // D-23: Sniper Execution (Shadow Limit)
                                     _execution.execute_sniper(&final_proposal).await;
                                 },
                                 taleb::RiskVerdict::Veto(reason) => {
                                     warn!("RISK: VETOED. Reason: {}", reason);
                                 },
                                 taleb::RiskVerdict::Panic => {
                                     error!("RISK: PANIC! HALTING EXECUTION.");
                                     break;
                                 }
                             }
                         },
                         Err(e) => {
                            eprintln!("❌ Brain Error: {}", e)
                        },
                    }
                    last_processed_yield = Instant::now();
            }
        }
        
        // Log local events
        if state.velocity.abs() > 8.0 {
             println!("⚠️ REFLEX: High Velocity Event! V={:.2}", state.velocity);
        }

        tokio::time::sleep(tick_rate).await;
    }
    
    Ok(())
}
