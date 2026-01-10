# Legislative Log Interpretations

**Log Source:** `reflex_service` (Stdout / Grafana)
**Filter:** `target: "legislator"` OR `message.contains("VETO")`

The Trade Journal is the ultimate source of truth. Here is how to audit the AI's reasoning vs. The Pilot's Will.

## 1. The Veto Signature (Rust Layer)

When the Legislature overrides the Brain:

`WARN 🚫 RUST VETO: Sell Blocked by LongOnly Legislation`

* **Meaning:** The Brain saw a bearish setup and tried to Short/Sell. The Pilot's "Long Only" toggle overrode this to a HOLD.
* **Action:** No trade. System is complying with directives.

## 2. The Hibernation Signature

When the Red Button is active:

`WARN ☢️ HIBERNATION ACTIVE: SYSTEM IS READ-ONLY`

* **Frequency:** Every Orient Cycle (High frequency log).
* **Audit Check:** If trades occur while this log is streaming, **CRITICAL FAILURE** (Bypass/Leak).

## 3. The Update Signature

When parameters change via the Sidebar:

`INFO ⚖️ LEGISLATURE UPDATED: Bias="LongOnly", Aggression=1.5, MakerOnly=true`

* **Timestamp:** Verify this timestamp matches your manual click (Latency check).
* **Aggression:** Confirm the multiplier (e.g., 1.5x) matches the slider.

## 4. The Brain Alignment Check

If logs show:
`INFO Decision: BUY (Confidence: 90%)` -> `WARN RUST VETO`

**Diagnosis:** The AI is aggressive, but the Pilot is conservative.
**Resolution:**

* If AI is "Right" (Price went up), consider removing Veto.
* If AI is "Wrong" (Price crashed), the Veto saved capital. **Good Pilot.**
