use serde::Deserialize;

// ==============================================================================
// Sub-modules
// ==============================================================================
pub mod kraken;

// ==============================================================================
// 1. Internal Generalized Tick
// ==============================================================================
#[derive(Debug, Clone, Copy)]
pub struct Tick {
    pub timestamp: f64, // Unix Timestamp (ms)
    pub price: f64,
    pub quantity: f64,
}

// ==============================================================================
// 2. Binance Incoming Message (JSON)
// ==============================================================================
#[derive(Debug, Deserialize)]
pub struct BinanceTradeEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    
    #[serde(rename = "E")]
    pub event_time: u64,
    
    #[serde(rename = "s")]
    pub symbol: String,
    
    #[serde(rename = "p")]
    pub price: String, 
    
    #[serde(rename = "q")]
    pub quantity: String,
    
    #[serde(rename = "T")]
    pub trade_time: u64,
}

impl BinanceTradeEvent {
    pub fn to_tick(&self) -> Option<Tick> {
        let price = self.price.parse::<f64>().ok()?;
        let quantity = self.quantity.parse::<f64>().ok()?;
        
        Some(Tick {
            timestamp: self.trade_time as f64,
            price,
            quantity,
        })
    }
}

// ==============================================================================
// 3. Market State (Aggregation)
// ==============================================================================
pub struct MarketData {
    pub price: f64,
}

impl MarketData {
    pub fn new() -> Self {
        Self { price: 0.0 }
    }
    
    pub fn update_price(&mut self, price: f64) {
        self.price = price;
    }
}
