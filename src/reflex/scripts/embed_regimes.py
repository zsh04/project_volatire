import os
import sys
import logging
import yfinance as yf
import pandas as pd
import numpy as np
import lancedb
from lancedb.pydantic import LanceModel, Vector
from lancedb.embeddings import get_registry
from sentence_transformers import SentenceTransformer
from dotenv import load_dotenv

# Setup Logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("EmbeddingEngine")

load_dotenv()

# --- Configuration ---
DB_PATH = os.path.expanduser("~/.active_memory/voltaire")
TABLE_NAME = "voltaire_memories"
# We use a 768-dim model as requested
MODEL_NAME = "distilbert-base-nli-stsb-mean-tokens"

# --- Narrative Database ---
NARRATIVES = {
    "2008-09-15": "Lehman Brothers Bankruptcy. Global Financial Crisis. Extreme Systemic Fear.",
    "2008-09-29": "Stock Market Crash. Congress rejects bailout. Liquidity Crisis.",
    "2010-05-06": "Flash Crash. High Frequency Trading anomaly. Liquidity Evaporation.",
    "2011-08-08": "US Credit Downgrade. European Debt Crisis. High Volatility.",
    "2015-08-24": "China Devaluation Shock. Flash Crash. Global Growth Fears.",
    "2018-02-05": "Volmageddon. VIX ETP Implosion. XIV Termination.",
    "2018-12-24": "Christmas Eve Massacre. Fed Tightening Fears. Bear Market.",
    "2020-03-09": "Covid Crash. Oil Price War. Circuit Breaker Triggered.",
    "2020-03-12": "Black Thursday. Systemic Liquidity Failure. Cash Dash.",
    "2020-03-16": "Covid Panic. Fed Emergency Rate Cut. Market Meltdown.",
    "2020-03-23": "Fed Unlimited QE Announcement. The Bottom.",
    "2021-01-27": "GameStop Short Squeeze. Retail Maniacal Euphoria.",
    "2022-02-24": "Russia Invades Ukraine. Geopolitical Shock. Commodity Spike.",
    "2022-06-13": "CPI Shock. Aggressive Fed Hikes. Bear Market Lows.",
    "2022-11-11": "FTX Collapse. Crypto Contagion. Fraud Uncovered.",
    "2023-03-10": "SVB Bank Run. Regional Banking Crisis. Flight to Safety.",
}


# --- Schema ---
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


# --- Logic ---


def compute_kinematics(df, period=14):
    """
    Computes Velocity, Acceleration, Jerk on Log Returns.
    """
    df["log_ret"] = np.log(df["Close"] / df["Close"].shift(1))
    df["velocity"] = df["log_ret"].rolling(window=period).mean() * 100
    df["acceleration"] = df["velocity"].diff()
    df["jerk"] = df["acceleration"].diff()
    return df


def get_regime_label(row):
    """
    Determine regime label based on Volatility (VVIX) and Trend (Velocity).
    """
    vvix = row["VVIX"]
    vel = row["velocity"]

    if pd.isna(vvix) or pd.isna(vel):
        return "Unknown"

    if vvix > 150:
        return "Extreme Panic / Liquidity Crisis"
    elif vvix > 110:
        if vel < -0.5:
            return "Crash Dynamics"
        return "High Volatility / Correction"
    elif vvix < 85:
        if vel > 0:
            return "Low Vol Bull Grind"
        return "Complacency"
    else:
        if vel > 0:
            return "Normal Bull"
        return "Normal Bear"


def semantic_string_builder(date, row_dict):
    """
    Construct the semantic prompt.
    """
    date_str = date.strftime("%Y-%m-%d")
    narrative = NARRATIVES.get(date_str, "Standard Market Conditions.")

    prompt = (
        f"Date: {date_str}. "
        f"Market Regime: {row_dict['regime']}. "
        f"Narrative: {narrative}. "
        f"Velocity: {row_dict['velocity']:.2f}. "
        f"Acceleration: {row_dict['acceleration']:.4f}. "
        f"Jerk: {row_dict['jerk']:.4f}. "
        f"VVIX: {row_dict['VVIX']:.2f}."
    )
    return prompt, narrative


def embed_history():
    logger.info("🧠 Initializing Embedding Engine...")

    logger.info(f"🤖 Loading Model: {MODEL_NAME}...")
    model = SentenceTransformer(MODEL_NAME)

    logger.info("📉 Fetching Market Data (GSPC + VVIX)...")
    spx = yf.Ticker("^GSPC").history(start="2007-01-01")
    vvix = yf.Ticker("^VVIX").history(start="2007-01-01")

    df = pd.DataFrame(index=spx.index)
    df["Close"] = spx["Close"]
    df["VVIX"] = vvix["Close"]
    df = df.dropna()

    logger.info("🔭 Computing Feynman Kinematics...")
    df = compute_kinematics(df)
    df = df.dropna()

    memories = []

    logger.info(f"📝 Synthesizing {len(df)} memories...")

    for date, row in df.iterrows():
        regime = get_regime_label(row)
        row_dict = row.to_dict()
        row_dict["regime"] = regime

        text, narrative = semantic_string_builder(date, row_dict)

        memories.append(
            {
                "timestamp": date.strftime("%Y-%m-%d"),
                "regime": regime,
                "velocity": float(row["velocity"]),
                "acceleration": float(row["acceleration"]),
                "jerk": float(row["jerk"]),
                "vvix": float(row["VVIX"]),
                "narrative": narrative,
                "text": text,
            }
        )

    logger.info("⚡ Vectorizing text (Inference)...")
    texts = [m["text"] for m in memories]
    embeddings = model.encode(texts, show_progress_bar=True)

    for i, m in enumerate(memories):
        m["vector"] = embeddings[i]

    logger.info(f"💾 Upserting to LanceDB: {DB_PATH}")
    os.makedirs(DB_PATH, exist_ok=True)
    db = lancedb.connect(DB_PATH)

    tbl = db.create_table(TABLE_NAME, schema=MarketMemory768, mode="overwrite")
    tbl.add(memories)

    logger.info(f"✅ Success. Indexed {len(memories)} memories.")


if __name__ == "__main__":
    try:
        embed_history()
    except Exception as e:
        logger.error(f"💥 Failed: {e}")
        sys.exit(1)
