# Release v0.5.0: Telemetry Stabilization & System Ignition

## 🚀 Overview
Release v0.5.0 marks the transition from simulated data to **Live Telemetry**, ensuring the Frontend (React) receives generic, real-time physics and governance data from the Backend (Rust/Python) via Envoy. This release also introduces unified system lifecycle management.

## ✨ Key Features

### 1. Live Data Pipeline (The Pulse)
- **Zero-Stub Policy**: Removed all mock data simulations (Kepler, Boyd, Feynman) from the frontend.
- **Real-Time Physics**: Streaming velocity, acceleration, and entropy from `flux` engine @ 10Hz.
- **Latency Monitoring**: Integrated RTT measurement between `Reflex` (Rust) and `Brain` (Python).

### 2. System Lifecycle (Ignition)
- **`ignite.sh`**: One-click startup for Envoy, Reflex, Brain, and Frontend.
- **`shutdown.sh`**: Graceful termination of the entire stack.
- **Model Health Check**: Integrated verify step for `Chronos` and `DistilBERT` (Python 3.12).

### 3. Governance Telemetry
- **Staircase Visualization**: Live streaming of Risk Tiers (Q0-Max) and Progress % to the UI.
- **Audit Drift**: Live streaming of `drift_score` from the `AuditLoop`.

### 4. Infrastructure Fixes
- **CORS**: Fixed `Access-Control-Allow-Origin` issues in Envoy.
- **Binding**: Fixed IPv6 vs IPv4 binding mismatch (`0.0.0.0`).
- **Defensive UI**: Implemented strict null-checks to prevent White Screen of Death on malformed frames.

## 🛠️ Deployment Instructions

```bash
# 1. Update Dependencies
cd src/brain && poetry install && cd ../..
cd src/interface && npm install && cd ../..

# 2. Ignite System
./scripts/ignite.sh

# 3. Shutdown
./scripts/shutdown.sh
```

## ⚠️ Breaking Changes
- **Proto Schema**: `reflex.proto` updated. All clients must regenerate protos.
- **Envoy Config**: Requires `x-data-origin` header support.
