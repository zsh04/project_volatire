import json
import pandas as pd
from pathlib import Path
from datetime import datetime

LOG_PATH = Path("logs/hallucinations.jsonl")


def analyze_hallucinations():
    if not LOG_PATH.exists():
        print("✅ No hallucinations found. Brain is healthy.")
        return

    print(f"🔬 Starting Biopsy on {LOG_PATH}...")

    data = []
    with open(LOG_PATH, "r") as f:
        for line in f:
            try:
                data.append(json.loads(line))
            except Exception:
                continue

    if not data:
        print("✅ Log file empty.")
        return

    df = pd.DataFrame(data)

    print(f"⚠️ Found {len(df)} Nullified Packets.")

    # Simple Analysis: Group by Error Type
    error_counts = df["error"].value_counts()
    print("\n📊 Hallucination Distribution:")
    print(error_counts)

    # Suggestion Logic
    print("\n🧠 PROMPT EVOLUTION ENGINE SUGGESTIONS:")

    # Check for Numeric Hallucinations
    numeric_errors = df[
        df["error"].astype(str).str.contains("NumericHallucination", na=False)
    ]
    if not numeric_errors.empty:
        print(f"  - 🚨 {len(numeric_errors)} Numeric Hallucinations detected.")
        print("    -> ACTION: Decrease Temperature in grounding.j2.")
        print(
            "    -> ACTION: Reinforce 'referenced_price' constraint in System Prompt."
        )

    # Check for Regime Mismatch
    regime_errors = df[df["error"].astype(str).str.contains("RegimeMismatch", na=False)]
    if not regime_errors.empty:
        print(f"  - 🚨 {len(regime_errors)} Regime Mismatches detected.")
        print("    -> ACTION: Provide more explicit Regime definitions in prompt.")

    print("\n✅ Biopsy Complete. Report generated.")


if __name__ == "__main__":
    analyze_hallucinations()
