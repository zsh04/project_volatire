use crate::feynman::PhysicsState;
use crate::ledger::AccountState;
use tracing::warn;

// Risk Constants
pub const MAX_JERK: f64 = 25.0;
pub const MAX_ENTROPY: f64 = 0.90;
pub const MAX_DRAWDOWN: f64 = 0.02; // 2%
pub const BLACK_SWAN_JERK: f64 = 100.0;

#[derive(Debug, Clone)]
pub struct StrategyIntent {
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

    pub fn check(
        &self,
        physics: &PhysicsState,
        account: &AccountState,
        intent: &StrategyIntent,
    ) -> RiskVerdict {
        if !self.is_armed {
            return RiskVerdict::Allowed;
        }

        // 1. Critical Physics Check (Kill Switch)
        if physics.jerk.abs() > BLACK_SWAN_JERK {
            warn!("RISK: CRITICAL JERK DETECTED ({:.2}). TRIGGERING PANIC.", physics.jerk);
            return RiskVerdict::Panic;
        }

        // 2. Physics Veto
        if physics.jerk.abs() > MAX_JERK {
            return RiskVerdict::Veto(format!("Max Jerk Exceeded: {:.2}", physics.jerk));
        }
        if physics.entropy > MAX_ENTROPY {
            return RiskVerdict::Veto(format!("Max Entropy Exceeded: {:.2}", physics.entropy));
        }

        // 3. Capital Veto
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
        // For SELL, check BTC balance? (Optional but good)
        if intent.side == "SELL" {
            if intent.qty > account.btc_position {
                 return RiskVerdict::Veto(format!(
                    "Insufficient BTC: Need {:.4} > Have {:.4}",
                    intent.qty,
                    account.btc_position
                ));
            }
        }

        // b. Drawdown check
        // We calculate what the drawdown IS currently. If we are already below limit, stop trading?
        // Or if this trade MAKES us blow limit?
        // Usually: "If DailyLoss > MaxDrawdown -> REJECT NEW TRADES"
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jerk_veto() {
        let guardian = RiskGuardian::new();
        let mut physics = PhysicsState::default();
        let account = AccountState::default();
        let intent = StrategyIntent { side: "BUY".to_string(), price: 100.0, qty: 1.0 };

        physics.jerk = 50.0; // > 25.0

        let verdict = guardian.check(&physics, &account, &intent);
        assert!(matches!(verdict, RiskVerdict::Veto(ref r) if r.contains("Max Jerk")));
    }

    #[test]
    fn test_black_swan_panic() {
        let guardian = RiskGuardian::new();
        let mut physics = PhysicsState::default();
        let account = AccountState::default();
        let intent = StrategyIntent { side: "BUY".to_string(), price: 100.0, qty: 1.0 };

        physics.jerk = 150.0; // > 100.0

        let verdict = guardian.check(&physics, &account, &intent);
        assert_eq!(verdict, RiskVerdict::Panic);
    }

    #[test]
    fn test_insolvency() {
        let guardian = RiskGuardian::new();
        let physics = PhysicsState::default();
        let account = AccountState::new(500.0, 0.0); // 500 USDT
        
        // Buy 600 USDT worth
        let intent = StrategyIntent { side: "BUY".to_string(), price: 100.0, qty: 6.0 };

        let verdict = guardian.check(&physics, &account, &intent);
        assert!(matches!(verdict, RiskVerdict::Veto(ref r) if r.contains("Insufficient Funds")));
    }

    #[test]
    fn test_drawdown_stop() {
        let guardian = RiskGuardian::new();
        let mut physics = PhysicsState::default();
        physics.price = 40000.0;

        let mut account = AccountState::new(1000.0, 0.01); // 1000 USDT + 0.01 BTC
        // Set SOD. Let's say we started with Equiy 1600 (1000 + 0.01*60k)
        // Now price is 40k. Equity = 1000 + 400 = 1400.
        // Loss 200. Drawdown 200/1600 = 12.5%.
        account.set_start_of_day(1600.0);

        // Try to buy
        let intent = StrategyIntent { side: "BUY".to_string(), price: 40000.0, qty: 0.001 };

        let verdict = guardian.check(&physics, &account, &intent);
        assert!(matches!(verdict, RiskVerdict::Veto(ref r) if r.contains("Max Drawdown")));
    }
}
