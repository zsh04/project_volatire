// Directive-86: Sovereign Interface Law (The Bridge)
// High-priority command channel for pilot strategic oversight

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::time::Instant;

/// Sovereign commands that bypass the autonomous OODA loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereignCommand {
    /// Emergency stop - halt all trading immediately
    Kill,
    
    /// Block the next trade decision
    Veto,
    
    /// Enter tactical pause - observe but don't trade
    Pause,
    
    /// Resume trading from tactical pause
    Resume,
    
    /// Close all open positions immediately
    CloseAll,
    
    /// Override Hypatia sentiment with manual weight (0.0-1.0)
    SetSentimentOverride(f64),
    
    /// Clear sentiment override, return to autonomous
    ClearSentimentOverride,
}

/// Authority Bridge - manages sovereign command channel
pub struct AuthorityBridge {
    command_rx: mpsc::UnboundedReceiver<SovereignCommand>,
    tactical_pause: bool,
    sentiment_override: Option<f64>,
    last_command_latency_us: u64,
    total_commands_processed: u64,
}

impl AuthorityBridge {
    /// Create new authority bridge and return sender for external use
    pub fn new() -> (Self, mpsc::UnboundedSender<SovereignCommand>) {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let bridge = Self {
            command_rx: rx,
            tactical_pause: false,
            sentiment_override: None,
            last_command_latency_us: 0,
            total_commands_processed: 0,
        };
        
        tracing::info!("🎛️ Authority Bridge initialized");
        
        (bridge, tx)
    }
    
    /// Check for sovereign commands at start of OODA loop
    /// CRITICAL: Must be called before any autonomous logic
    /// Returns Some(command) if intervention required
    pub fn check_intervention(&mut self) -> Option<SovereignCommand> {
        let start = Instant::now();
        
        // Non-blocking check (required for <10μs latency)
        match self.command_rx.try_recv() {
            Ok(cmd) => {
                let latency_us = start.elapsed().as_micros() as u64;
                self.last_command_latency_us = latency_us;
                self.total_commands_processed += 1;
                
                // Update internal state based on command
                match &cmd {
                    SovereignCommand::Pause => {
                        self.tactical_pause = true;
                        tracing::warn!("⏸️ TACTICAL PAUSE ENABLED");
                    }
                    SovereignCommand::Resume => {
                        self.tactical_pause = false;
                        tracing::info!("▶️ TACTICAL PAUSE DISABLED");
                    }
                    SovereignCommand::SetSentimentOverride(val) => {
                        self.sentiment_override = Some(*val);
                        tracing::warn!(
                            "🎚️ SENTIMENT OVERRIDE: {:.2} (manual control)",
                            val
                        );
                    }
                    SovereignCommand::ClearSentimentOverride => {
                        self.sentiment_override = None;
                        tracing::info!("🎚️ SENTIMENT OVERRIDE CLEARED");
                    }
                    SovereignCommand::Kill => {
                        tracing::error!("🛑 SOVEREIGN KILL COMMAND RECEIVED");
                    }
                    SovereignCommand::Veto => {
                        tracing::warn!("⛔ SOVEREIGN VETO");
                    }
                    SovereignCommand::CloseAll => {
                        tracing::warn!("📛 CLOSE ALL POSITIONS");
                    }
                }
                
                // Log latency warning if threshold exceeded
                if latency_us > 10 {
                    tracing::warn!(
                        "⚠️ Sovereign command latency: {}μs (threshold: 10μs)",
                        latency_us
                    );
                }
                
                Some(cmd)
            }
            Err(_) => None,
        }
    }
    
    /// Check if system is in tactical pause
    pub fn is_paused(&self) -> bool {
        self.tactical_pause
    }
    
    /// Get current sentiment override value if set
    pub fn sentiment_override(&self) -> Option<f64> {
        self.sentiment_override
    }
    
    /// Get last command processing latency in microseconds
    pub fn last_command_latency_us(&self) -> u64 {
        self.last_command_latency_us
    }
    
    /// Get total number of sovereign commands processed
    pub fn total_commands(&self) -> u64 {
        self.total_commands_processed
    }
}

impl Default for AuthorityBridge {
    fn default() -> Self {
        let (bridge, _tx) = Self::new();
        bridge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authority_bridge_creation() {
        let (bridge, _tx) = AuthorityBridge::new();
        assert!(!bridge.is_paused());
        assert!(bridge.sentiment_override().is_none());
        assert_eq!(bridge.total_commands(), 0);
    }

    #[test]
    fn test_tactical_pause() {
        let (mut bridge, tx) = AuthorityBridge::new();
        
        // Send pause command
        tx.send(SovereignCommand::Pause).unwrap();
        
        // Check intervention
        let cmd = bridge.check_intervention();
        assert!(matches!(cmd, Some(SovereignCommand::Pause)));
        assert!(bridge.is_paused());
        
        // Send resume command
        tx.send(SovereignCommand::Resume).unwrap();
        let cmd = bridge.check_intervention();
        assert!(matches!(cmd, Some(SovereignCommand::Resume)));
        assert!(!bridge.is_paused());
    }

    #[test]
    fn test_sentiment_override() {
        let (mut bridge, tx) = AuthorityBridge::new();
        
        // Set override
        tx.send(SovereignCommand::SetSentimentOverride(0.3)).unwrap();
        bridge.check_intervention();
        
        assert_eq!(bridge.sentiment_override(), Some(0.3));
        
        // Clear override
        tx.send(SovereignCommand::ClearSentimentOverride).unwrap();
        bridge.check_intervention();
        
        assert_eq!(bridge.sentiment_override(), None);
    }

    #[test]
    fn test_command_latency_tracking() {
        let (mut bridge, tx) = AuthorityBridge::new();
        
        tx.send(SovereignCommand::Veto).unwrap();
        bridge.check_intervention();
        
        // Latency should be tracked (likely < 10μs in test)
        assert!(bridge.last_command_latency_us() < 1000);
        assert_eq!(bridge.total_commands(), 1);
    }

    #[test]
    fn test_multiple_commands() {
        let (mut bridge, tx) = AuthorityBridge::new();
        
        // Send multiple commands
        tx.send(SovereignCommand::Pause).unwrap();
        tx.send(SovereignCommand::SetSentimentOverride(0.5)).unwrap();
        
        // First command
        bridge.check_intervention();
        assert!(bridge.is_paused());
        
        // Second command
        bridge.check_intervention();
        assert_eq!(bridge.sentiment_override(), Some(0.5));
        
        assert_eq!(bridge.total_commands(), 2);
    }
}
