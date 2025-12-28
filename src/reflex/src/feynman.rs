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
    pub efficiency_index: f64, // Kaufman Efficiency Ratio
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
            efficiency_index: 0.0,
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
            // Return previous state with new price to avoid DivByZero
            let mut safe_state = self.prev_state;
            safe_state.price = price; 
            return safe_state;
        }

        // 2. Update History
        if self.history.len() >= self.window_size {
            self.history.pop_front();
        }
        self.history.push_back((timestamp, price));

        // 3. Welford's Algorithm (Volatility)
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

        // 6. Efficiency Index (Kaufman)
        let efficiency_index = self.calculate_efficiency();

        let new_state = PhysicsState {
            timestamp,
            price,
            velocity,
            acceleration,
            jerk,
            volatility,
            entropy,
            efficiency_index,
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
            .map(|(prev, curr)| curr.1 - prev.1) 
            .collect();
        
        if returns.is_empty() { return 0.0; }

        let min = returns.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = returns.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return 0.0; // No variation
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

    fn calculate_efficiency(&self) -> f64 {
        // Kaufman Efficiency Ratio (ER)
        // ER = |Direction| / Volatility
        // Direction = Price_t - Price_{t-n}
        // Volatility = Sum(|Price_i - Price_{i-1}|)
        
        if self.history.len() < 5 { return 0.0; }
        
        let start_price = self.history.front().unwrap().1;
        let end_price = self.history.back().unwrap().1;
        
        let direction = (end_price - start_price).abs();
        
        // Sum of absolute changes
        let volatility: f64 = self.history.iter()
            .zip(self.history.iter().skip(1))
            .map(|(prev, curr)| (curr.1 - prev.1).abs())
            .sum();

        if volatility < f64::EPSILON {
            if direction > 0.0 { return 1.0; } // Pure jump?
            return 0.0;
        }

        direction / volatility
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
        assert_eq!(s1.acceleration, 1.0); // First step accl is 1.0

        // t=2, p=2 (v=1, a=0)
        let s2 = engine.update(2.0, 2.0);
        assert_eq!(s2.velocity, 1.0);
        assert_eq!(s2.acceleration, 0.0); 
        assert_eq!(s2.jerk, -1.0); 
        
        // t=3, p=3 (v=1, a=0, j=0)
        let s3 = engine.update(3.0, 3.0);
        assert_eq!(s3.jerk, 0.0); 
    }

    #[test]
    fn test_jerk_spike() {
        let mut engine = PhysicsEngine::new(50);
        // Base state
        engine.update(100.0, 0.0);
        engine.update(100.0, 1.0);
        engine.update(100.0, 2.0); // v=0

        // Sudden Spike
        let s = engine.update(110.0, 3.0);
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
        
        assert!(!s.velocity.is_nan());
        assert_eq!(s.price, 105.0);
    }

    #[test]
    fn test_efficiency_ratio() {
        let mut engine = PhysicsEngine::new(10);
        
        // 1. Trending (Linear) -> ER = 1.0
        // Prices: 0, 1, 2, 3, 4, 5
        // Direction = |5 - 0| = 5
        // Volatility = |1-0| + |2-1| + ... = 1+1+1+1+1 = 5
        // ER = 5/5 = 1.0
        for i in 0..=5 {
            engine.update(i as f64, i as f64);
        }
        let s = engine.update(6.0, 6.0);
        assert!((s.efficiency_index - 1.0).abs() < 1e-6, "Linear trend should have ER=1");

        // 2. Chopping -> ER Low
        // Prices: 10, 11, 10, 11, 10, 11
        // Direction (start to end) = |11 - 10| = 1
        // Volatility = |11-10| + |10-11| + ... = 1+1+1+1+1 = 5
        // ER = 1/5 = 0.2
        let mut engine_chop = PhysicsEngine::new(10);
        engine_chop.update(10.0, 0.0);
        engine_chop.update(11.0, 1.0); // +1
        engine_chop.update(10.0, 2.0); // +1
        engine_chop.update(11.0, 3.0); // +1
        engine_chop.update(10.0, 4.0); // +1
        let s_chop = engine_chop.update(11.0, 5.0); // +1
        
        // Direction = |11-10| = 1. Vol = 5. ER = 0.2.
        assert!((s_chop.efficiency_index - 0.2).abs() < 1e-6);
    }
}
