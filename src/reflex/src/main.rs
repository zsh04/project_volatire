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
    let mut physics = feynman::PhysicsEngine::new(1000); // 1000 tick history
    
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
            
            // B. Log (Telemetry) - throttle logs if needed, but for verification: log all
            // Format: [PHYSICS] t=... p=... v=... a=... j=... H=... ER=...
            info!(
                "REFLEX PHYSICS: p={:.2} v={:.4} a={:.4} j={:.4} H={:.2} ER={:.2} | Vol={:.4}",
                state.price,
                state.velocity,
                state.acceleration,
                state.jerk,
                state.entropy,
                state.efficiency_index,
                state.volatility
            );
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
