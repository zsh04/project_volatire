// Directive-56: The Genesis Tick - Live Feed Runner
// Binary for running OODA loop on live market data in shadow mode

use reflex::client;
use reflex::feynman;
use reflex::market;
use reflex::ingest;
use reflex::telemetry;
use reflex::audit;
use reflex::db;
use reflex::governor;

use std::time::Instant;
use tracing::{info, warn, error};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load Environment
    match dotenvy::dotenv() {
        Ok(path) => println!("✅ Loaded .env from {:?}", path),
        Err(e) => println!("⚠️ Failed to load .env: {}", e),
    }

    // Initialize Telemetry
    telemetry::init_telemetry().map_err(|e| e as Box<dyn std::error::Error>)?;
    info!("🚀 Voltaire Reflex Engine - LIVE MODE (D-56: The Genesis Tick)");

    // --- Configuration ---
    let _db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://admin:quest@localhost:8812/qdb".to_string());
    let ilp_host = std::env::var("QUESTDB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let ilp_port = std::env::var("QUESTDB_ILP_PORT").unwrap_or_else(|_| "9009".to_string());
    let ilp_addr = format!("{}:{}", ilp_host, ilp_port);

    let live_symbol = std::env::var("LIVE_SYMBOL").unwrap_or("btcusdt".to_string());
    let shadow_mode = std::env::var("SHADOW_EXECUTION").unwrap_or("true".to_string()) == "true";

    if !shadow_mode {
        warn!("⚠️ SHADOW_EXECUTION=false detected. This will execute REAL ORDERS!");
        warn!("⚠️ Directive-56 is SHADOW-ONLY. Forcing SHADOW_EXECUTION=true.");
    }

    info!("📡 Live Symbol: {} | Shadow Mode: ✅ ENABLED", live_symbol.to_uppercase());

    // --- Database Connections ---
    println!("Connecting to QuestDB at {}...", ilp_addr);
    let auditor = audit::QuestBridge::new(
        &ilp_addr,
        &ilp_host,
        "admin",
        "quest",
        "qdb"
    ).await;

    if auditor.check_connection().await {
        println!("✅ QuestDB Connection: ESTABLISHED");
    } else {
        error!("❌ QuestDB Connection: FAILED");
        return Err("QuestDB connection failed".into());
    }

    // --- DragonflyDB State Store ---
    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://localhost:6379/".to_string());
    println!("⚡ Connecting to DragonflyDB at {}...", redis_url);
    
    let state_store = match db::state::RedisStateStore::new(&redis_url).await {
        Ok(s) => {
            match s.ping().await {
                Ok(_) => {
                    println!("✅ DragonflyDB Connection: ESTABLISHED");
                    Some(s)
                },
                Err(e) => {
                    error!("❌ DragonflyDB PING FAILED: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            error!("❌ DragonflyDB Connection FAILED: {}", e);
            None
        }
    };

    // --- Telemetry Channels (D-50, D-51, D-52) ---
    let (forensic_tx, forensic_rx) = mpsc::channel(1024);
    let (mirror_tx, mirror_rx) = mpsc::channel(1024);
    let (decay_tx, decay_rx) = mpsc::channel(1024);
    let (_decay_fill_tx, decay_fill_rx) = mpsc::channel(1024);

    // Spawn Forensic Logger
    let logger_auditor = auditor.clone();
    tokio::spawn(async move {
        let scribe = telemetry::forensics::ForensicLogger::new(forensic_rx, logger_auditor);
        scribe.run().await;
    });

    // Spawn Mirror Engine
    tokio::spawn(async move {
        telemetry::mirror::MirrorEngine::new(mirror_rx).run().await;
    });

    // Spawn Decay Monitor
    tokio::spawn(async move {
        telemetry::decay::DecayMonitor::new(decay_rx, decay_fill_rx).run().await;
    });

    // --- OODA Core Initialization ---
    // In live_runner, we must panic if state store is missing as it's critical
    let store_for_ooda = state_store.clone().expect("Redis State Store is required for Live Runner");
    let mut ooda = governor::ooda_loop::OODACore::new(
        live_symbol.clone(),
        Some(forensic_tx),
        Some(mirror_tx),
        Some(decay_tx),
        store_for_ooda
    );

    // --- Connect to Brain Service ---
    let brain_url = std::env::var("BRAIN_SERVICE_URL").unwrap_or("http://[::1]:50052".to_string());
    info!("🔌 Connecting to Brain Service at {}...", brain_url);
    
    let mut brain_client = match client::BrainClient::connect(brain_url).await {
        Ok(c) => {
            info!("✅ Connected to Brain Service");
            Some(c)
        },
        Err(e) => {
            warn!("⚠️ Failed to connect to Brain: {}. Running AUTONOMOUS.", e);
            None
        }
    };

    // --- Live Feed Connection ---
    let (tick_tx, mut tick_rx) = mpsc::channel::<market::Tick>(10_000);
    
    info!("📡 CONNECTING TO LIVE FEED: {}", live_symbol.to_uppercase());
    let symbol_for_ingest = live_symbol.clone(); // Clone before moving into spawn
    tokio::spawn(async move {
        ingest::connect(&symbol_for_ingest, tick_tx).await;
    });

    // --- Physics Engine ---
    let mut feynman = feynman::PhysicsEngine::new(2000);

    // --- Metrics ---
    let meter = opentelemetry::global::meter("reflex_live");
    let heartbeat = meter.u64_counter("live_heartbeat").init();
    let tick_counter = meter.u64_counter("live_ticks_received").init();
    let latency_hist = meter.f64_histogram("ooda_latency_ms").init();
    
    let kv = vec![
        opentelemetry::KeyValue::new("mode", "live"),
        opentelemetry::KeyValue::new("symbol", live_symbol.clone()),
    ];

    info!("♻️ ENTERING LIVE OODA LOOP (SHADOW MODE)...");
    
    let mut loop_count: u64 = 0;
    let mut last_tick_time = Instant::now();
    
    while let Some(tick) = tick_rx.recv().await {
        loop_count += 1;
        let loop_start = Instant::now();

        // Metrics
        tick_counter.add(1, &kv);
        if loop_count % 100 == 0 {
            heartbeat.add(1, &kv);
        }

        // Archive Tick to Historian (D-50)
        auditor.log_tick(
            &live_symbol.to_uppercase(),
            tick.price,
            tick.quantity,
            (tick.timestamp * 1_000_000.0) as u64,
        );
        
        // Physics Update
        let spread = if let (Some(b), Some(a)) = (tick.bid, tick.ask) { a - b } else { 0.0 };
        let physics = feynman.update(tick.price, tick.timestamp, 0, spread);

        // OODA Orient
        let ooda_state = ooda.orient(physics.clone(), 0, brain_client.as_mut(), "NEUTRAL".to_string()).await;

        // OODA Decide
        let default_legislation = reflex::governor::legislator::LegislativeState::default();
        let decision = ooda.decide(&ooda_state, &default_legislation);

        // Shadow Execution Bypass
        // Action is an enum, use matches! or pattern matching
        use reflex::governor::ooda_loop::Action;
        if !matches!(decision.action, Action::Hold) {
            info!(
                "👻 SHADOW: Would execute {:?} {} @ {:.2} (Confidence: {:.2})",
                decision.action,
                live_symbol.to_uppercase(),
                tick.price,
                decision.confidence
            );
            // No real order submission in shadow mode
        }

        // Sync to DragonflyDB (D-42)
        if let Some(store) = &state_store {
            if let Err(e) = store.update_kinetics(&live_symbol.to_uppercase(), &physics).await {
                warn!("⚠️ Failed to sync kinetics: {}", e);
            }
        }

        // Latency Measurement
        let latency_ms = loop_start.elapsed().as_secs_f64() * 1000.0;
        latency_hist.record(latency_ms, &kv);

        if latency_ms > 1.0 {
            warn!("⚠️ OODA latency exceeded 1ms: {:.3}ms", latency_ms);
        }

        // Tick Rate Stats
        if loop_count % 1000 == 0 {
            let elapsed = last_tick_time.elapsed().as_secs_f64();
            let tps = 1000.0 / elapsed;
            info!("📊 Processed 1000 ticks in {:.2}s ({:.1} ticks/sec)", elapsed, tps);
            last_tick_time = Instant::now();
        }
    }

    Ok(())
}
