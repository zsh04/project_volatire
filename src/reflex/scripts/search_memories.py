import os
import sys
import logging
import lancedb
from lancedb.pydantic import LanceModel, Vector
from sentence_transformers import SentenceTransformer

# Setup Logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("CortexSearch")

# --- Configuration ---
DB_PATH = os.path.expanduser("~/.active_memory/voltaire")
TABLE_NAME = "voltaire_memories"
MODEL_NAME = "distilbert-base-nli-stsb-mean-tokens"  # 768 dim


class MarketMemory768(LanceModel):
    timestamp: str
    vector: Vector(768)
    regime: str
    velocity: float
    acceleration: float
    jerk: float
    vvix: float
    narrative: str
    text: str


def query_memory(query_text, limit=5):
    logger.info(f"🔎 Searching for: '{query_text}'")

    # 1. Connect
    if not os.path.exists(DB_PATH):
        logger.error(f"❌ DB not found at {DB_PATH}. Run embed_regimes.py first.")
        return

    db = lancedb.connect(DB_PATH)
    if TABLE_NAME not in db.table_names():
        logger.error(f"❌ Table {TABLE_NAME} not found.")
        return

    tbl = db.open_table(TABLE_NAME)

    # 2. Embed Query
    logger.info("🧠 Vectorizing query...")
    model = SentenceTransformer(MODEL_NAME)
    query_vec = model.encode(query_text)

    # 3. Search
    results = tbl.search(query_vec).limit(limit).to_pandas()

    # 4. Display
    logger.info(f"✅ Found {len(results)} matches:\n")
    print(
        results[["timestamp", "regime", "narrative", "_distance"]].to_markdown(
            index=False
        )
    )

    # 5. Acceptance Check
    # Check for GFC (2008) or Covid (2020) if query implies crisis
    if "Crisis" in query_text or "Liquidity" in query_text:
        timestamps = results["timestamp"].tolist()
        has_2008 = any("2008" in t for t in timestamps)
        has_2020 = any("2020" in t for t in timestamps)

        if has_2008 or has_2020:
            logger.info("✅ Semantic Parity Test: PASSED (Found 2008/2020)")
        else:
            logger.error("❌ Semantic Parity Test: FAILED (Did not find 2008/2020)")


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--query", type=str, default="Systemic Liquidity Crisis", help="Search query"
    )
    args = parser.parse_args()

    try:
        query_memory(args.query)
    except Exception as e:
        logger.error(f"💥 Search failed: {e}")
        sys.exit(1)
