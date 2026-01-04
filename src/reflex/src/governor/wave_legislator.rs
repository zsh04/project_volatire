use num_complex::Complex64;
use serde::{Deserialize, Serialize};

/// The verdict returned by the Wave Legislator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaveVerdict {
    /// High probability calculation - Go for Pre-Ignition
    Tunneling {
        probability: f64,
        target_price: f64,
    },
    /// Standard Newtonian Breakout
    Breakout {
        confidence: f64,
    },
    /// Wave function is too chaotic - Hold
    Decoherence {
        entropy: f64,
    },
    /// Not enough energy to tunnel - Hold
    BarrierBlocked,
}

pub struct WaveLegislator {
    // Configuration
    tunneling_threshold: f64, // e.g. 0.95 (Q95)
    
    // State
    current_psi: Complex64,   // Current Wave State (simplified)
    coherence_score: f64,     // 0.0 - 1.0
}

impl WaveLegislator {
    pub fn new(tunneling_threshold: f64) -> Self {
        Self {
            tunneling_threshold,
            current_psi: Complex64::default(),
            coherence_score: 1.0,
        }
    }

    pub fn set_tunneling_threshold(&mut self, new_threshold: f64) {
        self.tunneling_threshold = new_threshold.max(0.0).min(1.0);
    }

    pub fn tunneling_threshold(&self) -> f64 {
        self.tunneling_threshold
    }

    /// Evaluates the market physics to detect Quantum Tunneling potential.
    /// 
    /// # Arguments
    /// * `velocity` - Market Velocity (Momentum)
    /// * `entropy` - Market Entropy (Uncertainty)
    /// * `price` - Current Price
    /// * `barrier_V` - The Potential Barrier (Resistance Level)
    pub fn evaluate_tunneling(
        &mut self,
        velocity: f64,
        entropy: f64,
        price: f64,
        barrier_V: f64,
    ) -> WaveVerdict {
        // 1. Decoherence Check (Safety)
        // High entropy destroys wave coherence
        if entropy > 0.8 {
            self.coherence_score = (self.coherence_score * 0.9).max(0.0);
            return WaveVerdict::Decoherence { entropy };
        } else {
            self.coherence_score = (self.coherence_score + 0.1).min(1.0);
        }

        // 2. Calculate Energies
        // Kinetic Energy (T) ~ v^2 / 2m (assume mass=1)
        let kinetic_T = velocity.powi(2) * 0.5;
        
        // Potential Barrier (V)
        let potential_V = barrier_V;

        // 3. Logic Branch
        if kinetic_T > potential_V {
            // Classical Breakout: Energy exceeds barrier
            return WaveVerdict::Breakout { confidence: 1.0 };
        } else {
            // Quantum Regime: T < V
            // Calculate Tunneling Probability P = exp(-2 * sqrt(2m(V-T)) / h_bar)
            // Simplified for trading: P ~ exp(-(V-T)) scaled by coherence
            
            let energy_deficit = potential_V - kinetic_T;
            let tunneling_prob = (-energy_deficit).exp() * self.coherence_score;

            if tunneling_prob > self.tunneling_threshold {
                // Pre-Ignition Signal
                 return WaveVerdict::Tunneling {
                    probability: tunneling_prob,
                    target_price: price + (velocity * 2.0), // Projected landing zone
                };
            }
        }

        WaveVerdict::BarrierBlocked
    }
}
