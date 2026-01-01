import sys
import unittest
import pandas as pd
import os

# Ensure src/brain/src is in path
sys.path.append(os.path.join(os.path.dirname(__file__), "../src"))

from boyd.engine import BoydStrategist


class TestBoydStrategist(unittest.TestCase):
    def setUp(self):
        self.strategist = BoydStrategist()
        self.mock_forecast = pd.DataFrame(
            {"p10": [100] * 10, "p50": [105] * 10, "p90": [115] * 10}
        )
        # Base skew logic from engine:
        # P90(115) - P50(105) = 10
        # P50(105) - P10(100) = 5
        # Skew = 10 - 5 = +5 (Positive Skew)

    def test_hypatia_veto(self):
        """Scenario A: Hypatia Veto should force HOLD."""
        regime = {"regime": "CRASH"}
        valuation = {"price": 100, "fair_value": 110}  # Undervalued (Should be Long)

        decision = self.strategist.decide(
            market_data={},
            valuation=valuation,
            regime=regime,
            forecast=self.mock_forecast,
        )

        print(f"\n[Test A] Veto Check: {decision.reason}")
        self.assertEqual(decision.action, "HOLD")
        self.assertIn("Hypatia Veto", decision.reason)

    def test_long_confluence(self):
        """Scenario B: Undervalued + Positive Skew -> LONG"""
        regime = {"regime": "BULL_TREND"}
        valuation = {
            "price": 100,
            "fair_value": 101,
        }  # Undervalued by > 0.5% (100 < 101)
        # Mock forecast has Positive Skew (+5)

        decision = self.strategist.decide(
            market_data={},
            valuation=valuation,
            regime=regime,
            forecast=self.mock_forecast,
        )

        print(f"\n[Test B] Long Check: {decision.reason}")
        self.assertEqual(decision.action, "LONG")
        self.assertEqual(decision.confidence, 0.8)

    def test_short_confluence(self):
        """Scenario C: Overvalued + Negative Skew -> SHORT"""
        regime = {"regime": "BEAR_TREND"}
        valuation = {"price": 100, "fair_value": 90}  # Overvalued

        # create negative skew forecast
        # P90-P50 = 5
        # P50-P10 = 10
        # Skew = 5 - 10 = -5
        neg_skew_forecast = pd.DataFrame(
            {"p10": [90] * 10, "p50": [100] * 10, "p90": [105] * 10}
        )

        decision = self.strategist.decide(
            market_data={},
            valuation=valuation,
            regime=regime,
            forecast=neg_skew_forecast,
        )

        print(f"\n[Test C] Short Check: {decision.reason}")
        self.assertEqual(decision.action, "SHORT")

    def test_conflict_scenario(self):
        """Scenario D: Undervalued (Bullish) but Negative Skew (Bearish) -> HOLD"""
        regime = {"regime": "SIDEWAYS"}
        valuation = {"price": 100, "fair_value": 110}  # Undervalued (Bullish Signal)

        # create negative skew forecast (Bearish Signal)
        neg_skew_forecast = pd.DataFrame(
            {"p10": [90] * 10, "p50": [100] * 10, "p90": [105] * 10}
        )

        decision = self.strategist.decide(
            market_data={},
            valuation=valuation,
            regime=regime,
            forecast=neg_skew_forecast,
        )

        print(f"\n[Test D] Conflict Check: {decision.reason}")
        self.assertEqual(decision.action, "HOLD")
        self.assertIn("No Confluence", decision.reason)


if __name__ == "__main__":
    unittest.main()
