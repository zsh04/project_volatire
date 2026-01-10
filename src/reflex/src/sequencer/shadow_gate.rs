use std::collections::HashMap;
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use tracing::{info, warn};
use crate::governor::ooda_loop::{Decision, Action};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowOrder {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub qty: f64,
    pub limit_price: f64,
    pub created_at: u128, // Nanos since EPOCH (Instant is not Serializable)
    pub status: ShadowStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShadowStatus {
    Pending,
    Filled(f64, u128), // Fill Price, Timestamp
    Cancelled,
}

pub struct ShadowGate {
    pub symbol: String,
    pub virtual_book: HashMap<String, ShadowOrder>,
    pub latency_simulation_ms: u64,
    pub symbol: String, // D-110: Parameterized Symbol
}

impl ShadowGate {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            virtual_book: HashMap::new(),
            latency_simulation_ms: 500, // D-54: Exchange Latency Sim
            symbol,
        }
    }

    /// Submits a virtual order to the shadow book
    pub fn submit_order(&mut self, decision: &Decision, price: f64) {
        let (side, qty, limit_price) = match decision.action {
            Action::Buy(q) => ("BUY", q, price), // Market/Limit at current price
            Action::Sell(q) => ("SELL", q, price),
            _ => return, // Hold/Halt -> No Order
        };

        if qty <= 0.0 { return; }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let id = format!("{}-{}", side, now);
        
        // For simplicity in Phase 7, we treat these as "Limit Orders at Signal Price"
        // In reality, they might be Market orders, but we track slippage against this price.
        let order = ShadowOrder {
            id: id.clone(),
<<<<<<< HEAD
            symbol: self.symbol.clone(),
=======
            symbol: self.symbol.clone(), // D-110: Parameterized
>>>>>>> feb49d06 (pushing local changes.)
            side: side.to_string(),
            qty,
            limit_price,
            created_at: now, // Anchor for latency check
            status: ShadowStatus::Pending,
        };

        tracing::info!("👻 SHADOW ORDER SUBMITTED: {} {} @ {:.2}", side, qty, limit_price);
        self.virtual_book.insert(id, order);
    }

    /// Checks for fills based on current market price and simulated latency
    pub fn check_fills(&mut self, current_price: f64) {
        let mut filled_ids = Vec::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let latency_ns = (self.latency_simulation_ms as u128) * 1_000_000;

        for (id, order) in self.virtual_book.iter_mut() {
            if order.status != ShadowStatus::Pending { continue; }

            // 1. Latency Check (The "Travel Time")
            if now < order.created_at + latency_ns {
                // Too soon, packet is "in flight"
                continue;
            }

            // 2. Price Check (The "Matching Engine")
            // BUY: If Current Price <= Limit Price (we wanted to buy at X, price is now X or lower)
            // SELL: If Current Price >= Limit Price (we wanted to sell at X, price is now X or higher)
            // Note: This is simplified. Real matching requires depth.
            // For "Shadow Mode", we assume instant liquidity at BBO if price crosses.
            
            let is_fill = match order.side.as_str() {
                "BUY" => current_price <= order.limit_price,
                "SELL" => current_price >= order.limit_price,
                _ => false,
            };

            if is_fill {
                let fill_ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
                
                // Calculate Slippage (Alpha Decay)
                // Slippage = |Fill Price - Intended Price|
                let slippage = (current_price - order.limit_price).abs();
                tracing::info!("👻 SHADOW FILL: {} Filled @ {:.2} (Slippage: {:.2})", id, current_price, slippage);
                
                order.status = ShadowStatus::Filled(current_price, fill_ts);
                filled_ids.push(id.clone());
            }
        }
        
        // Cleanup or Archive? For now we keep them to avoid reprocessing, 
        // but in prod we'd move them to a 'filled_log'.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_shadow_fill_mechanics() {
        let mut gate = ShadowGate::new("BTC-USDT".to_string());
        // Lower latency for test speed
        gate.latency_simulation_ms = 10;
        
        let decision = Decision {
            action: Action::Buy(0.5),
            reason: "Test Buy".to_string(),
            confidence: 1.0,
        };
        
        // 1. Submit Order at 50000.0
        gate.submit_order(&decision, 50000.0);
        assert_eq!(gate.virtual_book.len(), 1);
        
        // 2. Check Fills immediately - Should be rejected by latency
        gate.check_fills(49990.0); // Price dipped, should fill if instant
        // Order pending?
        let order = gate.virtual_book.values().next().unwrap();
        assert_eq!(order.status, ShadowStatus::Pending);
        
        // 3. Wait > latency
        thread::sleep(Duration::from_millis(15));
        
        // 4. Check Fills - Price still favorable (49990.0 < 50000.0)
        gate.check_fills(49990.0);
        
        let order = gate.virtual_book.values().next().unwrap();
        if let ShadowStatus::Filled(price, _) = order.status {
            assert_eq!(price, 49990.0);
        } else {
            panic!("Order should be filled! Status: {:?}", order.status);
        }
    }
}
