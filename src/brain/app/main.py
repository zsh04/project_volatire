import asyncio
import structlog
import grpc
from src.brain.app.generated import brain_pb2_grpc
from src.brain.app.server import BrainServicer
import signal

# Configure Structlog
structlog.configure(
    processors=[structlog.processors.TimeStamper(fmt="iso"), structlog.processors.JSONRenderer()],
    logger_factory=structlog.PrintLoggerFactory(),
)

logger = structlog.get_logger()


async def serve() -> None:
    """
    Start the BrainD gRPC Server.
    """
    server = grpc.aio.server()
    brain_pb2_grpc.add_BrainServiceServicer_to_server(BrainServicer(), server)
    listen_addr = "[::]:50052"
    server.add_insecure_port(listen_addr)

    logger.info("braind_startup", address=listen_addr, status="waking_up")

    await server.start()

    async def graceful_shutdown(sig: int, loop: asyncio.AbstractEventLoop):
        logger.info("braind_shutdown_signal_received", signal=sig)
        await server.stop(5)
        loop.stop()

    # Signal handlers
    loop = asyncio.get_running_loop()

    def handle_signal() -> None:
        asyncio.create_task(shutdown_wrapper(server))

    for sig in (signal.SIGINT, signal.SIGTERM):
        loop.add_signal_handler(sig, handle_signal)

    try:
        await server.wait_for_termination()
    except asyncio.CancelledError:
        logger.info("braind_shutdown_complete")


async def shutdown_wrapper(server):
    logger.info("braind_stopping_server")
    await server.stop(5)
    logger.info("braind_stopped")


if __name__ == "__main__":
    try:
        asyncio.run(serve())
    except KeyboardInterrupt:
        pass
