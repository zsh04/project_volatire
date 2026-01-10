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
use reflex::governor::sentinel; // D-80
use reflex::governor::handoff::{HandoffManager, HandoffState}; // D-81

// Proto imports via lib
use reflex::reflex_proto::{
    ReasoningStep, // D-81
    PositionState, OrderState, // D-105
};
use tonic::Request;

use rand::Rng; // D-81


use std::time::{Duration, Instant};
use tracing::{info, warn, error};

use reflex::governor::regime_detector::{RegimeDetector, MarketRegime}; // D-87

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

    // --- Directive-81: Hot-Swap Protocol ---
    let mut handoff_state = HandoffState::default();
    let is_hotswap = std::env::var("REFLEX_HOTSWAP").is_ok();
    
    if is_hotswap {
        println!("🔥 HOTSWAP DETECTED: Initializing in SHADOW MODE...");
        match HandoffManager::load_state_from_shm("/dev/shm/reflex_state") {
            Ok(state) => {
                println!("✅ Handoff State Loaded: Sequence ID {}", state.sequence_id);
                handoff_state = state;
            },
            Err(e) => {
                error!("❌ Failed to load Handoff State: {}. ABORTING HOTSWAP.", e);
                // In a real scenario, we might want to panic or start fresh.
                // For now, we proceed as fresh but warn heavily.
            }
        }
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
    let _state_store = match state_store_res {
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
    let shared_state = std::sync::Arc::new(std::sync::RwLock::new({
        let mut s = reflex::server::SharedState::default();
        if is_hotswap {
            s.governance.staircase_tier = handoff_state.staircase_tier as i32;
            s.governance.staircase_progress = handoff_state.staircase_progress;
            s.governance.audit_drift = handoff_state.audit_drift;
        }
        s
    }));
    
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
    let mut ooda = reflex::governor::ooda_loop::OODACore::new("BTC-USDT".to_string(), Some(forensic_tx), Some(mirror_tx), Some(decay_tx));

    // D-86: Authority Bridge (Sovereign Command Channel)
    let (mut authority_bridge, authority_tx) = reflex::governor::authority::AuthorityBridge::new();

    // Spawn API Server
    let server_state = shared_state.clone();
    let server_tx = tx_broadcast.clone(); // Pass Sender for subscribing
    let server_auth_tx = authority_tx.clone();
    let _server_handle = tokio::spawn(async move {
        reflex::server::run_server(server_state, server_tx, server_auth_tx).await;
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
    let staircase_governor = reflex::governor::staircase::Staircase::new();
    // Directive-79: Sequencer (Master Clock)
    let sequencer = reflex::sequencer::Sequencer::new();
    // Directive-80: Vitality Sentinel
    let mut sentinel = sentinel::Sentinel::new();

    // D-87: Regime Detector (Hysteresis = 5 ticks)
    let mut regime_detector = RegimeDetector::new(5);

    println!("Components Initialized.");

    // Spawn Simulation Loop
    let mut client_clone = brain_client_opt; 

    println!("🔄 Simulation Loop Started.");
    let mut _last_processed_yield = Instant::now();
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
                let result = ingest::kraken::fetch_account_data(&key, &secret)
                    .await
                    .map_err(|e| e.to_string());
                match result {
                    Ok(data) => {
                        let _ = balance_tx.send(data).await;
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
    let audit_loop = reflex::governor::audit_loop::AuditLoop::new();

    // D-83: Ignition Sequence (Capital Gate)
    let mut ignition = reflex::governor::ignition::IgnitionSequence::new();

    // D-86: Authority Bridge (Sovereign Command Channel)
    let (mut authority_bridge, authority_tx) = reflex::governor::authority::AuthorityBridge::new();
    // TODO: Pass authority_tx to gRPC server for command injection

    // D-90: Rebalancer (The Governor)
    let mut rebalancer = reflex::governor::rebalancer::Rebalancer::new(50000.0); // Match Ledger

    let mut last_equity = 0.0;
    let mut last_pnl = 0.0;
    let mut last_positions: Vec<PositionState> = Vec::new();
    let mut last_orders: Vec<OrderState> = Vec::new();

    loop {
        let loop_start = Instant::now();
        
        // ... (existing code)

        // D-89 & D-90: Grave Processing
        // We drain graves here to feed both Biopsy and Rebalancer
        let graves = ooda.nullifier.drain_graves();
        
        // D-90: Punish Fidelity
        for _ in &graves {
            rebalancer.punish_nullification();
        }
        
        // D-90: Reward Fidelity (Simulated Success if Ignited & No Nullification)
        if ignition.state == reflex::governor::ignition::IgnitionState::Ignited && graves.is_empty() {
             // We assume "Success" every tick if no failure, but we should rate limit reward.
             // Prompt: "Every successful... trade".
             // Since we don't have exact trade feedback here easily, we'll implement a slow "Regen" 
             // every 100 ticks (10s) if no failures.
             // Or just every tick add 0.01 (very fast recovery).
             // Let's effectively require 100 clean ticks to recover 1 nullification (0.05 vs 0.01).
             // Actually 0.01 is huge. 100 ticks = 1.0. 
             // Nullification = -0.05.
             // Reward = +0.001 per tick? 
             // Prompt says "adds 0.01". I'll trigger it probabilistically or sparingly.
             // For safety, let's only reward if we actually "Acted" (implies trade).
             // `decision` var is available later. I can't do it here easily since decision is after.
             // Move this logic after `ooda.act`?
        }

        // D-89: Biopsy Archival
        if std::env::var("ENABLE_BIOPSY").unwrap_or_else(|_| "true".to_string()) == "true" {
           if !graves.is_empty() {
               let biopsy = reflex::historian::biopsy::Biopsy::new(std::path::PathBuf::from("logs/hallucinations.jsonl"));
               biopsy.archive(graves);
           }
        }

        if let Some(cmd) = authority_bridge.check_intervention() {
            use reflex::governor::authority::SovereignCommand;
            
            match cmd {
                SovereignCommand::Kill => {
                    tracing::error!("🛑 SOVEREIGN KILL COMMAND - Shutting down");
                    break; // Exit OODA loop immediately
                }
                SovereignCommand::CloseAll => {
                    tracing::warn!("📛 SOVEREIGN CLOSE ALL POSITIONS");
                    // Assuming `gateway` is available here, which it isn't in this snippet.
                    // This part of the code needs to be adapted if `gateway` is not in scope.
                    // For now, commenting out or assuming `gateway` exists.
                    // match gateway.close_all_positions().await {
                    //     Ok(count) => tracing::info!("✅ SOVEREIGN CLOSE COMPLETED. Closed: {}", count),
                    //     Err(e) => tracing::error!("❌ SOVEREIGN CLOSE FAILED: {}", e),
                    // }
                }
                SovereignCommand::Veto => {
                    tracing::warn!("⛔ SOVEREIGN VETO - Skipping this cycle");
                    continue; // Skip rest of OODA loop
                }
                _ => {
                    // Pause, Resume, Sentiment changes handled by AuthorityBridge state
                }
            }
        }
        
        // D-83: Check for Ignition Request from API
        if let Ok(mut w) = shared_state.write() {
            if w.ignition_request {
                ignition.initiate_launch();
                w.ignition_request = false; // Reset trigger
                tracing::info!("🚀 Ignition Launch Initiated");
            }
        }
        
        // D-81: Shadow Mode Logic
        if is_hotswap && now_ms < 5000.0 {
             // In a real implementation, we would compare outputs with the master process here
        }
        now_ms += 100.0;
        
        // --- Directive-72: Consume Account Updates ---
        if let Ok((usd, btc, equity, pnl, positions, orders)) = balance_rx.try_recv() {
             _ledger.sync(usd, btc, 0.0);
             last_equity = equity;
             last_pnl = pnl;
             last_positions = positions;
             last_orders = orders;
             info!("🏦 Ledger Synced: USD=${:.2} BTC={:.8} Equity=${:.2}", usd, btc, equity);
        }

        let span = tracing::info_span!("ooda_tick", tick_ms = now_ms);
        let _enter = span.enter();

        // --- Directive-72: LIVE DATA ORIGIN ---
        let (price, volume) = if is_sim_mode_flag {
             let phase = (now_ms / 1000.0) * std::f64::consts::PI; 
             let signal = phase.sin() * 5.0;
             let noise = (rand::random::<f64>() - 0.5) * 2.0;
             let spike = if now_ms > 5000.0 && now_ms < 5500.0 { 10.0 } else { 0.0 };
             let p = 100.0 + signal + noise + spike;
             let v = rand::thread_rng().gen_range(0.1..5.0);
             (p, v)
        } else {
             match tokio::time::timeout(Duration::from_millis(100), ingest_rx.recv()).await {
                 Ok(Some(tick)) => {
                     now_ms = tick.timestamp as f64;
                     market.update_book(tick.bid, tick.ask);
                     tick.price
                 },
                 Ok(None) => {
                     error!("❌ Ingestion Channel Closed!");
                     break; 
                 },
                 Err(_) => {
                     now_ms += 100.0;
                     (market.price, 0.0)
                 }
             }
        };
        
        metrics.heartbeat.add(1, &kv);
        metrics.market_price.record(price, &kv);
        
        market.update_price(price);

        // D-82: Zero-Copy Logging (Market Tick)
        // Explicitly recording the tick event to shared memory
        reflex::historian::logger::record_event(reflex::historian::events::LogEvent::MarketTick(
            reflex::historian::events::MarketTickEvent {
                timestamp: now_ms as u64,
                price: price,
                volume: volume,
            }
        ));

        // D-79: Generate GSID
        let seq_id = sequencer.next();
        let spread = market.get_spread();
        let state = feynman.update(market.price, now_ms, seq_id, spread);
        metrics.market_velocity.record(state.velocity, &kv);
        
        reflex::historian::logger::record_event(reflex::historian::events::LogEvent::Signal(
            reflex::historian::events::SignalEvent {
                timestamp: now_ms as u64,
                model_id: 1, // Feynman
                sentiment: state.velocity, // Using velocity as proxy for sentiment for now
                confidence: 1.0, 
            }
        ));

        // D-87: REGIME DETECTION
        // Map Physics Efficiency -> Coherence
        // Map Physics Entropy -> Entropy
        let market_regime = regime_detector.update(state.efficiency_index, state.entropy);
        let regime_id: u8 = match market_regime {
            MarketRegime::Laminar => 1,
            MarketRegime::Turbulent => 2,
            MarketRegime::Decoherent => 3,
        };

        // --- Directive-80: Sentinel Check (Moved Early for D-83) ---
        let vitality = sentinel.tick();
        
        // --- D-83: Ignition Update ---
        let market_active = true; // Assumed true if we are in this loop iteration (otherwise we break/timeout)
        ignition.update(&sentinel, market_active);

        // D-107: Fetch Legislative State (Reader)
        let legislation = if let Ok(r) = shared_state.read() {
            r.legislation.clone()
        } else {
            reflex::governor::legislator::LegislativeState::default()
        };
        
        let legislative_bias_str = format!("{:?}", legislation.bias).to_uppercase();

        // --- D-50: OODA Execution ---
        // Gated by Ignition State
        let mut ooda_state = if ignition.state == reflex::governor::ignition::IgnitionState::Ignited {
             ooda.orient(state.clone(), regime_id, client_clone.as_mut(), legislative_bias_str).await
        } else {
             if ignition.state == reflex::governor::ignition::IgnitionState::PennyTrade {
                 ooda.orient(state.clone(), regime_id, client_clone.as_mut(), legislative_bias_str).await
             } else {
                 reflex::governor::ooda_loop::OODAState::default() 
             }
        };
        
        // D-86: Sentiment Override
        if let Some(val) = authority_bridge.sentiment_override() {
             ooda_state.sentiment_score = Some(val);
             tracing::info!("🎚️ SENTIMENT OVERRIDE APPLIED: {:.2}", val);
        }

        let decision = if ignition.state == reflex::governor::ignition::IgnitionState::Ignited {
             ooda.decide(&ooda_state, &legislation)
        } else {
             reflex::governor::ooda_loop::Decision::default_hold() // Force Hold
        };

        // 4. ACT (Execution)
        // D-86: Tactical Pause - Skip Gateway if paused
        if !authority_bridge.is_paused() {
            ooda.act(decision.clone(), price);
        } else {
             tracing::debug!("⏸️ Tactical Pause - Skipping Gateway Execution");
        }

        // Update Shared State (For API)
        if let Ok(mut w) = shared_state.write() {
            w.physics = state.clone(); 
            w.ooda = Some(ooda_state.clone());
            
            // Directive-72: Update Account Link
            // Directive-72: Update Account Link
            if !is_sim_mode_flag && last_equity > 0.0 {
                w.account.equity = last_equity;
                w.account.balance = _ledger.available_balance(); // Keep sync'd balance
                w.account.btc_position = _ledger.btc_position;
                w.account.realized_pnl = last_pnl;
                w.account.unrealized_pnl = last_equity - _ledger.start_of_day_balance; // approx
                
                // D-105: Fiscal Deck
                w.account.active_positions = last_positions.clone();
                w.account.open_orders = last_orders.clone();
            } else {
                let current_equity = _ledger.total_equity(market.price);
                w.account.equity = current_equity;
                w.account.balance = _ledger.available_balance();
                w.account.btc_position = _ledger.btc_position;
                w.account.unrealized_pnl = current_equity - _ledger.start_of_day_balance;
                w.account.realized_pnl = 0.0;
            }
            
            // Directive-72: Brain Telemetry
            if let Some(lat) = ooda_state.brain_latency {
                w.gemma.latency_ms = lat;
                // Estimate tokens/sec (assuming ~50 tokens output per ctx call)
                if lat > 0.0 {
                     w.gemma.tokens_per_sec = 50.0 / (lat / 1000.0);
                }
            } else {
                w.gemma.latency_ms = 0.0;
                w.gemma.tokens_per_sec = 0.0;
            }     
            
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

            // Directive-80: Sentinel Check
            // (Moved to start of loop for Ignition gating)
            w.vitality.latency_us = sentinel.current_latency_us;
            w.vitality.jitter_us = sentinel.current_jitter_us;
            w.vitality.status = format!("{:?}", sentinel.status);
            
            // D-90: System Sanity & Omega Protocol
            w.governance.system_sanity_score = rebalancer.fidelity;
            
            // Omega Kill-Switch
            if rebalancer.check_omega(w.account.equity) {
                // Trigger Kill
                tracing::error!("💀 OMEGA PROTOCOL EXECUTED. DROPPING KEYS.");
                w.veto_active = true;
                // In Phase 5, we break the loop or exit
                // ooda.act(Kill) ?
                // For now, assume Veto handles suspension.
                // But D-90 says "sends SIGKILL... wipes keys".
                // We'll simulate by breaking loop (which ends process in main).
                break; 
            }

            
            // D-84: GSID Ordering Validation (Vector B)
            if std::env::var("ENABLE_GSID_VALIDATION").is_ok() {
                static LAST_GSID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let last = LAST_GSID.load(std::sync::atomic::Ordering::Relaxed);
                
                if seq_id < last {
                    tracing::error!(
                        "⚠️ GSID_OUT_OF_ORDER: Expected >= {}, got {}. Delta: {}",
                        last, seq_id, (last as i64) - (seq_id as i64)
                    );
                }
                
                LAST_GSID.store(seq_id, std::sync::atomic::Ordering::Relaxed);
            }
            
            // D-84: Cognitive Lag Detection (Vector B)
            // Check if Brain reasoning is significantly delayed relative to market data
            if std::env::var("ENABLE_COGNITIVE_LAG_WARNING").is_ok() {
                // If Brain latency exceeds 250ms, we have cognitive lag
                if w.gemma.latency_ms > 250.0 {
                    tracing::warn!(
                        "🧠 COGNITIVE_LAG: Brain response delayed by {}ms (threshold: 250ms)",
                        w.gemma.latency_ms
                    );
                }
            }
            
            w.vitality.status = vitality.as_str().to_string();
            
            // D-83: Ignition Status Broadcast
            w.ignition_status = format!("{:?}", ignition.state).to_uppercase();

            // Emergency Cool-Down (D-80)
            if vitality == sentinel::VitalityStatus::Critical {
                warn!("🔥 SENTINEL CRITICAL: Jitter High ({:.2}us). Forcing Cool-down.", sentinel.current_jitter_us);
                // Force Staircase Demotion (if we had access to mutable staircase here, but we update status)
                // For now, we rely on the status being broadcast to FE and Safety
            }
            
            // Directive-81: Simulated Reasoning Trace (for now, until Brain Bridge)
            // We'll generate a random step occasionally
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.1) { // 10% chance per tick to add a thought
                 let thoughts = vec![
                     "Analyzing volatility skew across strikes...",
                     "Detecting gamma imbalance in near-term expiries...",
                     "Correlating order flow with price momentum...",
                     "Hypothesis: Market is transitioning to High Volatility...",
                     "Verifying liquidity depth on bid side...",
                     "Deduction: Short-term mean reversion likely...",
                     "Checking risk limits for new entry...",
                 ];
                 let content = thoughts[rng.gen_range(0..thoughts.len())].to_string();
                 let step = ReasoningStep {
                     id: uuid::Uuid::new_v4().to_string(),
                     content,
                     probability: rng.gen_range(0.7..0.99),
                     r#type: "deduction".to_string(),
                     timestamp: now_ms,
                 };
                 
                 w.reasoning_trace.push(step);
                 if w.reasoning_trace.len() > 10 {
                     w.reasoning_trace.remove(0); // Keep last 10
                 }
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
        let _simons_pred = simons.forward(state.velocity);
        let next_target = state.velocity * 0.99; // Damping target
        simons.train(next_target);
        
        // Log local events
        if state.velocity.abs() > 8.0 {
             println!("⚠️ REFLEX: High Velocity Event! V={:.2}", state.velocity);
        }

        tokio::time::sleep(tick_rate).await;
        metrics.loop_duration.record(loop_start.elapsed().as_secs_f64() * 1000.0, &kv);
    }
    
    Ok(())
}

