import asyncio
import logging
import sys
import os
import grpc

# Add src and generated to path
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
sys.path.append(os.path.join(os.path.dirname(os.path.abspath(__file__)), "generated"))

from server import BrainService
from generated import brain_pb2_grpc


async def serve():
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
