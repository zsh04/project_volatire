import os
import sys
import logging

# Force LIVE environment to test safety
os.environ["VOLTAIRE_ENV"] = "LIVE"

# Add src path
sys.path.append(os.path.join(os.path.dirname(__file__), "../src"))

try:
    from kepler.engine import KeplerOracle

    print("🔮 Initializing KeplerOracle in LIVE mode...")
    oracle = KeplerOracle()

    if oracle._mock_mode:
        print("❌ FAILURE: Oracle is in MOCK MODE despite ENV=LIVE!")
        sys.exit(1)
    else:
        print(
            "✅ SUCCESS: Oracle loaded in REAL mode (or crashed successfully if deps missing)."
        )

except ImportError as e:
    # If it crashes due to active refusal to mock, that is ALSO a success for this test
    # BUT if we actually want to verify deps are present, this is a failure of the environment.
    # The user asked: "verify brain env". So we want it to work.
    print(
        f"⚠️ Dependency Missing (Expected if env is incomplete, but Safety Check worked): {e}"
    )
    # If we are verifying the environment is READY, this is a fail.
    # If we are verifying the CODE is SAFE, this is a pass.
    # Let's assume we want it to work.
    sys.exit(1)

except Exception as e:
    if "PANIC" in str(e) or "Fail fast" in str(e):
        print(f"✅ SAFETY CHECK PASSED: System panicked as expected in LIVE mode: {e}")
        sys.exit(0)
    else:
        print(f"💥 Unexpected Error: {e}")
        sys.exit(1)
