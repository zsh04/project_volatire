use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512, Digest};
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;
use reqwest;
use tracing::{info, error};

type HmacSha512 = Hmac<Sha512>;

pub struct KrakenClient {
    api_key: String,
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
    
    /// Place a test limit order on Kraken
    /// pair: e.g., "XBTUSD"
    /// side: "buy" or "sell"
    /// volume: quantity in base currency
    /// price: limit price
    pub async fn place_test_order(
        &self,
        pair: &str,
        side: &str,
        volume: f64,
        price: f64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        let mut params = std::collections::HashMap::new();
        params.insert("pair", pair.to_string());
        params.insert("type", side.to_string());
        params.insert("ordertype", "limit".to_string());
        params.insert("price", price.to_string());
        params.insert("volume", volume.to_string());
        params.insert("validate", "true".to_string()); // TEST ONLY - validate without execution
        
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
            .to_string();
        params.insert("nonce", nonce.clone());
        
        // Sign request
        let path = "/0/private/AddOrder";
        let signature = self.sign(path, &params)?;
        
        // Build request
        let url = format!("{}{}", self.base_url, path);
        let client = reqwest::Client::new();
        
        info!("🔐 Kraken: Placing TEST order (validate=true) - {} {} @ {} (Vol: {})", side.to_uppercase(), pair, price, volume);
        
        let response: reqwest::Response = client
            .post(&url)
            .header("API-Key", &self.api_key)
            .header("API-Sign", signature)
            .form(&params)
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
    
    fn sign(&self, path: &str, params: &std::collections::HashMap<&str, String>) 
        -> Result<String, Box<dyn std::error::Error>> 
    {
        let nonce = params.get("nonce").ok_or("Missing nonce")?;
        
        // Build POST data
        let mut post_data = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>();
        post_data.sort(); // Kraken requires sorted params
        let post_data_str = post_data.join("&");
        
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
        let result = client.place_test_order("XBTUSD", "buy", 0.001, 30000.0).await;
        
        assert!(result.is_ok(), "Order validation failed: {:?}", result.err());
    }
}
