import os
import logging
import lancedb
from lancedb.pydantic import LanceModel, Vector
from sentence_transformers import SentenceTransformer

logger = logging.getLogger("HypatiaMemory")


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
        self.table = None
        self.model = None

        self._connect()

    def _connect(self):
        try:
            if not os.path.exists(self.db_path):
                logger.warning(
                    f"⚠️ Memory DB not found at {self.db_path}. Running without Long-Term Memory."
                )
                return

            self.db = lancedb.connect(self.db_path)
            if "voltaire_memories" not in self.db.table_names():
                logger.warning("⚠️ Table 'voltaire_memories' not found.")
                return

            self.table = self.db.open_table("voltaire_memories")

            logger.info(f"🧠 Loading Embedding Model: {self.model_name}...")
            self.model = SentenceTransformer(self.model_name)
            self.connected = True
            logger.info("✅ Memory Engine Online.")
        except Exception as e:
            logger.error(f"❌ Memory Init Failed: {e}")

    def find_nearest_regime(self, query_text: str, limit=1):
        """
        Embeds the query and searches for the nearest market memory.
        """
        if not self.connected:
            return None, 999.0

        try:
            # Vectorize
            query_vec = self.model.encode(query_text)

            # Search
            results = self.table.search(query_vec).limit(limit).to_pandas()

            if results.empty:
                return None, 999.0

            best_match = results.iloc[0]
            regime_name = best_match["regime"]
            narrative = best_match["narrative"]
            distance = best_match["_distance"]

            return f"{regime_name} ({narrative})", distance

        except Exception as e:
            logger.error(f"Search Error: {e}")
            return None, 999.0
