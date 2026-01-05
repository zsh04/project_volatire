use crate::historian::shm_buffer::ShmRingBuffer;

use std::time::Duration;
use std::env;

pub struct Archiver {
    buffer: ShmRingBuffer,
    stress_mode: bool,
    flush_interval_ms: u64,
}

impl Archiver {
    pub fn new() -> Self {
        let buffer = ShmRingBuffer::new().expect("Failed to attach to SHM");
        
        // D-84: Stress mode configuration for Vector C
        let stress_mode = env::var("HISTORIAN_STRESS_MODE").is_ok();
        let flush_interval_ms = env::var("HISTORIAN_FLUSH_INTERVAL_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100); // Default 100ms
        
        if stress_mode {
            tracing::info!(
                "📊 Archiver starting in STRESS MODE (flush every {}ms)",
                flush_interval_ms
            );
        }
        
        Self { buffer, stress_mode, flush_interval_ms }
    }

    pub fn run(&mut self) {
        loop {
            let batch_size = if self.stress_mode { 1000 } else { 100 };
            let events = self.buffer.read_batch(batch_size);
            
            if events.is_empty() {
                // Sleep if no events
                let sleep_ms = if self.stress_mode {
                    self.flush_interval_ms
                } else {
                    10
                };
                std::thread::sleep(Duration::from_millis(sleep_ms));
                continue;
            }

            // D-84: In stress mode, simulate heavy I/O by writing to /dev/null
            // In production, this would write to QuestDB
            if self.stress_mode {
                // Simulate expensive I/O operation
                use std::io::Write;
                let mut sink = std::io::sink();
                for event in &events {
                    // Serialize event to bytes (simulated)
                    let bytes = format!("{:?}\n", event).into_bytes();
                    sink.write_all(&bytes).ok();
                }
                sink.flush().ok();
            } else {
                // Normal mode: just consume events
                for _event in events {
                    // Real implementation: Insert into QuestDB
                }
            }
        }
    }
}
