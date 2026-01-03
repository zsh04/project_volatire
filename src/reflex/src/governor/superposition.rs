use crate::governor::ooda_loop::PhysicsState;

pub struct RiemannEngine;

impl RiemannEngine {
    /// Calculates P_Riemann: The probability that the market is in an "Orderly" (Riemannian) state.
    /// Returns 0.0 (Chaotic/MeanRev) to 1.0 (Orderly/Momentum).
    pub fn calculate_riemann_probability(
        physics: &PhysicsState, 
        entropy: f64, 
        efficiency: f64,
        simons_confidence: f64 // 0.0 to 1.0
    ) -> f64 {
        // 1. Structural Noise Guard (Flash Crash)
        // If absolute jerk is massive, market is discontinuous/broken.
        if physics.jerk.abs() > 50.0 {
            return 0.0;
        }

        // 2. Normalization (Heuristic Deciles 0.0 to 1.0)
        // ideally 0 = Bad for Momentum, 1 = Good for Momentum
        
        // Efficiency: Direct mapping. 1.0 is pure trend.
        let n_eta = efficiency.clamp(0.0, 1.0);
        
        // Entropy: Inverse. High entropy (randomness) is bad for simple momentum.
        // Assuming Entropy range 0..3ish.
        let n_entropy = (1.0 - (entropy / 3.0)).clamp(0.0, 1.0);
        
        // Jerk: Inverse. Low jerk is smooth trend.
        // Normalize 0..1.0 range usually found in stable moves.
        let n_jerk = (1.0 - physics.jerk.abs().clamp(0.0, 1.0)).clamp(0.0, 1.0);

        // Confidence: Direct.
        let n_conf = simons_confidence.clamp(0.0, 1.0);

        // 3. Weighted Consensus
        // Directive: "If eta > 0.85, favor Momentum even if Entropy is elevated"
        // Base Weights
        let w_eta = 0.4;
        let w_entropy = 0.2;
        let w_jerk = 0.2;
        let w_conf = 0.2;
        
        let mut raw_score = (n_eta * w_eta) + (n_entropy * w_entropy) + (n_jerk * w_jerk) + (n_conf * w_conf);
        
        // Boost for Laminar Flow
        if efficiency > 0.85 {
            raw_score += 0.2; // Significant boost
        }

        // 4. Sigmoid Smoothing
        // Center around 0.5, steepness 10
        let k = 10.0;
        let x0 = 0.5;
        let sigmoid = 1.0 / (1.0 + (-k * (raw_score - x0)).exp());

        sigmoid.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_purity() {
        // High Efficiency, Moderate Entropy, Low Jerk -> Should be High Momentum
        let p = PhysicsState {
            symbol: "BTC".to_string(),
            price: 100.0,
            velocity: 1.0,
            acceleration: 0.0,
            jerk: 0.01,
            basis: 0.0,
        };
        let entropy = 1.5; // Moderate disorder
        let efficiency = 0.9; // Very High Efficiency (Laminar)
        let conf = 0.8;

        let riemann_prob = RiemannEngine::calculate_riemann_probability(&p, entropy, efficiency, conf);
        
        println!("Trend Purity Score: {}", riemann_prob);
        assert!(riemann_prob > 0.70, "Failed Trend Purity! Score: {}", riemann_prob);
    }

    #[test]
    fn test_structural_noise() {
        // Flash Crash Scenario
        let p = PhysicsState {
            symbol: "BTC".to_string(),
            price: 100.0,
            velocity: -100.0,
            acceleration: -500.0,
            jerk: 60.0, // > 50.0 Threshold
            basis: 0.0,
        };
        
        let riemann_prob = RiemannEngine::calculate_riemann_probability(&p, 0.5, 0.5, 0.5);
        
        assert_eq!(riemann_prob, 0.0, "Failed Structural Noise Guard!");
    }

    #[test]
    fn test_benchmark_speed() {
        let p = PhysicsState {
            symbol: "BTC".to_string(),
            price: 100.0,
            velocity: 1.0,
            acceleration: 0.0,
            jerk: 0.01,
            basis: 0.0,
        };
        
        let start = std::time::Instant::now();
        for _ in 0..10_000 {
            std::hint::black_box(RiemannEngine::calculate_riemann_probability(&p, 1.5, 0.9, 0.8));
        }
        let elapsed = start.elapsed();
        let avg = elapsed.as_nanos() / 10_000;
        
        println!("Avg Latency: {} ns", avg);
        assert!(avg < 10_000, "Too slow! {} ns", avg); // < 10us = 10,000ns
    }
}
