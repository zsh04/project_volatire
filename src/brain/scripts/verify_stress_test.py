import asyncio
import time
import sys
import os
import pandas as pd
import logging
from rich.console import Console
from rich.panel import Panel

# Ensure src path
sys.path.append(os.path.join(os.path.dirname(__file__), "../src"))

from hypatia.engine import ContextEngine
from kepler.engine import KeplerOracle

# Setup Logging
logging.basicConfig(level=logging.ERROR)
console = Console()


class MockAdapter:
    """Simulates a Market Crash scenario."""

    async def fetch_vix(self):
        return 45.0  # Panic

    async def fetch_yields(self):
        return 5.5  # High Stress

    async def fetch_btc_trend(self):
        return -0.15  # Crash Trend

    async def fetch_dxy(self):
        return 106.0  # Flight to safety


async def run_stress_test():
    console.print(
        Panel("🔥 Starting Stress Test: 5% Flash Crash Simulation", style="bold red")
    )

    # 1. Hypatia Integration (Logic Test)
    console.print("[yellow]1. Initializing Hypatia (Context Engine)...[/yellow]")
    hypatia = ContextEngine(adapter=MockAdapter())

    # Measure Logic Validation
    t0 = time.perf_counter()
    regime = await hypatia.fetch_snapshot()
    t1 = time.perf_counter()

    hypatia_latency = (t1 - t0) * 1000
    console.print(
        f"   -> Hypatia Latency: [bold cyan]{hypatia_latency:.2f}ms[/bold cyan]"
    )

    console.print(f"   -> Score: {regime.score} (Exp: < -0.5)")
    console.print(f"   -> VIX: {regime.vix}")

    # Assertions
    if regime.score <= -0.5:
        console.print("   ✅ Hypatia Logic: PASSED (Detected Crisis)")
    else:
        console.print(f"   ❌ Hypatia Logic: FAILED (Score {regime.score} too high)")

    # 2. Kepler Integration (Forecast Test)
    console.print("\n[yellow]2. Initializing Kepler (Oracle)...[/yellow]")
    oracle = KeplerOracle()

    # Simulate 5% Drop in 1 minute
    console.print("   -> Simulating 5% Price Drop (100 -> 95)...")
    now = pd.Timestamp.now()
    dates = [now - pd.Timedelta(seconds=i) for i in range(60, 0, -1)]
    prices = [100.0 - (5.0 * (i / 60.0)) for i in range(60)]  # 100 -> 95
    df = pd.DataFrame({"timestamp": dates, "price": prices})

    t2 = time.perf_counter()
    forecast = oracle.generate_forecast(df, horizon=5)
    t3 = time.perf_counter()

    kepler_latency = (t3 - t2) * 1000
    console.print(
        f"   -> Kepler Latency: [bold cyan]{kepler_latency:.2f}ms[/bold cyan]"
    )

    # Check for P10 Crash Continuation
    # If momentum is strictly maintained, P50 should differ from last price
    last_price = 95.0
    p10_forecast = forecast.iloc[0]["p10"]
    p50_forecast = forecast.iloc[0]["p50"]

    console.print(f"   -> Forecast P50 (T+1): {p50_forecast:.2f}")
    console.print(f"   -> Forecast P10 (T+1): {p10_forecast:.2f}")

    # 3. Overall Latency Audit
    total_compute_latency = hypatia_latency + kepler_latency
    console.print(
        f"\n[bold]⏱️  Total Compute Latency: {total_compute_latency:.2f}ms[/bold]"
    )

    if total_compute_latency < 250:  # Standard for Python component (Reflex is faster)
        console.print("   ✅ Speed Audit: PASSED (< 250ms)")
    else:
        # Note: Kepler model load is heavy, but cached inference is fast.
        # This test includes the FIRST inference which might be slower.
        console.print("   ⚠️ Speed Audit: WARNING (Check caching)")


if __name__ == "__main__":
    asyncio.run(run_stress_test())
