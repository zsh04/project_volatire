import os
import asyncio
import logging
import feedparser
import aiohttp
from typing import List, Dict, Any, Tuple
from collections import deque
from dotenv import load_dotenv

# Load Env from Root
# Ensure we look at the project root for .env
# Assuming we are in src/brain/src/hypatia/news.py
# Root is ../../../../
project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../../"))
load_dotenv(os.path.join(project_root, ".env"))

logger = logging.getLogger(__name__)

# --- CONFIGURATION ---
# Updated to Developer V2 endpoint based on live verification
CRYPTOPANIC_BASE_URL = "https://cryptopanic.com/api/developer/v2/posts/"
RSS_FEEDS = [
    "https://cointelegraph.com/rss",
    "https://www.coindesk.com/arc/outboundfeeds/rss/",
]

# Tier 1: Sieve Keywords
URGENT_KEYWORDS = {
    "HACK",
    "EXPLOIT",
    "FREEZE",
    "HALT",
    "BANKRUPT",
    "INSOLVENT",
    "ARREST",
    "RAID",
    "SEIZE",
    "DEPEG",
    "COLLAPSE",
    "LIQUIDATION",
}
MACRO_KEYWORDS = {
    "SEC",
    "LAWSUIT",
    "FED",
    "RATE HIKE",
    "CPI",
    "INFLATION",
    "FOMC",
    "REGULATION",
}


