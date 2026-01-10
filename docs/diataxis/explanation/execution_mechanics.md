# Execution Mechanics: The Actor Model

**Module:** `src/reflex/src/execution/`
**Component:** `ExecutionAdapter`

The Execution Adapter is the "muscle" of the system. It is responsible for
translating abstract `TradeProposal` intents into concrete wire protocols
(Exchange Calls), while strictly adhering to API Rate Limits.

## 1. Rate Limiting (Token Bucket)

Reflex uses a strict **Token Bucket** algorithm to prevent Exchange IP bans.

* **Capacity:** 20 Tokens (Burst capability).
* **Refill Rate:** 10 Tokens/sec.
* **Cost:** 1 Token per Order.

```rust
pub struct TokenBucket {
    capacity: f64,
    tokens: Mutex<f64>,
    refill_rate_per_sec: f64,
}
```

## 2. Execution Paths

The Actor supports two distinct execution modalities depending on urgency.

### A. The Sniper Path (`execute_sniper`)

**Usage:** Standard Entries, Rebalancing (Alpha Capture).

* **Logic:** "Shadow Limit" Chasing.
* The system places a Limit Order at the `proposal.price`.
* If not filled immediately, the system conceptually "chases" the price (in a
    loop, though `actor.rs` currently mocks the fill).
* **Constraints:** Strictly obeys rate limits. If bucket is empty, it **Warns
    and Drops** the frame (anticipating the next Tick will retry).

### B. The Nuclear Path (`execute_nuclear`)

**Usage:** Risk Shroud Breaches, Stop Losses, Black Swans.

* **Logic:** **IOC (Immediate-Or-Cancel)** Market Order.
* **Goal:** Liquidity at *any* cost.
* **Constraints:** Theoretically strictly rate limited, but prioritized. Log
    message implies "Retrying immediately" strategies would be deployed here in
    production to ensure exit even towards penalty thresholds.

## 3. Latency Simulation

The current implementation explicitly mocks network latency to ensure the OODA
loop simulation remains realistic (< 500us internal, + Simulated Network RTT).
