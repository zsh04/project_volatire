# Recovery Protocols: State Decoherence

**Module:** `src/reflex/src/ingest/`
**Severity**: High (Risk of Stale Pricing)

## 1. WebSocket Decoherence

Socket disconnections are treated as "The Norm," not exceptions.

### Reconnection Logic (`kraken.rs`)

The ingestion loop is wrapped in a permanent `loop {}` block.

1. **Detection**: `read.next().await` returns `None` or `Error`.
2. **State**: The channel transmitter `tx` is dropped (or kept alive depending on architecture).
3. **Backoff**: Fixed **5-second sleep** (`tokio::time::sleep(Duration::from_secs(5))`) to prevent thundering herds against the API.
4. **Resubscribe**: On reconnect, the subscription payload (`{"event": "subscribe", ...}`) is resent immediately.

## 2. Account State Resync (`Directive-72`)

If the WebSocket feed for *Balances* or *Orders* drops, the system might drift from the Exchange's truth.

### The REST Poller

Reflex runs a low-frequency background poller (separate from the fast tick loop) that calls:

* `/0/private/TradeBalance` (Equity)
* `/0/private/OpenPositions`
* `/0/private/OpenOrders`

**Drift Detection:**
If `Local_Equity != Remote_Equity`, the system triggers an `ACCOUNT_DRIFT` warning log.

### Manual Reset Protocol

If the Pilot notices a drift in the HUD ("Vitality" mismatch):

1. **Engage HIBERNATION** (Tactical Sidebar > Hibernate).
    * This stops all *new* OODA decisions.
2. **Restart Reflex**: The `live_runner` binary must be restarted to flush the in-memory `PortfolioStore`.
    * `docker restart reflex_service` (or equivalent).
3. **Verify**: Check "Equity" matches Kraken Dashboard.
4. **Disengage Hibernation**.
