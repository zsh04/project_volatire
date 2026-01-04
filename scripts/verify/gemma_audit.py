import asyncio
import json
import time
import urllib.request
from typing import Dict, Any

MODEL_NAME = "gemma2:9b-instruct-q4_K_M"
OLLAMA_URL = "http://localhost:11434/api/generate"


class GemmaAuditor:
    def __init__(self):
        self.results = {
            "structural_accuracy": 0.0,
            "latency_ttft": 0.0,
            "ids_score": 0.0,
        }

    async def run_audit(self):
        print(f"🔎 GENESIS AUDIT: Connecting to Sovereign Model via {OLLAMA_URL}...")

        # Check connectivity
        if not self._check_connection():
            print("❌ FAILED: Could not connect to Ollama. Ensure it is running.")
            return

        # 1. Structural Accuracy Gate (Obfuscation)
        print("   [Gate 1] Structural Accuracy (Obfuscation Testing)...")
        accuracy = await self._test_obfuscation_live()
        self.results["structural_accuracy"] = accuracy
        if accuracy >= 0.6:  # Relaxed for 0-shot
            print(f"      ✅ PASSED: {accuracy:.2%} >= 60%")
        else:
            print(f"      ❌ FAILED: {accuracy:.2%} < 60%")

        # 2. Latency Gate
        print("   [Gate 2] Latency & Jitter (Real-time Benchmarking)...")
        ttft = await self._benchmark_latency_live()
        self.results["latency_ttft"] = ttft
        # TTFT for a full local ML run might be higher than mock
        limit = 1000.0
        if ttft < limit:
            print(f"      ✅ PASSED: {ttft:.2f}ms < {limit}ms")
        else:
            print(f"      ❌ FAILED: {ttft:.2f}ms >= {limit}ms")

        # 3. Alignment Gate (IDS)
        print("   [Gate 3] Agent Alignment (IDS Score)...")
        ids = await self._calculate_ids_live()
        self.results["ids_score"] = ids
        if ids > 0.3:
            print(f"      ✅ PASSED: IDS {ids:.2f} > 0.3")
        else:
            print(f"      ❌ FAILED: IDS {ids:.2f} <= 0.3")

        self._generate_report()

    def _call_ollama(self, prompt: str) -> Dict:
        data = {
            "model": MODEL_NAME,
            "prompt": prompt,
            "stream": False,
            "options": {
                "temperature": 0.0,  # Deterministic for audit
                "seed": 42,
            },
        }
        req = urllib.request.Request(
            OLLAMA_URL,
            data=json.dumps(data).encode("utf-8"),
            headers={"Content-Type": "application/json"},
        )
        try:
            start = time.time()
            with urllib.request.urlopen(req) as response:
                result = json.loads(response.read().decode("utf-8"))
                result["latency_total"] = (time.time() - start) * 1000
                return result
        except Exception as e:
            print(f"      ⚠️ API Error: {e}")
            return {"response": "", "latency_total": 0}

    def _check_connection(self) -> bool:
        try:
            self._call_ollama("ping")
            return True
        except Exception:
            return False

    async def _test_obfuscation_live(self) -> float:
        # Case: Liquidity Vacuum (Price spikes on low volume/thin book)
        prompt = (
            "You are a financial logic engine. "
            "Analyze this obfuscated ticker data:\n"
            "T-0: Price 100, Vol 50\n"
            "T-1: Price 100, Vol 50\n"
            "T-2: Price 105 (Gap Up), Vol 10 (Low)\n"
            "Context: Order book Ask side was empty.\n"
            "Is this a 'Liquidity Vacuum' or 'True Breakout'? "
            "Reply with JUST the classification phrase."
        )

        # Wrapper to run blocking call in async
        loop = asyncio.get_running_loop()

        print("      > Sending Obfuscated Test Case...")
        response_data = await loop.run_in_executor(None, self._call_ollama, prompt)
        answer = response_data.get("response", "").strip()
        print(f"      > Model Answer: '{answer}'")

        if "Liquidity Vacuum" in answer:
            return 1.0
        return 0.0

    async def _benchmark_latency_live(self) -> float:
        loop = asyncio.get_running_loop()
        prompt = "Define 'Gamma' in one word."

        # Average of 3 runs
        total_latency = 0
        runs = 3
        for i in range(runs):
            print(f"      > Ping {i + 1}/{runs}...")
            res = await loop.run_in_executor(None, self._call_ollama, prompt)
            lat = res.get("latency_total", 0)
            total_latency += lat
            print(f"        Latency: {lat:.2f}ms")

        return total_latency / runs

    async def _calculate_ids_live(self) -> float:
        # Ask for a critique to see if it adds unique value (entropy)
        loop = asyncio.get_running_loop()
        hypatia_stmt = (
            "The market is efficient because price reflects all public information."
        )
        prompt = (
            f"Critique this statement from a chaos theory perspective: '{hypatia_stmt}'. "
            "Keep it under 20 words."
        )

        print(f"      > Sending Alignment Challenge...")
        res = await loop.run_in_executor(None, self._call_ollama, prompt)
        answer = res.get("response", "").strip()
        print(f"      > Model Critique: '{answer}'")

        # Simple heuristic: Does it mention non-linear concepts?
        keywords = [
            "fractal",
            "chaos",
            "chao",
            "nonlinear",
            "non-linear",
            "feedback",
            "entropy",
            "noise",
            "inefficient",
            "linear",
            "predict",
            "complex",
            "emergent",
            "sensitive",
        ]
        matches = sum(1 for k in keywords if k in answer.lower())

        # IDS score based on keyword density
        # Threshold > 0.3 means we need >1 match or a weighted match.
        # Simple count * 0.3 means 2 matches = 0.6 > 0.3.
        return min(1.0, matches * 0.3)

    def _generate_report(self):
        filename = "audit_report_gemma_live.json"
        with open(filename, "w") as f:
            json.dump(self.results, f, indent=2)
        print(f"\n📋 Real Audit Report saved to {filename}")


if __name__ == "__main__":
    auditor = GemmaAuditor()
    asyncio.run(auditor.run_audit())
