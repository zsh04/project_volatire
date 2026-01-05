use std::time::{Duration, Instant};
use crate::governor::sentinel::Sentinel;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IgnitionState {
    Hibernation,
    HardwareCheck, // Gate 1: Sentinel Stability
    WarmingUp,     // Gate 2: Market Data Flow
    PennyTrade,    // Gate 3: Live Connectivity Test
    AwaitingGemma, // Gate 4: Logic/Physics Audit
    Ignited,       // Live Trading Enabled
}

pub struct IgnitionSequence {
    pub state: IgnitionState,
    pub hardware_last_checked: Instant,
    pub warmup_start: Option<Instant>,
    pub penny_trade_id: Option<u64>,
}

impl IgnitionSequence {
    pub fn new() -> Self {
        Self {
            state: IgnitionState::Hibernation,
            hardware_last_checked: Instant::now(),
            warmup_start: None,
            penny_trade_id: None,
        }
    }

    /// User manually triggers the start sequence (e.g., from HUD)
    pub fn initiate_launch(&mut self) {
        if self.state == IgnitionState::Hibernation {
            self.state = IgnitionState::HardwareCheck;
            println!("[IGNITION] Sequence Initiated. Checking Hardware...");
        }
    }

    pub fn abort(&mut self) {
        self.state = IgnitionState::Hibernation;
        self.warmup_start = None;
        println!("[IGNITION] ABORTED. Returning to Hibernation.");
    }

    pub fn update(&mut self, sentinel: &Sentinel, market_active: bool) {
        match self.state {
            IgnitionState::Hibernation => {
                // Do nothing until triggered
            },
            IgnitionState::HardwareCheck => {
                // Gate 1: Helper function in Sentinel checks for 300s of stability
                // For development speed, we might use a shorter window if flagged, 
                // but requirement is 300s.
                if sentinel.is_stable_for(Duration::from_secs(300)) {
                    println!("[IGNITION] Hardware Integrity Verified. Warming Up...");
                    self.state = IgnitionState::WarmingUp;
                    self.warmup_start = Some(Instant::now());
                } else {
                     // If we just entered, we wait. If unstable, strict reset logic handled by Sentinel's last_instability
                }
            },
            IgnitionState::WarmingUp => {
                // Gate 2: 60s of Market Data
                if !market_active {
                    // Reset if flow stops
                    self.warmup_start = Some(Instant::now()); 
                    return;
                }
                
                if let Some(start) = self.warmup_start {
                    if start.elapsed() >= Duration::from_secs(60) {
                         println!("[IGNITION] Warmup Complete. Proceeding to Penny Trade...");
                         self.state = IgnitionState::PennyTrade;
                    }
                }
            },
            IgnitionState::PennyTrade => {
                // Gate 3: Penny Trade
                // Logic handled by OrderManager integration. 
                // We wait for external confirmation that penny trade filled.
                // For now, we assume it's pending.
            },
            IgnitionState::AwaitingGemma => {
                // Gate 4: Gemma Blessing
                // Awaiting explicit "Allow" from Brain.
            },
            IgnitionState::Ignited => {
                // Live
            }
        }
    }
    
    // Called when Penny Trade confirms fill
    pub fn confirm_penny_trade(&mut self) {
        if self.state == IgnitionState::PennyTrade {
            println!("[IGNITION] Penny Trade Confirmed. Awaiting Gemma...");
             // Skip Gemma for now in this iteration, or move to AwaitingGemma
            self.state = IgnitionState::AwaitingGemma;
        }
    }

    // Called when Brain confirms Laminar flow
    pub fn confirm_gemma_blessing(&mut self) {
        if self.state == IgnitionState::AwaitingGemma {
            println!("[IGNITION] Gemma Logic Verified. SYSTEMS IGNITED.");
            self.state = IgnitionState::Ignited;
        }
    }
}
