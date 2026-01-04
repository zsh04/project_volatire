
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use url::Url;
use tracing::{info, error, warn};
use std::time::Duration;
use crate::market::{Tick, BinanceTradeEvent};

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
    let url_str = format!("wss://stream.binance.com:9443/ws/{}@trade", lower_symbol);
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

async fn connect_loop(url: &Url, tx: &mpsc::Sender<Tick>) -> Result<(), Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async(url).await?;
    info!("Ingest: Connected to Binance Stream.");
    let (_write, mut read) = ws_stream.split();
    
    // Subscribe (Binance doesn't need explicit subscribe for single stream URL)
    
    while let Some(msg) = read.next().await {
        let msg = msg?;

        match msg {
            Message::Text(text) => {
                if let Ok(event) = serde_json::from_str::<BinanceTradeEvent>(&text) {
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
