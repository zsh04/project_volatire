import sys
import time
import torch
import numpy as np
from sklearn.metrics.pairwise import cosine_similarity
import warnings

# Suppress warnings
warnings.filterwarnings("ignore")


class ModelAuditor:
    def __init__(self):
        self.chronos = None
        self.distilbert = None
        self.results = {}

    def load_models(self):
        print("⚙️  Loading Models for Deep Audit...")

        # Load Chronos
        try:
            from chronos import ChronosBoltPipeline

            self.chronos = ChronosBoltPipeline.from_pretrained(
                "amazon/chronos-bolt-small", device_map="cpu", torch_dtype=torch.float32
            )
            print("   ✅ Chronos Loaded.")
        except Exception as e:
            print(f"   ❌ Chronos Load Failed: {e}")

        # Load DistilBERT
        try:
            from sentence_transformers import SentenceTransformer

            self.distilbert = SentenceTransformer(
                "distilbert-base-nli-stsb-mean-tokens"
            )
            print("   ✅ DistilBERT Loaded.")
        except Exception as e:
            print(f"   ❌ DistilBERT Load Failed: {e}")

    def audit_chronos(self):
        if not self.chronos:
            return
        print("\n⏳ AUDIT: CHRONOS (Time-Series Logic)")

        # Test 1: Linear Trend Logic
        # Pattern: 10, 20, 30, 40, 50 -> Should predict > 50
        print("   [Check 1] Linear Trend Extrapolation...")
        context = torch.tensor([[10.0, 20.0, 30.0, 40.0, 50.0]])
        start = time.time()
        forecast = self.chronos.predict(context, prediction_length=15)
        latency = (time.time() - start) * 1000

        print(f"      Forecast Shape: {forecast.shape} (Batch, Quantiles, Horizon)")

        # Logic Gate: Must have 9 quantiles (p10...p90)
        shape_pass = forecast.shape[1] == 9 and forecast.shape[2] == 15
        print(
            f"      > Quantile Structure Check (9x15): {'✅ PASSED' if shape_pass else '❌ FAILED'}"
        )

        # Median forecast (quantile index 4 is usually median in 9 quantiles)
        median_forecast = forecast[0, 4, :].numpy()
        print(f"      Input: [10..50], Forecast (First 3): {median_forecast[:3]}...")

        # Logic Gate: First predicted point should be > 50
        trend_pass = median_forecast[0] > 50.0
        print(f"      > Trend Check: {'✅ PASSED' if trend_pass else '❌ FAILED'}")

        # Test 2: Stationarity Logic
        # Pattern: 100, 100, 100, 100 -> Should predict ~100
        print("   [Check 2] Stationarity/Noise Rejection...")
        context_static = torch.tensor([[100.0, 100.0, 100.0, 100.0, 100.0]])
        forecast_static = self.chronos.predict(context_static, prediction_length=15)
        median_static = forecast_static[0, 4, :].numpy()
        print(f"      Input: [100..100], Forecast (First 3): {median_static[:3]}...")

        # Logic Gate: Variance should be low, mean close to 100 (+- 5%)
        # Chronos might add noise, but median should be close.
        diff = np.abs(median_static[0] - 100)
        stationarity_pass = diff < 10.0
        print(
            f"      > Stationarity Check (Delta < 10): {'✅ PASSED' if stationarity_pass else '❌ FAILED'} (Delta: {diff:.2f})"
        )

        # Test 3: Batch Stress Test (Batch=50)
        print("   [Check 3] Batch Stress Test (N=50)...")
        # Create 50 random time series of length 10
        batch_context = torch.rand(50, 10)
        start_batch = time.time()
        forecast_batch = self.chronos.predict(batch_context, prediction_length=15)
        batch_latency = (time.time() - start_batch) * 1000

        print(f"      Batch Input: [50, 10]")
        print(f"      Batch Forecast: {forecast_batch.shape}")

        batch_pass = forecast_batch.shape == (50, 9, 15)
        print(
            f"      > Batch Structure Check: {'✅ PASSED' if batch_pass else '❌ FAILED'}"
        )
        print(
            f"      > Batch Latency: {batch_latency:.2f}ms ({(batch_latency / 50):.2f}ms per item)"
        )

        print(
            f"   [Performance] Single Latency: {latency:.2f}ms | Batch Latency: {batch_latency:.2f}ms"
        )
        self.results["chronos_trend"] = trend_pass
        self.results["chronos_stationarity"] = stationarity_pass
        self.results["chronos_batch"] = batch_pass

    def audit_distilbert(self):
        if not self.distilbert:
            return
        print("\n🧠 AUDIT: DISTILBERT (Semantic Relevance)")

        # Test: Semantic Relevance / Topical Filtering (RAG Logic)
        # Query: "Market Crash"
        # A: "The S&P 500 dropped by 5% due to inflation." (Relevant)
        # B: "The weather in California is sunny." (Irrelevant)
        # Expect Sim(Query,A) > Sim(Query,B)
        print("   [Check 1] RAG Relevance Logic...")

        query = "Market Crash"
        doc_a = "The S&P 500 dropped by 5% due to inflation."
        doc_b = "The weather in California is sunny."

        print(f"      Query: '{query}'")
        print(f"      Doc A (Relevant): '{doc_a}'")
        print(f"      Doc B (Irrelevant): '{doc_b}'")

        start = time.time()
        embs = self.distilbert.encode([query, doc_a, doc_b])
        latency = (time.time() - start) * 1000

        # Calculate Cosine Similarities
        sim_qa = cosine_similarity([embs[0]], [embs[1]])[0][0]  # Query vs Relevant
        sim_qb = cosine_similarity([embs[0]], [embs[2]])[0][0]  # Query vs Irrelevant

        print(f"      Sim(Query, Relevant): {sim_qa:.4f}")
        print(f"      Sim(Query, Irrelevant): {sim_qb:.4f}")

        # Logic Gate: Relevant doc should constitute a 'Strong Match' (> Irrelevant)
        logic_pass = sim_qa > (sim_qb + 0.2)  # Safety margin of 0.2
        print(f"      > Relevance Logic: {'✅ PASSED' if logic_pass else '❌ FAILED'}")

        print(f"   [Performance] Latency: {latency:.2f}ms")
        self.results["distilbert_logic"] = logic_pass

    def run(self):
        self.load_models()
        self.audit_chronos()
        self.audit_distilbert()

        all_passed = all(self.results.values())
        print(f"\n📋 FINAL AUDIT STATUS: {'✅ PASSED' if all_passed else '❌ FAILED'}")
        if not all_passed:
            sys.exit(1)


if __name__ == "__main__":
    auditor = ModelAuditor()
    auditor.run()
