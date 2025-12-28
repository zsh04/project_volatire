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
        // 1. Calculate Time Step for Instantaneous Acceleration/Jerk
        let dt_step = timestamp - self.prev_state.timestamp;
        
        // Guard: Zero Time Delta (Microsecond collision)
        if dt_step <= 0.0 && self.count > 0 {
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
        // Note: Welford tracks volatility of "step returns", not windowed returns.
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

        // 4. Derivatives (Windowed Velocity)
        // Find price ~100ms ago for velocity calculation to smooth noise
        let window_lookback = 100.0; // ms
        let (past_ts, past_price) = self.find_tick_at(timestamp - window_lookback);
        
        let dt_window = timestamp - past_ts;
        let velocity = if dt_window > f64::EPSILON {
            (price - past_price) / dt_window
        } else {
            0.0
        };

        // Acceleration and Jerk are "Instantaneous changes in the Windowed Vector"
        // a = dv/dt (where dt is the step time, not window time)
        let acceleration = if dt_step > f64::EPSILON {
            (velocity - self.prev_state.velocity) / dt_step
        } else {
            0.0
        };

        let jerk = if dt_step > f64::EPSILON {
            (acceleration - self.prev_state.acceleration) / dt_step
        } else {
            0.0
        };

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

    // Helper: Find a tick closest to target_ts in history
    fn find_tick_at(&self, target_ts: f64) -> (f64, f64) {
        if self.history.is_empty() {
            return (0.0, 0.0);
        }
        
        // Scan backwards. Since history is sorted, we stop when we go past target.
        // Actually, we want the tick *closest* to target_ts or the first one *before* it?
        // Let's take the first one <= target_ts, or just the oldest if none exist.
        
        for (ts, price) in self.history.iter().rev() {
            if *ts <= target_ts {
                return (*ts, *price);
            }
        }
        
        // If we didn't find any tick older than target (i.e., history is shorter than window),
        // return the oldest tick available.
        self.history.front().copied().unwrap_or((0.0, 0.0))
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

        // t=100, p=100 (v = (100-0)/(100-0) = 1.0)
        let s1 = engine.update(100.0, 100.0);
        assert_eq!(s1.velocity, 1.0);
        
        // a = (v1 - v0) / (t1 - t0) = (1.0 - 0.0) / 100.0 = 0.01
        assert!((s1.acceleration - 0.01).abs() < 1e-6); 

        // t=200, p=200 (v = (200-100)/(200-100) = 1.0)
        let s2 = engine.update(200.0, 200.0);
        assert_eq!(s2.velocity, 1.0);
        
        // a = (1.0 - 1.0) / 100 = 0.0
        assert_eq!(s2.acceleration, 0.0); 
        
        // j = (0.0 - 0.01) / 100 = -0.0001
        assert!((s2.jerk - -0.0001).abs() < 1e-6); 
    }

    #[test]
    fn test_jerk_spike() {
        let mut engine = PhysicsEngine::new(50);
        // Base state
        engine.update(100.0, 0.0);
        engine.update(100.0, 1.0);
        engine.update(100.0, 2.0); // v=0

        // Sudden Spike
        // With windowed logic (lookback 100ms), we find the tick at t=0 (price 100).
        // dt_window = 3.0 - 0.0 = 3.0
        // v = (110 - 100) / 3.0 = 3.333...
        // a = (3.333 - 0) / 1.0 = 3.333...
        // j = (3.333 - 0) / 1.0 = 3.333...
        
        let s = engine.update(110.0, 3.0);
        assert!((s.velocity - 3.3333333).abs() < 1e-5);
        assert!((s.acceleration - 3.3333333).abs() < 1e-5);
        assert!((s.jerk - 3.3333333).abs() < 1e-5);
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
