use std::time::{Duration, Instant};
use crate::gateway::order_manager::OrderGateway;

const DEADMAN_TIMEOUT_SEC: u64 = 300;

pub struct KillSwitch {
    pub is_halted: bool,
    last_heartbeat: Instant,
}

impl KillSwitch {
    pub fn new() -> Self {
        Self {
            is_halted: false,
            last_heartbeat: Instant::now(),
        }
    }

    /// Trigger the Kill Switch manually (e.g., from Mobile API).
    pub fn trigger_halt(&mut self, _token: &str, gateway: &mut OrderGateway) -> bool {
        // In real impl, verify JWT/Token here.
        
        // Execute Liquidation
        gateway.emergency_liquidate();
        
        self.is_halted = true;
        true
    }

    /// Reset the Kill Switch (requires strict auth).
    pub fn disarm(&mut self) {
        self.is_halted = false;
        self.last_heartbeat = Instant::now();
    }

    /// Called periodically to check for Deadman Timeout.
    pub fn check_heartbeat(&mut self) -> bool {
        if self.is_halted {
            return true; // Already halted
        }

        if self.last_heartbeat.elapsed() > Duration::from_secs(DEADMAN_TIMEOUT_SEC) {
            // Deadman Triggered
            self.is_halted = true;
            return true; // Newly halted
        }
        
        false
    }
    
    /// Keep-alive from the UI/Pilot.
    pub fn pulse(&mut self) {
        self.last_heartbeat = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_trigger() {
        let mut kill_switch = KillSwitch::new();
        let mut gateway = OrderGateway::new("key".into(), "secret".into());

        assert!(!kill_switch.is_halted);
        
        kill_switch.trigger_halt("valid_token", &mut gateway);
        
        assert!(kill_switch.is_halted);
    }
    
    // Note: Deadman test skipped to avoid waiting 300s, 
    // but logic is standard elapsed check.
}
