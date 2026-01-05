use crate::auditor::nullifier::NullifiedPacket;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use chrono::SecondsFormat;

pub struct Biopsy {
    log_path: PathBuf,
}

impl Biopsy {
    pub fn new(log_path: PathBuf) -> Self {
        Self { log_path }
    }

    pub fn archive(&self, packets: Vec<NullifiedPacket>) {
        if packets.is_empty() {
            return;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .expect("Biopsy: Failed to open hallucination log");

        for packet in packets {
            // Manual JSON serialization to avoid serde overhead if possible, 
            // but for Biopsy we prefer structured data.
            // Using a simple format:
            // {"timestamp": "...", "error": "...", "reasoning": "..."}
            
            let ts = packet.timestamp; // Instant is hard to serialize to absolute time without anchor.
            // In main/nullifier, we might want SystemTime. 
            // For now, let's assume NullifiedPacket has been updated to use SystemTime or we ignore exact wall clock in this MVP 
            // and just use current write time.
            
            let now = chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true);
            
            let json_line = format!(
                "{{\"timestamp\": \"{}\", \"error\": \"{:?}\", \"reasoning\": \"{}\"}}\n",
                now,
                packet.error,
                packet.raw_reasoning.replace("\"", "\\\"").replace("\n", " ") // Basic escape
            );

            if let Err(e) = file.write_all(json_line.as_bytes()) {
                eprintln!("Biopsy: Write failed: {}", e);
            }
        }
    }
}
