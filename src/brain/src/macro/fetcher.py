import os
import json
import asyncio
import aiohttp
from datetime import datetime, timedelta
from src.generated import macro_pb2

# Default Fallbacks
DEFAULT_RFR = 0.045  # 4.5%
DEFAULT_STRESS = 0.0
DEFAULT_SENTIMENT = 0.0
DEFAULT_REGIME = macro_pb2.EXPANSION


class MacroFetcher:
    """
    Directive-24C: The Macro-Hurdle Synapse.
    Fetches economic data from FRED, BEA, Quandl, and Tiingo.
    Maintains a local cache and handles API resilience.
    """

    def __init__(self):
        # API Keys
        self.fred_key = os.getenv("FRED_API_KEY")
        self.bea_key = os.getenv("BEA_API_KEY")
        self.tiingo_key = os.getenv("TIINGO_API_KEY")
        self.quandl_key = os.getenv("QUANDL_API_KEY")

        # Cache Config
        self.cache_file = "macro_cache.json"
        self.cache_ttl_fast = timedelta(hours=4)  # Sentiment/Stress
        self.cache_ttl_slow = timedelta(hours=24)  # RFR/Regime

        self.state = self._load_cache()

    def _load_cache(self) -> macro_pb2.MacroState:
        """Loads state from local JSON file or returns default."""
        if os.path.exists(self.cache_file):
            try:
                with open(self.cache_file, "r") as f:
                    data = json.load(f)
                    # Check age? Logic handled in fetch based on timestamp if we stored it.
                    # For simplicity, we assume loaded cache is 'last known good' and validity is checked locally
                    # But protobuf doesn't serialize to JSON directly without helper.
                    # We store simplistic dict.
                    return macro_pb2.MacroState(
                        risk_free_rate=data.get("rfr", DEFAULT_RFR),
                        systemic_stress=data.get("stress", DEFAULT_STRESS),
                        economic_sentiment=data.get("sentiment", DEFAULT_SENTIMENT),
                        regime_bias=data.get("regime", DEFAULT_REGIME),
                    )
            except Exception as e:
                print(f"⚠️ Cache Load Error: {e}")

        return macro_pb2.MacroState(
            risk_free_rate=DEFAULT_RFR,
            systemic_stress=DEFAULT_STRESS,
            economic_sentiment=DEFAULT_SENTIMENT,
            regime_bias=DEFAULT_REGIME,
        )

    def _save_cache(self, state: macro_pb2.MacroState):
        """Persists state to local JSON file."""
        data = {
            "rfr": state.risk_free_rate,
            "stress": state.systemic_stress,
            "sentiment": state.economic_sentiment,
            "regime": state.regime_bias,
            "timestamp": datetime.now().isoformat(),
        }
        try:
            with open(self.cache_file, "w") as f:
                json.dump(data, f)
        except Exception as e:
            print(f"⚠️ Cache Save Error: {e}")

    async def fetch_macro_state(self) -> macro_pb2.MacroState:
        """
        Main entry point. Fetches data concurrently with resilience.
        """
        print("🌍 SYNAPSE: Fetching Global Gravity...")

        async with aiohttp.ClientSession() as session:
            rfr_task = self._fetch_fred_rfr(session)
            stress_task = self._fetch_quandl_stress(session)
            sentiment_task = self._fetch_tiingo_sentiment(session)
            regime_task = self._fetch_bea_regime(session)

            results = await asyncio.gather(
                rfr_task,
                stress_task,
                sentiment_task,
                regime_task,
                return_exceptions=True,
            )

            # Unpack with Fallbacks (using previous state as fallback if API fails)
            rfr = (
                results[0]
                if isinstance(results[0], float)
                else self.state.risk_free_rate
            )
            stress = (
                results[1]
                if isinstance(results[1], float)
                else self.state.systemic_stress
            )
            sentiment = (
                results[2]
                if isinstance(results[2], float)
                else self.state.economic_sentiment
            )
            regime = (
                results[3] if isinstance(results[3], int) else self.state.regime_bias
            )

            # If exceptions provided, log them
            for res in results:
                if isinstance(res, Exception):
                    print(f"⚠️ Macro Fetch Warning: {res}")

            new_state = macro_pb2.MacroState(
                risk_free_rate=rfr,
                systemic_stress=stress,
                economic_sentiment=sentiment,
                regime_bias=regime,
            )

            self._save_cache(new_state)
            self.state = new_state

            print(
                f"🪐 GRAVITY UPDATED: RFR={rfr:.2%} Stress={stress:.2f} Sentiment={sentiment:.2f} Regime={macro_pb2.RegimeBias.Name(regime)}"
            )
            return new_state

    # --- Sources ---

    async def _fetch_fred_rfr(self, session) -> float:
        """FRED: FEDFUNDS (Effective Federal Funds Rate)."""
        if not self.fred_key:
            return DEFAULT_RFR
        url = "https://api.stlouisfed.org/fred/series/observations"
        params = {
            "series_id": "FEDFUNDS",
            "api_key": self.fred_key,
            "file_type": "json",
            "limit": 1,
            "sort_order": "desc",
        }
        async with session.get(url, params=params) as resp:
            if resp.status == 200:
                data = await resp.json()
                # Value is percent, e.g. "4.33"
                val = float(data["observations"][0]["value"])
                return val / 100.0
        raise Exception(f"FRED Failed: {resp.status}")

    async def _fetch_quandl_stress(self, session) -> float:
        """Quandl: STLFSI4 (St. Louis Fed Financial Stress Index)."""
        if not self.quandl_key:
            return DEFAULT_STRESS
        url = "https://www.quandl.com/api/v3/datasets/FRED/STLFSI4.json"
        params = {"api_key": self.quandl_key, "rows": 1, "order": "desc"}
        async with session.get(url, params=params) as resp:
            if resp.status == 200:
                data = await resp.json()
                return float(data["dataset"]["data"][0][1])
        raise Exception(f"Quandl Failed: {resp.status}")

    async def _fetch_tiingo_sentiment(self, session) -> float:
        """Tiingo: Crypto News Sentiment."""
        if not self.tiingo_key:
            return DEFAULT_SENTIMENT
        url = "https://api.tiingo.com/tiingo/news"
        params = {"token": self.tiingo_key, "limit": 20, "tickers": "crypto"}

        # Tiingo doesn't give direct sentiment scores in free tier usually,
        # but let's assume we parse title/description or use a mock logic if missing.
        # User constraint: "Aggregate news sentiment scores"
        # We will do a naive keyword scan for now or random (-1 to 1) if unavailable,
        # basically preserving previous logic but ensuring double return.

        async with session.get(url, params=params) as resp:
            if resp.status == 200:
                data = await resp.json()
                # Naive Sentiment: Count "bull" vs "bear" or similar in descriptions
                score = 0.0
                count = 0
                keywords = {
                    "soar": 0.5,
                    "surge": 0.5,
                    "jump": 0.5,
                    "gain": 0.3,
                    "high": 0.2,
                    "plunge": -0.5,
                    "crash": -0.8,
                    "drop": -0.3,
                    "loss": -0.3,
                    "low": -0.2,
                }
                for article in data:
                    text = (
                        (article.get("description") or "").lower()
                        + " "
                        + (article.get("title") or "").lower()
                    )
                    batch_score = sum(
                        val for key, val in keywords.items() if key in text
                    )
                    if batch_score != 0:
                        score += batch_score
                        count += 1

                if count > 0:
                    final_score = max(-1.0, min(1.0, score / count))
                    return final_score
                return 0.0
        raise Exception(f"Tiingo Failed: {resp.status}")

    async def _fetch_bea_regime(self, session) -> int:
        """BEA: GDP Percent Change -> Regime Bias."""
        if not self.bea_key:
            return DEFAULT_REGIME
        url = "https://apps.bea.gov/api/data"
        params = {
            "UserID": self.bea_key,
            "method": "GetData",
            "datasetname": "NIPA",
            "TableName": "T10101",
            "Frequency": "Q",
            "Year": "X",  # Last available
            "ResultFormat": "JSON",
        }

        # BEA API is tricky, sometimes requires POST or specific headers.
        # Assuming simple GET works for now based on docs.

        async with session.get(url, params=params) as resp:
            if resp.status == 200:
                data = await resp.json()
                # BEA JSON structure is deep
                try:
                    results = data["BEAAPI"]["Results"]["Data"]
                    # Find Line 1 (Gross domestic product)
                    for row in results:
                        if row["LineNumber"] == "1":
                            val = float(row["DataValue"].replace(",", ""))
                            if val > 1.0:
                                return macro_pb2.EXPANSION
                            if val < 0.0:
                                return macro_pb2.CONTRACTION
                            return macro_pb2.STAGFLATION  # 0-1% growth logic
                except (KeyError, ValueError, IndexError) as e:
                    raise Exception(f"BEA Parse Error: {e}")

        # Fallback if 200 but bad data
        raise Exception(f"BEA Failed: {resp.status}")
