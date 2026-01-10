import structlog
from src.brain.app.generated import brain_pb2, brain_pb2_grpc
import grpc

logger = structlog.get_logger()


class BrainServicer(brain_pb2_grpc.BrainServiceServicer):
    """
    Hypatia's Logic Core.
    Implements the reasoning and forecasting endpoints for the Cortex.
    """

    async def Reason(
        self, request: brain_pb2.StateVector, context: grpc.aio.ServicerContext
    ) -> brain_pb2.StrategyIntent:
        """
        Reflex asks for a decision based on current market state.
        """
        log = logger.bind(method="Reason", price=request.price, vol=request.vol_cluster)

        # Placeholder Logic (P1)
        action = "HOLD"
        confidence = 0.5

        log.info("reasoning_complete", action=action, confidence=confidence)

        return brain_pb2.StrategyIntent(
            action=action, confidence=confidence, model_used="hypatia-v0-stub"
        )

    async def Forecast(
        self, request: brain_pb2.HistoryWindow, context: grpc.aio.ServicerContext
    ) -> brain_pb2.ForecastResult:
        """
        Reflex asks for a future price/volatility forecast.
        """
        log = logger.bind(method="Forecast", window_size=request.window_size)

        # Placeholder Logic (P1)
        p50 = 100.0  # Dummy
        if request.prices:
            p50 = request.prices[-1]

        log.info("forecast_generated", p50=p50)

        return brain_pb2.ForecastResult(p10=p50 * 0.99, p50=p50, p90=p50 * 1.01, horizon=10.0)

    async def Heartbeat(
        self, request: brain_pb2.Pulse, context: grpc.aio.ServicerContext
    ) -> brain_pb2.PulseAck:
        """
        Reflex checks if Brain is alive.
        """
        # log.debug("pulse_received", timestamp=request.timestamp) # Debug level to avoid spam
        return brain_pb2.PulseAck(alive=True, timestamp=request.timestamp)

    async def NotifyRegimeChange(
        self, request: brain_pb2.RegimeEvent, context: grpc.aio.ServicerContext
    ) -> brain_pb2.Empty:
        """
        Reflex notifies of a regime change.
        """
        logger.info("regime_change_detected", regime=request.regime, vol=request.volatility)
        return brain_pb2.Empty()

    async def ResetState(
        self, request: brain_pb2.Empty, context: grpc.aio.ServicerContext
    ) -> brain_pb2.Ack:
        """
        Reflex triggers a hard reset of the Brain's internal state (AMR).
        """
        logger.warning("⚡ BRAIN RESET REQUESTED (AMR)")
        # In a real implementation, we would flush KV cache / memory here.
        return brain_pb2.Ack(success=True, message="Brain State Reset")
