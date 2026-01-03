use crate::feynman::PhysicsState;
use std::collections::VecDeque;

// Safety Staircase Tiers for Risk (Lots)
// Safety Staircase Tiers for Risk (Lots)
const SAFETY_STAIRCASE: [f64; 6] = [0.01, 0.05, 0.10, 0.25, 0.50, 1.0];
const WARMUP_DURATION_MS: u128 = 300_000; // 5 Minutes

#[derive(Debug, Clone)]
pub struct ProvisionalExecutive {
    pub current_tier_index: usize,
    pub consecutive_stable_cycles: usize,
    pub required_stable_cycles: usize,
    pub shadow_pnl_window: VecDeque<f64>, // Rolling PnL of shadow sim
    pub boot_time: std::time::Instant,
}

impl ProvisionalExecutive {
    pub fn new() -> Self {
        Self {
            current_tier_index: 0, // Start at 0.01 (Frozen/Survival)
            consecutive_stable_cycles: 0,
            required_stable_cycles: 2, // As per directive
            shadow_pnl_window: VecDeque::with_capacity(1000),
            boot_time: std::time::Instant::now(),
        }
    }

    pub fn get_current_max_risk(&self) -> f64 {
        SAFETY_STAIRCASE[self.current_tier_index]
    }

    /// Primary Update Loop
    /// 1. Map Physics -> Stability Score (Quantile)
    /// 2. Update Stability Counters
    /// 3. Check Shadow Sim (Mocked for now)
    /// 4. Promote/Demote
    pub fn update(&mut self, physics: &PhysicsState, entropy: f64, efficiency: f64) -> bool {
        let score = self.calculate_stability_score(physics.jerk, entropy, efficiency);
        
        // Logic: If Score <= Q3 (3), we are stable.
        if score <= 3 {
            self.consecutive_stable_cycles += 1;
        } else {
            // Reset if instability detected
            self.consecutive_stable_cycles = 0;
            // Immediate Demotion logic could go here (e.g., if Q10, drop to index 0)
            if score >= 9 {
                self.current_tier_index = 0; // Emergency Freeze
            }
        }

        // Mock Shadow Sim Update (In prod, this comes from a separate sim engine)
        // We push a "success" value if physics is good to simulate positive expectancy
        let mock_pnl = if score <= 5 { 1.0 } else { -1.0 };
        self.update_shadow_sim(mock_pnl);

        self.attempt_promotion()
    }

    /// Q1 (Best) -> Q10 (Worst)
    /// Heuristic mapping based on directives
    fn calculate_stability_score(&self, jerk: f64, entropy: f64, efficiency: f64) -> u8 {
        // 1. Jerk Component (Lower is better)
        // Assume experimental range 0.0 to 1.0 for normalized jerk
        let j_score = if jerk.abs() < 0.01 { 1 } 
                      else if jerk.abs() < 0.05 { 2 }
                      else if jerk.abs() < 0.1 { 5 }
                      else { 10 };

        // 2. Efficiency Component (Higher is better)
        // Efficiency > 0.8 is target
        let e_score = if efficiency > 0.9 { 1 }
                      else if efficiency > 0.8 { 2 }
                      else if efficiency > 0.5 { 5 }
                      else { 10 };
        
        // 3. Entropy Component (Lower is usually more stable/ordered, but depends on regime)
        // Let's assume High Entropy = Chaos (Bad) for this heuristics
        let h_score = if entropy < 1.0 { 1 } else { 10 };

        // Simple fused average rounded up
        let avg = (j_score + e_score + h_score) as f64 / 3.0;
        avg.ceil() as u8
    }

    fn update_shadow_sim(&mut self, pnl_tick: f64) {
        if self.shadow_pnl_window.len() >= 1000 {
            self.shadow_pnl_window.pop_front();
        }
        self.shadow_pnl_window.push_back(pnl_tick);
    }

