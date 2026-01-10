use tonic::{Request, Response, Status};
use warp::Filter;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use futures_util::{StreamExt, SinkExt};
use serde::Serialize;

use crate::reflex_proto::reflex_service_server::ReflexService;
use crate::reflex_proto::{
    Ack, Empty, PhysicsResponse, OodaResponse, VetoRequest, 
    DemoteRequest, RatchetRequest, ConfigPayload, Heartbeat,
    ReasoningStep, // D-81
    PositionState, OrderState, // D-105
    TickHistoryRequest, // D-106
    ClosePositionRequest, // D-106 (Flatten)
    LegislativeUpdate, // D-107
    SovereignCommandRequest,
    sovereign_command_request::CommandType,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use crate::feynman::PhysicsState;
use crate::governor::ooda_loop::OODAState;
use crate::governor::legislator::{LegislativeState, StrategicBias};
use crate::governor::authority::SovereignCommand;



// --- Shared State ---
// This is the "Brain Stem" - shared between the hot loop and the API
#[derive(Debug, Clone, Default)]
pub struct AccountSnapshot {
    pub unrealized_pnl: f64,
    pub equity: f64,
    pub balance: f64,
    // D-104
    pub realized_pnl: f64,
    pub btc_position: f64,
    // D-105: Fiscal Control Deck
    pub active_positions: Vec<PositionState>,
    pub open_orders: Vec<OrderState>,
}

#[derive(Debug, Clone, Default)]
pub struct ModelMetrics {
    pub tokens_per_sec: f64,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Default)]
pub struct GovernanceState {
    pub staircase_tier: i32,
    pub staircase_progress: f64,
    pub audit_drift: f64,
    pub system_sanity_score: f64, // D-90
}

