# API Glossary

**Status:** Public Reference
**Scope:** Core Class Interfaces for Integration.

## 1. Safety & Physics (Reflex)

### `PhysicsEngine`

The kinematic core. Tracks the "Physical State" of the market.

* **File:** `src/reflex/src/feynman.rs`
* **Method:** `update(price: f64, timestamp: f64) -> PhysicsState`
* **Output:** Velocity, Acceleration, Jerk, Entropy, Efficiency.

### `RiskGuardian`

The "Iron Gate" policy engine.

* **File:** `src/reflex/src/taleb/mod.rs`
* **Method:** `check(physics, account, intent, forecast...) -> RiskVerdict`
* **Output:** `Allowed`, `Veto(Reason)`, `Panic`.
* **Key Logic:** Omega Ratio < 1.5 $\to$ Veto.

## 2. Intelligence (Brain)

### `KeplerOracle`

The probabilistic forecasting engine (Chronos-Bolt).

* **File:** `src/brain/src/kepler/engine.py`
* **Method:** `generate_forecast(df: pd.DataFrame, horizon: int) -> pd.DataFrame`
* **Output:** Time-series of Quantiles (P10, P50, P90).

### `BoydStrategist`

The OODA Loop decision synthesizer.

* **File:** `src/brain/src/boyd/engine.py`
* **Method:** `decide(market_data, valuation, regime, forecast) -> TradeDecision`
* **Logic:**
  * **Long:** Undervalued + Positive Skew (P90-P50 > P50-P10).
  * **Short:** Overvalued + Negative Skew.

## 3. Neural Networks

### `EchoStateNetwork` (Simons)

Reservoir Computing for non-linear valuation.

* **File:** `src/reflex/src/simons.rs`
* **Method:** `train(target: f64)`, `forward(input: f64) -> f64`.
* **Architecture:** Fixed Reservoir ($W_{res}$), Learned Readout ($W_{out}$) via RLS.
