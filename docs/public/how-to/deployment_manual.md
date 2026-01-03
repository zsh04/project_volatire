# Deployment Manual

**Public Guide**

## 1. Prerequisites

* **Docker & Docker Compose**: v20.10+
* **Rust Toolchain**: Stable (1.80+)
* **Python**: 3.10+
* **Hardware**: Min 16GB RAM, AVX2 Support (for Vector DB).

## 2. Infrastructure Stack

The system runs on a unified Docker stack defined in `docker/`:

1. **Voltaire Stack**: `docker/telemetry-stack.yaml` (OTel Collector, database bridges).
2. **Core Stack**: `docker-compose.yaml` (Dragonfly, QuestDB, App Services).

### Starting the Infrastructure

```bash
# 1. Start Telemetry Grid (Collector)
docker-compose -f docker/telemetry-stack.yaml up -d

# 2. Start Data Plane (Dragonfly & QuestDB)
docker-compose up -d dragonfly questdb
```

## 3. Configuration

Copy `.env.example` to `.env` and populate the required API keys.
> **Note**: For production, ensure `ENV=PROD` to enable safety guardrails.

### Critical Variables

* `DATABASE_URL`: QuestDB Wire Protocol (default port 8812).
* `OTEL_EXPORTER_OTLP_ENDPOINT`: Collector endpoint (default `http://localhost:4318`).
* `ALPACA_API_KEY`: Execution credentials.

## 4. Database Initialization

### QuestDB

The system automatically creates required tables (OHLCV, Logs) on startup if they don't exist.
Admin Console: `http://localhost:9000`

### DragonflyDB (Redis)

State is ephemeral-first but persisted to disk. No manual schema init required.

## 5. Deployment Checklist

- [ ] `.env` credentials secured.
* [ ] Telemetry Collector running (`cc_pulse`).
* [ ] Databases reachable (`pg_isready`).
* [ ] `reflex` binary compiled (`cargo build --release`).
