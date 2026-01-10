import logging
import time  # Fixed import

import pandas as pd
from collections import deque
from datetime import datetime
from rich.console import Console
from generated import brain_pb2, brain_pb2_grpc
from model_router import ModelRouter  # D-95

from hypatia.engine import ContextEngine
from kepler.engine import KeplerOracle
from boyd.engine import BoydStrategist

# Setup Rich Console
console = Console()
logger = logging.getLogger(__name__)


class BrainService(brain_pb2_grpc.BrainServiceServicer):
    def __init__(self):
        self.router = ModelRouter()
        self.router.load_adapters("config/adapters.json")  # Mock path
        print("Brain Service Initialized.")
        self.hypatia = ContextEngine()
        self.kepler = KeplerOracle()
        self.boyd = BoydStrategist()

        self.price_history = deque(maxlen=100)
        self.timestamps = deque(maxlen=100)

        # Telemetry Instruments
        from opentelemetry import metrics

        meter = metrics.get_meter("voltaire.brain")
        self.inference_duration = meter.create_histogram(
            name="brain_inference_duration",
            description="Time taken for Brain Reason() logic",
            unit="ms",
        )
        self.inference_confidence = meter.create_histogram(
            name="brain_inference_confidence",
            description="Confidence level of generated signals",
        )
        self.signals_generated = meter.create_counter(
            name="brain_signals_generated",
            description="Count of signals by action type",
        )

        from opentelemetry import trace

        self.tracer = trace.get_tracer("voltaire.brain")
        logger.info("✨ CORTEX Online.")

    async def Reason(self, request, context):
        """
        The Main Loop entry point.
        Reflex sends StateVector -> Brain returns StrategyIntent.
        """
        with self.tracer.start_as_current_span("brain_reason_loop"):
            # 1. Update Short-term Memory
            start_time = datetime.now()
            self.price_history.append(request.price)
            self.timestamps.append(start_time)

        # Log Stimulus (Rich)
        console.print(
            f"[bold magenta]🧠 CORTEX:[/bold magenta] Stimulus | "
            f"P: [green]{request.price:.2f}[/green] | "
            f"V: [cyan]{request.velocity:.2f}[/cyan] | "
            f"S: [blue]{request.simons_prediction:.2f}[/blue]"
        )

        # 2. Warmup Check
        if len(self.price_history) < 20:
            return brain_pb2.StrategyIntent(
                action="HOLD", confidence=0.0, model_used="warming_up"
            )

        # 3. Parallel Perception (Hypatia & Kepler)
        # We need a DataFrame for Kepler
        df = pd.DataFrame(
            {"timestamp": list(self.timestamps), "price": list(self.price_history)}
        )

        # Launch Async Tasks
        regime_task = self.hypatia.fetch_snapshot()

        # Kepler is synchronous (for now, unless we thread it),
        # but let's run it.
        # Ideally, we run CPU-bound work in executor if it blocks > 10ms.
        # efficient-chronos-bolt is fast.

        try:
            regime = await regime_task
            forecast = self.kepler.generate_forecast(df, horizon=10)
        except Exception as e:
            logger.error(f"Perception Error: {e}")
            return brain_pb2.StrategyIntent(
                action="HOLD", confidence=0.0, model_used="error_fallback"
            )

        # 4. Boyd Decision (The Strategist)
        # Prepare inputs
        valuation = {
            "price": request.price,
            "fair_value": request.price
            * (1.0 + request.simons_prediction),  # Simons predicts returns?
            # Actually, Simons returns "predicted_next_val".
            # If simons_prediction is raw value, we compare.
            # If simons_prediction is alpha, we adapt.
            # D-11 says: simons.forward(velocity) -> next_target (velocity).
            # So simons_prediction is predicted VELOCITY.
            # Let's approximate Fair Value = Price + Predicted_Velocity
        }
        # Refinement: Simons is ESN on velocity.
        # If predicted velocity is positive => Price will go up.
        # So "Fair Value" (next step) = Price + Pred_Vel.
        valuation["fair_value"] = request.price + request.simons_prediction

        decision = self.boyd.decide(
            market_data={"velocity": request.velocity, "entropy": request.entropy},
            valuation=valuation,
            regime=regime.to_dict() if hasattr(regime, "to_dict") else vars(regime),
            forecast=forecast,
            legislative_bias=getattr(request, "legislative_bias", "NEUTRAL"),  # D-107
        )

        # Log Decision
        color = "white"
        if decision.action == "LONG":
            color = "green"
        if decision.action == "SHORT":
            color = "red"

        console.print(
            f"[bold {color}]⚡ BOYD DECISION: {decision.action}[/bold {color}] ({decision.reason})"
        )

        if forecast is not None and not forecast.empty:
            next_step = forecast.iloc[0]
            f_p10 = next_step.get("p10", 0.0)
            f_p20 = next_step.get("p20", 0.0)
            f_p50 = next_step.get("p50", 0.0)
            f_p80 = next_step.get("p80", 0.0)
            f_p90 = next_step.get("p90", 0.0)
            # Assuming timestamp is pd.Timestamp
            ts = next_step.get("timestamp")
            f_ts = int(ts.timestamp() * 1000) if ts else 0
        else:
            f_p10, f_p20, f_p50, f_p80, f_p90, f_ts = 0.0, 0.0, 0.0, 0.0, 0.0, 0

        # Record Metrics
        duration_ms = (datetime.now() - start_time).total_seconds() * 1000
        self.inference_duration.record(duration_ms)
        self.inference_confidence.record(decision.confidence)
        self.signals_generated.add(1, {"action": decision.action})

        return brain_pb2.StrategyIntent(
            action=decision.action,
            confidence=decision.confidence,
            model_used="boyd-v1",
            forecast_p10=f_p10,
            forecast_p50=f_p50,
            forecast_p90=f_p90,
            forecast_p20=f_p20,
            forecast_p80=f_p80,
            forecast_timestamp=f_ts,
        )

    async def Heartbeat(self, request, context):
        return brain_pb2.PulseAck(alive=True, timestamp=request.timestamp)

    async def NotifyRegimeChange(self, request, context):
        console.print(f"[bold red]🚨 REGIME CHANGE:[/bold red] {request.regime}")
        return brain_pb2.Empty()

    async def Forecast(self, request, context):
        # Allow direct forecast requests (bypass Boyd)
        return brain_pb2.ForecastResult(p50=0.0)

    async def AddMemoryCheckpoint(self, request, context):
        """
        D-102: Saves Episodic Memory (Trade/Veto)
        """
        try:
            checkpoint = {
                "timestamp": request.timestamp,
                "type": request.type,
                "payload": request.payload,
                "outcome": request.outcome,
                "venue": request.venue,
                "vector_text": request.vector_text,
            }
            # Use Hypatia's Memory Engine
            # Note: We need to ensure logic runs in thread/async properly if blocking
            # but lancedb add is reasonably fast.
            self.hypatia.memory_engine.add_episodic(checkpoint)

            console.print(
                f"[bold green]💾 MEMORY:[/bold green] Checkpoint ({request.type}) saved."
            )
            return brain_pb2.Ack(success=True, message="Checkpoint Saved")
        except Exception as e:
            logger.error(f"Checkpoint Error: {e}")
            return brain_pb2.Ack(success=False, message=str(e))

    async def GetContext(self, request, context):
        start = time.time()

        # D-95: Extract intended adapter from request
        # If active_adapter is not present or empty, fallback to router inference or default.
        adapter_id = getattr(request, "active_adapter", None)
        if adapter_id:
             flavor = adapter_id
        else:
             flavor = self.router.inference("Market Data...")

        reasoning = (
            f"{flavor}: Market implies hold. Acceleration {getattr(request, 'acceleration', 0.0):.2f}"
        )
        """
        D-54: Live Semantic Context.
        Combines Real-Time Sentiment (D-38) + Deep Memory (D-37) + D-87.5 LLM Grounding.
        """
        # D-87.5: Extract Truth Envelope
        truth_envelope = getattr(request, "truth_envelope", None)

        # Call Hypatia Engine
        ctx_data = await self.hypatia.fetch_context(
            request.price, request.velocity, truth_envelope_json=truth_envelope
        )

        duration_ns = int((datetime.now() - start).total_seconds() * 1e9)

        # Metrics Log for Audit extraction
        print(f"[METRICS] gemma_latency_ms={duration_ns / 1e6:.4f}", flush=True)

        console.print(
            f"[bold yellow]🧠 CONTEXT:[/bold yellow] "
            f"Regime: [cyan]{ctx_data['nearest_regime']}[/cyan] | "
            f"Sent: [magenta]{ctx_data['sentiment_score']:.2f}[/magenta] | "
            f"Reasoning: {ctx_data.get('reasoning', 'N/A')[:30]}..."
        )

        # Prepare context packet for Boyd, similar to Reason's market_data, valuation, regime
        context_pkt = {
            "market_data": {"velocity": request.velocity, "entropy": request.entropy},
            "valuation": {
                "price": request.price,
                "fair_value": request.price
                + request.simons_prediction,  # Assuming simons_prediction is velocity
            },
            "regime": ctx_data.get(
                "nearest_regime", "Unknown"
            ),  # Simplified for context
            "forecast": None,  # No forecast available in GetContext
        }

        if self.boyd:
            decision = self.boyd.decide(
                **context_pkt
            )  # Pass dictionary as keyword arguments
            if decision.action == "VETO":
                reasoning += f" [BOYD VETO: {decision.reason}]"

        return brain_pb2.ContextResponse(
            sentiment_score=ctx_data["sentiment_score"],
            nearest_regime=ctx_data["nearest_regime"],
            regime_distance=ctx_data["regime_distance"],
            computation_time_ns=duration_ns,
            reasoning=reasoning,  # Use the potentially modified reasoning
            referenced_price=ctx_data.get("referenced_price", 0.0),
        )
