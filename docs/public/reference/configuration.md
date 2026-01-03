# Configuration Reference

**Public Spec**

## Environment Variables (`.env`)

### System Identity

* `ENV`: `PROD` or `DEV`. Controls log verbosity and safety overrides.
* `OTEL_SERVICE_NAME`: The identifier for traces (e.g., `cc_engine_metal`).

### Database Connections

* `DATABASE_URL`: Postgres-wire connection string for QuestDB.
  * Format: `postgresql://user:pass@host:8812/db`
* `QUESTDB_ILP_PORT`: Influx Line Protocol port (9009) for high-speed ingest.
* `CLOUDFLARE_STORAGE_URL`: S3-compatible R2 endpoint for data lake.

### Market Data Services

* `ALPACA_API_KEY` / `ALPACA_API_SECRET`: For Execution and Market Data.
* `BINANCE_API_KEY`: For Crypto Data.
* `FRED_API_KEY`: For Macroeconomic Data (Risk-Free Rate).

### Telemetry (Grafana Cloud)

* `GRAFANA_CLOUD_ENDPOINT`: OTLP HTTP Endpoint.
* `GRAFANA_CLOUD_AUTH`: Basic Auth Token (Base64).

## System Constraints

* **Max Leverage**: Defined by `MAX_LEVERAGE` (Default: 1.0x).
* **Fat Finger Cap**: Defined by `FAT_FINGER_CAP_PCT` (Default: 20% of NAV).
* **Health Check**: Watchdog timer defaults to `600s`.

## Docker Network

Services communicate over the `voltaire_network`.

* `cc_pulse`: Telemetry Gateway.
* `dragonfly`: State Store.
* `questdb`: Time Series Store.
