# Project V Technical Reference Manual

**Version**: 2.3
**Status**: Active / Production Ready
**Codename**: Cottage Prime

## 1. System Architecture

**Project V** is a hybrid-metal autonomous trading system. It utilizes a **Decoupled Microservices** pattern to fuse the speed of **Rust** (The Reflex) with the cognitive flexibility of **Python** (The Brain).

### High-Level Topology

graph TD
    User[Trader] -->|View| FE[Frontend (Glass Cockpit)]
    FE -->|WebSocket| Gateway[Gateway (Node.js)]

    subgraph "The Nervous System (gRPC)"
        Reflex[ReflexD (Rust)] <-->|Pulse/State| Brain[BrainD (Python)]
    end
    
    subgraph "The Reflex (Body)"
        Reflex -->|Ticks| Feynman[Feynman (Physics)]
        Reflex -->|Signal| Simons[Simons (ESN)]
        Reflex -->|Veto| Taleb[Taleb (Risk)]
        Taleb -->|Execute| Exchange[Alpaca/Bybit]
    end
    
    subgraph "The Brain (Mind)"
        Brain -->|Ask| Kepler[Chronos-Bolt (Forecast)]
        Brain -->|Ask| Boyd[Gemma 2 (Strategy)]
        Brain -->|Ask| Hypatia[DistilBERT (News)]
    end
    
    subgraph "Storage Trinity"
        Reflex -->|State| Dragonfly[(Dragonfly - Hot)]
        Reflex -->|History| QuestDB[(QuestDB - Cold)]
        Brain -->|Context| LanceDB[(LanceDB - Warm)]
    end

2. Core Services
 2.1 ReflexD (The Body)
  Location: src/reflexL
  anguage: Rust (Tokio/Tonic)
  Responsibility:
   Ingest: Direct WebSocket connection to Exchange (L1 Data).
   Physics Engine: Calculates Mass, Velocity, Jerk, and Entropy in $O(1)$.
   Risk Engine: Enforces the Ratchet Protocol (Kill Switch) and Maker-Only Mode.
   Survivability: Operates independently. If BrainD disconnects, ReflexD enters Safe Mode (Cash).
 2.2 BrainD (The Mind)
  Location: src/brain
  Language: Python 3.12 (Pydantic AI)
  Responsibility:
   Reasoning: Large Language Model (Gemma 2 9B) processing via Ollama.
   Forecasting: Probabilistic Time-Series generation (Chronos-Bolt).
   Context: RAG (Retrieval Augmented Generation) via LanceDB.
 2.3 Gateway
  Location: src/gateway
  Language: Node.js
  Responsibility:
   UI Bridge: Exposes ReflexD streams to the Frontend via WebSockets.
   Auth: Handles user sessions (JWT) for the Glass Cockpit.

1. Data Definitions (The Synapse)
Communication occurs strictly via gRPC Protobufs.

 The State Vector ($\Psi$)
 Defined in protos/brain.proto. This is the "Truth" sent from Body to Mind.

 Field,Type,Description
 timestamp,double,Unix epoch (microsecond precision).
 price,double,Last traded price.
 velocity,double,Log return rate: ln(Pt​/Pt−1​).
 jerk,double,Change in acceleration: da/dt.
 entropy,double,Shannon entropy of the last 100 ticks.
 efficiency_index,double,Thermodynamic Efficiency (η=Work/Energy).

 The Strategy Intent
 Defined in protos/brain.proto. This is the "Command" sent from Mind to Body.

 Field,Type,Description
 regime,string,"""RIEMANN"" (Trend) or ""SHANNON"" (Noise)."
 conviction,double,Confidence score (0.0 to 1.0).
 risk_scalar,double,Multiplier for position sizing (0.0 to 1.0).
 halt_trading,bool,Emergency Kill Switch trigger.

1. Component Detail
 4.1 Physics Engine (Feynman)
  Implementation: Pure Rust.
  Logic: Uses Incremental Statistics (Welford's Algorithm) to update variance and entropy without re-scanning history windows.
  Latency: < 10 microseconds per tick.

 4.2 Risk Engine (Taleb)
  The Ratchet: A one-way function. It can tighten limits instantly based on Jerk or Drawdown, but requires legislative (config) changes to loosen them.
  The Shiver: An autonomic loop that reduces position size to 0.01 lots if efficiency_index drops below threshold.

 4.3 Reasoning Engine (Boyd)
  Implementation: Pydantic AI.
  Model: gemma2:9b (via Ollama).
  Output: Strictly typed StrategyIntent object. No text parsing.

1. Infrastructure (Docker)

 Service,Image,Role,Port
 Dragonfly,dragonflydb/dragonfly,"Hot Storage. Ephemeral state, Pub/Sub.",:6379
 QuestDB,questdb/questdb,Cold Storage. Infinite tick history.,:9000
 LanceDB,(Embedded),Warm Storage. Vector context.,N/A
 Ollama,ollama/gemma2:9b,Inference. LLM Host.,:11434

1. Observability

 Metrics (Prometheus/Grafana)
 All metrics are prefixed with cc. (Curiosity Cottage).
  cc.physics.jerk: Gauge. Current market turbulence.
  cc.risk.drawdown: Gauge. Current % deviation from High Water Mark.
  cc.system.efficiency: Gauge. Rolling $\eta$ index.
  cc.rpc.latency: Histogram. Brain/Reflex round-trip time.
  
  Logs (Loki/Grafana)
   Format: JSON Structured Logging.
   Levels:
    INFO: Trade executed, State updated.
    WARN: Efficiency low (Maker-Only Mode active).
    ERROR: gRPC Pulse missed (Stale Consciousness).
    FATAL: Gap-to-Zero detected.
