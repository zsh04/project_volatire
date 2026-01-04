use serde::Deserialize;
use crate::market::Tick;

// ==============================================================================
// Kraken WebSocket Message Structures
// ==============================================================================

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum KrakenMessage {
    // Subscription status/heartbeat
    Event(KrakenEvent),
    // Trade data array
    Trade(Vec<serde_json::Value>),
}

#[derive(Debug, Deserialize)]
pub struct KrakenEvent {
    pub event: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub pair: Option<String>,
    #[serde(default)]
    pub subscription: Option<serde_json::Value>,
}

// Kraken trade format: [channelID, [[price, volume, time, side, orderType, misc]], channelName, pair]
pub fn parse_kraken_trade(msg: &str) -> Option<Vec<Tick>> {
    let value: serde_json::Value = serde_json::from_str(msg).ok()?;
    
    if !value.is_array() {
        return None;
    }
    
    let arr = value.as_array()?;
    if arr.len() < 4 {
        return None;
    }
    
    // Check if this is a trade message (channel name should be "trade")
    if let Some(channel_name) = arr.get(2).and_then(|v| v.as_str()) {
        if channel_name != "trade" {
            return None;
        }
    } else {
        return None;
    }
    
    // Extract trade array (index 1)
    let trades = arr.get(1)?.as_array()?;
    let mut ticks = Vec::new();
    
    for trade_data in trades {
        let trade_arr = trade_data.as_array()?;
        if trade_arr.len() < 3 {
            continue;
        }
        
        let price: f64 = trade_arr.get(0)?.as_str()?.parse().ok()?;
        let volume: f64 = trade_arr.get(1)?.as_str()?.parse().ok()?;
        let timestamp: f64 = trade_arr.get(2)?.as_f64()?;
        
        ticks.push(Tick {
            timestamp: timestamp * 1000.0, // Convert to milliseconds
            price,
            quantity: volume,
        });
    }
    
    Some(ticks)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_kraken_trade() {
        let msg = r#"[0,[["50000.10","0.05",1704240000.123456,"b","l",""]],"trade","XBT/USD"]"#;
        let ticks = parse_kraken_trade(msg).unwrap();
        
        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0].price, 50000.10);
        assert_eq!(ticks[0].quantity, 0.05);
    }
}
