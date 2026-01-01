use tokio_postgres::Client;

pub async fn run_migrations(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 MIGRATION: Checking QuestDB Schema...");

    // 1. OHLCV 1-Minute (Nanosecond Precision, Week Partition)
    // using TIMESTAMP for QuestDB (which is micro/nano depending on config, usually micro in older, nano in v9+)
    // User requested "TIMESTAMP_NS" if available or standard TIMESTAMP with awareness.
    // QuestDB standard SQL uses `TIMESTAMP` type which is high precision.
    let ohlcv_1min = "
        CREATE TABLE IF NOT EXISTS ohlcv_1min (
            ts TIMESTAMP,
            symbol SYMBOL capacity 256 CACHE,
            open DOUBLE,
            high DOUBLE,
            low DOUBLE,
            close DOUBLE,
            volume DOUBLE
        ) TIMESTAMP(ts) PARTITION BY WEEK;
    ";

    // 2. OHLCV 1-Day (Unified)
    let ohlcv_1d = "
        CREATE TABLE IF NOT EXISTS ohlcv_1d (
            ts TIMESTAMP,
            symbol SYMBOL capacity 256 CACHE,
            open DOUBLE,
            high DOUBLE,
            low DOUBLE,
            close DOUBLE,
            volume DOUBLE
        ) TIMESTAMP(ts) PARTITION BY YEAR;
    ";

    // 3. Friction Ledger (Antifragile Accounting)
    let friction = "
        CREATE TABLE IF NOT EXISTS friction_ledger (
            ts TIMESTAMP,
            symbol SYMBOL capacity 256 CACHE,
            order_id SYMBOL capacity 256 CACHE,
            realized_slippage_bps DOUBLE,
            fee_native DOUBLE,
            tax_buffer DOUBLE
        ) TIMESTAMP(ts) PARTITION BY MONTH;
    ";

    client.execute(ohlcv_1min, &[]).await?;
    client.execute(ohlcv_1d, &[]).await?;
    client.execute(friction, &[]).await?;

    println!("✅ MIGRATION: Schema Synchronized.");
    Ok(())
}