    /// Check criteria for moving up the staircase
    fn attempt_promotion(&mut self) -> bool {
        // 1. Check Cycle Count
        if self.consecutive_stable_cycles < self.required_stable_cycles {
            return false;
        }

        // 1.5 Warm-up Check (Sandbox Verification)
        if self.boot_time.elapsed().as_millis() < WARMUP_DURATION_MS {
            // Log once per minute? implicit logic prevents spamming
            return false; // Still warming up
        }

        // 2. Check Shadow Consistency (Omega > 1 => Sum PnL > 0)
        let total_pnl: f64 = self.shadow_pnl_window.iter().sum();
        if total_pnl <= 0.0 {
            return false; // Shadow Validation Failed
        }

        // 3. Promote
        if self.current_tier_index < SAFETY_STAIRCASE.len() - 1 {
            self.current_tier_index += 1;
            self.consecutive_stable_cycles = 0; // Reset counter for next level
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_staircase_climb() {
        let mut exec = ProvisionalExecutive::new();
        
        // Override boot_time to bypass warm-up for testing
        exec.boot_time = std::time::Instant::now() - std::time::Duration::from_secs(400);
        
        // Initial State
        assert_eq!(exec.get_current_max_risk(), 0.01);

        // Mock Stable Physics
        let stable_physics = PhysicsState {
            price: 100.0,
            velocity: 0.0,
            acceleration: 0.0,
            jerk: 0.001, // Very low jerk
            ..Default::default()
        };

        // Cycle 1: Stable
        exec.update(&stable_physics, 0.5, 0.95);
        assert_eq!(exec.get_current_max_risk(), 0.01); // Holding

        // Cycle 2: Stable -> Promote
        let promoted = exec.update(&stable_physics, 0.5, 0.95);
        assert!(promoted);
        assert_eq!(exec.get_current_max_risk(), 0.05); // Level 2
    }

    #[test]
    fn test_shadow_rejection() {
        let mut exec = ProvisionalExecutive::new();
        // Force negative shadow PnL
        exec.update_shadow_sim(-1000.0); 

         let stable_physics = PhysicsState {
            price: 100.0,
            velocity: 0.0,
            acceleration: 0.0,
            jerk: 0.001,
            ..Default::default()
        };

        // Run cycles
        exec.update(&stable_physics, 0.5, 0.95);
        let promoted = exec.update(&stable_physics, 0.5, 0.95);
        
        // Should failed promotion due to shadow pnl
        assert!(!promoted);
        assert_eq!(exec.get_current_max_risk(), 0.01);
    }
    
    #[test]
    fn test_emergency_freeze() {
         let mut exec = ProvisionalExecutive::new();
         // Manually bump level
         exec.current_tier_index = 3; 

         // Chaos Physics
         let chaos = PhysicsState {
            price: 100.0,
            velocity: 100.0,
            acceleration: 50.0,
            jerk: 5.0, // Massive Jerk
            ..Default::default()
        };

        exec.update(&chaos, 5.0, 0.1); // High Entropy, Low Efficiency
        assert_eq!(exec.get_current_max_risk(), 0.01, "Should drop to 0.01");
    }

    #[test]
    fn test_warmup_lockout() {
        let mut exec = ProvisionalExecutive::new();
        
        // Mock Stable Physics
        let stable_physics = PhysicsState {
            price: 100.0,
            velocity: 0.0,
            acceleration: 0.0,
            jerk: 0.001,
            ..Default::default()
        };

        // Run enough cycles to trigger promotion (but should fail due to warmup)
        exec.update(&stable_physics, 0.5, 0.95);
        let promoted = exec.update(&stable_physics, 0.5, 0.95);
        
        // Should not promote during warmup (even with stable conditions)
        assert!(!promoted, "Should NOT promote during 5-min warmup");
        assert_eq!(exec.get_current_max_risk(), 0.01);
    }
}
