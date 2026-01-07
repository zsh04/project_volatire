import sys
import os
import logging

# Add src to path (assuming CWD is src/brain)
sys.path.append(os.path.abspath("src"))

from hypatia.memory import MemoryEngine, ProceduralKB
import time

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("VerifyMemory")


def verify():
    logger.info("🧠 Initializing MemoryEngine...")
    # Use default model (DistilBERT ~ 768 dim) to match schema
    engine = MemoryEngine(db_path="~/.active_memory/audit_test")

    if not engine.connected:
        logger.error("Failed to connect.")
        return

    # 1. Test Procedural Memory (SOPs)
    logger.info("📘 Seeding Procedural Memory (SOPs)...")
    sops = [
        {
            "timestamp": int(time.time()),
            "regime": "LAMINAR_BULL",
            "narrative": "Buy the dip when RSP > 50 and VIX < 15. Hold trend.",
            "venue": "ALL",
            "outcome": "PROFIT",
        },
        {
            "timestamp": int(time.time()),
            "regime": "TURBULENT_BEAR",
            "narrative": "Short rallies. Tight stops. Avoid Kraken during outages.",
            "venue": "KRAKEN",
            "outcome": "PROFIT",
        },
    ]

    # Manually add to table (since we didn't add bulk add method yet, or just iterate)
    # Actually we didn't add a method to add procedural in MemoryEngine, let's use the table directly or add it.
    # Wait, I didn't add add_procedural to MemoryEngine. I should fix that or use table.

    # Adding via table directly for now to test logic, but I should add the method.
    for sop in sops:
        vec = engine.model.encode(sop["narrative"])
        entry = {
            "timestamp": sop["timestamp"],
            "vector": vec,
            "regime": sop["regime"],
            "narrative": sop["narrative"],
            "venue": sop["venue"],
            "outcome": sop["outcome"],
        }
        engine.procedural_table.add([entry])

    logger.info("✅ SOPs Seeded.")

    # 2. Test Retrieval with Filter
    logger.info("🔍 Testing Procedural Search...")

    # Case A: Generic Search
    regime, dist = engine.find_nearest_regime("Low volatility trending up", venue="ALL")
    logger.info(f"   > Search 'Low vol': {regime} (Dist: {dist:.4f})")
    assert "LAMINAR_BULL" in regime, f"Expected LAMINAR_BULL, got {regime}"

    # Case B: Venue Specific
    regime, dist = engine.find_nearest_regime("High volatility crash", venue="KRAKEN")
    logger.info(f"   > Search 'High vol' @ KRAKEN: {regime} (Dist: {dist:.4f})")
    assert "TURBULENT_BEAR" in regime, f"Expected TURBULENT_BEAR, got {regime}"

    # 3. Test Episodic Memory
    logger.info("📸 Testing Episodic Checkpoint...")
    checkpoint = {
        "type": "TRADE",
        "timestamp": int(time.time() * 1000),
        "payload": '{"symbol": "BTC/USD", "size": 1.0}',
        "outcome": "PROFIT",
        "venue": "KRAKEN",
        "vector_text": "Executed BUY BTC/USD on KRAKEN during dip.",
    }
    engine.add_episodic(checkpoint)

    # Verify it exists
    count = len(engine.episodic_table.to_pandas())
    logger.info(f"   > Episodic Count: {count}")
    assert count > 0, "Episodic memory not saved."

    logger.info("🎉 Memory Verification PASSED.")


if __name__ == "__main__":
    verify()
