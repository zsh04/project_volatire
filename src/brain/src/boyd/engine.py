import logging
import pandas as pd
from typing import Dict, Any, Optional
from .structure import TradeDecision

logger = logging.getLogger(__name__)


class BoydStrategist:
    """
    The OODA Loop Core.
    Synthesizes inputs from Feynman, Simons, Hypatia, and Kepler to make trading decisions.
    """

    def __init__(self):
        self.market_state = None

    def decide(
        self,
        market_data: Optional[Dict[str, Any]],  # Feynman Snapshot
        valuation: Optional[Dict[str, Any]],  # Simons Valuation
        regime: Optional[Dict[str, Any]],  # Hypatia Regime
        forecast: Optional[pd.DataFrame],  # Kepler Forecast (P10-P90)
    ) -> TradeDecision:
        """
        Executes the OODA Loop Strategy Logic.
        """

        # 0. Default Safe Decision
        decision = TradeDecision(
            action="HOLD",
            confidence=0.0,
            reason="Initializing / Insufficient Data",
            meta={},
        )

        try:
            # --- 1. HYPATIA VETO ---
            # If we are in a crash or extreme volatility, we DO NOT trade.
            if regime:
                current_regime = regime.get("regime", "UNKNOWN")
                decision.meta["regime"] = current_regime

                if current_regime in ["CRASH", "EXTREME_VOLATILITY"]:
                    decision.action = "HOLD"
                    decision.confidence = 1.0
                    decision.reason = f"Hypatia Veto: Active Regime is {current_regime}"
                    return decision

            # Ensure we have minimum viable inputs for active trading
            if not valuation or forecast is None or forecast.empty:
                decision.reason = "Missing input data (Valuation or Forecast)"
                return decision

            # --- 2. KEPLER SKEW ---
            # Skew = (Right Tail) - (Left Tail)
            # Right Tail = P90 - P50
            # Left Tail  = P50 - P10

            # We take the average skew over the forecast horizon to be robust
            p90 = forecast["p90"].mean()
            p50 = forecast["p50"].mean()
            p10 = forecast["p10"].mean()

            right_tail = p90 - p50
            left_tail = p50 - p10

            skew = right_tail - left_tail
            decision.meta["skew"] = skew

            # Positive Skew => Probabilistic mass is shifting upwards (Upside potential)
            # Negative Skew => Probabilistic mass is shifting downwards (High downside risk)

            # --- 3. SIMONS VALUATION ---
            fair_value = valuation.get("fair_value", 0.0)
            current_price = valuation.get("price", 0.0)

            if current_price <= 0:
                decision.reason = "Invalid Price Data"
                return decision

            deviation = (current_price - fair_value) / current_price
            decision.meta["valuation_deviation"] = deviation

            # deviation > 0 => Overvalued (Price > Fair)
            # deviation < 0 => Undervalued (Price < Fair)

            # --- 4. CONFLUENCE CHECK ---

            CONFIDENCE_THRESHOLD = 0.6

            # LONG SCENARIO
            # 1. Undervalued (Price < Fair Value)
            # 2. Positive Skew (Forecast predicts upside tail)
            if deviation < -0.005 and skew > 0:
                decision.action = "LONG"
                decision.confidence = 0.8
                decision.reason = "Golden Confluence: Undervalued + Bullish Skew"

            # SHORT SCENARIO
            # 1. Overvalued (Price > Fair Value)
            # 2. Negative Skew (Forecast predicts downside tail)
            elif deviation > 0.005 and skew < 0:
                decision.action = "SHORT"
                decision.confidence = 0.8
                decision.reason = "Golden Confluence: Overvalued + Bearish Skew"

            else:
                decision.action = "HOLD"
                decision.confidence = 0.5
                decision.reason = "No Confluence: Validation & Forecast Disagree"

        except Exception as e:
            logger.error(f"Boyd Error: {e}")
            decision.action = "HOLD"
            decision.reason = f"Internal Error: {str(e)}"

        return decision
