
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use url::Url;
use tracing::{info, error, warn};
use std::time::Duration;
use crate::market::{Tick, BinanceTradeEvent};
use serde::Deserialize;

pub mod kraken;

// Main dispatcher - selects exchange based on environment
pub async fn connect(symbol: &str, tx: mpsc::Sender<Tick>) {
    let exchange = std::env::var("EXCHANGE").unwrap_or("BINANCE".to_string());
    
    match exchange.to_uppercase().as_str() {
        "KRAKEN" => {
            info!("📡 Dispatching to KRAKEN WebSocket");
            kraken::connect_kraken(symbol, tx).await;
        },
        _ => {
            info!("📡 Dispatching to BINANCE WebSocket (default)");
            connect_binance(symbol, tx).await;
        }
    }
}

// Binance WebSocket client
pub async fn connect_binance(symbol: &str, tx: mpsc::Sender<Tick>) {
    let lower_symbol = symbol.to_lowercase();
    // Use bookTicker for best bid/ask
    let url_str = format!("wss://stream.binance.com:9443/ws/{}@bookTicker", lower_symbol);
    let url = Url::parse(&url_str).expect("Invalid Binance WS URL");

    info!("Ingest: Initializing Binance connection to {}", url);

    loop {
        match connect_loop(&url, &tx).await {
            Ok(_) => {
                warn!("Ingest: Connection closed gracefully. Reconnecting in 5s...");
            }
            Err(e) => {
                error!("Ingest: Connection error: {}. Reconnecting in 5s...", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// Binance BookTicker Event
#[derive(Debug, Deserialize)]
pub struct BinanceBookTickerEvent {
    #[serde(rename = "u")]
    pub update_id: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "b")]
    pub bid_price: String,
    #[serde(rename = "B")]
    pub bid_qty: String,
    #[serde(rename = "a")]
    pub ask_price: String,
    #[serde(rename = "A")]
    pub ask_qty: String,
}

impl BinanceBookTickerEvent {
    pub fn to_tick(&self) -> Option<Tick> {
        let bid = self.bid_price.parse::<f64>().ok()?;
        let ask = self.ask_price.parse::<f64>().ok()?;

        let mid_price = (bid + ask) / 2.0;

        Some(Tick {
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as f64,
            price: mid_price, // Approximate price using Mid-Price
            quantity: 0.0,
            bid: Some(bid),
            ask: Some(ask),
        })
    }
}

async fn connect_loop(url: &Url, tx: &mpsc::Sender<Tick>) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async(url).await?;
    info!("Ingest: Connected to Binance Stream.");
    let (_write, mut read) = ws_stream.split();
    
    // Subscribe (Binance doesn't need explicit subscribe for single stream URL)
    
    while let Some(msg) = read.next().await {
        let msg = msg?;

        match msg {
            Message::Text(text) => {
                // Try to parse as BookTicker
                if let Ok(event) = serde_json::from_str::<BinanceBookTickerEvent>(&text) {
                     if let Some(tick) = event.to_tick() {
                         if let Err(e) = tx.send(tick).await {
                             return Err(format!("Channel closed: {}", e).into());
                         }
                     }
                } else if let Ok(event) = serde_json::from_str::<BinanceTradeEvent>(&text) {
                    // Fallback or if we were using combined stream (though this function is bound to specific URL)
                    if let Some(tick) = event.to_tick() {
                        if let Err(e) = tx.send(tick).await {
                             return Err(format!("Channel closed: {}", e).into());
                        }
                    }
                }
            }
            Message::Ping(_) | Message::Pong(_) => {}
            Message::Close(_) => return Ok(()),
            _ => {}
        }
    }

    Ok(())
}
