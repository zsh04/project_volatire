// Modules are now in lib.rs
use reflex::client;
use reflex::feynman;
use reflex::market;
use reflex::ledger;
use reflex::taleb;
use reflex::audit;
use reflex::simons;
use reflex::execution;
use reflex::telemetry;
use reflex::sim;
use reflex::db;
use reflex::ingest;

// Proto imports via lib
// use reflex::reflex_proto;
// use reflex::brain_proto;


use std::time::{Duration, Instant};
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 0. Load Configuration (Env)
    match dotenvy::dotenv() {
        Ok(path) => println!("✅ Loaded .env from {:?}", path),
        Err(e) => println!("⚠️ Failed to load .env: {}", e),
    }

    // 1. Initialize Telemetry (Directive-47)
    telemetry::init_telemetry().map_err(|e| e as Box<dyn std::error::Error>)?;
    info!("Voltaire Reflex Engine v1.0.0 (Phase 5) - Telemetry Active");

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

    // --- Directive-69: Genesis Orchestrator ---
    let orchestrator = reflex::governor::supervise::GenesisOrchestrator::new();
    if !orchestrator.orchestrate().await {
        error!("Genesis Aborted. Check Logs.");
        // In Prod: return Err("Genesis Failed".into());
        // In Dev: We warn but might proceed if overriding
    }

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

    // --- Directive-41 & 42: State Supercharger & Kinetic Pipe ---
    let redis_url = "redis://localhost:6379/";
    println!("⚡ Connecting to DragonflyDB (L1 State) at {}...", redis_url);
    
    // Initialize Pool
    let state_store_res = db::state::RedisStateStore::new(redis_url).await;
    let state_store = match state_store_res {
        Ok(s) => {
             match s.ping().await {
                 Ok(_) => {
                     println!("✅ DragonflyDB Connection: ESTABLISHED");
                     Some(s)
                 },
                 Err(e) => {
                     eprintln!("❌ DragonflyDB PING FAILED: {}", e);
                     None
                 }
             }
        },
        Err(e) => {
            eprintln!("❌ DragonflyDB Connection INIT FAILED: {}", e);
            None
        }
    };

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


    // --- Directive-49: Control Surface ---
    let (tx_broadcast, _rx_broadcast) = tokio::sync::broadcast::channel::<reflex::server::SharedState>(100);
    let shared_state = std::sync::Arc::new(std::sync::RwLock::new(reflex::server::SharedState::default()));
    
    // --- Directive-50: Internal Historian (Forensic Logger) ---
    let (forensic_tx, forensic_rx) = tokio::sync::mpsc::channel(1024);
    let logger_auditor = _auditor.clone(); // Assumes QuestBridge is Clone
    tokio::spawn(async move {
        let scribe = telemetry::forensics::ForensicLogger::new(forensic_rx, logger_auditor);
        scribe.run().await;
    });

    // --- Directive-51: The Mirror Reality (Synthetic Drift Detection) ---
    let (mirror_tx, mirror_rx) = tokio::sync::mpsc::channel(1024);
    tokio::spawn(async move {
        // Run Mirror Actor
        telemetry::mirror::MirrorEngine::new(mirror_rx).run().await;
    });

    // --- Directive-52: The Decay Monitor (Alpha Validation) ---
    // Channel for Decisions (Intent)
    let (decay_tx, decay_rx) = tokio::sync::mpsc::channel(1024);
    // Channel for Fills (Reality)
    let (_decay_fill_tx, decay_fill_rx) = tokio::sync::mpsc::channel(1024);

    tokio::spawn(async move {
        // Run Decay Actor
        telemetry::decay::DecayMonitor::new(decay_rx, decay_fill_rx).run().await;
    });

    // Update OODA Core with Decay Channel
    let mut ooda = reflex::governor::ooda_loop::OODACore::new(Some(forensic_tx), Some(mirror_tx), Some(decay_tx));

    // Spawn API Server
    let server_state = shared_state.clone();
    let server_tx = tx_broadcast.clone(); // Pass Sender for subscribing
    let _server_handle = tokio::spawn(async move {
        reflex::server::run_server(server_state, server_tx).await;
    });


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
             // In prod this might be fatal, but for dev autonomous is okay
            eprintln!("⚠️ Failed to connect to BrainD: {}. Running in AUTONOMOUS mode.", e);
            None
        }
    };

    // Components
    let mut market = market::MarketData::new();
    let mut feynman = feynman::PhysicsEngine::new(2000);
    let mut _ledger = ledger::AccountState::new(50000.0, 0.0);
    let _taleb = taleb::RiskGuardian::new();
    let mut simons = simons::EchoStateNetwork::new(100);
    let _execution = execution::actor::ExecutionAdapter::new();
    // Directive-64: Safety Staircase (Real Instance)
    let mut staircase_governor = reflex::governor::staircase::Staircase::new();
    println!("Components Initialized.");

    // Spawn Simulation Loop
    let mut client_clone = brain_client_opt; 

    println!("🔄 Simulation Loop Started.");
    let mut last_processed_yield = Instant::now();
    let tick_rate = Duration::from_micros(100); 
    let mut now_ms = 0.0;

    // --- Metrics Setup ---
    let meter = opentelemetry::global::meter("reflex_engine");
    let metrics = telemetry::metrics::EngineMetrics::new(&meter);
    let kv = [opentelemetry::KeyValue::new("mode", "simulation")];

    // --- Directive-72: Ingestion Spawning ---
    let (ingest_tx, mut ingest_rx) = tokio::sync::mpsc::channel(100);
    let is_sim_mode_flag = false; 

    if !is_sim_mode_flag {
        println!("🚀 LIVE MODE: Connecting to Kraken Ingestion...");
        tokio::spawn(async move {
            ingest::kraken::connect_kraken("XBT/USD", ingest_tx).await;
        });
    }

    // --- Directive-72: Account Sync Channel ---
    let (balance_tx, mut balance_rx) = tokio::sync::mpsc::channel(10);
    if !is_sim_mode_flag {
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            
            let key = std::env::var("KRAKEN_API_KEY").unwrap_or_default();
            let secret = std::env::var("KRAKEN_API_SECRET").unwrap_or_default();
            
            if key.is_empty() {
                warn!("⚠️ KRAKEN KEYS MISSING. Account Sync Disabled.");
                return;
            }

            loop {
                let result = ingest::kraken::fetch_account_balance(&key, &secret)
                    .await
                    .map_err(|e| e.to_string());
                match result {
                    Ok((usd, btc)) => {
                        let _ = balance_tx.send((usd, btc)).await;
                    },
                    Err(e) => {
                        error!("❌ Account Sync Failed: {}", e);
                    }
                }
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
    }

    // --- Directive-66: Audit Loop (Real Instance) ---
    let mut audit_loop = reflex::governor::audit_loop::AuditLoop::new();

    loop {
        let loop_start = Instant::now();
        now_ms += 100.0;
        
        // --- Directive-72: Consume Account Updates ---
        if let Ok((usd, btc)) = balance_rx.try_recv() {
             _ledger.sync(usd, btc, 0.0);
             info!("🏦 Ledger Synced: USD=${:.2} BTC={:.8}", usd, btc);
        }

        let span = tracing::info_span!("ooda_tick", tick_ms = now_ms);
        let _enter = span.enter();

        // --- Directive-72: LIVE DATA ORIGIN ---
        let price = if is_sim_mode_flag {
             let phase = (now_ms / 1000.0) * std::f64::consts::PI; 
             let signal = phase.sin() * 5.0;
             let noise = (rand::random::<f64>() - 0.5) * 2.0;
             let spike = if now_ms > 5000.0 && now_ms < 5500.0 { 10.0 } else { 0.0 };
             100.0 + signal + noise + spike
        } else {
             match tokio::time::timeout(Duration::from_millis(100), ingest_rx.recv()).await {
                 Ok(Some(tick)) => {
                     now_ms = tick.timestamp as f64;
                     tick.price
                 },
                 Ok(None) => {
                     error!("❌ Ingestion Channel Closed!");
                     break; 
                 },
                 Err(_) => {
                     now_ms += 100.0;
                     market.price 
                 }
             }
        };
        
        metrics.heartbeat.add(1, &kv);
        metrics.market_price.record(price, &kv);
        
        market.update_price(price);
        let state = feynman.update(market.price, now_ms);
        metrics.market_velocity.record(state.velocity, &kv);
        
        // --- D-50: OODA Execution ---
        let ooda_state = ooda.orient(state.clone(), client_clone.as_mut()).await;
        let _decision = ooda.decide(&ooda_state);

        // Update Shared State (For API)
        if let Ok(mut w) = shared_state.write() {
            w.physics = state.clone(); 
            w.ooda = Some(ooda_state.clone());
            
            // Directive-72: Update Account Link
            let current_equity = _ledger.total_equity(market.price);
            w.account.equity = current_equity;
            w.account.balance = _ledger.available_balance();
            w.account.unrealized_pnl = current_equity - _ledger.start_of_day_balance;
            
            // Directive-72: Brain Telemetry (Updated below if connected)
            // Default/Fallback values
            // w.gemma.tokens_per_sec = 0.0; // Set via Brain response if available
            // w.gemma.latency_ms = 0.0;     
            
            // Directive-64: Update Governance Link
            w.governance.staircase_tier = staircase_governor.tier();
            w.governance.staircase_progress = staircase_governor.progress();
            w.governance.audit_drift = audit_loop.drift_score;

            // Check Veto (Directive-68 & 72)
            if w.veto_active {
                tracing::warn!("⛔ VETO ACTIVE (Origin Guard/Kill Switch). Suspending Execution Loop.");
                tokio::time::sleep(tick_rate * 10).await; // Slow down loop significantly
                continue; // Skip Brain Logic & Execution
            }
        }

        // Broadcast State (Fire & Forget)
        // We reconstruct a lightweight shared state or just update fields
        // ideally we broadcast the immutable state. 
        // For simplicity, we clone the struct.
        let broadcast_payload = {
            let r = shared_state.read().unwrap();
            r.clone()
        };
        let _ = tx_broadcast.send(broadcast_payload);
        
        // Simons Prediction (Echo State Network)
        let simons_pred = simons.forward(state.velocity);
        let next_target = state.velocity * 0.99; // Damping target
        simons.train(next_target);

        // --- Directive-42: Kinetic Pipe Sync ---
        // Fire-and-forget logic for state updates (don't block heavily)
        if let Some(store) = &state_store {
            // We clone efficiently if needed or just ref. state is Copy?
            // PhysicsState is Copy (derived in feynman.rs)
            if let Err(e) = store.update_kinetics("BTC-USDT", &state).await {
                eprintln!("⚠️ Failed to sync kinetics: {}", e);
            }
        }

        // Talk to Brain
        if let Some(client) = &mut client_clone {
                // Throttle requests to 2Hz (every 500ms)
                if last_processed_yield.elapsed() > Duration::from_millis(500) {
                     let rtt_start = Instant::now(); // Measure Latency
                     match client.reason(
                         market.price,
                         state.velocity,
                         state.efficiency_index, 
                         state.entropy,
                         simons_pred
                     ).await {
                         Ok(intent) => {
                             let latency_ms = rtt_start.elapsed().as_millis() as f64;
                             
                             // Update Telemetry with Real Latency
                             if let Ok(mut w) = shared_state.write() {
                                 w.gemma.latency_ms = latency_ms;
                                 w.gemma.tokens_per_sec = 0.0; // Requires proto update for real value
                             }

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
                                     metrics.signal_processed.add(1, &kv);
                                     info!("RISK: ALLOWED. Executing Strategy: {:?}", final_proposal);
                                     
                                     // --- D-21: Friction Logging ---
                                     // ... (existing code)
                                     
                                     // ... existing accounting logic ...
                                     // D-23: Sniper Execution (Shadow Limit)
                                     _execution.execute_sniper(&final_proposal).await;
                                 },
                                 taleb::RiskVerdict::Veto(reason) => {
                                     metrics.risk_vetos.add(1, &kv);
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
        metrics.loop_duration.record(loop_start.elapsed().as_secs_f64() * 1000.0, &kv);
    }
    
    Ok(())
}

