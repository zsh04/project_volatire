import asyncio
import sys
import os

# Add src/brain to sys.path to resolve 'src.' imports if needed
# Assuming structure: voltaire/src/brain/src/macro/fetcher.py
# If running from voltaire/
sys.path.append(os.path.abspath("src/brain"))

from src.macro.fetcher import MacroFetcher


async def main():
    print("🧪 Verifying Macro Fetcher...")

    # Mock Env for safety validation (defaults)
    # real keys might be in real .env loaded by main app

    fetcher = MacroFetcher()

    # 1. Fetch State (should use defaults if keys missing)
    state = await fetcher.fetch_macro_state()

    print("\n📊 Resulting State:")
    print(f"  RFR: {state.risk_free_rate}")
    print(f"  Stress: {state.systemic_stress}")
    print(f"  Sentiment: {state.economic_sentiment}")
    print(f"  Regime: {state.regime_bias} (0=Expansion)")

    # 2. Check Cache creation
    if os.path.exists("macro_cache.json"):
        print("\n✅ Cache file created.")
    else:
        print("\n❌ Cache file NOT created.")


if __name__ == "__main__":
    asyncio.run(main())
