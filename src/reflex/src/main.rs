use tonic::transport::Server;
use tracing::info;

// Import the generated code
pub mod reflex_proto {
    // The build script generates the code in OUT_DIR.
    // We include it here.
    tonic::include_proto!("reflex");
}

pub mod brain_proto {
     tonic::include_proto!("brain");
}

// Declare local modules
pub mod feynman;
pub mod market;
pub mod ingest;
pub mod ledger;
pub mod taleb;
pub mod simons;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Telemetry
    tracing_subscriber::fmt::init();
    info!("Reflex (The Body) is Waking Up...");

    // 2. Bind Address
    let addr = "127.0.0.1:50051".parse()?;
    info!("ReflexD listening on {}", addr);

    // 3. Reflex Core Init
    let mut physics = feynman::PhysicsEngine::new(2000); // 2000 tick capacity
    let mut _ledger = ledger::AccountState::new(0.0, 0.0); // Shadow Ledger (Empty init)
    let _risk = taleb::RiskGuardian::new(); // Taleb
    let mut simons = simons::EchoStateNetwork::new(100); // Simons (100 neurons)
    
    info!("🛡️ Taleb Risk Engine: ARMED");
    info!("🧠 Simons Pattern Matcher: ONLINE (N=100)");

    // 4. Ingest (Eyes)
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    // Connect to Binance.com (Directive-07 Requirement)
    tokio::spawn(async move {
        ingest::connect("btcusdt", tx).await; 
    });

    // 5. Main Event Loop (The Pulse)
    tokio::spawn(async move {
        info!("Reflex: Main Loop Started. Waiting for Market Data...");
        
        while let Some(tick) = rx.recv().await {
            // A. Update Physics (Feynman)
            let state = physics.update(tick.price, tick.timestamp);
            
            // B. Simons (Pattern Matcher)
            // Train on current observation (Velocity) - "Thinking Fast"
            simons.train(state.velocity);
            let prediction = simons.forward(state.velocity);

            // C. Log "Significant Events"
            // Threshold: Velocity > $1.0/sec or Acceleration > $0.1/sec^2
            if state.velocity.abs() > 1.0 || state.acceleration.abs() > 0.1 || state.jerk.abs() > 0.1 {
                 info!(
                    "REFLEX EVENT: v={:.2} (Pred={:.2}) a={:.2} H={:.2}",
                    state.velocity, prediction, state.acceleration, state.entropy
                );
            }
        }
    });

    // 6. Instantiate Service
    let reflex_service = server::MyReflexService::default();

    // 4. Start Server
    Server::builder()
        .add_service(reflex_proto::reflex_service_server::ReflexServiceServer::new(reflex_service))
        .serve(addr)
        .await?;
    
    Ok(())
}
