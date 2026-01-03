use tonic::{Request, Response, Status};
use warp::Filter;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use futures_util::{StreamExt, SinkExt};
use serde::Serialize;

use crate::reflex_proto::reflex_service_server::ReflexService;
use crate::reflex_proto::{
    Ack, Empty, PhysicsResponse, OodaResponse, VetoRequest, 
    DemoteRequest, RatchetRequest, ConfigPayload, Heartbeat
};
use crate::feynman::PhysicsState;
use crate::governor::ooda_loop::OODAState;



// --- Shared State ---
// This is the "Brain Stem" - shared between the hot loop and the API
#[derive(Debug, Clone)]
pub struct SharedState {
    pub physics: PhysicsState,
    pub ooda: Option<OODAState>,
    pub veto_active: bool,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            physics: PhysicsState::default(),
            ooda: None,
            veto_active: false,
        }
    }
}

pub type SafeState = Arc<RwLock<SharedState>>;

// --- WebSocket Message ---
#[derive(Debug, Clone, Serialize)]
struct KineticHUD {
    ts: f64,
    price: f64,
    velocity: f64,
    jerk: f64,
    entropy: f64,
    decision: String,
}

// --- gRPC Service ---
#[derive(Debug)]
pub struct ReflexServerImpl {
    pub state: SafeState,
}

#[tonic::async_trait]
impl ReflexService for ReflexServerImpl {
    async fn get_physics(&self, _request: Request<Empty>) -> Result<Response<PhysicsResponse>, Status> {
        let r = self.state.read().map_err(|_| Status::internal("Lock poisoned"))?;
        
        Ok(Response::new(PhysicsResponse {
            price: r.physics.price,
            velocity: r.physics.velocity,
            acceleration: r.physics.acceleration,
            jerk: r.physics.jerk,
            entropy: r.physics.entropy,
            efficiency_index: r.physics.efficiency_index,
            timestamp: r.physics.timestamp,
        }))
    }

    async fn get_ooda(&self, _request: Request<Empty>) -> Result<Response<OodaResponse>, Status> {
        let r = self.state.read().map_err(|_| Status::internal("Lock poisoned"))?;
        
        if let Some(ooda) = &r.ooda {
             Ok(Response::new(OodaResponse {
                physics: Some(PhysicsResponse {
                    price: ooda.physics.price,
                    velocity: ooda.physics.velocity,
                    acceleration: ooda.physics.acceleration,
                    jerk: ooda.physics.jerk,
                    entropy: ooda.physics.entropy,
                    efficiency_index: ooda.physics.efficiency_index,
                    timestamp: ooda.physics.timestamp,
                }),
                sentiment_score: ooda.sentiment_score,
                nearest_regime: ooda.nearest_regime.as_ref().map(|s| s.clone()),
                decision: "HOLD".to_string(), // Placeholder, needs OODA decision field
                weights: std::collections::HashMap::new(),
             }))
        } else {
            Err(Status::unavailable("OODA not initialized"))
        }
    }

    async fn trigger_veto(&self, request: Request<VetoRequest>) -> Result<Response<Ack>, Status> {
        let req = request.into_inner();
        let mut w = self.state.write().map_err(|_| Status::internal("Lock poisoned"))?;
        
        w.veto_active = true;
        tracing::warn!("☢️ MANUAL VETO TRIGGERED by {}: {}", req.operator, req.reason);
        
        Ok(Response::new(Ack { success: true, message: "Veto Triggered".into() }))
    }

    async fn demote_provisional(&self, _req: Request<DemoteRequest>) -> Result<Response<Ack>, Status> {
         Ok(Response::new(Ack { success: true, message: "Use Provisional API (Upcoming)".into() }))
    }

    // --- Legacy Stubs ---
    async fn trigger_ratchet(&self, request: Request<RatchetRequest>) -> Result<Response<Ack>, Status> {
        let req = request.into_inner();
        tracing::info!("🔧 RATCHET TRIGGERED: Level {:?} | Reason: {}", req.level, req.reason);

        match req.level {
            0 => {} // IDLE
            1 => {} // TIGHTEN
            2 => {} // FREEZE
            3 => { 
                // KILL SWITCH
                tracing::error!("☢️ SYSTEM HALT COMMAND RECEIVED. INITIATING SHUTDOWN.");
                // We write to shared state so main loop can see it (if it checks)
                // Or we just exit. For safety in Phase 5, let's force exit after a brief delay to allow Ack to send.
                tokio::spawn(async move {
                     tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                     std::process::exit(0);
                });
                return Ok(Response::new(Ack { success: true, message: "SYSTEM HALTING NOW".into() }));
            }
            _ => {}
        }

        Ok(Response::new(Ack { success: true, message: "Ratchet Updated".into() }))
    }
    async fn update_config(&self, _req: Request<ConfigPayload>) -> Result<Response<Ack>, Status> {
        Ok(Response::new(Ack { success: true, message: "Legacy stub".into() }))
    }
    type GetStreamStream = tokio_stream::wrappers::ReceiverStream<Result<Heartbeat, Status>>;
    async fn get_stream(&self, _req: Request<Empty>) -> Result<Response<Self::GetStreamStream>, Status> {
        Err(Status::unimplemented("Use WebSocket /ws instead"))
    }
}

// --- Launcher ---
pub async fn run_server(
    state: SafeState, 
    tx: broadcast::Sender<SharedState>
) {
    // 1. gRPC Server
    let grpc_state = state.clone();
    let grpc_addr = "[::1]:50051".parse().unwrap();
    let reflex_service = crate::reflex_proto::reflex_service_server::ReflexServiceServer::new(ReflexServerImpl { state: grpc_state });

    tracing::info!("🚀 API Surface (gRPC) listening on {}", grpc_addr);
    
    // Spawn gRPC
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(reflex_service)
            .serve(grpc_addr)
            .await
            .expect("gRPC Server Failed");
    });

    // 2. WebSocket Server (Warp)
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let my_rx = tx.subscribe(); // Each client gets a receiver
            ws.on_upgrade(move |socket| handle_socket(socket, my_rx))
        });

    tracing::info!("📡 WebSocket Surface listening on 0.0.0.0:3030");
    warp::serve(ws_route).run(([0, 0, 0, 0], 3030)).await;
}


async fn handle_socket(ws: warp::ws::WebSocket, mut rx: broadcast::Receiver<SharedState>) {
    let (mut tx_ws, _rx_ws) = ws.split();

    while let Ok(state) = rx.recv().await {
        // Transform internal state to Public HUD JSON
        let hud = KineticHUD {
            ts: state.physics.timestamp,
            price: state.physics.price,
            velocity: state.physics.velocity,
            jerk: state.physics.jerk,
            entropy: state.physics.entropy,
            decision: if state.veto_active { "VETO".into() } else { "ACTIVE".into() },
        };

        if let Ok(json) = serde_json::to_string(&hud) {
             if tx_ws.send(warp::ws::Message::text(json)).await.is_err() {
                 break; // Client disconnected
             }
        }
    }
}
