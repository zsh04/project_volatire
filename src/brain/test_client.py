import asyncio
import logging
import sys
import os
import grpc

sys.path.append(os.path.dirname(os.path.abspath(__file__)) + "/src")
sys.path.append(os.path.dirname(os.path.abspath(__file__)) + "/src/generated")

from generated import brain_pb2, brain_pb2_grpc


async def run():
    async with grpc.aio.insecure_channel("localhost:50052") as channel:
        stub = brain_pb2_grpc.BrainServiceStub(channel)
        print("Sending Reason request...")
        response = await stub.Reason(
            brain_pb2.StateVector(
                price=100.0,
                velocity=5.0,
                vol_cluster=0.1,
                entropy=0.5,
                simons_prediction=1.0,
            )
        )
        print("Response received:")
        print(f"Action: {response.action}")
        print(f"Confidence: {response.confidence}")


if __name__ == "__main__":
    asyncio.run(run())
