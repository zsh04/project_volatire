use serde::Deserialize;

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
// Reference: {"e": "trade", "E": 123456789, "s": "BTCUSDT", "p": "0.001", "q": "100", "T": 123456785}
#[derive(Debug, Deserialize)]
pub struct BinanceTradeEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    
    #[serde(rename = "E")]
    pub event_time: u64,
    
    #[serde(rename = "s")]
    pub symbol: String,
    
    #[serde(rename = "p")]
    pub price: String, // Prices come as strings to preserve precision
    
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
