use crate::auditor::truth_envelope::TruthEnvelope;
use rand::Rng; // Requirement: Chaotic Randomness
use tracing::warn;

pub struct RedTeam {
    pub active: bool,
    pub skew_prob: f64,
    pub flash_prob: f64,
    pub poison_prob: f64,
}

impl RedTeam {
    pub fn new() -> Self {
        // Default to ACTIVE but low probability for "Background Radiation" testing
        // In real prod, this is disabled by default.
        Self {
            active: true, 
            skew_prob: 0.1,  // 10% chance of clock skew
            flash_prob: 0.05, // 5% chance of flash crash
            poison_prob: 0.1, // 10% chance of sentiment poison
        }
    }

    #[cfg(test)]
    pub fn all_out_war() -> Self {
        Self {
            active: true,
            skew_prob: 1.0,
            flash_prob: 1.0,
            poison_prob: 1.0,
        }
    }

    pub fn inject_chaos(&self, truth: &mut TruthEnvelope) {
        if !self.active { return; }
        
        let mut rng = rand::thread_rng();

        // 1. Vector A: Temporal Skew (The "Lagging Feed")
        if rng.gen_bool(self.skew_prob) {
            warn!("🔴 RED TEAM: Injecting Temporal Skew (-500ms)");
            // Simulated by altering the timestamp relative to "now" checks downstream
            // Or just mutating the record to look old.
            truth.timestamp -= 0.5; 
        }

        // 2. Vector B: The "Lying Exchange" (Flash Crash)
        if rng.gen_bool(self.flash_prob) {
             let shock = if rng.gen_bool(0.5) { 1.05 } else { 0.95 }; // +/- 5%
             warn!("🔴 RED TEAM: Injecting Price Flash (* {:.2})", shock);
             truth.mid_price *= shock;
             // Also spike acceleration to allow jerk checks to catch it if price check fails
             truth.acceleration *= 10.0; 
        }

        // 3. Vector C: Sentiment Poisoning (The "Hallucination")
        if rng.gen_bool(self.poison_prob) {
             // Invert sentiment against reality
             // If physics says crash (accel < 0), we say pure euphoria (> 0.9).
             if truth.acceleration < 0.0 {
                 warn!("🔴 RED TEAM: Injecting Sentiment Poison (Euphoria in Crash)");
                 truth.sentiment_score = 0.95;
             } else {
                 warn!("🔴 RED TEAM: Injecting Sentiment Poison (Panic in rally)");
                 truth.sentiment_score = -0.95;
             }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_vectors() {
        let red_team = RedTeam::all_out_war();
        let mut truth = TruthEnvelope::default();
        truth.timestamp = 1000.0;
        truth.mid_price = 100.0;
        truth.acceleration = -5.0; // Crash scenario

        red_team.inject_chaos(&mut truth);

        // 1. Verify Skew
        assert!(truth.timestamp < 1000.0, "Temporal Skew failed");

        // 2. Verify Flash
        assert_ne!(truth.mid_price, 100.0, "Flash Crash failed");

        // 3. Verify Poison (Should be Euphoric > 0.9 despite crash)
        assert!(truth.sentiment_score > 0.9, "Sentiment Poison failed");
    }
}
