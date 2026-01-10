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
    
    // Subscribe to ticker feed (provides spread)
    // Kraken format: {"event":"subscribe","pair":["XBT/USD"],"subscription":{"name":"ticker"}}
    let subscribe_msg = serde_json::json!({
        "event": "subscribe",
        "pair": [pair],
        "subscription": {
            "name": "ticker"
        }
    });

    // D-110: Also subscribe to spread for bid/ask perception
    let subscribe_spread = serde_json::json!({
        "event": "subscribe",
        "pair": [pair],
        "subscription": {
            "name": "spread"
        }
    });
    
    let sub_text = serde_json::to_string(&subscribe_msg)?;
    write.send(Message::Text(sub_text)).await?;

    let spread_text = serde_json::to_string(&subscribe_spread)?;
    write.send(Message::Text(spread_text)).await?;

    info!("Kraken Ingest: Subscribed to {} ticker & spread", pair);
    
    while let Some(msg) = read.next().await {
        let msg = msg?;

        match msg {
            Message::Text(text) => {
                // Try to parse as ticker message
                if let Some(tick) = kraken::parse_kraken_ticker(&text) {
                    if let Err(e) = tx.send(tick).await {
                        return Err(format!("Channel closed: {}", e).into());
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

use crate::reflex_proto::{PositionState, OrderState};

async fn kraken_request(api_key: &str, api_secret: &str, path: &str, payload: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.kraken.com{}", path);
    
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis().to_string();
    let full_payload = format!("nonce={}&{}", nonce, payload);

    let mut sha256 = Sha256::new();
    sha256.update(nonce.as_bytes());
    sha256.update(full_payload.as_bytes());
    let sha256_hash = sha256.finalize();

    let mut mac = Hmac::<Sha512>::new_from_slice(&general_purpose::STANDARD.decode(api_secret)?)?;
    mac.update(path.as_bytes());
    mac.update(&sha256_hash);
    let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());

    let resp = client.post(&url)
        .header("API-Key", api_key)
        .header("API-Sign", signature)
        .body(full_payload)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(err) = resp.get("error").and_then(|e| e.as_array()) {
        if !err.is_empty() {
             return Err(format!("Kraken API Error: {:?}", err).into());
        }
    }

    resp.get("result").cloned().ok_or("No result field".into())
}

pub async fn fetch_account_data(api_key: &str, api_secret: &str) -> Result<(f64, f64, f64, f64, Vec<PositionState>, Vec<OrderState>), Box<dyn std::error::Error>> {
    // 1. TradeBalance (Equity)
    let tb_res = kraken_request(api_key, api_secret, "/0/private/TradeBalance", "asset=ZUSD").await?;
    let equity = tb_res.get("eb").and_then(|v| v.as_str()).unwrap_or("0").parse::<f64>().unwrap_or(0.0);

    // 2. Balance (Assets, Cash)
    let b_res = kraken_request(api_key, api_secret, "/0/private/Balance", "").await?;
    let zusd = b_res.get("ZUSD").or_else(|| b_res.get("USDT")).and_then(|v| v.as_str()).unwrap_or("0").parse::<f64>().unwrap_or(0.0);
    let xxbt = b_res.get("XXBT").or_else(|| b_res.get("XBT")).and_then(|v| v.as_str()).unwrap_or("0").parse::<f64>().unwrap_or(0.0);

    // 3. OpenPositions
    // Returns dict of txid -> { pair, time, type, cost, vol, net... }
    let mut positions = Vec::new();
    if let Ok(op_res) = kraken_request(api_key, api_secret, "/0/private/OpenPositions", "docalcs=true").await {
        if let Some(obj) = op_res.as_object() {
            for (_, val) in obj {
                let symbol = val.get("pair").and_then(|v| v.as_str()).unwrap_or("UNKNOWN").to_string();
                let vol = val.get("vol").and_then(|v| v.as_str()).unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let cost = val.get("cost").and_then(|v| v.as_str()).unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let net = val.get("net").and_then(|v| v.as_str()).unwrap_or("0").parse::<f64>().unwrap_or(0.0); // Unrealized PnL? Check API. 'net' is usually PnL in docalcs mode.
                // Actually 'net' might be something else. 'value' - 'cost' = pnl.
                let value = val.get("value").and_then(|v| v.as_str()).unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let pnl = if value != 0.0 { value - cost } else { net }; // fallback scheme
                
                let time = val.get("time").and_then(|v| v.as_f64()).unwrap_or(0.0) as i64; // Unix float
                
                // Entry price = cost / vol
                let entry = if vol != 0.0 { cost / vol } else { 0.0 };

                positions.push(PositionState {
                    symbol,
                    net_size: vol,
                    avg_entry_price: entry,
                    unrealized_pnl: pnl,
                    entry_timestamp: time * 1000, // s to ms
                    current_price: 0.0, // Filled by ticker in main?
                });
            }
        }
    }

    // 4. OpenOrders
    let mut orders = Vec::new();
    if let Ok(oo_res) = kraken_request(api_key, api_secret, "/0/private/OpenOrders", "").await {
        if let Some(open) = oo_res.get("open").and_then(|v| v.as_object()) {
            for (id, val) in open {
                let desc = val.get("descr");
                let pair = desc.and_then(|d| d.get("pair")).and_then(|s| s.as_str()).unwrap_or("?").to_string();
                let side = desc.and_then(|d| d.get("type")).and_then(|s| s.as_str()).unwrap_or("?").to_string();
                let price = desc.and_then(|d| d.get("price")).and_then(|s| s.as_str()).unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let vol = val.get("vol").and_then(|v| v.as_str()).unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let opentm = val.get("opentm").and_then(|v| v.as_f64()).unwrap_or(0.0) as i64;

                orders.push(OrderState {
                    order_id: id.clone(),
                    symbol: pair,
                    side: side.to_uppercase(),
                    quantity: vol,
                    limit_price: price,
                    status: "OPEN".to_string(),
                    timestamp: opentm * 1000,
                });
            }
        }
    }
    
    let total_unrealized: f64 = positions.iter().map(|p| p.unrealized_pnl).sum();
    // Kraken Equity 'eb' usually includes unrealized pnl.
    
    info!("💰 Account Sync: Equity=${:.2} | PnL=${:.2}", equity, total_unrealized);

    Ok((zusd, xxbt, equity, total_unrealized, positions, orders))
}
