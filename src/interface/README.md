# Command Deck - Setup Guide

The Voltaire Command Deck is a real-time trading HUD built with Next.js 15 and gRPC-web.

---

## Quick Start (Demo Mode)

```bash
cd src/interface
npm install --legacy-peer-deps
npm run dev
```

Navigate to `http://localhost:3000` - the UI will show **simulated data**.

---

## Live Mode Setup

### Prerequisites

1. **Reflex gRPC Server** running on `localhost:50051`
2. **Docker** for Envoy proxy
3. **protoc** compiler for TypeScript generation (optional)

### Step 1: Start Envoy Proxy

```bash
cd infra
docker-compose -f docker-compose.envoy.yaml up -d
```

This starts Envoy on port `8080`, translating HTTP/gRPC-web → gRPC for browser clients.

**Verify**:

```bash
curl http://localhost:9901/ready  # Envoy admin health check
```

### Step 2: Start Reflex gRPC Server

```bash
cd src/reflex
cargo run --release --bin live_runner
```

Ensure the gRPC server is listening on `0.0.0.0:50051`.

### Step 3: Generate TypeScript Stubs (Optional)

> **Note**: Proto generation requires `protoc` and plugins. If skipped, you can use a mock client or manually define types.

Install `protoc`:

```bash
brew install protobuf  # macOS
```

Install gRPC-web plugin:

```bash
npm install -g grpc-web
```

Generate stubs:

```bash
cd src/interface
chmod +x scripts/generate-protos.sh
./scripts/generate-protos.sh
```

This creates `lib/grpc/generated/reflex_*` TypeScript files.

### Step 4: Enable Live Telemetry

```bash
cd src/interface
echo "NEXT_PUBLIC_USE_LIVE_TELEMETRY=true" > .env.local
echo "NEXT_PUBLIC_ENVOY_URL=http://localhost:8080" >> .env.local
```

Restart the dev server:

```bash
npm run dev
```

---

## Architecture

```
Browser (localhost:3000)
    ↓ HTTP/gRPC-web
Envoy Proxy (localhost:8080)
    ↓ gRPC
Reflex Server (localhost:50051)
```

---

## Mode Toggle

| Mode | Variable | Data Source |
|------|----------|-------------|
| **Demo** | `NEXT_PUBLIC_USE_LIVE_TELEMETRY=false` | Simulated (sine waves) |
| **Live** | `NEXT_PUBLIC_USE_LIVE_TELEMETRY=true` | Real gRPC stream |

Both modes run simultaneously - demo is fallback if live stream fails.

---

## Components

| Component | Purpose |
|-----------|---------|
| `ConsensusMeter` | Simons/Hypatia alignment gauge |
| `ReasoningStream` | Scrollable decision log |
| `RiemannCloud` | 3D physics visualization (Three.js) |
| `SafetyStaircase` | Risk tier display |
| `MarketInternals` | VIX/TICK/Alpha metrics |

---

## Troubleshooting

### Envoy not starting

```bash
docker logs voltaire-envoy
```

Check for port conflicts on `8080` or `9901`.

### gRPC connection refused

Ensure Reflex is running:

```bash
lsof -i :50051
```

### Proto generation fails

Manually install dependencies:

```bash
npm install --save-dev grpc-tools ts-protoc-gen --legacy-peer-deps
```

### React 19 peer dependency warnings

Use `--legacy-peer-deps` for all npm install commands:

```bash
npm install --legacy-peer-deps
```

---

## Performance

- **Target**: 60fps during 1000 tick/sec
- **Physics updates**: 150ms interval (Tick-to-ACK budget)
- **OODA updates**: 1s interval
- **Three.js**: GPU-accelerated WebGL

---

## Next Steps

1. Add WebAuthn for sovereign controls
2. Implement mobile swipe gestures
3. Add Web Worker for telemetry parsing
4. Performance profiling under load

---

**Status**: Phase 4 Complete ✅  
**Dependencies**: 724 packages  
**Files**: 13 components + infrastructure
