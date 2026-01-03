import asyncio
from datetime import datetime
from .models import MarketRegime
from .adapter import HypatiaAdapter
from .news import NewsEngine


from .memory import MemoryEngine


class ContextEngine:
    def __init__(self, adapter=None):
        self.adapter = adapter or HypatiaAdapter()
        self.news_engine = NewsEngine()
        self.memory_engine = MemoryEngine()  # D-37
        self.current_regime = MarketRegime(timestamp=datetime.now(), score=0.0)
        self._news_task = None

    async def start_background_tasks(self):
        """
        Starts the News Ingest Loop.
        Should be called by Server on startup.
        """
        if not self._news_task:
            self._news_task = asyncio.create_task(self.news_engine.fetch_news())

    async def fetch_context(self, price: float, velocity: float) -> dict:
        """
        Returns Semantic Context for D-54.
        Combines Real-time Sentiment (D-38) with Historical Memory (D-37).
        """
        # Ensure Logic Engines are running
        if not self._news_task:
            await self.start_background_tasks()

        # 1. Get Sentiment (Fast, In-Memory)
        sentiment_score = self.news_engine.latest_sentiment

        # 2. Get Memory (LanceDB Search)
        # Construct Query from Physics + Narrative
        # "Market is falling fast. News: SEC Lawsuit triggers selloff."
        narrative = self.news_engine.latest_headline
        query = f"Price: {price:.2f}, Velocity: {velocity:.2f}. Headlines: {narrative}"

        regime_label, distance = self.memory_engine.find_nearest_regime(query)

        if regime_label is None:
            regime_label = "Unknown Frontier"

        return {
            "sentiment_score": sentiment_score,
            "nearest_regime": regime_label,
            "regime_distance": distance,
        }

    async def fetch_snapshot(self) -> MarketRegime:
        """
        Gathers all peripheral data and computes the regime score.
        """
        # Ensure News Engine is running (Safety check)
        if not self._news_task:
            await self.start_background_tasks()

        # Fetch in parallel
        vix_task = self.adapter.fetch_vix()
        btc_trend_task = self.adapter.fetch_btc_trend()
        yields_task = self.adapter.fetch_yields()
        dxy_task = self.adapter.fetch_dxy()  # Actually fetching UUP proxy for now

        vix, btc_trend, yields_10y, dxy_val = await asyncio.gather(
            vix_task, btc_trend_task, yields_task, dxy_task
        )

        # Calculate Score
        score = 0.5  # Start slightly optimistic (Neutral-Bullish default)

        # 1. Volatility Check
        if vix > 30.0:
            score -= 0.5
        elif vix < 15.0:
            score += 0.2

        # 2. Yield Stress
        if yields_10y > 5.0:  # High rates
            score -= 0.2

        # 3. BTC Trend
        if btc_trend > 0.10:  # > 10% above SMA
            score += 0.3
        elif btc_trend < -0.05:  # Below SMA
            score -= 0.4

        # 4. News Veto (Critical Risk)
        if self.news_engine.is_panic:
            # Force Extreme Fear
            score = -1.0
            # Modify VIX explicitly in object to signal panic if desired,
            # or just rely on the score and Regime mapping.
            # Ideally we check the regime map later.

        # 5. Clamp
        score = max(-1.0, min(1.0, score))

        # Determine Regime Label
        if self.news_engine.is_panic:
            regime_label = "CRASH"  # Override for News Panic
        else:
            regime_label = "NEUTRAL"  # Default, logic elsewhere might refine this
            if score > 0.6:
                regime_label = "BULL_TREND"
            if score < -0.6:
                regime_label = "BEAR_TREND"

        self.current_regime = MarketRegime(
            timestamp=datetime.now(),
            score=score,
            vix=vix,
            dxy=dxy_val,
            yield_10y=yields_10y,
            btc_trend_score=btc_trend,
        )

        # Inject Regime Label into the Object if it supports it (Snapshot might not have it field-wise but we use it in Boyd)
        # Boyd reads `regime.to_dict()["regime"]` if available, or just score.
        # Let's verify MarketRegime model.
        # For now, adding dynamic attribute for compatibility.
        self.current_regime.regime = regime_label

        return self.current_regime
