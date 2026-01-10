# The Pilot’s Operating Manual: Ops

**Module:** `src/interface/components/HUD/TacticalSidebar.tsx`
**Backend:** `src/reflex/src/governor/legislator.rs`

This document defines the expected behavior of the "Tactical Overrides" (Directive-107).

## 1. Maker-Only Mode

**Icon:** Lock
**Function:** Forces the `ExecutionAdapter` to **Post Only**.

* **Behavior:**
  * If `Action::Buy`, we place a Limit at $Bid$.
  * We **DO NOT** cross the spread.
  * If the price moves away, we chase (Shadow Limit) but never pay Taker fees.
* **Use Case:** Low volatility accumulation.
* **Risk:** Non-execution (Opportunity Cost).

## 2. Hibernate (The Red Button)

**Icon:** PauseCircle
**Function:** Immediate System Read-Only Mode.

* **Behavior:**
  * `OODA Loop`: Continues to tick (observe).
  * `Decide`: **FORCED HOLD**. All signals are Vetoed.
  * `Act`: No orders sent.
* **Use Case:**
  * Account Drift detected.
  * Exchange API error.
  * "Something feels wrong."

## 3. Snap-to-Break-Even

**Icon:** Zap
**Function:** Emergency Exit to Neutrality.

* **Behavior:**
  * Scan all open positions.
  * Calculate $Price_{entry}$ + Fees.
  * Place Limit Orders at Break-Even Price.
* **Note:** This is *not* a Market Close. It protects capital without taking a loss.

## 4. Strategic Bias

**Controls:** `Long Only` | `Neutral` | `Short Only`
**Backend Logic:**
The Legislative Layer wraps the OODA Decision.

* **If Long Only:** `Action::Sell` is mutated to `Action::Hold`.
* **If Short Only:** `Action::Buy` is mutated to `Action::Hold`.
* **Closing Positions:** Closing a Long is a Sell *action*, but contextually defined as "Reduce". (Current implementation blocks *Opening* new distinct directions, need to verify if distinct logic separates "Close" from "Short"). *Protocol Note: Current V1 implementation simply Vetoes the verb. Pilot must switch to Neutral to close positions manually if automation fails.*
