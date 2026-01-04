use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt}; // Added SinkExt for .send()
use tokio::sync::mpsc;
use url::Url;
use tracing::{info, error, warn};
use std::time::Duration;
use crate::market::{Tick, kraken};
// --- Account Sync Logic (Directive-72) ---
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest, Sha512};
use base64::{Engine as _, engine::general_purpose};
use std::collections::HashMap;

pub enum Exchange {
    Binance,
    Kraken,
}

pub async fn connect_kraken(pair: &str, tx: mpsc::Sender<Tick>) {
    // Kraken WebSocket URL for public spot feeds
    let url_str = "wss://ws.kraken.com";
    let url = Url::parse(url_str).expect("Invalid Kraken WS URL");

    info!("Kraken Ingest: Initializing connection to {}", url);

    loop {
        match connect_kraken_loop(&url, pair, &tx).await {
            Ok(_) => {
                warn!("Kraken Ingest: Connection closed gracefully. Reconnecting in 5s...");
            }
            Err(e) => {
                error!("Kraken Ingest: Connection error: {}. Reconnecting in 5s...", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn connect_kraken_loop(url: &Url, pair: &str, tx: &mpsc::Sender<Tick>) 
    -> Result<(), Box<dyn std::error::Error>> 
{
    let (ws_stream, _) = connect_async(url).await?;
    info!("Kraken Ingest: Connected to WebSocket");
    
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to trade feed
    // Kraken format: {"event":"subscribe","pair":["XBT/USD"],"subscription":{"name":"trade"}}
    let subscribe_msg = serde_json::json!({
        "event": "subscribe",
        "pair": [pair],
        "subscription": {
            "name": "trade"
        }
    });
    
    let sub_text = serde_json::to_string(&subscribe_msg)?;
    write.send(Message::Text(sub_text)).await?;
    info!("Kraken Ingest: Subscribed to {} trades", pair);
    
    while let Some(msg) = read.next().await {
        let msg = msg?;

        match msg {
            Message::Text(text) => {
                // Try to parse as trade message
                if let Some(ticks) = kraken::parse_kraken_trade(&text) {
                    for tick in ticks {
                        if let Err(e) = tx.send(tick).await {
                            return Err(format!("Channel closed: {}", e).into());
                        }
                    }
                }
                // Ignore subscription confirmations and heartbeats
            }
            Message::Ping(_) | Message::Pong(_) => {}
            Message::Close(_) => return Ok(()),
            _ => {}
        }
    }

    Ok(())
}

// Original Binance connection (unchanged)
pub async fn connect_binance(symbol: &str, tx: mpsc::Sender<Tick>) {
    let lower_symbol = symbol.to_lowercase();
    let url_str = format!("wss://stream.binance.com:9443/ws/{}@trade", lower_symbol);
    let url = Url::parse(&url_str).expect("Invalid Binance WS URL");

    info!("Binance Ingest: Initializing connection to {}", url);

    loop {
        match connect_binance_loop(&url, &tx).await {
            Ok(_) => {
                warn!("Binance Ingest: Connection closed gracefully. Reconnecting in 5s...");
            }
            Err(e) => {
                error!("Binance Ingest: Connection error: {}. Reconnecting in 5s...", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn connect_binance_loop(url: &Url, tx: &mpsc::Sender<Tick>) 
    -> Result<(), Box<dyn std::error::Error>> 
{
    let (ws_stream, _) = connect_async(url).await?;
    info!("Binance Ingest: Connected to WebSocket");
    let (_write, mut read) = ws_stream.split();
    
    while let Some(msg) = read.next().await {
        let msg = msg?;

        match msg {
            Message::Text(text) => {
                if let Ok(event) = serde_json::from_str::<crate::market::BinanceTradeEvent>(&text) {
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

pub async fn fetch_account_balance(api_key: &str, api_secret: &str) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let uri_path = "/0/private/Balance";
    let url = format!("https://api.kraken.com{}", uri_path);
    
    // Nonce
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis().to_string();
    let payload = format!("nonce={}", nonce);

    // Sign
    let mut sha256 = Sha256::new();
    sha256.update(nonce.as_bytes());
    sha256.update(payload.as_bytes()); // nonce=... is the POST data
    let sha256_hash = sha256.finalize();

    let mut mac = Hmac::<Sha512>::new_from_slice(&general_purpose::STANDARD.decode(api_secret)?)?;
    mac.update(uri_path.as_bytes());
    mac.update(&sha256_hash);
    let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());

    let resp = client.post(&url)
        .header("API-Key", api_key)
        .header("API-Sign", signature)
        .body(payload)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    // Parse Response (e.g. {"error":[],"result":{"ZUSD":"1000.0","XXBT":"0.5"}})
    if let Some(err) = resp.get("error").and_then(|e| e.as_array()) {
        if !err.is_empty() {
             return Err(format!("Kraken API Error: {:?}", err).into());
        }
    }

    let result = resp.get("result").ok_or("No result field")?;
    
    // Kraken uses ZUSD for USD and XXBT for Bitcoin
    let usd = result.get("ZUSD").or_else(|| result.get("USD"))
              .and_then(|v| v.as_str())
              .and_then(|v| v.parse::<f64>().ok())
              .unwrap_or(0.0);
              
    let btc = result.get("XXBT").or_else(|| result.get("XBT"))
              .and_then(|v| v.as_str())
              .and_then(|v| v.parse::<f64>().ok())
              .unwrap_or(0.0);

    info!("💰 Account Sync: USD=${:.2}, BTC={:.8}", usd, btc);
    Ok((usd, btc))
}
