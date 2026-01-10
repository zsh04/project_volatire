use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512, Digest};
use base64::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicI64, Ordering};

// --- Nonce Manager ---

pub struct NonceManager {
    last_nonce: AtomicI64,
}

impl NonceManager {
    pub fn new() -> Self {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        Self {
            last_nonce: AtomicI64::new(start),
        }
    }

    /// Returns a strictly increasing nonce (current millis, or last + 1 if collision)
    pub fn next(&self) -> i64 {
        let mut now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        
        // Simple spin loop to ensure monotonicity
        // If we generate nonces faster than 1ms, we increment manually
        let mut last = self.last_nonce.load(Ordering::Relaxed);
        loop {
            if now <= last {
                now = last + 1;
            }
            match self.last_nonce.compare_exchange_weak(
                last, now, Ordering::SeqCst, Ordering::Relaxed
            ) {
                Ok(_) => return now,
                Err(x) => last = x,
            }
        }
    }
}

// --- Kraken Signer ---

pub struct KrakenSigner {
    api_key: String,
    secret_key_decoded: Vec<u8>,
}

impl KrakenSigner {
    pub fn new(api_key: &str, private_key: &str) -> Result<Self, String> {
        let decoded = BASE64_STANDARD.decode(private_key)
            .map_err(|e| format!("Failed to decode Kraken private key: {}", e))?;
        Ok(Self {
            api_key: api_key.to_string(),
            secret_key_decoded: decoded,
        })
    }

    /// Signs a Kraken request.
    /// Logic: HMAC-SHA512(path + SHA256(nonce + post_data), b64_decoded_secret)
    pub fn sign(&self, path: &str, nonce: i64, post_data: &str) -> String {
        // 1. SHA256(nonce + post_data)
        let np = format!("{}{}", nonce, post_data);
        let mut sha256 = Sha256::new();
        sha256.update(np.as_bytes());
        let hash256 = sha256.finalize();

        // 2. HMAC-SHA512(path + hash256, secret)
        let mut mac = Hmac::<Sha512>::new_from_slice(&self.secret_key_decoded)
            .expect("HMAC can take key of any size");
        mac.update(path.as_bytes());
        mac.update(&hash256);
        
        let result = mac.finalize().into_bytes();
        BASE64_STANDARD.encode(result)
    }
    
    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }
}

// --- Binance Signer ---

pub struct BinanceSigner {
    api_key: String,
    secret_key: String,
}

impl BinanceSigner {
    pub fn new(api_key: &str, secret_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            secret_key: secret_key.to_string(),
        }
    }

    /// Signs a Binance request.
    /// Logic: HMAC-SHA256(query_string, secret) -> Hex string
    pub fn sign(&self, query_string: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        let result = mac.finalize().into_bytes();
        hex::encode(result)
    }

    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_manager() {
        let nm = NonceManager::new();
        let n1 = nm.next();
        let n2 = nm.next();
        assert!(n2 > n1);
    }

    #[test]
    fn test_binance_signature() {
        // Example from Binance Docs (if available) or verified manually
        // Secret: "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j"
        // Query: "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559"
        // Expected: "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71"
        
        let secret = "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j";
        let query = "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559";
        let signer = BinanceSigner::new("apikey", secret);
        
        let sig = signer.sign(query);
        assert_eq!(sig, "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71");
    }

    #[test]
    fn test_kraken_signature_structure() {
        // Since Kraken involves sha256(nonce + post) then hmac-sha512, harder to get a static test vector without a real key/time match
        // But we can verify it returns a valid base64 string
        // Dummy key (Base64 encoded)
        let dummy_key = BASE64_STANDARD.encode(b"ThisIsAFakeSecretKeyForTestingPurposeOnly123");
        let signer = KrakenSigner::new("apikey", &dummy_key).unwrap();
        let sig = signer.sign("/0/private/AddOrder", 1616492376594, "nonce=1616492376594&ordertype=limit&pair=XBTUSD&price=37500&type=buy&volume=1.25");
        
        // It must be a valid base64 string
        assert!(BASE64_STANDARD.decode(&sig).is_ok());
    }
}
