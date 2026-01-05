use std::sync::atomic::{AtomicU64, Ordering}; // Restored
pub mod sync_gate;
pub mod shadow_gate;

/// A thread-safe generator for Global Sequence IDs (GSID).
/// Ensures strict monotonicity for all system events.
pub struct Sequencer {
    counter: AtomicU64,
}

impl Sequencer {
    /// Create a new Sequencer starting from 1 (0 is reserved for "Genesis" or "Unknown")
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
        }
    }

    /// Get the next unique sequence ID.
    /// This operation is atomic and ensures each ID is strictly greater than the last.
    pub fn next(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Get the current sequence ID without incrementing (for inspection).
    pub fn current(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
}

// Global static instance could be used, or passed down.
// For now, struct is sufficient, likely instantiated in main/server.
