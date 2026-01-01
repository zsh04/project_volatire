import sys
import asyncio
import unittest
from datetime import datetime

# Adjust path
sys.path.append("src")
from brain.src.hypatia.news import NewsEngine


class TestNewsEngine(unittest.TestCase):
    def setUp(self):
        self.engine = NewsEngine()

    def test_sieve(self):
        """Verify Keyword Filter"""
        boring = "Bitcoin stays stable at 50k"
        urgent = "EXPLOIT detected in DeFi Protocol"
        macro = "FED likely to Pause Rate Hikes"

        r1 = self.engine._sieve(boring)
        r2 = self.engine._sieve(urgent)
        r3 = self.engine._sieve(macro)

        self.assertFalse(r1["is_urgent"] or r1["is_macro"])
        self.assertTrue(r2["is_urgent"])
        self.assertTrue(r3["is_macro"])

    def test_heuristic_score(self):
        """Verify Heuristic Scoring"""
        # URGENT + BEARISH -> PANIC
        headline = "Exchange HALTS withdrawals immediately after HACK"
        params = {"is_urgent": True, "is_macro": False}

        score, is_panic = self.engine._score(headline, params)
        print(f"\nHeadline: {headline} -> Score: {score}, Panic: {is_panic}")

        self.assertTrue(is_panic)
        self.assertLess(score, 0.0)

    def test_deduplication(self):
        t = "Same Headline"
        self.assertFalse(self.engine._is_duplicate(t))
        self.assertTrue(self.engine._is_duplicate(t))


if __name__ == "__main__":
    unittest.main()
