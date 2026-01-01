import asyncio
import sys
import os

# Adjust path to include src/brain/src
current_dir = os.path.dirname(os.path.abspath(__file__))
src_dir = os.path.abspath(os.path.join(current_dir, "../src"))
sys.path.append(src_dir)

from hypatia.engine import ContextEngine
from rich.console import Console
from rich.panel import Panel

console = Console()


async def main():
    console.print("[bold cyan]🔮 HYPATIA: Initializing Context Engine...[/bold cyan]")

    engine = ContextEngine()

    try:
        regime = await engine.fetch_snapshot()

        console.print(
            Panel(
                f"Timestamp: [yellow]{regime.timestamp}[/yellow]\n"
                f"Score: [bold {'green' if regime.score > 0 else 'red'}]{regime.score:.2f}[/]\n"
                f"VIX: {regime.vix:.2f}\n"
                f"Yields: {regime.yield_10y:.2f}%\n"
                f"BTC Trend: {regime.btc_trend_score:.2%}\n"
                f"Leverage Cap: {regime.leverage_scalar:.1f}x",
                title="Market Regime Snapshot",
                subtitle=f"Risk On: {regime.is_risk_on}",
            )
        )

    except Exception as e:
        console.print_exception()


if __name__ == "__main__":
    asyncio.run(main())
