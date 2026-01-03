import os
import sys
import json
import logging
import pandas as pd
import lancedb
from lancedb.pydantic import LanceModel, Vector
from sentence_transformers import SentenceTransformer
from datetime import datetime

# Setup Logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("ReflectionLoop")

# --- Configuration ---
DB_PATH = os.path.expanduser("~/.active_memory/voltaire")
LESSON_TABLE = "voltaire_lessons"
# Using the same model as D-37/38 for consistency
MODEL_NAME = "distilbert-base-nli-stsb-mean-tokens"


# --- Schema ---
class TradeLesson(LanceModel):
    timestamp_ns: int  # Nano-precision timestamp of the trade
    symbol: str
    pnl: float
    regime: str
    mistake_type: str  # e.g., "Sentiment Divergence", "Physics Violation"
    lesson_text: str  # Natural language summary
    vector: Vector(768)


# --- Logic ---


class Reflector:
    def __init__(self):
        logger.info("🪞 Initializing Reflection Engine...")
        self.model = SentenceTransformer(MODEL_NAME)
        self.db = lancedb.connect(DB_PATH)

    def analyze_logs(self, log_path):
        logger.info(f"📖 Reading Trade Logs: {log_path}")

        try:
            with open(log_path, "r") as f:
                logs = json.load(f)
        except Exception as e:
            logger.error(f"❌ Failed to load logs: {e}")
            return []

        losses = [trade for trade in logs if trade["pnl"] < 0]
        logger.info(f"🔍 Found {len(losses)} losses out of {len(logs)} trades.")

        lessons = []
        for trade in losses:
            lesson = self.critique(trade)
            if lesson:
                lessons.append(lesson)

        return lessons

    def critique(self, trade):
        """
        Attribution Analysis: Why did we lose?
        """
        reasons = []

        # 1. Check Physics (VVIX)
        # Using mock data context for the purpose of the script,
        # in prod this would fetch from R2/QuestDB based on trade.timestamp
        vvix = trade.get("context", {}).get("vvix", 0)
        if vvix > 150:
            reasons.append(
                "Extreme Volatility (VVIX > 150). System should have been in Bunker Mode."
            )

        # 2. Check Sentiment
        sentiment_score = trade.get("context", {}).get("sentiment", 0)
        side = trade["side"]
        if side == "BUY" and sentiment_score < 0:
            reasons.append("Sentiment Divergence (Longed into Bad News).")
        elif side == "SELL" and sentiment_score > 0:
            reasons.append("Sentiment Divergence (Shorted into Good News).")

        # 3. Check Physics (Velocity)
        velocity = trade.get("context", {}).get("velocity", 0)
        if side == "BUY" and velocity < -2.0:
            reasons.append("Kinematics Violation (Catching a Falling Knife).")

        if not reasons:
            return None  # Random noise loss, no structural lesson

        reason_str = " + ".join(reasons)
        lesson_text = (
            f"LOSS on {trade['symbol']}. REASON: {reason_str}. ACTION: Reduce Beta."
        )

        return {
            "timestamp_ns": trade["timestamp_ns"],
            "symbol": trade["symbol"],
            "pnl": trade["pnl"],
            "regime": trade.get("context", {}).get("regime", "Unknown"),
            "mistake_type": "Structural Fault"
            if len(reasons) > 1
            else "Single Failure",
            "lesson_text": lesson_text,
        }

    def store_lessons(self, lessons):
        if not lessons:
            logger.info(
                "✅ No structural lessons found. Losses were within noise tolerance."
            )
            return

        logger.info(f"🧠 Encoding {len(lessons)} lessons for long-term memory...")

        # Vectorize
        texts = [l["lesson_text"] for l in lessons]
        vectors = self.model.encode(texts)

        data = []
        for i, l in enumerate(lessons):
            l["vector"] = vectors[i]
            data.append(l)

        # Upsert
        logger.info(f"💾 Upserting to {LESSON_TABLE}...")

        # Create table if not exists
        if LESSON_TABLE in self.db.table_names():
            tbl = self.db.open_table(LESSON_TABLE)
            tbl.add(data)
        else:
            tbl = self.db.create_table(LESSON_TABLE, schema=TradeLesson)
            tbl.add(data)

        logger.info(f"✅ Stored {len(data)} lessons.")

    def generate_report(self, lessons):
        report_path = "reflection_report.md"
        with open(report_path, "w") as f:
            f.write("# 🪞 Hypatia's Reflection Report\n\n")
            f.write(f"**Date:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
            f.write(f"**Lessons Learned:** {len(lessons)}\n\n")

            for l in lessons:
                f.write(f"### Trade: {l['symbol']}\n")
                f.write(f"- **PnL:** ${l['pnl']:.2f}\n")
                f.write(f"- **Critique:** {l['lesson_text']}\n")
                f.write(f"- **Regime:** {l['regime']}\n")
                f.write("---\n")
        logger.info(f"📄 Report generated: {report_path}")


if __name__ == "__main__":
    # Dummy Data Generation for Verification
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("--dummy", action="store_true", help="Run with dummy data")
    args = parser.parse_args()

    reflector = Reflector()

    if args.dummy:
        logger.info("🧪 Running in MOCK Mode...")
        dummy_logs = [
            {
                "timestamp_ns": 1678800000000000000,
                "symbol": "BTC-USD",
                "side": "BUY",
                "pnl": -500.0,
                "context": {
                    "vvix": 160,
                    "sentiment": -1,
                    "velocity": -5.0,
                    "regime": "Extreme Panic",
                },
            },
            {
                "timestamp_ns": 1678800000000000000,
                "symbol": "ETH-USD",
                "side": "BUY",
                "pnl": 150.0,
                "context": {"vvix": 160, "sentiment": -1},
            },
            {
                "timestamp_ns": 1678800000000000000,
                "symbol": "SPY",
                "side": "BUY",
                "pnl": -200.0,
                "context": {
                    "vvix": 80,
                    "sentiment": -1,
                    "velocity": 0.5,
                    "regime": "Normal Bull",
                },
            },
        ]

        with open("sim_trades.json", "w") as f:
            json.dump(dummy_logs, f)

        lessons = reflector.analyze_logs("sim_trades.json")
        reflector.store_lessons(lessons)
        reflector.generate_report(lessons)

        # Cleanup
        if os.path.exists("sim_trades.json"):
            os.remove("sim_trades.json")
    else:
        # Prod mode: expect real file
        if not os.path.exists("sim_trades.json"):
            logger.error("❌ No sim_trades.json found.")
        else:
            lessons = reflector.analyze_logs("sim_trades.json")
            reflector.store_lessons(lessons)
            reflector.generate_report(lessons)
