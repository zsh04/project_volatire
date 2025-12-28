
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use url::Url;
use tracing::{info, error, warn};
use std::time::Duration;
use crate::market::{Tick, BinanceTradeEvent};

pub async fn connect(symbol: &str, tx: mpsc::Sender<Tick>) {
    let lower_symbol = symbol.to_lowercase();
    let url_str = format!("wss://stream.binance.com:9443/ws/{}@trade", lower_symbol);
    let url = Url::parse(&url_str).expect("Invalid Binance WS URL");

    info!("Ingest: Initializing connection to {}", url);

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

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = msg?;
        
        match msg {
            Message::Text(text) => {
                // PERFORMANCE: In Phase 2, avoid String allocation here. Use zero-copy parsing (e.g., simd-json)
                // or parse directly from the bytes. For now (Phase 1), serde_json::from_str is acceptable.
                if let Ok(event) = serde_json::from_str::<BinanceTradeEvent>(&text) {
                    if let Some(tick) = event.to_tick() {
                        if let Err(e) = tx.send(tick).await {
                             // If channel is closed, main loop is dead. Exit.
                             return Err(format!("Channel closed: {}", e).into());
                        }
                    }
                } else {
                    warn!("Ingest: Failed to parse message: {}", text);
                }
            }
            Message::Ping(_) | Message::Pong(_) => {}
            Message::Close(_) => return Ok(()),
            _ => {}
        }
    }

    Ok(())
}
