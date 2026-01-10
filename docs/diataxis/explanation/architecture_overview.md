# System Architecture

The Voltaire system is a high-frequency algorithmic trading platform built on a hybrid Rust/Python architecture. It employs an OODA (Observe-Orient-Decide-Act) loop to process market data and generate trading decisions with sub-millisecond latency.

## High-Level Overview

The system consists of three primary subsystems:

1. **Reflex (The Kernel)**: A high-performance Rust executable responsible for market data ingestion, state management, risk checks, and order execution. It hosts the OODA loop.
2. **Brain (The Cogitation Layer)**: A Python-based service providing machine learning inference (Chronos, FinBERT) and complex reasoning capabilities via gRPC.
3. **Interface (The Control Plane)**: A Next.js/React frontend for real-time telemetry, visualization, and manual intervention.

```mermaid
graph TD
    Market[Market Data (Kraken/Binance)] -->|WebSocket| Ingest[Reflex: Ingestion]
    Ingest -->|Tick| Physics[Reflex: Physics Engine]
    Physics -->|State| OODA[Reflex: OODA Loop]
    OODA -->|GRPC| Brain[Brain Service]
    Brain -->|Inference| OODA
    OODA -->|Decision| Risk[Reflex: Risk Governor]
    Risk -->|Order| Gateway[Reflex: Order Gateway]
    Gateway -->|API| Exchange[Exchange Execution]

    subgraph Data Persistence
        OODA -->|Async| QuestDB[(QuestDB: Metrics)]
        OODA -->|Sync/Async| Redis[(DragonflyDB: State)]
    end
```

## Core Components

### Reflex (Rust)

* **Physics Engine**: Calculates kinetic derivatives (velocity, acceleration, jerk) of price and volume in real-time.
* **OODA Core**: The central event loop that orchestrates data flow. `Orient` phase aggregates data, `Decide` phase applies logic.
* **Governance Modules**:
  * *Legislator*: Applies strategic overrides (High/Low aggression).
  * *Sentinel*: Monitors system health (latency, jitter).
  * *Auditor*: Validates reasoning/decisions against safety constraints.

### Brain (Python)

* **Chronos**: Probabilistic time-series forecasting.
* **FinBERT**: Sentiment analysis on news/social feeds.
* **LLM Integration**: Optional high-level reasoning via Ollama/Gemma.

### Governance & Safety

The system prioritizes safety through multiple layers:

* **Hard Vetoes**: Code-level checks for excessive risk (e.g., max position size).
* **Circuit Breakers**: Automatic hibernation if latency exceeds thresholds.
* **Sovereign Intervention**: Manual kill-switches and overrides via the frontend.
