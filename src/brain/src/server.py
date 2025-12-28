import logging
import asyncio
from rich.console import Console
from rich.logging import RichHandler
from generated import brain_pb2, brain_pb2_grpc

# Setup Rich Console
console = Console()


class BrainService(brain_pb2_grpc.BrainServiceServicer):
    async def Reason(self, request, context):
        # Log the stimulus
        # Format: 🧠 CORTEX: Received Stimulus | Price: 95,xxx | V: +20.5 | Simons: +0.6
        console.print(
            f"[bold magenta]🧠 CORTEX:[/bold magenta] Received Stimulus | "
            f"Price: [green]{request.price:.2f}[/green] | "
            f"V: [cyan]{request.velocity:.2f}[/cyan] | "
            f"H: [yellow]{request.entropy:.2f}[/yellow] | "
            f"Simons: [blue]{request.simons_prediction:.2f}[/blue]"
        )

        # Return Intent
        return brain_pb2.StrategyIntent(
            action="HOLD", confidence=0.5, model_used="hypatia-v0"
        )

    async def Heartbeat(self, request, context):
        return brain_pb2.PulseAck(alive=True, timestamp=request.timestamp)

    async def NotifyRegimeChange(self, request, context):
        console.print(f"[bold red]🚨 REGIME CHANGE:[/bold red] {request.regime}")
        return brain_pb2.Empty()

    async def Forecast(self, request, context):
        return brain_pb2.ForecastResult(p50=0.0)