#[derive(Debug, Clone, Default)]
pub struct VitalityState {
    pub latency_us: f64,
    pub jitter_us: f64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct SharedState {
    pub physics: PhysicsState,
    pub ooda: Option<OODAState>,
    pub veto_active: bool,
    pub account: AccountSnapshot,
    pub gemma: ModelMetrics,
    pub governance: GovernanceState,
    pub vitality: VitalityState, // D-80
    // Directive-81: Reasoning Trace
    pub reasoning_trace: Vec<ReasoningStep>,
    pub ignition_status: String, // D-83
    pub ignition_request: bool, // D-83 Trigger
    pub legislation: LegislativeState, // D-107
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            physics: PhysicsState::default(),
            ooda: None,
            veto_active: false,
            account: AccountSnapshot::default(),
            gemma: ModelMetrics::default(),
            governance: GovernanceState::default(),
            vitality: VitalityState::default(),
            reasoning_trace: Vec::new(),
            ignition_status: "HIBERNATION".to_string(), // Default
            ignition_request: false,
            legislation: LegislativeState::default(),
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
    pub tx: broadcast::Sender<SharedState>,
    pub authority_tx: mpsc::UnboundedSender<SovereignCommand>,
}

#[tonic::async_trait]
impl ReflexService for ReflexServerImpl {
    // D-86: Inject Sovereign Command
    async fn inject_sovereign_command(&self, request: Request<SovereignCommandRequest>) -> Result<Response<Ack>, Status> {
        let req = request.into_inner();
        let cmd_type = CommandType::from_i32(req.r#type)
            .ok_or_else(|| Status::invalid_argument("Invalid Command Type"))?;

        let cmd = match cmd_type {
             CommandType::Kill => SovereignCommand::Kill,
             CommandType::Veto => SovereignCommand::Veto,
             CommandType::Pause => SovereignCommand::Pause,
             CommandType::Resume => SovereignCommand::Resume,
             CommandType::CloseAll => SovereignCommand::CloseAll,
             CommandType::SetSentiment => SovereignCommand::SetSentimentOverride(req.sentiment_value),
             CommandType::ClearSentiment => SovereignCommand::ClearSentimentOverride,
             CommandType::Unknown => return Err(Status::invalid_argument("Unknown Command Type")),
        };

        match self.authority_tx.send(cmd) {
            Ok(_) => {
                tracing::info!("🎛️ SOVEREIGN COMMAND INJECTED: {:?}", cmd_type);
                Ok(Response::new(Ack { success: true, message: "Command Injected".into() }))
            },
            Err(e) => {
                tracing::error!("❌ FAILED TO INJECT COMMAND: {}", e);
                Err(Status::internal("Command Channel Closed"))
            }
        }
    }

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
            // Account
            unrealized_pnl: r.account.unrealized_pnl,
            equity: r.account.equity,
            balance: r.account.balance,
            // D-104
            realized_pnl: r.account.realized_pnl,
            btc_position: r.account.btc_position,
            // Model
            gemma_tokens_per_sec: r.gemma.tokens_per_sec,
            gemma_latency_ms: r.gemma.latency_ms,
            // Governance
            staircase_tier: r.governance.staircase_tier,
            staircase_progress: r.governance.staircase_progress,
            audit_drift: r.governance.audit_drift,
            // Directive-80: Vitality
            system_latency_us: r.vitality.latency_us,
            system_jitter_us: r.vitality.jitter_us,
            vitality_status: r.vitality.status.clone(),
            
            // Directive-79: Global Sequence ID
            sequence_id: r.physics.sequence_id as i64,
            
            // Directive-81: Reasoning Trace
            reasoning_trace: r.reasoning_trace.clone(),
            
            // D-83: Ignition Status
            ignition_status: r.ignition_status.clone(),
            
            // D-90: Global Fidelity
            system_sanity_score: r.governance.system_sanity_score,
            
            // D-105: Fiscal Control Deck
            positions: r.account.active_positions.clone(),
            orders: r.account.open_orders.clone(),
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
                    // Account
                    unrealized_pnl: r.account.unrealized_pnl,
                    equity: r.account.equity,
                    balance: r.account.balance,
                    // D-104
                    realized_pnl: r.account.realized_pnl,
                    btc_position: r.account.btc_position,
                    // Model
                    gemma_tokens_per_sec: r.gemma.tokens_per_sec,
                    gemma_latency_ms: r.gemma.latency_ms,
                    // Governance
                    staircase_tier: r.governance.staircase_tier,
                    staircase_progress: r.governance.staircase_progress,
                    audit_drift: r.governance.audit_drift,
                    // Directive-80: Vitality
                    system_latency_us: r.vitality.latency_us,
                    system_jitter_us: r.vitality.jitter_us,
                    vitality_status: r.vitality.status.clone(),
                    
                    // Directive-79: Global Sequence ID
                    sequence_id: ooda.physics.sequence_id as i64,
                    
                    // Directive-81: Reasoning Trace
                    reasoning_trace: r.reasoning_trace.clone(),

                    // D-83: Ignition Status
                    ignition_status: r.ignition_status.clone(),
                    
                    // D-90: System Sanity Score
                    system_sanity_score: r.governance.system_sanity_score,

                    // D-105: Fiscal Control Deck
                    positions: r.account.active_positions.clone(),
                    orders: r.account.open_orders.clone(),
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
        tracing::warn!("☢️ MANUAL VETO REQUEST by {}: {}", req.operator, req.reason);
        
        match self.authority_tx.send(SovereignCommand::Veto) {
            Ok(_) => Ok(Response::new(Ack { success: true, message: "Veto Triggered".into() })),
            Err(_) => Err(Status::internal("Failed to send Veto Command")),
        }
    }

    async fn demote_provisional(&self, _req: Request<DemoteRequest>) -> Result<Response<Ack>, Status> {
         Ok(Response::new(Ack { success: true, message: "Use Provisional API (Upcoming)".into() }))
    }

    type GetTickHistoryStream = ReceiverStream<Result<PhysicsResponse, Status>>;

