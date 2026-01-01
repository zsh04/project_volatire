import sys
import os
import pandas as pd
import numpy as np

# ensure src/brain/src is in path
sys.path.append(os.path.join(os.path.dirname(__file__), "../src"))

from kepler.engine import KeplerOracle


def test_kepler_oracle():
    print("🔮 Testing Kepler Oracle...")

    # 1. Instantiate (Should warn about Mock Mode)
    oracle = KeplerOracle()
    assert oracle is not None

    # 2. Create Dummy Data
    dates = pd.date_range(end=pd.Timestamp.now(), periods=50, freq="1min")
    df = pd.DataFrame({"timestamp": dates, "price": [100 + i * 0.1 for i in range(50)]})

    # 3. Generate Forecast
    print("\n📊 Generating Forecast (Horizon=10)...")
    forecast = oracle.generate_forecast(df, horizon=10)

    # 4. Validate Structure
    print("Result Head:\n", forecast.head(3))

    required_cols = ["timestamp"] + [f"p{i}0" for i in range(1, 10)]
    for col in required_cols:
        assert col in forecast.columns, f"Missing column {col}"

    print(f"✅ Column Check Passed: {required_cols}")

    # 5. Validate Logic (P90 > P50 > P10)
    row = forecast.iloc[0]
    assert row["p90"] > row["p50"] > row["p10"], "Quantiles are not ordered!"
    print("✅ Quantile Logic Passed (Fan Chart verified)")

    # 6. Validate Cache
    print("\n🕒 Testing Cache...")
    t1 = pd.Timestamp.now()
    _ = oracle.generate_forecast(df)  # Should be instant
    elapsed = (pd.Timestamp.now() - t1).total_seconds()

    assert elapsed < 0.1, f"Cache too slow! {elapsed}s"
    print(f"✅ Cache Hit confirmed ({elapsed:.4f}s)")

    print("\n✨ Kepler Oracle Verification COMPLETE.")


if __name__ == "__main__":
    test_kepler_oracle()
