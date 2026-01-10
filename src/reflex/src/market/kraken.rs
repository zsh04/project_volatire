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

// Kraken ticker format:
// [
//   channelID,
//   {
//     "a": ["52609.60000", 0, "0.400"],
//     "b": ["52609.50000", 0, "0.400"],
//     "c": ["52609.60000", "0.00000000"],
//     "v": ["3736.21980665", "5318.57777176"],
//     "p": ["52932.96200", "52857.25197"],
//     "t": [30896, 45296],
//     "l": ["51655.00000", "51655.00000"],
//     "h": ["53555.00000", "53555.00000"],
//     "o": ["52329.80000", "52062.40000"]
//   },
//   "ticker",
//   "XBT/USD"
// ]
pub fn parse_kraken_ticker(msg: &str) -> Option<Tick> {
    let value: serde_json::Value = serde_json::from_str(msg).ok()?;

    if !value.is_array() {
        return None;
    }

    let arr = value.as_array()?;
    if arr.len() < 4 {
        return None;
    }

    // Check if channel name is "ticker"
    if let Some(channel_name) = arr.get(2).and_then(|v| v.as_str()) {
        if channel_name != "ticker" {
            return None;
        }
    } else {
        return None;
    }

    // Extract ticker object (index 1)
    let ticker = arr.get(1)?.as_object()?;

    // Last Trade Price (c[0])
    let c_arr = ticker.get("c")?.as_array()?;
    let price: f64 = c_arr.get(0)?.as_str()?.parse().ok()?;

    // Best Ask (a[0])
    let a_arr = ticker.get("a")?.as_array()?;
    let ask: f64 = a_arr.get(0)?.as_str()?.parse().ok()?;

    // Best Bid (b[0])
    let b_arr = ticker.get("b")?.as_array()?;
    let bid: f64 = b_arr.get(0)?.as_str()?.parse().ok()?;

    Some(Tick {
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as f64,
        price,
        quantity: 0.0, // Ticker update doesn't have last trade volume in a simple way (c[1] is volume of last trade)
        bid: Some(bid),
        ask: Some(ask),
    })
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
    
    // Check Channel Name
    let channel_name = arr.get(2).and_then(|v| v.as_str())?;
    
    match channel_name {
        "trade" => {
            // ... (Existing Trade Logic)
            let trades = arr.get(1)?.as_array()?;
            let mut ticks = Vec::new();

            for trade_data in trades {
                let trade_arr = trade_data.as_array()?;
                if trade_arr.len() < 3 { continue; }

                let price: f64 = trade_arr.get(0)?.as_str()?.parse().ok()?;
                let volume: f64 = trade_arr.get(1)?.as_str()?.parse().ok()?;
                let timestamp: f64 = trade_arr.get(2)?.as_f64()?;

                ticks.push(Tick {
                    timestamp: timestamp * 1000.0,
                    price,
                    quantity: volume,
                    bid: None,
                    ask: None,
                });
            }
            Some(ticks)
        },
        "spread" => {
            // Spread format: [bid, ask, timestamp, bidVol, askVol]
            let spread_data = arr.get(1)?.as_array()?;
            if spread_data.len() < 3 { return None; }

            let bid: f64 = spread_data.get(0)?.as_str()?.parse().ok()?;
            let ask: f64 = spread_data.get(1)?.as_str()?.parse().ok()?;
            let timestamp: f64 = spread_data.get(2)?.as_f64()?;

            // Treat spread update as a Tick with 0 volume but valid bid/ask
            let tick = Tick {
                timestamp: timestamp * 1000.0,
                price: (bid + ask) / 2.0, // Mid price as proxy
                quantity: 0.0,
                bid: Some(bid),
                ask: Some(ask),
            };
            Some(vec![tick])
        },
        _ => None,
    }
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

    #[test]
    fn test_parse_kraken_ticker() {
        let msg = r#"[345,{"a":["52609.60000",0,"0.400"],"b":["52609.50000",0,"0.400"],"c":["52609.60000","0.00000000"],"v":["3736.21980665","5318.57777176"],"p":["52932.96200","52857.25197"],"t":[30896,45296],"l":["51655.00000","51655.00000"],"h":["53555.00000","53555.00000"],"o":["52329.80000","52062.40000"]},"ticker","XBT/USD"]"#;
        let tick = parse_kraken_ticker(msg).unwrap();

        assert_eq!(tick.price, 52609.60);
        assert_eq!(tick.ask, Some(52609.60));
        assert_eq!(tick.bid, Some(52609.50));
    }
}
