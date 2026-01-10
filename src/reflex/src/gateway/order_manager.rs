 // Or Sha512 depending on exchange
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::governor::wave_legislator::WaveVerdict;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)] // Not sending this directly, so minimal derives
pub struct PrimedOrder {
    pub symbol: String,
    pub side: Side,
    pub qty: f64,
    pub price: Option<f64>, // Null for market orders
    
    // The raw payload bytes ready for the socket
    // We pre-serialize and pre-sign everything here
    pub payload: Vec<u8>, 
    
    // Timestamps for audit (nanoseconds)
    pub t_decision: u128, // When logic said "Maybe"
    pub t_primed: u128,   // When we finished signing
}

pub struct OrderGateway {
    // Configuration
    api_key: String,
    api_secret: String,
    
    // The "Hot Buffer" (Pre-allocated memory)
    // Holds the fully constructed, signed packet ready to send
    hot_buffer: Option<PrimedOrder>,
}

impl OrderGateway {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
            hot_buffer: None,
        }
    }

    /// Primary Logic: converts a "Tunneling" verdict into a "Primed Order".
    /// This is the "Pre-Ignition" phase.
    pub fn prime_order(&mut self, verdict: &WaveVerdict, symbol: &str) {
        match verdict {
            WaveVerdict::Tunneling { probability: _, target_price } => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
                
                // 1. Construct the payload (Simulation for now)
                // In real impl, this would be JSON or FIX bytes
                let payload_str = format!(
                    r#"{{"event":"addOrder","pair":"{}","type":"limit","price":{},"ordertype":"limit"}}"#, 
                    symbol, target_price
                );
                
                // 2. Sign (Simulated HMAC)
                // Real signing is expensive, so we do it HERE, not at trigger time
                // let mut mac = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes()).unwrap();
                // mac.update(payload_str.as_bytes());
                // let signature = mac.finalize().into_bytes();

                let final_payload = payload_str.into_bytes();
                // final_payload.extend_from_slice(&signature);

                // 3. Buffer it
                self.hot_buffer = Some(PrimedOrder {
                    symbol: symbol.to_string(),
                    side: Side::Buy, // Simplified for this context
                    qty: 1.0,        // Default unit
                    price: Some(*target_price),
                    payload: final_payload,
                    t_decision: now,
                    t_primed: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
                });
                
                // println!("Gateway: Order PRIMED for Tunneling Event (Prob: {:.2})", probability);
            },
            _ => {
                // If verdict is Breakout, we might fire immediately, but for priming we ignore
            }
        }
    }

    /// Phase B: The Trigger
    /// D-61 confirms the move. We send bytes immediately.
    /// Returns the timestamp of "Wire Send"
    pub fn fire_instant(&mut self, jerk: f64) -> Option<u128> {
        // 1. Micro-Veto (Last Look)
        // If jerk is massive and negative (sudden crash acceleration), we ABORT
        if jerk < -10.0 {
            // println!("Gateway: MICRO-VETO triggered by Jerk ({:.2}). Order Aborted.", jerk);
            self.hot_buffer = None;
            return None;
        }

        if let Some(_order) = &self.hot_buffer {
            // 2. "Send" (access network socket here)
            // ws_stream.send(order.payload).await...
            
            // println!("Gateway: 🔥 FIRED! Payload: {:?}", String::from_utf8_lossy(&order.payload));
            
            let t_fire = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            
            // 3. Clear Buffer
            self.hot_buffer = None;
            
            return Some(t_fire);
        }

        None
    }

    /// Directive-68: Emergency Liquidation (Kill Switch)
    /// Clears any buffer and returns a simulated count of positions closed.
    pub fn emergency_liquidate(&mut self) -> usize {
        // 1. Drop the hot buffer (Cancel pending)
        self.hot_buffer = None;
        
        // 2. In a real system, we would iterate known positions and fire Market Sells.
        // Here we simulate it.
        // println!("Gateway: ☢️ EMERGENCY LIQUIDATION TRIGGERED ☢️");
        
        // Return simulated number of closed positions
        1 // Assume 1 position for simulation
    }
    /// Directive-86: Sovereign Close All
    /// Calls emergency liquidation to flatten all positions
    pub async fn close_all_positions(&mut self) -> Result<usize, String> {
        // In a real implementation, this would iterate active orders and send CancelAll
        // followed by Market Close orders for all positions.
        let count = self.emergency_liquidate();
        tracing::warn!("📛 GATEWAY: Sovereign CloseAll executed. Liquidated {} positions/orders.", count);
        Ok(count)
    }

    /// D-106: Flatten Single Position
    pub async fn close_position(&mut self, symbol: &str) -> Result<bool, String> {
        // In real implementation:
        // 1. Get position size from AccountSnapshot
        // 2. Send Market Order (Sell if Long, Buy if Short)
        // 3. Wait for fill
        tracing::warn!("📛 GATEWAY: Flattening position for {}", symbol);

        // Simulating success
        Ok(true)
    }
}
