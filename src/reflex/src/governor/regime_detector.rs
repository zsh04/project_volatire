use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketRegime {
    /// Regime 0: Sharp, singular peak in |ψ|^2. Consistent Phase Gradient.
    /// Safety Staircase allowed to Level Up.
    Laminar, 
    
    /// Regime 1: Multiple peaks, rapid oscillations. 
    /// Safety Staircase locked. Mean-Reversion logic.
    Turbulent,
    
    /// Regime 2: Wave function spreads across price axis. Zero identifiable phase.
    /// Soft Veto triggers. Exit all positions.
    Decoherent,
}

pub struct RegimeDetector {
    current_regime: MarketRegime,
    
    // Hysteresis State
    pending_regime: Option<MarketRegime>,
    confirmation_counter: u32,
    required_confirmations: u32,
}

impl RegimeDetector {
    pub fn new(required_confirmations: u32) -> Self {
        Self {
            current_regime: MarketRegime::Laminar, // Default to optimistic, Staircase checks will catch up
            pending_regime: None,
            confirmation_counter: 0,
            required_confirmations,
        }
    }

    pub fn current_regime(&self) -> MarketRegime {
        self.current_regime
    }

    /// Update the detector with new wave metrics.
    /// 
    /// # Arguments
    /// * `coherence` - Derived from Wave Function (0.0 to 1.0). High is Laminar.
    /// * `entropy` - Market Entropy (0.0 to 1.0). High is Decoherent.
    pub fn update(&mut self, coherence: f64, entropy: f64) -> MarketRegime {
        let raw_regime = self.classify_snapshot(coherence, entropy);
        self.apply_hysteresis(raw_regime)
    }

    fn classify_snapshot(&self, coherence: f64, entropy: f64) -> MarketRegime {
        // Thresholds based on Directive-65 requirements
        // Laminar: High Coherence, Low Entropy
        // Decoherent: High Entropy (regardless of coherence usually, but low coherence implied)
        // Turbulent: In between
        
        if entropy > 0.8 || coherence < 0.2 {
            return MarketRegime::Decoherent;
        }

        if coherence > 0.7 && entropy < 0.4 {
            return MarketRegime::Laminar;
        }

        // Default to Turbulent (Regime 1) for mixed signals
        MarketRegime::Turbulent
    }

    fn apply_hysteresis(&mut self, candidate: MarketRegime) -> MarketRegime {
        // If candidate matches current, reset pending
        if candidate == self.current_regime {
            self.pending_regime = None;
            self.confirmation_counter = 0;
            return self.current_regime;
        }

        // If candidate matches pending, increment counter
        if let Some(pending) = self.pending_regime {
            if pending == candidate {
                self.confirmation_counter += 1;
                
                if self.confirmation_counter >= self.required_confirmations {
                    // Transition confirmed
                    self.current_regime = candidate;
                    self.pending_regime = None;
                    self.confirmation_counter = 0;
                }
            } else {
                // Candidate changed mid-transition, reset
                self.pending_regime = Some(candidate);
                self.confirmation_counter = 1;
            }
        } else {
            // New transition candidate
            self.pending_regime = Some(candidate);
            self.confirmation_counter = 1;
        }

        self.current_regime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let detector = RegimeDetector::new(3);
        assert_eq!(detector.current_regime(), MarketRegime::Laminar);
    }

    #[test]
    fn test_instant_classification() {
        let detector = RegimeDetector::new(0); // No hysteresis
        let regime = detector.classify_snapshot(0.9, 0.1);
        assert_eq!(regime, MarketRegime::Laminar);

        let regime = detector.classify_snapshot(0.1, 0.9);
        assert_eq!(regime, MarketRegime::Decoherent);

        let regime = detector.classify_snapshot(0.5, 0.5);
        assert_eq!(regime, MarketRegime::Turbulent);
    }

    #[test]
    fn test_hysteresis_transition() {
        let mut detector = RegimeDetector::new(3);
        
        // Start Laminar
        assert_eq!(detector.current_regime, MarketRegime::Laminar);

        // Feed Decoherent signals
        // 1
        detector.update(0.1, 0.9); 
        assert_eq!(detector.current_regime, MarketRegime::Laminar, "Should hold regime 1/3");

        // 2
        detector.update(0.1, 0.9);
        assert_eq!(detector.current_regime, MarketRegime::Laminar, "Should hold regime 2/3");

        // 3 (Transition)
        detector.update(0.1, 0.9);
        assert_eq!(detector.current_regime, MarketRegime::Decoherent, "Should transition at 3/3");
    }

    #[test]
    fn test_hysteresis_reset() {
        let mut detector = RegimeDetector::new(3);
        
        // Feed Decoherent (1/3)
        detector.update(0.1, 0.9);
        assert_eq!(detector.pending_regime, Some(MarketRegime::Decoherent));

        // Feed Laminar (Reset)
        detector.update(0.9, 0.1); 
        assert_eq!(detector.confirmation_counter, 0, "Counter should reset");
        assert_eq!(detector.pending_regime, None, "Pending should clear");
    }
}
