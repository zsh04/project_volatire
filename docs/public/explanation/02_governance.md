# Governance: The Kinetic Constitution

State: **Active**
Version: **1.0** (Phase 4 Verified)

The Governance layer ("The Governor") is the supreme authority in the Voltaire system. It operates on an **OODA Loop** (Observe, Orient, Decide, Act) to ensure that every trade is physically sound, semantically verified, and constitutionally compliant.

## 1. The Core Philosophy

**"Safety > Profit."**
The system prioritizes survival over maximizing returns. This is enforced through strict, zero-trust gates.

## 2. The OODA Loop

Implemented in `src/reflex/src/governor/ooda_loop.rs`.

1. **Observe**: Ingest real-time market physics (Velocity, Acceleration, Jerk) and financial news.
2. **Orient**: Recall historical regimes (LanceDB) and check sentiment (DistilBERT).
    * *Jitter Fallback*: If semantic checks exceed latency budgets (e.g. > 20ms), the system degrades to "Blind Mode" with reduced risk sizing.
3. **Decide**:
    * **Nuclear Veto**: Checks for Qualitative Panic + Quantitative Chaos.
    * **Provisional Limits**: Checks stability tier.
    * **Strategy**: Calculates $P_{\text{Riemann}}$ (Momentum vs Mean Rev).
4. **Act**: Atomically executes the decision (Buy, Sell, Hold, Halt).

## 3. The Nuclear Veto (Directive-45)

Implemented in `src/reflex/src/brain/veto_gate.rs`.

A **Double-Key** mechanism that triggers a `Checkmate` (System Halt) only if:

* **Narrative is Toxic**: Sentinel Sentiment Score $< -0.90$.
* **Physics is Broken**: Absolute Jerk ($|j|$) $> 50.0$.
* **Math is Negative**: Taleb Omega Ratio $< 1.0$.

This prevents "Ghost Halts" from fake news while ensuring we respect true black swan events.

## 4. The Safety Staircase (Directive-43)

Implemented in `src/reflex/src/governor/provisional.rs`.

The system cannot "jump" to full risk. It must climb a staircase of trust based on **Stability Quantiles**.

* **Tiers**: 0.01 $\rightarrow$ 0.05 $\rightarrow$ 0.10 ... $\rightarrow$ 1.0 Lots.
* **Promotion**: Requires $N$ consecutive stable cycles (Low Jerk, High Efficiency).
* **Demotion**: Instant reset to Tier 0 if stability breaks.

## 5. Strategy Superposition (Directive-44)

Implemented in `src/reflex/src/governor/superposition.rs`.

We do not use binary logic for strategy selection. We calculate $P_{\text{Riemann}}$:

* **Laminar Flow**: High Efficiency ($\eta > 0.85$) boosts Momentum allocation.
* **Structural Noise**: High Entropy/Jerk collapses the probability to 0.0 (Mean Reversion/Cash).
