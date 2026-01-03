import sys
import os
import asyncio

# Setup Path FIRST
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
sys.path.append(os.path.join(os.path.dirname(os.path.abspath(__file__)), "generated"))

import logging

import dotenv
import telemetry
from server import BrainService
from generated import brain_pb2_grpc
import grpc


async def serve():
    # 0. Load Env
    dotenv.load_dotenv()

    # 1. Init Telemetry
    print("DEBUG: Initializing Telemetry...", flush=True)
    trace_provider, _, _ = telemetry.init_telemetry()
    print("DEBUG: Telemetry Initialized.", flush=True)

    # 2. Force Startup Trace
    tracer = trace_provider.get_tracer(__name__)
    with tracer.start_as_current_span("brain_startup_check") as span:
        span.set_attribute("version", "1.0.0")
        print("DEBUG: Startup Span Sent.", flush=True)

    server = grpc.aio.server()
    brain_pb2_grpc.add_BrainServiceServicer_to_server(BrainService(), server)
    listen_addr = "[::]:50052"
    server.add_insecure_port(listen_addr)
    print(f"🔮 BrainD (The Mind) listening on {listen_addr}")
    await server.start()
    await server.wait_for_termination()


if __name__ == "__main__":
    try:
        asyncio.run(serve())
    except KeyboardInterrupt:
        print("\n🛑 BrainD Shutdown")
