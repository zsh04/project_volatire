# Running in Live Mode

This guide explains how to configure and run the Voltaire system in "Live" mode (connected to real exchange feeds), potentially with "Shadow Execution" enabled.

## Prerequisites

* Rust Toolchain (latest stable)
* QuestDB (running on default port 9009)
* DragonflyDB/Redis (running on default port 6379)
* Environment Variables (.env) configured.

## Configuration

Edit your `.env` file to set the target symbol and mode:

```bash
# Target Asset
LIVE_SYMBOL=btcusdt

# Shadow Execution (True = Virtual Fills, False = Real Orders)
SHADOW_EXECUTION=true

# Database Config
DATABASE_URL=postgresql://admin:quest@localhost:8812/qdb
QUESTDB_HOST=localhost
REDIS_URL=redis://localhost:6379/
```

> [!WARNING]
> Setting `SHADOW_EXECUTION=false` will enable **REAL MONEY TRADING**. Ensure you have adequate safeguards and capital limits before disabling shadow mode.

## Execution

To start the live runner binary:

```bash
cargo run --bin live_runner --release
```

## Monitoring

Once running, the system will emit telemetry to the configured endpoints.

1. Open the **Interface** (localhost:3000) to view the Command Deck.
2. Data origin will be flagged as `LIVE` in the UI.
3. Check terminal output for "OODA Loop" status logs:
    * `👻 SHADOW`: Indicates virtual execution.
    * `⚡ SENT ORDER`: Indicates real execution.
