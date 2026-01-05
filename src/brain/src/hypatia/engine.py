import asyncio
import json
import logging
import os
import requests
from pathlib import Path
from jinja2 import Template
from datetime import datetime
from .models import MarketRegime
from .adapter import HypatiaAdapter
from .news import NewsEngine
from .memory import MemoryEngine

logger = logging.getLogger(__name__)

class ContextEngine:
    def __init__(self, adapter=None):
        self.adapter = adapter or HypatiaAdapter()
        self.news_engine = NewsEngine()
        self.memory_engine = MemoryEngine()  # D-37
        self.current_regime = MarketRegime(timestamp=datetime.now(), score=0.0)
        self._news_task = None

        # D-87.5: Load Template
        template_path = Path(__file__).parent.parent.parent / "templates" / "grounding.j2"
        try:
             with open(template_path, "r") as f:
                 self.prompt_template = Template(f.read())
        except Exception as e:
            logger.error(f"Failed to load grounding.j2: {e}")
            self.prompt_template = None

    async def start_background_tasks(self):
        """
        Starts the News Ingest Loop.
        Should be called by Server on startup.
        """
        if not self._news_task:
            self._news_task = asyncio.create_task(self.news_engine.fetch_news())

    async def fetch_context(self, price: float, velocity: float, truth_envelope_json: str = None) -> dict:
        """
        Returns Semantic Context for D-54.
        Combines Real-time Sentiment (D-38), Historical Memory (D-37), and D-87.5 LLM Reasoning.
        """
        # Ensure Logic Engines are running
        if not self._news_task:
            await self.start_background_tasks()

        # 1. Get Sentiment (Fast, In-Memory)
        sentiment_score = self.news_engine.latest_sentiment

        # 2. Get Memory (LanceDB Search)
        narrative = self.news_engine.latest_headline
        query = f"Price: {price:.2f}, Velocity: {velocity:.2f}. Headlines: {narrative}"
        regime_label, distance = self.memory_engine.find_nearest_regime(query)

        if regime_label is None:
            regime_label = "Unknown Frontier"

        # D-87.5: LLM Reasoning Injection
        reasoning = "Initializing..."
        referenced_price = 0.0
        
        if truth_envelope_json:
            try:
                envelope = json.loads(truth_envelope_json)
                reasoning_packet = await self._generate_reasoning(envelope)
                if reasoning_packet:
                    reasoning_data = json.loads(reasoning_packet)
                    reasoning = reasoning_data.get("reasoning", "Analysis Failed")
                    referenced_price = float(reasoning_data.get("referenced_price", 0.0))
                    # Optional: Verify GSID match here or handle explicit errors
            except Exception as e:
                logger.error(f"D-87.5 LLM Failure: {e}")
                
        return {
            "sentiment_score": sentiment_score,
            "nearest_regime": regime_label,
            "regime_distance": distance,
            "reasoning": reasoning, # D-87.5
            "referenced_price": referenced_price, # D-87.5
        }

    async def _generate_reasoning(self, envelope: dict) -> str:
        """
        D-87.5: Logic for Contextual Injection.
        Renders template and calls Gemma Sidecar.
        """
        if not self.prompt_template:
            return None

        # 1. Render Prompt
        prompt = self.prompt_template.render(envelope=envelope)

        # 2. Call Gemma (Ollama Sidecar)
        # Using synchronous requests in async wrapper (should be async client ideally, but kept simple for now)
        try:
            # Assume Ollama at localhost:11434
            # Model: gemma:latest or user defined. Using 'gemma:2b' or similar as placeholder or 'gemma:9b' if available
            OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434/api/generate")
            MODEL_NAME = os.environ.get("OLLAMA_MODEL", "gemma2") 

            payload = {
                "model": MODEL_NAME,
                "prompt": prompt,
                "stream": False,
                "format": "json" # Enforce JSON Schema
            }
            
            # Offload blocking IO to thread
            response = await asyncio.to_thread(requests.post, OLLAMA_URL, json=payload, timeout=0.5) # 500ms timeout
            
            if response.status_code == 200:
                return response.json().get("response", "")
            else:
                logger.error(f"Ollama Error: {response.text}")
                return None

        except Exception as e:
            logger.error(f"LLM Call Failed: {e}")
            return None

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
