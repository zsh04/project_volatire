use tracing::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

// Directive-107: The Legislature
// Manages manual overrides and strategic bias injected by the Pilot.

#[derive(Debug, Clone, PartialEq)]
pub enum StrategicBias {
    Neutral,
    LongOnly,
    ShortOnly,
}

impl Default for StrategicBias {
    fn default() -> Self {
        Self::Neutral
    }
}

impl From<&str> for StrategicBias {
    fn from(s: &str) -> Self {
        match s {
            "LONG_ONLY" => Self::LongOnly,
            "SHORT_ONLY" => Self::ShortOnly,
            _ => Self::Neutral,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LegislativeState {
    pub bias: StrategicBias,
    pub aggression: f64, // Fidelity Multiplier (0.1 - 2.0)
    pub maker_only: bool,
    pub hibernation: bool,
}

impl Default for LegislativeState {
    fn default() -> Self {
        Self {
            bias: StrategicBias::Neutral,
            aggression: 1.0,
            maker_only: false,
            hibernation: false,
        }
    }
}

pub struct Legislator {
    state: Arc<RwLock<LegislativeState>>,
}

impl Legislator {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(LegislativeState::default())),
        }
    }

    pub async fn update(&self, bias: &str, aggression: f64, maker_only: bool, hibernation: bool) {
        let mut w_guard = self.state.write().await;
        w_guard.bias = StrategicBias::from(bias);
        w_guard.aggression = aggression.clamp(0.1, 2.0);
        w_guard.maker_only = maker_only;
        w_guard.hibernation = hibernation;
        
        info!(
            "⚖️ LEGISLATURE UPDATED: Bias={:?}, Aggression={:.2}, MakerOnly={}, Hibernation={}",
            w_guard.bias, w_guard.aggression, w_guard.maker_only, w_guard.hibernation
        );

        if hibernation {
            warn!("☢️ HIBERNATION ACTIVE: SYSTEM IS READ-ONLY");
        }
    }

    pub async fn get_state(&self) -> LegislativeState {
        self.state.read().await.clone()
    }
}
