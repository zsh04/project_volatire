use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512, Digest};
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;
use reqwest;
use tracing::{info, error};

type HmacSha512 = Hmac<Sha512>;

#[derive(Debug)]
pub struct KrakenClient {
    api_key: String,
    #[allow(dead_code)]
    private_key: Vec<u8>,
    base_url: String,
}

impl KrakenClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = std::env::var("KRAKEN_API_KEY")?;
        let private_key_b64 = std::env::var("KRAKEN_PRIVATE_KEY")?;
        
        // Decode base64 private key
        let private_key = general_purpose::STANDARD.decode(&private_key_b64)?;
        
        Ok(Self {
            api_key,
            private_key,
            base_url: "https://api.kraken.com".to_string(),
        })
    }
    
    /// Place an order on Kraken
    /// pair: e.g., "XBTUSD"
    /// side: "buy" or "sell"
    /// volume: quantity in base currency
    /// price: limit price
    /// validate_only: if true, validate inputs only (no execution)
    pub async fn place_order(
        &self,
        pair: &str,
        side: &str,
        volume: f64,
        price: f64,
        validate_only: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let mut params = std::collections::HashMap::new();
        params.insert("pair", pair.to_string());
        params.insert("type", side.to_string());
        params.insert("ordertype", "limit".to_string());
        params.insert("price", price.to_string());
        params.insert("volume", volume.to_string());
        
        if validate_only {
            params.insert("validate", "true".to_string());
        }
        
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
            .to_string();
        params.insert("nonce", nonce.clone());
        
        // Sign request
        // Manually build POST body to ensure order matches signature
        let mut post_data = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>();
        post_data.sort(); // Must match sorting in sign()
        let post_data_str = post_data.join("&");

        // Sign using the EXACT string we will send
        let path = "/0/private/AddOrder";
        let signature = self.sign(path, &nonce, &post_data_str)?;
        
        let url = format!("{}{}", self.base_url, path);
        let client = reqwest::Client::new();
        
        if validate_only {
            info!("🔐 Kraken: Placing VALIDATION order - {} {} @ {} (Vol: {})", side.to_uppercase(), pair, price, volume);
        } else {
            info!("🚨 Kraken: Placing LIVE order - {} {} @ {} (Vol: {})", side.to_uppercase(), pair, price, volume);
        }
        
        let response: reqwest::Response = client
            .post(&url)
            .header("API-Key", &self.api_key)
            .header("API-Sign", signature)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(post_data_str) // Send exact string
            .send()
            .await?;
        
        let status = response.status();
        let body: Value = response.json().await?;
        
        if status.is_success() {
            if let Some(result) = body.get("result") {
                info!("✅ Kraken Order Validation: {}", serde_json::to_string_pretty(&result)?);
                Ok(serde_json::to_string(&result)?)
            } else if let Some(error) = body.get("error") {
                error!("❌ Kraken API Error: {}", error);
                Err(format!("Kraken error: {}", error).into())
            } else {
                Ok(body.to_string())
            }
        } else {
            error!("❌ Kraken HTTP Error {}: {}", status, body);
            Err(format!("HTTP {}: {}", status, body).into())
        }
    }

    /// Cancel an order on Kraken
    /// txid: The transaction ID (order ID) to cancel
    pub async fn cancel_order(
        &self,
        txid: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut params = std::collections::HashMap::new();
        params.insert("txid", txid.to_string());

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
            .to_string();
        params.insert("nonce", nonce.clone());

        let mut post_data = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>();
        post_data.sort();
        let post_data_str = post_data.join("&");

        let path = "/0/private/CancelOrder";
        let signature = self.sign(path, &nonce, &post_data_str)?;

        let url = format!("{}{}", self.base_url, path);
        let client = reqwest::Client::new();

        info!("🗑️ Kraken: Cancelling order {}", txid);

        let response: reqwest::Response = client
            .post(&url)
            .header("API-Key", &self.api_key)
            .header("API-Sign", signature)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(post_data_str)
            .send()
            .await?;

        let status = response.status();
        let body: Value = response.json().await?;

        if status.is_success() {
            if let Some(result) = body.get("result") {
                info!("✅ Kraken Order Cancelled: {}", serde_json::to_string_pretty(&result)?);
                Ok(serde_json::to_string(&result)?)
            } else if let Some(error) = body.get("error") {
                error!("❌ Kraken API Error: {}", error);
                Err(format!("Kraken error: {}", error).into())
            } else {
                Ok(body.to_string())
            }
        } else {
            error!("❌ Kraken HTTP Error {}: {}", status, body);
            Err(format!("HTTP {}: {}", status, body).into())
        }
    }
    
    fn sign(&self, path: &str, nonce: &str, post_data_str: &str) 
        -> Result<String, Box<dyn std::error::Error>> 
    {
        // SHA256(nonce + postdata)
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", nonce, post_data_str));
        let sha256_hash = hasher.finalize();
        
        // path + sha256
        let mut message = path.as_bytes().to_vec();
        message.extend_from_slice(&sha256_hash);
        
        // HMAC-SHA512
        let mut mac = HmacSha512::new_from_slice(&self.private_key)?;
        mac.update(&message);
        let signature = mac.finalize().into_bytes();
        
        // Base64 encode
        Ok(general_purpose::STANDARD.encode(signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires valid API keys
    async fn test_kraken_order_validation() {
        dotenvy::dotenv().ok();
        
        let client = KrakenClient::new().expect("Failed to create client");
        let result = client.place_order("XBTUSD", "buy", 0.001, 30000.0, true).await;
        
        assert!(result.is_ok(), "Order validation failed: {:?}", result.err());
    }
}
