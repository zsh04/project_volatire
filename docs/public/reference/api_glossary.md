# API Glossary: Key Interfaces

## RiskGuardian

**Location:** `reflex/src/taleb/mod.rs` (Rust)

The **Iron Gate**. A deterministic state machine that validates every `TradeProposal` against physics and financial constraints.

**Key Method:** `check(...)`

```rust
pub fn check(
    physics: &PhysicsState,
    account: &AccountState,
    intent: &TradeProposal,
    forecast: (P10, P50, P90),
    hurdle_rate: f64
) -> RiskVerdict
```

**Checks:**

1. **TTL:** Is the forecast < 60s old?
2. **Jerk:** Is momentum stable? (`Jerk < 25`)
3. **Entropy:** Is the market orderly? (`Entropy < 0.90`)
4. **Omega:** Is the risk/reward asymmetric? (`Omega > 1.5`)
5. **Insolvency:** Do we have enough cash?

---

## KeplerOracle

**Location:** `src/brain/src/kepler/engine.py` (Python)
**Models:** `amazon/chronos-bolt-small`

The **Prophet**. Wraps the Chronos-Bolt Probabilistic Time Series model.

**Key Method:** `generate_forecast(df, horizon=10)`

- **Input:** DataFrame with `price` history.
- **Output:** DataFrame with quantiles `p10`...`p90` for next `horizon` steps.
- **Features:**
  - **Caching:** Caches forecasts for 60s to prevent GPU overload.
  - **Mock Mode:** Auto-fallbacks to synthetic "Fan Charts" if GPU/Torch fails.

---

## PhysicsEngine

**Location:** `reflex/src/feynman.rs` (Rust)

The **Observer**. Maintains a rolling window of market history to compute derivatives and thermodynamics.

**Key Method:** `update(price, timestamp) -> PhysicsState`

- **State:**
  - `velocity`: 1st Derivative (Trend).
  - `acceleration`: 2nd Derivative (Momentum).
  - `jerk`: 3rd Derivative (Crash Risk).
  - `volatility`: Realized Vol (Welford).
  - `efficiency`: Fractal Dimension proxy.
