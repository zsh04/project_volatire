pub struct BESKelly;

impl BESKelly {
    /// Calculates the friction-adjusted optimal fraction.
    ///
    /// - capital: Total available equity.
    /// - price: Current asset price.
    /// - target_price: The specialized 'Target' (e.g., P90 for Long).
    /// - stop_price: The specialized 'Stop' (e.g., P10 for Long).
    /// - confidence: Signal strength (0.0 to 1.0) -> interprets as probability of "Win".
    ///
    /// Constants:
    /// - FEE: 0.005 (0.5% Taker assumed for safety)
    /// - SLIPPAGE: 0.001 (0.1%)
    ///
    /// Returns:
    /// - amount: The USD amount to deploy.
    pub fn allocate(
        capital: f64,
        price: f64,
        target_price: f64,
        stop_price: f64,
        confidence: f64,
    ) -> f64 {
        let base_fee = 0.005; // 0.5% exchange fee
        let slippage = 0.001; // 0.1% expected drift
        let _tax_buffer = 0.0; // Ignored for Phase 1 execution sizing, handled in accountant?

        let frictional_cost_pct = base_fee + slippage;
        let friction_amt = price * frictional_cost_pct;

        // 1. Calculate Net Payoffs
        // "Win" means hitting Target.
        // Net Win = (Target - Entry) - Friction
        let gross_win = target_price - price;
        let net_win = gross_win - friction_amt;

        // "Loss" means hitting Stop.
        // Net Loss = (Entry - Stop) + Friction (Loss is magnitude)
        let gross_loss = price - stop_price;
        let net_loss = gross_loss + friction_amt;

        // 2. Frictions Veto
        // If the expected 'Win' is consumed by friction, don't trade.
        if net_win <= 0.0 {
            return 0.0;
        }
        if net_loss <= 0.0 {
            // Should be impossible if Stop < Price, but safety check.
            // If Stop > Price (Short?), logic differs. assuming Long for now.
            return 0.0; 
        }

        // 3. Kelly Criterion
        // f = p/a - q/b
        // Where:
        // p = probability of win (confidence)
        // q = 1 - p
        // b = odds = NetGain / NetLoss
        // f = p - q/b (Simpler form is often f = p - q/b where b is payoff ratio)
        // Let's use standard: f = (p(b+1) - 1) / b
        // Here b = net_win / net_loss
        
        let b = net_win / net_loss;
        let p = confidence;
        let q = 1.0 - p;

        let f_star = (p * b - q) / b;

        // 4. Safety Constraints
        // a. No negatives
        if f_star <= 0.0 {
            return 0.0;
        }
        
        // b. Half-Kelly (Safety)
        let f_safe = f_star * 0.5;

        // c. Cap (Never risk more than 20% of equity on one trade in Phase 2)
        let f_capped = f_safe.min(0.20);

        capital * f_capped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kelly_positive() {
        // High confidence, good RR.
        // Price 100, Target 110 (Win 10), Stop 95 (Loss 5). Odds = 2:1.
        // Conf 0.6.
        // b = 2.
        // f = (0.6 * 2 - 0.4) / 2 = 0.8 / 2 = 0.4.
        // Half Kelly = 0.2.
        // Friction reduces this slightly.
        let alloc = BESKelly::allocate(1000.0, 100.0, 110.0, 95.0, 0.6);
        println!("Alloc: {}", alloc);
        assert!(alloc > 0.0);
        assert!(alloc < 250.0); // Should be around 20% max cap.
    }

    #[test]
    fn test_friction_kill() {
        // Tiny win, huge friction.
        // Price 100, Target 100.1, Stop 99.
        // Friction ~0.6% = 0.60.
        // Gross Win 0.1. Net Win = -0.5.
        // Should output 0.
        let alloc = BESKelly::allocate(1000.0, 100.0, 100.1, 99.0, 0.6);
        assert_eq!(alloc, 0.0);
    }
}
