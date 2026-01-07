import os
import logging
import time
import lancedb
from lancedb.pydantic import LanceModel, Vector
from sentence_transformers import SentenceTransformer

logger = logging.getLogger("HypatiaMemory")


# D-102: Procedural Memory (Standard Operating Procedures)
class ProceduralKB(LanceModel):
    timestamp: int
    vector: Vector(768)
    regime: str
    narrative: str  # The SOP text
    venue: str  # "ALL" or specific exchange
    outcome: str  # "PROFIT", "LOSS" (Success rate of this SOP)


# D-102: Episodic Memory (The Black Box Trade Log)
class EpisodicEvent(LanceModel):
    timestamp: int
    vector: Vector(768)  # Vectorized Reasoning or Market Condition
    type: str  # "TRADE", "VETO", "HALT"
    payload: str  # JSON Dump of State
    outcome: str  # "PROFIT", "LOSS", "N/A"
    venue: str  # "KRAKEN"
    reasoning: str


class MemoryEngine:
    def __init__(
        self,
        db_path="~/.active_memory/voltaire",
        model_name="distilbert-base-nli-stsb-mean-tokens",
    ):
        self.db_path = os.path.expanduser(db_path)
        self.model_name = model_name
        self.connected = False
        self.db = None
        self.procedural_table = None
        self.episodic_table = None
        self.model = None

        self._connect()

    def _connect(self):
        try:
            if not os.path.exists(self.db_path):
                os.makedirs(self.db_path, exist_ok=True)
                logger.info(f"📁 Created Memory DB path at {self.db_path}")

            self.db = lancedb.connect(self.db_path)

            # Initialize Procedural Table
            if "voltaire_procedural" in self.db.table_names():
                self.procedural_table = self.db.open_table("voltaire_procedural")
            else:
                self.procedural_table = self.db.create_table(
                    "voltaire_procedural", schema=ProceduralKB
                )

            # Initialize Episodic Table
            if "voltaire_episodic" in self.db.table_names():
                self.episodic_table = self.db.open_table("voltaire_episodic")
            else:
                self.episodic_table = self.db.create_table(
                    "voltaire_episodic", schema=EpisodicEvent
                )

            logger.info(f"🧠 Loading Embedding Model: {self.model_name}...")
            self.model = SentenceTransformer(self.model_name)
            self.connected = True
            logger.info("✅ Memory Engine Online (Unified Tier).")
        except Exception as e:
            logger.error(f"❌ Memory Init Failed: {e}")

    def add_episodic(self, checkpoint: dict):
        """
        D-102: Saves a 'Flashbulb Memory' of a trade or veto.
        """
        if not self.connected:
            return
        try:
            vector_text = checkpoint.get("vector_text", "")
            if not vector_text:
                # Fallback to payload string if no text provided
                vector_text = str(checkpoint.get("payload", ""))

            vec = self.model.encode(vector_text)

            entry = EpisodicEvent(
                timestamp=checkpoint.get("timestamp", int(time.time() * 1000)),
                vector=vec,
                type=checkpoint.get("type", "UNKNOWN"),
                payload=checkpoint.get("payload", "{}"),
                outcome=checkpoint.get("outcome", "N/A"),
                venue=checkpoint.get("venue", "ALL"),
                reasoning=vector_text,
            )
            self.episodic_table.add([entry])
            logger.info(f"💾 Episodic Memory Saved: {checkpoint.get('type')}")
        except Exception as e:
            logger.error(f"Failed to save episodic memory: {e}")

    def add_procedural(self, entry: dict):
        """
        D-102: Adds a Standard Operating Procedure (SOP) to Procedural Memory.
        """
        if not self.connected:
            return
        try:
            narrative = entry.get("narrative", "")
            vec = self.model.encode(narrative)

            row = ProceduralKB(
                timestamp=entry.get("timestamp", int(time.time())),
                vector=vec,
                regime=entry.get("regime", "UNKNOWN"),
                narrative=narrative,
                venue=entry.get("venue", "ALL"),
                outcome=entry.get("outcome", "unknown"),
            )
            self.procedural_table.add([row])
            logger.info(f"📘 Procedural SOP Saved: {entry.get('regime')}")
        except Exception as e:
            logger.error(f"Failed to save procedural memory: {e}")

    def find_nearest_regime(self, query_text: str, limit=1, venue="ALL"):
        """
        D-102: Searches Procedural Memory for SOPs.
        Applies Geospatial Filter (Venue Preference).
        """
        if not self.connected:
            return None, 999.0

        try:
            # Vectorize
            query_vec = self.model.encode(query_text)

            # Search Procedural Table with wider net
            results = (
                self.procedural_table.search(query_vec).limit(limit * 5).to_pandas()
            )

            if results.empty:
                return None, 999.0

            best_match = None

            # D-102 Bias: Strict Venue Prioritization
            if venue != "ALL":
                # 1. Try to find Exact Venue Match
                exact_venue = results[results["venue"] == venue]
                if not exact_venue.empty:
                    best_match = exact_venue.iloc[0]

            # 2. If no exact match (or venue was ALL), try to find Universal (ALL) match
            if best_match is None:
                # We prioritize "ALL" over a mismatching venue
                universal = results[results["venue"] == "ALL"]
                if not universal.empty:
                    best_match = universal.iloc[0]

            # 3. Last Resort: Just the closest vector
            if best_match is None:
                best_match = results.iloc[0]

            regime_name = best_match["regime"]
            narrative = best_match["narrative"]
            distance = best_match["_distance"]

            return f"{regime_name} ({narrative})", distance

        except Exception as e:
            logger.error(f"Search Error: {e}")
            return None, 999.0

    def get_trade_biopsy(self, trade_id=None, start_time=None, end_time=None):
        """
        D-106: Forensic Time Machine - Biopsy
        Retrieves the 'Reasoning Trace' (Context) for a specific trade or time window.
        """
        if not self.connected:
            return []

        try:
            # For now, we query the episodic memory for entries roughly matching the timeframe
            # or payload containing the trade_id.

            # This is a naive implementation. In a real system, we'd query by exact timestamp range.
            # LanceDB filtering support is improving.

            # Pandas fallback for filtering
            df = self.episodic_table.to_pandas()

            if start_time and end_time:
                mask = (df["timestamp"] >= start_time) & (df["timestamp"] <= end_time)
                df = df[mask]

            if trade_id:
                # Basic string contains check in payload
                df = df[df["payload"].astype(str).str.contains(trade_id, na=False)]

            return df.to_dict("records")

        except Exception as e:
            logger.error(f"Biopsy Failed: {e}")
            return []
