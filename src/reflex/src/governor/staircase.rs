use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskTier {
    Q0 = 0, // Tier 1: 0.01 lots (Survival / Floor)
    Q1 = 1, // Tier 2: 0.05 lots
    Q2 = 2, // Tier 3: 0.10 lots
    Q3 = 3, // Tier 4: 0.25 lots
    Q4 = 4, // Tier 5: 0.50 lots
    Max = 5, // Tier 6: 1.00 lots
}

impl RiskTier {
    pub fn position_size(&self) -> f64 {
        match self {
            RiskTier::Q0 => 0.01,
            RiskTier::Q1 => 0.05,
            RiskTier::Q2 => 0.10,
            RiskTier::Q3 => 0.25,
            RiskTier::Q4 => 0.50,
            RiskTier::Max => 1.00,
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            RiskTier::Q0 => Some(RiskTier::Q1),
            RiskTier::Q1 => Some(RiskTier::Q2),
            RiskTier::Q2 => Some(RiskTier::Q3),
            RiskTier::Q3 => Some(RiskTier::Q4),
            RiskTier::Q4 => Some(RiskTier::Max),
            RiskTier::Max => None,
        }
    }

    pub fn prev(&self) -> Option<Self> {
        match self {
            RiskTier::Q0 => None,
            RiskTier::Q1 => Some(RiskTier::Q0),
            RiskTier::Q2 => Some(RiskTier::Q1),
            RiskTier::Q3 => Some(RiskTier::Q2),
            RiskTier::Q4 => Some(RiskTier::Q3),
            RiskTier::Max => Some(RiskTier::Q4),
        }
    }
}

pub struct Staircase {
    pub current_tier: RiskTier,
    consecutive_tight_fills: u32,
    veto_count: u32,
    last_veto_time: Option<Instant>,
    cooldown_until: Option<Instant>,
}

impl Staircase {
    pub fn new() -> Self {
        Self {
            current_tier: RiskTier::Q0,
            consecutive_tight_fills: 0,
            veto_count: 0,
            last_veto_time: None,
            cooldown_until: None,
        }
    }

    /// Primary evaluation loop. 
    /// Should be called before every trade generation.
    pub fn get_position_size(&self) -> f64 {
        if self.is_in_cooldown() {
            return RiskTier::Q0.position_size();
        }
        self.current_tier.position_size()
    }

    pub fn is_in_cooldown(&self) -> bool {
        if let Some(until) = self.cooldown_until {
            Instant::now() < until
        } else {
            false
        }
    }

    // --- Telemetry Helpers ---
    pub fn tier(&self) -> i32 {
        self.current_tier as i32
    }

    pub fn progress(&self) -> f64 {
        if self.is_in_cooldown() {
            return 0.0;
        }
        (self.consecutive_tight_fills as f64 / 50.0).min(1.0)
    }

    /// Called after a trade execution to update fill quality metrics.
    /// slippage_bps: The difference between expected and realized price in basis points.
    pub fn register_fill(&mut self, slippage_bps: f64) {
        if slippage_bps.abs() <= 2.0 {
            self.consecutive_tight_fills += 1;
        } else {
            self.consecutive_tight_fills = 0; // Reset on poor fill
        }
    }

    /// Gates promotion based on specific criteria.
    /// consensus_score: 0.0 to 1.0 representing agreement between alpha models.
    pub fn try_promote(&mut self, consensus_score: f64) -> bool {
        if self.is_in_cooldown() {
            return false;
        }

        // Promotion Gate: 50 tight fills AND High Consensus
        if self.consecutive_tight_fills >= 50 && consensus_score > 0.85 {
            if let Some(next_tier) = self.current_tier.next() {
                self.current_tier = next_tier;
                self.consecutive_tight_fills = 0; // Reset counter after promotion
                return true;
            }
        }
        false
    }