class NewsEngine:
    """
    The Scrappy News Engine (Hypatia Expansion).
    Listens to CryptoPanic and RSS. Applies Sieve and Neural Scoring.
    """

    def __init__(self):
        self.api_key = os.getenv("CRYPTOPANIC_API_KEY")
        if not self.api_key:
            logger.warning(
                "⚠️ CRYPTOPANIC_API_KEY not found. NewsEngine will run in degraded mode (RSS only)."
            )

        self.seen_hashes = deque(maxlen=500)  # Deduplication
        self.latest_sentiment = 0.0  # -1.0 (Panic) to 1.0 (Euphoria)
        self.latest_headline = "Market is active."  # Default
        self.is_panic = False  # Critical Veto Flag

        # Load ONNX Model (Placeholder for now, defaulting to Mock Scorer if missing)
        self.onnx_session = None
        self._init_neural_engine()

    def _init_neural_engine(self):
        """
        Attempts to load ONNX model for CryptoBERT.
        Falls back to Heuristic Scorer if missing.
        """
        try:
            from transformers import AutoTokenizer
            from optimum.onnxruntime import ORTModelForSequenceClassification

            # Path relative to src/brain/src/hypatia/news.py
            # Model is in src/brain/models/crypto_bert_onnx
            model_path = os.path.abspath(
                os.path.join(os.path.dirname(__file__), "../../models/crypto_bert_onnx")
            )

            if os.path.exists(model_path):
                self.tokenizer = AutoTokenizer.from_pretrained(model_path)
                self.model = ORTModelForSequenceClassification.from_pretrained(
                    model_path
                )
                logger.info(f"✅ Neural Scorer Online: {model_path}")
            else:
                logger.warning(
                    f"⚠️ Neural Model not found at {model_path}. Using Heuristic Scorer."
                )
        except Exception as e:
            logger.error(f"❌ Neural Engine Init Failed: {e}. Using Heuristic Scorer.")

    async def fetch_news(self):
        """
        Main Loop (Async). Polling Strategy.
        """
        logger.info("📰 NewsEngine: Starting Ingest Loop...")
        while True:
            try:
                # 1. Fetch
                headlines = await self._ingest()

                # 2. Sieve & Score
                panic_detected = False

                for title, source in headlines:
                    if self._is_duplicate(title):
                        continue

                    # Tier 1: Keyword Sieve
                    params = self._sieve(title)
                    if params["is_urgent"] or params["is_macro"]:
                        # Tier 2: Score
                        sentiment, is_bearish_panic = self._score(title, params)

                        if is_bearish_panic:
                            logger.critical(
                                f"🚨 PANIC DETECTED: {title} (Source: {source})"
                            )
                            panic_detected = True

                        # Update rolling sentiment (decaying average)
                        self.latest_sentiment = (
                            0.8 * self.latest_sentiment + 0.2 * sentiment
                        )
                        self.latest_headline = title
                        logger.info(
                            f"📰 News ({source}): {title} | Sent: {sentiment:.2f}"
                        )

                # 3. Update Veto Flag
                # If ANY panic headline was found this cycle, raise flag.
                # In robust system, we might want a cooldown.
                # Here we latch it for this cycle.
                self.is_panic = panic_detected

            except Exception as e:
                logger.error(f"NewsLoop Error: {e}")

            await asyncio.sleep(60)  # 60s poll

    async def _ingest(self) -> List[Tuple[str, str]]:
        """
        Gathers raw headlines from API and RSS.
        """
        tasks = []
        if self.api_key:
            tasks.append(self._fetch_cryptopanic())

        for url in RSS_FEEDS:
            tasks.append(self._fetch_rss(url))

        results = await asyncio.gather(*tasks, return_exceptions=True)

        headlines = []
        for res in results:
            if isinstance(res, list):
                headlines.extend(res)
        return headlines

    async def _fetch_cryptopanic(self) -> List[Tuple[str, str]]:
        """
        Calls CryptoPanic API.
        """
        out = []
        try:
            async with aiohttp.ClientSession() as session:
                url = f"{CRYPTOPANIC_BASE_URL}?auth_token={self.api_key}&filter=hot&kind=news"
                async with session.get(url, timeout=10) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        for post in data.get("results", []):
                            out.append((post["title"], "CryptoPanic"))
        except Exception as e:
            logger.debug(f"CP Fetch Error: {e}")
        return out

    async def _fetch_rss(self, url: str) -> List[Tuple[str, str]]:
        """
        Parses RSS Feed.
        NOTE: feedparser is blocking. We should run in executor.
        """
        out = []
        try:
            loop = asyncio.get_running_loop()
            feed = await loop.run_in_executor(None, feedparser.parse, url)
            for entry in feed.entries[:5]:  # Top 5 only
                out.append((entry.title, "RSS"))
        except Exception as e:
            logger.debug(f"RSS Fetch Error ({url}): {e}")
        return out

    def _sieve(self, text: str) -> Dict[str, Any]:
        """
        Tier 1: Checks for Urgent/Macro keywords.
        """
        upper = text.upper()
        is_urgent = any(k in upper for k in URGENT_KEYWORDS)
        is_macro = any(k in upper for k in MACRO_KEYWORDS)
        return {"is_urgent": is_urgent, "is_macro": is_macro}

    def _score(self, text: str, params: Dict[str, Any]) -> Tuple[float, bool]:
        """
        Tier 2: Neural Scorer (CryptoBERT ONNX).
        Returns: (sentiment_score, is_panic_trigger)
        """
        is_panic = False
        score = 0.0

        if self.model and self.tokenizer:
            try:
                import torch

                inputs = self.tokenizer(
                    text, return_tensors="pt", truncation=True, max_length=128
                )
                outputs = self.model(**inputs)
                probs = torch.softmax(outputs.logits, dim=-1)[0].detach().numpy()

                # ElKulako/cryptobert mapping: 0: Bearish, 1: Neutral, 2: Bullish
                bearish_prob = float(probs[0])
                # neutral_prob = float(probs[1]) # Unused
                bullish_prob = float(probs[2])

                # Composite Score (-1 to 1)
                score = bullish_prob - bearish_prob

                # Panic Trigger: High Bearish Probability on Urgent News
                if params["is_urgent"] and bearish_prob > 0.9:
                    is_panic = True
                    logger.warning(
                        f"🚨 PANIC DETECTED (Neural): '{text}' (Bearish: {bearish_prob:.2f})"
                    )

                return score, is_panic

            except Exception as e:
                logger.error(f"Neural Inference Error: {e}. Falling back to Heuristic.")

        # Heuristic Scorer Fallback

        upper = text.upper()

        # Bearish Tokens (Expanded w/ Sieve keywords to ensure detection)
        bearish_tokens = [
            "CRASH",
            "PLUNGE",
            "DUMP",
            "SUED",
            "BAN",
            "UNKNOW",
            "SUSPEND",
            "HACK",
            "EXPLOIT",
            "FREEZE",
            "HALT",
            "BANKRUPT",
            "INSOLVENT",
            "ARREST",
            "RAID",
            "SEIZE",
            "DEPEG",
            "COLLAPSE",
            "LIQUIDATION",
        ]
        bullish_tokens = [
            "SURGE",
            "RECORD",
            "HIGH",
            "ETF",
            "APPROVE",
            "PARTNERSHIP",
            "LAUNCH",
            "ADOPTION",
        ]

        bear_count = sum(1 for t in bearish_tokens if t in upper)
        bull_count = sum(1 for t in bullish_tokens if t in upper)

        if bear_count > bull_count:
            score = -0.5 * bear_count
        elif bull_count > bear_count:
            score = 0.5 * bull_count

        # Panic Trigger logic
        if params["is_urgent"] and score < -0.4:
            is_panic = True

        return max(-1.0, min(1.0, score)), is_panic

    def _is_duplicate(self, title: str) -> bool:
        h = hash(title)
        if h in self.seen_hashes:
            return True
        self.seen_hashes.append(h)
        return False
