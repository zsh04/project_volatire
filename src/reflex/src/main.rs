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
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Telemetry
    tracing_subscriber::fmt::init();
    info!("Reflex (The Body) is Waking Up...");

    // 2. Bind Address
    let addr = "127.0.0.1:50051".parse()?;
    info!("ReflexD listening on {}", addr);
    
    // 3. Instantiate Service
    let reflex_service = server::MyReflexService::default();

    // 4. Start Server
    Server::builder()
        .add_service(reflex_proto::reflex_service_server::ReflexServiceServer::new(reflex_service))
        .serve(addr)
        .await?;
    
    Ok(())
}
