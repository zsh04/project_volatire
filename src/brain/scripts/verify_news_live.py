import sys
import asyncio
import os
import logging
from rich.console import Console

# Adjust path to find src
sys.path.append("src")
from brain.src.hypatia.news import NewsEngine

# Setup Logging
logging.basicConfig(level=logging.DEBUG)
console = Console()


async def main():
    console.print("[bold cyan]🚀 Starting Live News Verification...[/bold cyan]")

    engine = NewsEngine()
    # Force DEBUG on the engine logger explicitly if needed, but basicConfig coverage should suffice
    logging.getLogger("brain.src.hypatia.news").setLevel(logging.DEBUG)

    if not engine.api_key:
        console.print("[bold red]❌ No API Key found in Environment![/bold red]")
        return

    console.print(f"[yellow]🔑 API Key Prefix: {engine.api_key[:5]}...[/yellow]")

    console.print("[cyan]📡 Fetching Headlines (CryptoPanic + RSS)...[/cyan]")
    headlines = await engine._ingest()

    if not headlines:
        console.print("[bold red]❌ No headlines found! Check Network/Key.[/bold red]")
    else:
        console.print(
            f"[bold green]✅ Success! Fetched {len(headlines)} headlines.[/bold green]"
        )
        console.print("\n[bold white]Top 5 Headlines:[/bold white]")
        for i, (title, source) in enumerate(headlines[:5]):
            console.print(f"{i + 1}. [{source}] {title}")

        # Check for Panic in real data
        console.print("\n[bold white]Running Sieve & Score on Real Data:[/bold white]")

        # Inject a test case
        headlines.insert(
            0, ("CRASH: Bitcoin dumps 50% due to SEC ban", "TEST_INJECTED")
        )

        for title, source in headlines[:6]:
            params = engine._sieve(title)
            # Force score logging for verification even if not urgent
            score, is_panic = engine._score(title, params)

            color = "red" if is_panic else "green"
            console.print(
                f"[{color}]{title}[/{color}]\n   -> Sieve: {params} | Score: {score:.4f} | Panic: {is_panic}"
            )


if __name__ == "__main__":
    asyncio.run(main())
