use crate::historian::events::LogEvent;
use crate::historian::shm_buffer::ShmRingBuffer;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Global Singleton for the Historian (to allow easy access from anywhere like 'info!')
// In a pure architecture we might pass it down, but for a Logger replacement, Global is easier.
// Since we are Single-Producer (Main Thread), a Mutex is technically okay if contention is low (only main thread locks it).
// But 'Single Producer' requirement means we should only call this from the OODA loop thread.
// If multiple threads call it, the Mutex handles safety but adds overhead.

pub static HISTORIAN: Lazy<Mutex<Historian>> = Lazy::new(|| {
    let buffer = ShmRingBuffer::new().expect("Failed to initialize SHM Ring Buffer");
    Mutex::new(Historian { buffer })
});

pub struct Historian {
    buffer: ShmRingBuffer,
}

impl Historian {
    pub fn record(&mut self, event: LogEvent) {
        self.buffer.write(&event);
    }
}

// Public facade macro or function
pub fn record_event(event: LogEvent) {
    if let Ok(mut historian) = HISTORIAN.lock() {
        historian.record(event);
    } else {
        // Lock poisoned, ignore or eprintln
    }
}
