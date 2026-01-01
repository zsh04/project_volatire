use tonic::{Request, Response, Status};
use tracing::info;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

// Import specific items from the generated proto module
use crate::reflex_proto::reflex_service_server::ReflexService;
use crate::reflex_proto::{Ack, ConfigPayload, Empty, Heartbeat, RatchetRequest};

#[derive(Debug, Default)]
pub struct MyReflexService;

#[tonic::async_trait]
impl ReflexService for MyReflexService {
    // 1. The Ratchet
    async fn trigger_ratchet(
        &self,
        request: Request<RatchetRequest>,
    ) -> Result<Response<Ack>, Status> {
        let req = request.into_inner();
        info!("RATCCHET TRIGGERED: Level={:?} | Reason={}", req.level, req.reason);
        // TODO: Implement actual Ratchet logic (Taleb Persona)
        
        Ok(Response::new(Ack {
            success: true,
            message: "Ratchet Executed".to_string(),
        }))
    }

    // 2. The Configuration
    async fn update_config(
        &self,
        request: Request<ConfigPayload>,
    ) -> Result<Response<Ack>, Status> {
        let req = request.into_inner();
        info!("CONFIG UPDATE: {} = {}", req.key, req.value);
        
        Ok(Response::new(Ack {
            success: true,
            message: "Config Updated".to_string(),
        }))
    }

    // 3. The Status Stream
    type GetStreamStream = ReceiverStream<Result<Heartbeat, Status>>;

    async fn get_stream(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::GetStreamStream>, Status> {
        info!("Client connected to Heartbeat Stream");
        
        let (tx, rx) = mpsc::channel(4);
        
        tokio::spawn(async move {
            loop {
                // Simulate Physics Heartbeat
                let beat = Heartbeat {
                    timestamp: chrono::Utc::now().timestamp_millis() as f64,
                    brain_connected: false,
                    efficiency_index: 0.95, // High efficiency
                    p_riemann: 0.5,
                };
                
                if tx.send(Ok(beat)).await.is_err() {
                    info!("Client disconnected");
                    break;
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
