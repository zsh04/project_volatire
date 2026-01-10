# The 19ms Latency Budget

**Module:** `src/reflex/src/governor/ooda_loop.rs`
**Maximum Allowable Cycle:** 19ms (Tick-to-Trade)
**Jitter Threshold:** 20ms (Hard Drop)

## 1. The Budget Breakdown

Reflex operates on a strict "Physics-First" principle. If the Brain (Semantic Layer) cannot reason within the budget, it is bypassed.

| Phase | Component | Budget | Description |
| :--- | :--- | :--- | :--- |
| **Observe** | `IngestAdapter` | 2ms | Deserialization of WebSocket frames (Serde to Struc). |
| **Orient** | `Feynman` | <1ms | Calculation of Velocity ($v$), Acceleration ($a$), Jerk ($j$). |
| **Context** | `BrainClient` | **12ms** | gRPC RTT + Inference. The "Heavy" path. |
| **Decide** | `OODACore` | 1ms | Rule matching, Veto Gates, Legislative Checks. |
| **Act** | `ExecutionAdapter` | 3ms | Formatting Wire Protocol and Network transmission. |
| **Total** | | **~18-19ms** | |

## 2. Jitter Handling

The `orient` function in `ooda_loop.rs` implements a `tokio::time::timeout` wrapper around the Semantic Fetch.

```rust
// src/reflex/src/governor/ooda_loop.rs

match tokio::time::timeout(self.jitter_threshold, client.get_context(...)).await {
    Ok(Ok(ctx)) => {
        // ... Process Brain Context ...
    },
    Err(_) => {
        tracing::warn!("Brain Timeout (Jitter Violated)");
        (None, None, None) // Fallback to Blind Physics
    }
}
```

### The "Blind Mode" Fallback

If the budget is exceeded:

1. **Sentiment is Nullified**: `sentiment_score` becomes `None`.
2. **Risk Floor Applied**: Conviction is halved (`base_signal *= 0.5`).
3. **Vetoes Disabled**: Semantic Vetoes (e.g., "News Panic") cannot trigger if data is missing, so we rely on **Kinematic Vetoes** (Physics stops).

## 3. Garbage Collection & Noise

Reflex runs entirely on the stack or pre-allocated arenas where possible to avoid GC pauses (Rust ownership model). However, gRPC serialization can induce micro-latency. The **SyncGate** (`D-91`) measures wall-clock time at the start and end of the loop; if `elapsed > 150ms`, the decision is discarded as "Stale".
