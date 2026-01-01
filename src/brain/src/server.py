import logging

import pandas as pd
from collections import deque
from datetime import datetime
from rich.console import Console
from generated import brain_pb2, brain_pb2_grpc

from hypatia.engine import ContextEngine
from kepler.engine import KeplerOracle
from boyd.engine import BoydStrategist

# Setup Rich Console
console = Console()
logger = logging.getLogger(__name__)


class BrainService(brain_pb2_grpc.BrainServiceServicer):
    def __init__(self):
        logger.info("🧠 Initializing CORTEX Components...")
        self.hypatia = ContextEngine()
        self.kepler = KeplerOracle()
        self.boyd = BoydStrategist()

        # Internal Memory (Short-term)
        self.price_history = deque(maxlen=100)
        self.timestamps = deque(maxlen=100)

        logger.info("✨ CORTEX Online.")

    async def Reason(self, request, context):
        """
        The Main Loop entry point.
        Reflex sends StateVector -> Brain returns StrategyIntent.
        """
        # 1. Update Short-term Memory
        self.price_history.append(request.price)
        self.timestamps.append(datetime.now())

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
