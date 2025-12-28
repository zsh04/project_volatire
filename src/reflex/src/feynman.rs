use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

// ==============================================================================
// 1. The Physical State Vector ($\Psi$)
// ==============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PhysicsState {
    pub timestamp: f64,
    pub price: f64,
    pub velocity: f64,      // $v = dp/dt$
    pub acceleration: f64,  // $a = dv/dt$
    pub jerk: f64,          // $j = da/dt$
    pub volatility: f64,    // Realized Volatility ($\sigma$)
    pub entropy: f64,       // Shannon Entropy ($H$)
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self {
            timestamp: 0.0,
            price: 0.0,
            velocity: 0.0,
            acceleration: 0.0,
            jerk: 0.0,
            volatility: 0.0,
            entropy: 0.0,
        }
    }
}

// ==============================================================================
// 2. The Physics Engine
// ==============================================================================

pub struct PhysicsEngine {
    window_size: usize,
    history: VecDeque<(f64, f64)>, // (timestamp, price)
    
    // Welford's Online Algorithm State
    count: usize,
    mean: f64,
    m2: f64, // Sum of squares of differences from the current mean
    
    // Previous State for Derivatives
    prev_state: PhysicsState,
}

impl PhysicsEngine {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            history: VecDeque::with_capacity(window_size),
            count: 0,
            mean: 0.0,
            m2: 0.0,
            prev_state: PhysicsState::default(),
        }
    }

    pub fn update(&mut self, price: f64, timestamp: f64) -> PhysicsState {
        // 1. Calculate Time Delta
        let dt = timestamp - self.prev_state.timestamp;
        
        // Guard: Zero Time Delta (Microsecond collision)
        if dt <= 0.0 && self.count > 0 {
            // Return previous state to avoid Division by Zero
            // Update price but keep derivatives static (or zero them? static is safer for continuity)
            let mut safe_state = self.prev_state;
            safe_state.price = price; 
            return safe_state;
        }

        // 2. Update History (For Entropy)
        if self.history.len() >= self.window_size {
            self.history.pop_front();
        }
        self.history.push_back((timestamp, price));

        // 3. Welford's Algorithm (Volatility)
        // We track returns for volatility, not raw price
        let return_val = if self.prev_state.price != 0.0 {
            (price - self.prev_state.price) / self.prev_state.price
        } else {
            0.0
        };
        
        self.count += 1;
        let delta = return_val - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = return_val - self.mean;
        self.m2 += delta * delta2;
        
        let variance = if self.count < 2 {
            0.0
        } else {
            self.m2 / (self.count - 1) as f64
        };
        let volatility = variance.sqrt();

        // 4. Derivatives (Finite Difference)
        let velocity = if dt > 0.0 { (price - self.prev_state.price) / dt } else { 0.0 };
        let acceleration = if dt > 0.0 { (velocity - self.prev_state.velocity) / dt } else { 0.0 };
        let jerk = if dt > 0.0 { (acceleration - self.prev_state.acceleration) / dt } else { 0.0 };

        // 5. Entropy (Shannon)
        let entropy = self.calculate_entropy();

        let new_state = PhysicsState {
            timestamp,
            price,
            velocity,
            acceleration,
            jerk,
            volatility,
            entropy,
        };

        self.prev_state = new_state;
        new_state
    }

    fn calculate_entropy(&self) -> f64 {
        if self.history.len() < 10 {
            return 0.0;
        }

        // Calculate histograms of returns
        // Simple Histogram approach: 10 bins
        let returns: Vec<f64> = self.history.iter()
            .zip(self.history.iter().skip(1))
            .map(|(prev, curr)| curr.1 - prev.1) // Absolute differences for simplicity in this context
            .collect();
        
        if returns.is_empty() { return 0.0; }

        let min = returns.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = returns.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return 0.0; // No variation = Zero entropy
        }

        let bins = 10;
        let bin_width = (max - min) / bins as f64;
        let mut histogram = vec![0; bins];
        
        for r in &returns {
            let mut bin_idx = ((r - min) / bin_width).floor() as usize;
            if bin_idx >= bins { bin_idx = bins - 1; }
            histogram[bin_idx] += 1;
        }

        let total_count = returns.len() as f64;
        let mut entropy = 0.0;

        for count in histogram {
            if count > 0 {
                let p = count as f64 / total_count;
                entropy -= p * p.ln();
            }
        }
        
        entropy
    }
}

// ==============================================================================
// 3. Tests
// ==============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derivatives_linear_ramp() {
        let mut engine = PhysicsEngine::new(50);
        
        // t=0, p=0
        let s0 = engine.update(0.0, 0.0);
        assert_eq!(s0.velocity, 0.0);

        // t=1, p=1 (v=1, a=0)
        let s1 = engine.update(1.0, 1.0);
        assert_eq!(s1.velocity, 1.0);
        assert_eq!(s1.acceleration, 1.0); // First step accl is 1.0 because prev v was 0

        // t=2, p=2 (v=1, a=0)
        let s2 = engine.update(2.0, 2.0);
        assert_eq!(s2.velocity, 1.0);
        assert_eq!(s2.acceleration, 0.0); // Now it stabilizes
        assert_eq!(s2.jerk, -1.0); // Decelerating from initial acceleration
        
        // t=3, p=3 (v=1, a=0, j=0)
        let s3 = engine.update(3.0, 3.0);
        assert_eq!(s3.jerk, 0.0); // Stabilized
    }

    #[test]
    fn test_jerk_spike() {
        let mut engine = PhysicsEngine::new(50);
        // Base state
        engine.update(100.0, 0.0);
        engine.update(100.0, 1.0);
        engine.update(100.0, 2.0); // v=0, a=0

        // Sudden Spike
        let s = engine.update(110.0, 3.0);
        // v = 10/1 = 10
        // a = (10 - 0)/1 = 10
        // j = (10 - 0)/1 = 10
        assert_eq!(s.velocity, 10.0);
        assert_eq!(s.acceleration, 10.0);
        assert_eq!(s.jerk, 10.0);
    }

    #[test]
    fn test_entropy_constant() {
        let mut engine = PhysicsEngine::new(50);
        for i in 0..50 {
            let s = engine.update(100.0, i as f64);
            if i > 10 {
                assert_eq!(s.entropy, 0.0);
            }
        }
    }

    #[test]
    fn test_zero_dt_handling() {
        let mut engine = PhysicsEngine::new(50);
        engine.update(100.0, 1.0);
        let s = engine.update(105.0, 1.0); // Same timestamp!
        
        // Should not explode (NaN)
        assert!(!s.velocity.is_nan());
        assert_eq!(s.price, 105.0);
        // Should return previous velocity
        // Wait, logic says return previous state explicitly with new price
        // Previous state v was infinity? No, first update v is 0.
    }
}
