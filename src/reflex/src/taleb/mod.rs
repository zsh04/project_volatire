pub mod omega;
pub mod sizing;
pub mod shroud; // D-22 Risk Shroud

use crate::feynman::PhysicsState;
use crate::ledger::AccountState;
use tracing::warn;

// Risk Constants
pub const MAX_JERK: f64 = 25.0;
pub const MAX_ENTROPY: f64 = 0.90;
pub const MAX_DRAWDOWN: f64 = 0.02; // 2%
pub const BLACK_SWAN_JERK: f64 = 100.0;
pub const OMEGA_THRESHOLD: f64 = 1.5;

use crate::client::brain::StrategyIntent as BrainIntent;

#[derive(Debug, Clone)]
pub struct TradeProposal {
    pub side: String, // "BUY" or "SELL"
    pub price: f64,
    pub qty: f64,
}

#[derive(Debug, PartialEq)]
pub enum RiskVerdict {
    Allowed,
    Veto(String), // Reason
    Panic,        // Kill Switch
}

pub struct RiskGuardian {
    is_armed: bool,
}

impl Default for RiskGuardian {
    fn default() -> Self {
        Self { is_armed: true }
    }
}

impl RiskGuardian {
    pub fn new() -> Self {
        Self { is_armed: true }
    }

    /// Primary Gatekeeper Function
    pub fn check(
        &self,
        physics: &PhysicsState,
        account: &AccountState,
        intent: &TradeProposal,
        forecast_p10: f64,
        forecast_p50: f64,
        forecast_p90: f64,
        forecast_ts: i64, // Unix Millis
        hurdle_rate: f64, // Annualized Hurdle (e.g. 0.05)
    ) -> RiskVerdict {
        if !self.is_armed {
            return RiskVerdict::Allowed;
        }

        // --- 0. Hardening: Quantile TTL (D-20 Hardening) ---
        // Ensure forecast is fresh (e.g., < 60 seconds old).
        // Note: Using system time. Clock drift > 60s is unlikely but possible on docker.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        
        let age = now - forecast_ts;
        if age > 60_000 { // 60 Seconds TTL
             return RiskVerdict::Veto(format!("Forecast Stale: Age {}ms > 60000ms", age));
        }

        // --- 1. Critical Physics Check (Kill Switch) ---
        if physics.jerk.abs() > BLACK_SWAN_JERK {
            warn!("RISK: CRITICAL JERK DETECTED ({:.2}). TRIGGERING PANIC.", physics.jerk);
            return RiskVerdict::Panic;
        }

        // --- 2. Physics Veto ---
        if physics.jerk.abs() > MAX_JERK {
            return RiskVerdict::Veto(format!("Max Jerk Exceeded: {:.2}", physics.jerk));
        }
        if physics.entropy > MAX_ENTROPY {
            return RiskVerdict::Veto(format!("Max Entropy Exceeded: {:.2}", physics.entropy));
        }

        // --- 3. The Omega Sieve (Taleb Extension) ---
        // Verify that the Probability Distribution justifies the trade.
        // MAR (Min Acceptable Return) = Price * (1 + Daily_Hurdle + Frictions).
        
        // Annual Hurdle -> Daily Hurdle approx
        let daily_hurdle = hurdle_rate / 365.0;
        let friction_buffer = 0.001; // 10 bps buffer for verification
        
        let mar_threshold = intent.price * (1.0 + daily_hurdle + friction_buffer);
        
        let omega = omega::OmegaScorer::calculate(
            forecast_p10, 
            forecast_p50, 
            forecast_p90, 
            mar_threshold
        );

        if omega < OMEGA_THRESHOLD {
            return RiskVerdict::Veto(format!("Omega Fragility Veto: {:.2} < 1.5", omega));
        }

        // --- 4. Capital Veto ---
        // a. Insolvency / Balance check
        if intent.side == "BUY" {
            let cost = intent.price * intent.qty;
            if cost > account.available_balance() {
                return RiskVerdict::Veto(format!(
                    "Insufficient Funds: Cost {:.2} > Available {:.2}",
                    cost,
                    account.available_balance()
                ));
            }
        }
        
        // b. Drawdown check
        let current_dd = account.current_drawdown_pct(physics.price);
        if current_dd > MAX_DRAWDOWN {
            return RiskVerdict::Veto(format!(
                "Max Drawdown Exceeded: {:.2}% > {:.2}%",
                current_dd * 100.0,
                MAX_DRAWDOWN * 100.0
            ));
        }

        RiskVerdict::Allowed
    }