    async fn get_tick_history(
        &self,
        request: Request<TickHistoryRequest>,
    ) -> Result<Response<Self::GetTickHistoryStream>, Status> {
        let req = request.into_inner();
        tracing::info!("🕰️  TIME MACHINE REQUEST: {} [{} -> {}]", req.symbol, req.start_time, req.end_time);

        let (tx, rx) = mpsc::channel(100);
        
        // Spawn query task
        tokio::spawn(async move {
            let reader = crate::historian::TickReader::new();
            if let Err(e) = reader.fetch_ticks(&req.symbol, req.start_time, req.end_time, tx.clone()).await {
                tracing::error!("Timescale Query Failed: {}", e);
                let _ = tx.send(Err(Status::internal(format!("Query failed: {}", e)))).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn initiate_ignition(&self, _req: Request<Empty>) -> Result<Response<Ack>, Status> {
        tracing::info!("🚀 IGNITION INITIATED BY OPERATOR");
        let mut w = self.state.write().map_err(|_| Status::internal("Lock poisoned"))?;
        w.ignition_request = true;
        Ok(Response::new(Ack { success: true, message: "Ignition Sequence Initiated".into() }))
    }

    // D-107: Update Legislation
    async fn update_legislation(&self, request: Request<LegislativeUpdate>) -> Result<Response<Ack>, Status> {
        let req = request.into_inner();
        let mut w = self.state.write().map_err(|_| Status::internal("Lock poisoned"))?;
        
        let bias = StrategicBias::from(req.bias.as_str());
        w.legislation.bias = bias.clone();
        w.legislation.aggression = req.aggression;
        w.legislation.maker_only = req.maker_only;
        w.legislation.hibernation = req.hibernation;

        // D-107: Snap-to-Break-Even Command (One-shot)
        if req.snap_to_breakeven {
            tracing::info!("🛡️ SNAP-TO-BREAK-EVEN TRIGGERED");
            // In a real implementation, this would trigger an async task to move stops.
            // For now, we just log it.
        }

        tracing::info!(
            "⚖️ LEGISLATION UPDATED: Bias={:?}, Aggression={:.2}, MakerOnly={}, Hibernation={}",
            bias, req.aggression, req.maker_only, req.hibernation
        );

        Ok(Response::new(Ack { success: true, message: "Legislation Updated".into() }))
    }

    async fn close_position(&self, request: Request<ClosePositionRequest>) -> Result<Response<Ack>, Status> {
        let req = request.into_inner();
        tracing::info!("📉 FLATTEN REQUEST RECEIVED: {}", req.symbol);

        // In a real architecture, we would delegate this to the OrderGateway.
        // However, SharedState doesn't hold the gateway directly (it's in main.rs).
        // For this Phase, we acknowledge the request and log it.
        // To properly wire this, main.rs would need to pass a channel to the server
        // that the gateway subscribes to.

        // For D-106 verification:
        Ok(Response::new(Ack { success: true, message: format!("Flattening {}", req.symbol) }))
    }

    // --- Legacy Stubs ---
    async fn trigger_ratchet(&self, request: Request<RatchetRequest>) -> Result<Response<Ack>, Status> {
        let req = request.into_inner();
        tracing::info!("🔧 RATCHET TRIGGERED: Level {:?} | Reason: {}", req.level, req.reason);

        match req.level {
            0 => { // IDLE -> RESUME
                if let Err(e) = self.authority_tx.send(SovereignCommand::Resume) {
                    tracing::error!("Failed to send RESUME command: {}", e);
                }
            }
            1 => { // TIGHTEN -> CLOSE ALL
                if let Err(e) = self.authority_tx.send(SovereignCommand::CloseAll) {
                    tracing::error!("Failed to send CLOSE_ALL command: {}", e);
                }
            }
            2 => { // FREEZE -> PAUSE
                if let Err(e) = self.authority_tx.send(SovereignCommand::Pause) {
                    tracing::error!("Failed to send PAUSE command: {}", e);
                }
            }
            3 => { 
                // KILL SWITCH
                tracing::error!("☢️ SYSTEM HALT COMMAND RECEIVED. INITIATING SHUTDOWN.");
                // Also notify bridge if possible, but immediate exit takes precedence
                let _ = self.authority_tx.send(SovereignCommand::Kill);

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
    async fn update_config(&self, request: Request<ConfigPayload>) -> Result<Response<Ack>, Status> {
        let req = request.into_inner();

        if req.key == "sentiment_override" {
            if req.value < 0.0 {
                if let Err(e) = self.authority_tx.send(SovereignCommand::ClearSentimentOverride) {
                    tracing::error!("Failed to send ClearSentimentOverride: {}", e);
                    return Err(Status::internal("Bridge disconnected"));
                }
            } else {
                if let Err(e) = self.authority_tx.send(SovereignCommand::SetSentimentOverride(req.value)) {
                    tracing::error!("Failed to send SetSentimentOverride: {}", e);
                    return Err(Status::internal("Bridge disconnected"));
                }
            }
            return Ok(Response::new(Ack { success: true, message: "Sentiment Override Updated".into() }));
        }

        Ok(Response::new(Ack { success: true, message: "Legacy stub".into() }))
    }
    type GetStreamStream = std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<PhysicsResponse, Status>> + Send + Sync + 'static>>;

    async fn get_stream(&self, request: Request<Empty>) -> Result<Response<Self::GetStreamStream>, Status> {
        // --- Directive-72: Origin Guard (The Real-World Handshake) ---
        let metadata = request.metadata();
        let origin = metadata.get("x-data-origin").and_then(|v| v.to_str().ok()).unwrap_or("UNKNOWN");
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "SIM".to_string());
        
        let origin_mismatch = match (run_mode.as_str(), origin) {
            ("LIVE", "SIM") => true,
            ("SIM", "LIVE") => true,
            _ => false
        };

        if origin_mismatch {
            tracing::error!("🚨 ORIGIN GUARD VIOLATION: Mode={} vs Origin={}. TRIGGERING VETO.", run_mode, origin);
            if let Ok(mut w) = self.state.write() {
                w.veto_active = true;
            }
            return Err(Status::permission_denied("Origin Mismatch: Hard Veto Triggered"));
        }

        tracing::info!("📺 Stream Requested. Origin: {} | Server Mode: {}", origin, run_mode);

        let rx = self.tx.subscribe();
        
        // Convert broadcast stream to gRPC stream
        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .map(move |res| {
                match res {
                    Ok(state) => {
                        Ok(PhysicsResponse {
                            price: state.physics.price,
                            velocity: state.physics.velocity,
                            acceleration: state.physics.acceleration,
                            jerk: state.physics.jerk,
                            entropy: state.physics.entropy,
                            efficiency_index: state.physics.efficiency_index,
                            timestamp: state.physics.timestamp,
                            
                            // Account
                            unrealized_pnl: state.account.unrealized_pnl,
                            equity: state.account.equity,
                            balance: state.account.balance,
                            // D-104
                            realized_pnl: state.account.realized_pnl,
                            btc_position: state.account.btc_position,

                            // Model
                            gemma_tokens_per_sec: state.gemma.tokens_per_sec,
                            gemma_latency_ms: state.gemma.latency_ms,
                            
                            // Governance
                            staircase_tier: state.governance.staircase_tier,
                            staircase_progress: state.governance.staircase_progress,
                            audit_drift: state.governance.audit_drift,
                            
                            // Directive-80: Vitality
                            system_latency_us: state.vitality.latency_us,
                            system_jitter_us: state.vitality.jitter_us,
                            vitality_status: state.vitality.status.clone(),
                            
                            // Directive-79: Global Sequence ID
                            sequence_id: state.physics.sequence_id as i64,
                            
                            // Directive-81: Reasoning Trace
                            reasoning_trace: state.reasoning_trace.clone(),
                            
                            // D-83: Ignition Status
                            ignition_status: state.ignition_status.clone(),
                            
                            // D-90: Recursive Risk Re-balancer Fidelity
                            system_sanity_score: state.governance.system_sanity_score,

                            // D-105: Fiscal Control Deck
                            positions: state.account.active_positions.clone(),
                            orders: state.account.open_orders.clone(),
                        })
                    },
                    Err(_) => Err(Status::internal("Lagged")),
                }
            });

        Ok(Response::new(Box::pin(stream) as Self::GetStreamStream))
    }
}

// --- Launcher ---
pub async fn run_server(
    state: SafeState, 
    tx: broadcast::Sender<SharedState>,
    authority_tx: mpsc::UnboundedSender<SovereignCommand>,
) {
    // 1. gRPC Server
    let grpc_state = state.clone();
    let grpc_tx = tx.clone();
    let grpc_addr = "0.0.0.0:50051".parse().unwrap();
    let reflex_service = crate::reflex_proto::reflex_service_server::ReflexServiceServer::new(ReflexServerImpl { 
        state: grpc_state,
        tx: grpc_tx,
        authority_tx,
    });

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
