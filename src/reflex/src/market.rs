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
    // D-110: Perception
    pub bid: Option<f64>,
    pub ask: Option<f64>,
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
            bid: None,
            ask: None,
        })
    }
}

// ==============================================================================
// 3. Market State (Aggregation)
// ==============================================================================
pub struct MarketData {
    pub price: f64,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
}

impl MarketData {
    pub fn new() -> Self {
        Self {
            price: 0.0,
            best_bid: None,
            best_ask: None,
        }
    }
    
    pub fn update_price(&mut self, price: f64) {
        self.price = price;
    }

    pub fn update_book(&mut self, bid: Option<f64>, ask: Option<f64>) {
        if let Some(b) = bid { self.best_bid = Some(b); }
        if let Some(a) = ask { self.best_ask = Some(a); }
    }

    pub fn get_spread(&self) -> f64 {
        match (self.best_bid, self.best_ask) {
            (Some(b), Some(a)) => a - b,
            _ => 0.0,
        }
    }
}
