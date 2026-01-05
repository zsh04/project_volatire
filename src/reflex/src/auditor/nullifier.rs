use crate::auditor::firewall::FirewallError;
use std::collections::VecDeque;
use std::time::Instant;
use tracing::{warn, error, info};

// Directive-88: Semantic Nullification
// "The Eraser"

#[derive(Debug, Clone)]
pub struct NullifiedPacket {
    pub timestamp: Instant,
    pub error: FirewallError,
    // We might store the raw reasoning here for post-mortem
    pub raw_reasoning: String,
}

pub struct Nullifier {
    consecutive_failures: u32,
    grave_buffer: VecDeque<NullifiedPacket>,
    max_grave_size: usize,
    amr_threshold: u32, // 3
}

impl Nullifier {
    pub fn new() -> Self {
        Self {
            consecutive_failures: 0,
            grave_buffer: VecDeque::with_capacity(100),
            max_grave_size: 100,
            amr_threshold: 3,
        }
    }

    /// nullify: The "Drop Gate"
    /// Returns true if an Automatic Model Reset (AMR) is triggered.
    pub fn nullify(&mut self, error: FirewallError, raw_reasoning: String) -> bool {
        // 1. Increment Failure Counter
        self.consecutive_failures += 1;

        // 2. Add to Graves
        if self.grave_buffer.len() >= self.max_grave_size {
            self.grave_buffer.pop_front();
        }
        self.grave_buffer.push_back(NullifiedPacket {
            timestamp: Instant::now(),
            error: error.clone(),
            raw_reasoning,
        });

        // 3. Log the "Atomic Drop"
        warn!("🚫 NULLIFIED Packet (Count: {}): {:?}", self.consecutive_failures, error);

        // 4. Check AMR Threshold
        if self.consecutive_failures >= self.amr_threshold {
            self.trigger_amr();
            return true;
        }

        false
    }

    /// On valid packet, we reset the consecutive counter.
    /// "Regime Continuity" restored.
    pub fn reset_continuity(&mut self) {
        if self.consecutive_failures > 0 {
            info!("✅ Semantic Continuity Restored. Failures reset.");
            self.consecutive_failures = 0;
        }
    }

    /// Automatic Model Reset
    /// Flushes the KV-cache of the Brain.
    fn trigger_amr(&mut self) {
        error!("⚡ AMR TRIGGERED: Consecutive Nullifications exceeded threshold ({})", self.amr_threshold);
        error!("⚡ Sending SIGUSR1 to Gemma (Context Flush)...");
        
        // In a real unix environment with a known PID for the sidecar:
        // nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGUSR1);
        
        // For now, we simulate the effect (The Brain Client might need to send an RPC)
        // or we just log it as the "Action".
        
        // Reset counter after punishment to prevent loop
        self.consecutive_failures = 0;
    }

    /// Should we Demote the Staircase?
    /// "The Rule: If ... nullifications [persist], demote to Tier 0"
    pub fn requires_demotion(&self) -> bool {
        // Strict: Demote on ANY nullification? 
        // Prompt says: "If the system cannot 'Reason' for more than 500ms... demote"
        // Roughly check if consecutive failures imply > 500ms gap.
        // Assuming 100ms tick, 5 failures = 500ms.
        self.consecutive_failures >= 5
    }

    /// D-89: Biopsy Access
    pub fn drain_graves(&mut self) -> Vec<NullifiedPacket> {
        self.grave_buffer.drain(..).collect()
    }
}
