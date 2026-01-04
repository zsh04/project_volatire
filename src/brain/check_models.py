import sys
import os
import time
import torch
import warnings

# Suppress warnings for cleaner output
warnings.filterwarnings("ignore")


def check_chronos():
    print("\n⏳ Checking CHRONOS (Time-Series)...")
    try:
        from chronos import ChronosBoltPipeline

        model_name = "amazon/chronos-bolt-small"
        print(f"   > Loading {model_name}...")
        start = time.time()
        pipeline = ChronosBoltPipeline.from_pretrained(
            model_name,
            device_map="cpu",  # Force CPU for check to be safe
            torch_dtype=torch.float32,
        )
        print(f"   > Loaded in {(time.time() - start):.2f}s")

        # Test Inference
        print("   > Running dummy inference...")
        context = torch.tensor([[1.0, 2.0, 3.0, 4.0, 5.0]])
        forecast = pipeline.predict(context, prediction_length=3)
        print(f"   ✅ CHRONOS ALIVE. Forecast shape: {forecast.shape}")
        return True
    except ImportError:
        print("   ❌ FAILED: `chronos` package not installed.")
    except Exception as e:
        print(f"   ❌ FAILED: {e}")
    return False


def check_distilbert():
    print("\n🧠 Checking DISTILBERT (Embeddings)...")
    try:
        from sentence_transformers import SentenceTransformer

        model_name = "distilbert-base-nli-stsb-mean-tokens"
        print(f"   > Loading {model_name}...")
        start = time.time()
        model = SentenceTransformer(model_name)
        print(f"   > Loaded in {(time.time() - start):.2f}s")

        # Test Inference
        print("   > Running dummy embedding...")
        embeddings = model.encode(["The market is chaotic."])
        print(f"   ✅ DISTILBERT ALIVE. Embedding shape: {embeddings.shape}")
        return True
    except ImportError:
        print("   ❌ FAILED: `sentence-transformers` not installed.")
    except Exception as e:
        print(f"   ❌ FAILED: {e}")
    return False


if __name__ == "__main__":
    print(f"🔍 MODEL HEALTH CHECK | Python {sys.version.split()[0]}")
    c = check_chronos()
    d = check_distilbert()

    if c and d:
        print("\n✨ ALL SYSTEMS GO: Chronos & DistilBERT are operational.")
        sys.exit(0)
    else:
        print("\n⚠️ SOME SYSTEMS FAILED.")
        sys.exit(1)
