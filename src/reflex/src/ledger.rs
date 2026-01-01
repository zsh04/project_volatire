use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub usdt_balance: f64,
    pub btc_position: f64,
    pub locked_balance: f64, // USDT in open orders
    pub start_of_day_balance: f64, // For drawdown calc
}

impl Default for AccountState {
    fn default() -> Self {
        Self {
            usdt_balance: 0.0,
            btc_position: 0.0,
            locked_balance: 0.0,
            start_of_day_balance: 0.0,
        }
    }
}

impl AccountState {
    pub fn new(usdt: f64, btc: f64) -> Self {
        Self {
            usdt_balance: usdt,
            btc_position: btc,
            locked_balance: 0.0,
            start_of_day_balance: usdt, // Assuming starting full in USDT or calculating total equity?
            // For now, let's assume SOD is just the initial USDT for simplicity, 
            // or we need a price to calculate SOD equity. 
            // Let's refine: set SOD to usdt. If holding BTC, we'd need initial price.
        }
    }

    /// Set Start of Day Balance explicitly (e.g. after first sync with price)
    pub fn set_start_of_day(&mut self, total_equity: f64) {
        self.start_of_day_balance = total_equity;
    }

    /// Update local state based on an execution (Fill)
    pub fn update_fill(&mut self, side: &str, price: f64, qty: f64) {
        match side {
            "BUY" => {
                let cost = price * qty;
                self.usdt_balance -= cost;
                self.btc_position += qty;
                // If we had locked funds for this buy, release them?
                // Usually logic is: Place Order -> Lock Funds. Fill -> Unlock Funds & Deduct Balance.
                // For "Atomic Update" requested, let's assume we are just updating balances post-fill 
                // or post-decision. 
                // Implementation Note: If update_local handles just the result:
                // We'll assume locked_balance is managed separately or we decr it here.
                // Let's keep it simple for Directive-09: Direct impact on balances.
            }
            "SELL" => {
                let revenue = price * qty;
                self.usdt_balance += revenue;
                self.btc_position -= qty;
            }
            _ => {}
        }
    }

    /// Sync with Exchange API snapshot
    pub fn sync(&mut self, usdt: f64, btc: f64, locked: f64) {
        self.usdt_balance = usdt;
        self.btc_position = btc;
        self.locked_balance = locked;
    }

    pub fn available_balance(&self) -> f64 {
        self.usdt_balance - self.locked_balance
    }

    pub fn total_equity(&self, current_price: f64) -> f64 {
        self.usdt_balance + (self.btc_position * current_price)
    }

    pub fn current_drawdown_pct(&self, current_price: f64) -> f64 {
        if self.start_of_day_balance <= f64::EPSILON {
            return 0.0;
        }
        let equity = self.total_equity(current_price);
        // Drawdown is how far below SOD we are.
        // If equity > SOD, drawdown is 0 (or negative? Usually 0).
        let diff = self.start_of_day_balance - equity;
        if diff < 0.0 {
            0.0
        } else {
            diff / self.start_of_day_balance
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equity_and_drawdown() {
        let mut account = AccountState::new(1000.0, 0.0);
        account.set_start_of_day(1000.0);

        // Price goes up, no position -> Equity 1000, DD 0
        assert_eq!(account.total_equity(50000.0), 1000.0);
        assert_eq!(account.current_drawdown_pct(50000.0), 0.0);

        // Buy 0.01 BTC @ 50,000 (Cost 500)
        account.update_fill("BUY", 50000.0, 0.01);
        // USDT = 500, BTC = 0.01
        assert_eq!(account.usdt_balance, 500.0);
        assert_eq!(account.btc_position, 0.01);
        
        // Price stays 50k -> Equity = 500 + (0.01 * 50000) = 1000.
        assert_eq!(account.total_equity(50000.0), 1000.0);

        // Price drops to 40k -> Equity = 500 + (0.01 * 40000) = 900.
        // Loss 100. Drawdown = 100 / 1000 = 0.10 (10%)
        assert!((account.current_drawdown_pct(40000.0) - 0.10).abs() < 1e-6);
    }
}