    /// The "Emergency Slide". Checks for critical failure conditions.
    /// alpha_decay: 0.0 to 1.0 (e.g., 0.15 = 15%).
    pub fn check_emergency_slide(&mut self, alpha_decay: f64) -> bool {
        // Trigger 1: Alpha Decay Spike
        if alpha_decay > 0.15 {
            self.demote_to_floor("Alpha Decay Spike > 15%");
            return true;
        }
        false
    }

    /// Called when a Nuclear Veto is issued by the Risk Engine.
    pub fn register_veto(&mut self) {
        let now = Instant::now();
        
        // Check window (60 minutes)
        if let Some(last_time) = self.last_veto_time {
            if now.duration_since(last_time) > Duration::from_secs(3600) {
                self.veto_count = 0; // Reset if window expired
            }
        }

        self.veto_count += 1;
        self.last_veto_time = Some(now);

        // Trigger 2: 3 Vetoes in 60 mins -> Cooldown Lock
        if self.veto_count >= 3 {
            self.cooldown_until = Some(now + Duration::from_secs(4 * 3600)); // 4 hours
            self.demote_to_floor("3 Nuclear Vetoes within 60m");
            self.veto_count = 0; // Reset count after triggering lock
        }
    }

    fn demote_to_floor(&mut self, _reason: &str) {
        self.current_tier = RiskTier::Q0;
        self.consecutive_tight_fills = 0;
        // In a real system, we would log the `_reason` to the Historian here.
        // println!("STAIRCASE DEMOTION: {}", _reason); 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sc = Staircase::new();
        assert_eq!(sc.current_tier, RiskTier::Q0);
        assert_eq!(sc.get_position_size(), 0.01);
    }

    #[test]
    fn test_promotion_mechanics() {
        let mut sc = Staircase::new();

        // Simulate 49 tight fills (not enough)
        for _ in 0..49 {
            sc.register_fill(1.0);
        }
        let promoted = sc.try_promote(0.9);
        assert!(!promoted, "Should not promote at 49 fills");

        // 50th fill
        sc.register_fill(1.0);
        let promoted = sc.try_promote(0.9);
        assert!(promoted, "Should promote at 50 fills + high consensus");
        assert_eq!(sc.current_tier, RiskTier::Q1);
        assert_eq!(sc.consecutive_tight_fills, 0, "Counter should reset");
    }

    #[test]
    fn test_promotion_gates() {
        let mut sc = Staircase::new();

        // 50 fills but low consensus
        for _ in 0..50 {
            sc.register_fill(1.0);
        }
        let promoted = sc.try_promote(0.5); // Low consensus
        assert!(!promoted, "Should gated by consensus");
        assert_eq!(sc.current_tier, RiskTier::Q0);

        // Poor fill resets counter
        sc.register_fill(5.0); // Slippage > 2.0
        assert_eq!(sc.consecutive_tight_fills, 0);
    }

    #[test]
    fn test_emergency_slide() {
        let mut sc = Staircase::new();
        // Force promote to Max
        sc.current_tier = RiskTier::Max;
        
        let triggered = sc.check_emergency_slide(0.20); // 20% Alpha Decay
        assert!(triggered);
        assert_eq!(sc.current_tier, RiskTier::Q0, "Should slide to floor");
    }

    #[test]
    fn test_veto_lockout() {
        let mut sc = Staircase::new();
        
        // Trigger 3 vetoes rapidly
        sc.register_veto();
        sc.register_veto();
        sc.register_veto();

        assert!(sc.is_in_cooldown(), "Should be in cooldown");
        assert_eq!(sc.get_position_size(), 0.01, "Should be forced to min size");

        // Attempt promotion during cooldown
        sc.register_fill(1.0); // fill
        // ... (simulation of 50 fills skipped for brevity, but logic holds)
        // Force conditions for promotion manually for test
        sc.consecutive_tight_fills = 50; 
        
        let promoted = sc.try_promote(0.95);
        assert!(!promoted, "Cannot promote during cooldown");
    }
}
