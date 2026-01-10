use reflex::governor::ooda_loop::{OODACore, PhysicsState};
use reflex::governor::legislator::LegislativeState;
use reflex::db::state::RedisStateStore;
use reflex::telemetry;
use std::time::Duration;
use tracing::info;

#[tokio::test]
async fn test_telemetry_emission() {
    // 0. Load Env
    dotenvy::dotenv().ok();
    // 1. Init Telemetry
    telemetry::init_telemetry().unwrap();
    info!("TEST: Telemetry Initialized.");

    // 2. Setup OODA
    let store = RedisStateStore::new("redis://127.0.0.1:6379/").await.unwrap();
    let mut ooda = OODACore::new("BTC-USDT".to_string(), None, None, None, store);
    
    // 3. Create Mock State
    let physics = PhysicsState {
        timestamp: 1234567890.0,
        price: 50000.0,
        velocity: 10.0,
        acceleration: 0.5,
        jerk: 0.1,
        ..Default::default()
    };

    // 4. Run Cycle (Should generate spans)
    info!("TEST: Running OODA Cycle...");
    let state = ooda.orient(physics, 0, None, "Neutral".to_string()).await; // Negative sentiment simulated by None fallback
    let legislation = LegislativeState::default();
    let _decision = ooda.decide(&state, &legislation);
    
    // 5. Wait for batch flush (Batch processor default 1s or size)
    info!("TEST: Waiting for export...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // 6. Shutdown
    telemetry::shutdown_telemetry();
    info!("TEST: Complete.");
}
