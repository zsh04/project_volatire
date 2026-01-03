# Service Catalog: The Nervous System

**Reference:** `protos/brain.proto`, `reflex/src/server.rs`

## 1. The OODA Loop

The system operates on a continuous Observation-Orientation-Decision-Action loop, mediated by gRPC.

1. **Reflex (Body):** Ingests Market Data ($v, j, H$).
2. **Reflex -> Brain:** Sends `Reason(StateVector)`.
3. **Brain (Mind):** Consults Kepler (Forecast) + Boyd (Strategy).
4. **Brain -> Reflex:** Returns `StrategyIntent` (Buy/Sell + Confidence).
5. **Reflex:** Validates via `RiskGuardian`.
6. **Reflex:** Executes via `ExecutionActor`.

## 2. BrainService (gRPC)

**Port:** `50051` (Default)

| RPC Method | Input | Output | Description |
| :--- | :--- | :--- | :--- |
| `Reason` | `StateVector` | `StrategyIntent` | Core decision loop. Submits current physics state, receives trading orders. |
| `Forecast` | `HistoryWindow` | `ForecastResult` | Requests a pure price/volatility forecast (Kepler) without trade logic. |
| `Heartbeat` | `Pulse` | `PulseAck` | Liveness check to ensure the "Mind" is awake. |
| `NotifyRegimeChange` | `RegimeEvent` | `Empty` | Alerts Brain of macro shifts (e.g., Volatility Spikes). |

## 3. ReflexServer (gRPC)

**Port:** `50052` (Default)
**Component:** `reflex/src/server.rs`

Reflex acts as a server mainly for Control Plane interactions (Dashboard/CLI).

| RPC Method | Input | Output | Description |
| :--- | :--- | :--- | :--- |
| `GetStream` | `Empty` | `stream Heartbeat` | Telemetry stream for Dashboards (Simulates pulse). |
| `TriggerRatchet` | `RatchetRequest` | `Ack` | Manual intervention (e.g., "Tighten Stops"). |
| `UpdateConfig` | `ConfigPayload` | `Ack` | Hot-swap parameters (e.g., `MAX_JERK`). |