    /// Secondary Gatekeeper: The Risk Shroud (Exit Logic)
    pub fn check_shroud(
        &self,
        current_price: f64,
        intent: &BrainIntent,
        entropy: f64,
    ) -> shroud::ShroudVerdict {
        if !self.is_armed {
            return shroud::ShroudVerdict::Safe;
        }
        shroud::RiskShroud::new().check_shroud(current_price, intent, entropy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jerk_veto() {
        let guardian = RiskGuardian::new();
        let mut physics = PhysicsState::default();
        let account = AccountState::default();
        let intent = TradeProposal { side: "BUY".to_string(), price: 100.0, qty: 1.0 };

        physics.jerk = 50.0; // > 25.0

        // Forecast irrelevant for Jerk Veto, but needed for signature
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
        let verdict = guardian.check(&physics, &account, &intent, 90.0, 100.0, 110.0, now, 0.05);
        assert!(matches!(verdict, RiskVerdict::Veto(ref r) if r.contains("Max Jerk")));
    }

    #[test]
    fn test_black_swan_panic() {
        let guardian = RiskGuardian::new();
        let mut physics = PhysicsState::default();
        let account = AccountState::default();
        let intent = TradeProposal { side: "BUY".to_string(), price: 100.0, qty: 1.0 };

        physics.jerk = 150.0; // > 100.0

        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
        let verdict = guardian.check(&physics, &account, &intent, 90.0, 100.0, 110.0, now, 0.05);
        assert_eq!(verdict, RiskVerdict::Panic);
    }

    #[test]
    fn test_omega_veto_integration() {
        let guardian = RiskGuardian::new();
        let physics = PhysicsState::default();
        let account = AccountState::new(1000.0, 0.0);
        let intent = TradeProposal { side: "BUY".to_string(), price: 100.0, qty: 1.0 };

        // Bearish Forecast: P90 is close to P50, but P10 is far below.
        // P50=100. P90=102. P10=80.
        // Downside=20. Upside=2. Omega << 1.
        // Timestamp = 0 (Epoch) -> Should FAIL TTL check if armed.
        
        // I need to use a current timestamp for this test to reach Omega check.
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
        
        let verdict = guardian.check(&physics, &account, &intent, 80.0, 100.0, 102.0, now, 0.05);
        
        assert!(matches!(verdict, RiskVerdict::Veto(ref r) if r.contains("Omega Fragility")));
    }
    
    #[test]
    fn test_ttl_veto() {
        let guardian = RiskGuardian::new();
        let physics = PhysicsState::default();
        let account = AccountState::default();
        let intent = TradeProposal { side: "BUY".to_string(), price: 100.0, qty: 1.0 };
        
        // Stale Timestamp (Now - 70s)
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
        let past = now - 70_000;

        let verdict = guardian.check(&physics, &account, &intent, 90.0, 100.0, 110.0, past, 0.05);
        assert!(matches!(verdict, RiskVerdict::Veto(ref r) if r.contains("Forecast Stale")));
    }

    #[test]
    fn test_insolvency_check() {
        let guardian = RiskGuardian::new();
        let physics = PhysicsState::default();
        let account = AccountState::new(50.0, 0.0); // Only 50 USD
        let intent = TradeProposal { side: "BUY".to_string(), price: 100.0, qty: 1.0 }; // Cost 100

        // High Omega Forecast to bypass first gate
        // P50 > Threshold. Upside heavily favored.
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
        let verdict = guardian.check(&physics, &account, &intent, 99.0, 105.0, 110.0, now, 0.05);
        assert!(matches!(verdict, RiskVerdict::Veto(ref r) if r.contains("Insufficient Funds")));
    }
}
