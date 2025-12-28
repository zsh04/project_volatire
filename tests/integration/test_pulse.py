import pytest
import grpc
from src.brain.app.generated import reflex_pb2, reflex_pb2_grpc
from src.brain.app.generated import brain_pb2, brain_pb2_grpc


@pytest.mark.asyncio
async def test_reflex_pulse():
    """
    Verifies that the Reflex (Body) is listening and can process a Ratchet signal.
    """
    async with grpc.aio.insecure_channel("localhost:50051") as channel:
        stub = reflex_pb2_grpc.ReflexServiceStub(channel)

        # 1. Send a Ratchet Trigger (IDLE) - Lowest risk state
        # The enum value for IDLE is 0
        response = await stub.TriggerRatchet(
            reflex_pb2.RatchetRequest(level=0, reason="Integration Test Pulse")
        )

        assert response.success is True
        assert "Ratchet Executed" in response.message


@pytest.mark.asyncio
async def test_brain_pulse():
    """
    Verifies that the Brain (Mind) is listening and can Reason.
    """
    async with grpc.aio.insecure_channel("localhost:50052") as channel:
        stub = brain_pb2_grpc.BrainServiceStub(channel)

        # 1. Ask for a Reason (Strategy Intent)
        state = brain_pb2.StateVector(price=100.0, velocity=0.0, vol_cluster=0.5)

        intent = await stub.Reason(state)

        assert intent.action in ["BUY", "SELL", "HOLD"]
        assert intent.confidence > 0.0
        assert intent.model_used == "hypatia-v0-stub"


@pytest.mark.asyncio
async def test_synapse_integrity():
    """
    Conceptual test: Verifies that both defined protobufs are loadable.
    (Implicitly passed if imports above succeed)
    """
    assert True
