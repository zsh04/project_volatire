# Service Architecture

**Architecture:** Bi-Cameral (Two Minds)
**Communication:** gRPC (Protobuf)
**Latency Target:** < 5ms (Tick-to-Decision)

The system is split into two hemispheres:

1. **Reflex (Rust):** The Fast Brain. Handles Physics, Risk, and Execution.
2. **Brain (Python):** The Slow Brain. Handles Forecasting, Strategy, and Macro Reasoning.

## 1. Service Catalog

### BrainService (`brain.proto`)

The primary interface for the Python hemisphere.

| RPC | Type | Description |
|:---|:---|:---|
| `Reason` | Unary | **The Core Loop.** Reflex sends current state ($\Psi_t$); Brain returns Strategy Intent. |
| `Forecast` | Unary | Demand a probabilistic forecast (Chronos) for a given history window. |
| `NotifyRegimeChange` | Unary | Reflex alerts Brain of phase transitions (Laminar $\to$ Turbulent). |
| `Heartbeat` | Unary | Health check to ensure the Python process is alive and inference-ready. |

### ReflexService (`reflex.proto`)

The control plane for the Rust hemisphere.

| RPC | Type | Description |
|:---|:---|:---|
| `TriggerRatchet` | Unary | **Emergency Brake.** External command to tighten risk or panic (Kill Switch). |
| `UpdateConfig` | Unary | Dynamic parameter tuning (e.g., changing $\alpha$ for ESN). |
| `GetStream` | Streaming | Real-time telemetry stream for the UI (Vue/React). |

### MacroSynapse (`macro.proto`)

The conduit for Economic Gravity.

| RPC | Type | Description |
|:---|:---|:---|
| `SyncMacroState` | Unary | Pushes global macro factors (Interest Rates, GDP) from Brain $\to$ Reflex. |

## 2. The OODA Loop (Data Flow)

The Tick-to-Trade lifecycle follows the OODA (Observe, Orient, Decide, Act) loop:

1. **Observe (Reflex):**
    * `FastSocket` receives Market Tick.
    * `PhysicsEngine` computes Velocity, Acceleration, Jerk.
    * `Simons` (ESN) updates reservoir state.

2. **Orient (Brain):**
    * Reflex calls `Brain.Reason(StateVector)`.
    * Brain checks `Hypatia` (Context/Memory).
    * `Kepler` (Chronos) generates Forecast Distribution ($P_{10}, P_{50}, P_{90}$).

3. **Decide (Brain/Reflex):**
    * Brain generates `StrategyIntent` (Buy/Sell, Confidence).
    * Reflex receives Intent.
    * **Taleb (Risk)** audits the intent against:
        * Physics (Jerk check).
        * Math (Omega Ratio check).
        * Capital (Drawdown check).

4. **Act (Reflex):**
    * If `RiskVerdict::Allowed`: Order is sent to Exchange.
    * `AccountState` updated.
    * Telemetry emitted to GUI.
