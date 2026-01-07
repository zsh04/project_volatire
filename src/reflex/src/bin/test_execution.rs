use reflex::execution::kraken::KrakenClient;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // 2. Load Env
    dotenv().ok();
    info!("🚀 Starting Kraken Execution Test...");

    // 3a. Verify Key with Balance Check (Read-Only)
    let api_key = std::env::var("KRAKEN_API_KEY")?;
    let api_secret = std::env::var("KRAKEN_PRIVATE_KEY")?;
    
    info!("🧪 Verifying Keys via Balance Check...");
    match reflex::ingest::kraken::fetch_account_data(&api_key, &api_secret).await {
        Ok((usd, btc, equity, pnl, pos, ord)) => {
            info!("✅ Balance Check Passed:");
            info!("   USD: ${}", usd);
            info!("   BTC: {}", btc);
            info!("   Equity: ${}", equity);
            info!("   PnL: ${}", pnl);
            info!("   Positions: {}", pos.len());
            info!("   Orders: {}", ord.len());
        },
        Err(e) => {
            info!("❌ Balance Check Failed. Keys are invalid or missing permissions.");
            info!("Error: {:?}", e);
            return Err(e);
        }
    }

    // 3b. Initialize Client for Execution
    let client = KrakenClient::new().expect("Failed to initialize Kraken Client");
    info!("✅ Kraken Client Initialized");

    // 4. Define Order Parameters
    let pair = "XBTUSD";
    let side = "buy";
    let volume = 0.0001; // Min size
    let price = 10000.0; // Safe price far below market
    let validate_only = false; // LIVE FIRE MODE

    // 5. Execute
    if !validate_only {
        info!("🚨 WARNING: EXECUTING LIVE TRADE. MONITOR KRAKEN UI.");
    }
    info!("📡 Sending Order: {} {} @ {} (Validate={})", side.to_uppercase(), pair, price, validate_only);
    
    match client.place_order(pair, side, volume, price, validate_only).await {
        Ok(response) => {
            info!("✅ SUCCESS: Order Placed/Validated");
            info!("📄 Response: {}", response);
        },
        Err(e) => {
            info!("❌ FAILURE: Order Rejected");
            info!("Result: {:?}", e);
        }
    }

    Ok(())
}
