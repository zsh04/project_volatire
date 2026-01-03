import sys
import time
import logging
import torch
from transformers import AutoTokenizer, AutoModelForSequenceClassification
import numpy as np

# Setup Logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("SemanticLens")


class SentimentEngine:
    def __init__(
        self,
        model_name="mrm8488/distilroberta-finetuned-financial-news-sentiment-analysis",
    ):
        self.device = "mps" if torch.backends.mps.is_available() else "cpu"
        # Fallback to cuda if available (though likely MPS on mac)
        if torch.cuda.is_available():
            self.device = "cuda"

        logger.info(f"⚡ Initializing Sentiment Engine on {self.device.upper()}...")
        logger.info(f"🤖 Loading Model: {model_name}")

        try:
            self.tokenizer = AutoTokenizer.from_pretrained(model_name)
            self.model = AutoModelForSequenceClassification.from_pretrained(
                model_name
            ).to(self.device)
            self.model.eval()  # Set to inference mode
        except Exception as e:
            logger.error(f"💥 Failed to load model: {e}")
            raise e

        # Label Mapping for this specific model (usually: 0-negative, 1-neutral, 2-positive)
        # Check model config if possible, but standard for this repo is Neg, Neu, Pos.
        # We will verify in benchmark.
        self.id2label = self.model.config.id2label
        logger.info(f"🏷️ Label Map: {self.id2label}")

    def predict(self, text):
        """
        Returns sentiment score: -1 (Negative), 0 (Neutral), 1 (Positive)
        """
        inputs = self.tokenizer(
            text, return_tensors="pt", truncation=True, max_length=512
        ).to(self.device)
        with torch.no_grad():
            logits = self.model(**inputs).logits
            predicted_class = torch.argmax(logits, dim=1).item()

        # Map class ID to -1, 0, 1
        # mrm8488/distilroberta map: {0: 'negative', 1: 'neutral', 2: 'positive'}
        # We need to ensure we map correctly based on label string.
        label = self.id2label[predicted_class].lower()

        if "negative" in label:
            return -1
        if "positive" in label:
            return 1
        return 0

    def predict_batch(self, texts, batch_size=32):
        """
        High-throughput batch prediction.
        """
        results = []
        for i in range(0, len(texts), batch_size):
            batch = texts[i : i + batch_size]
            inputs = self.tokenizer(
                batch,
                return_tensors="pt",
                padding=True,
                truncation=True,
                max_length=512,
            ).to(self.device)

            with torch.no_grad():
                logits = self.model(**inputs).logits
                preds = torch.argmax(logits, dim=1).cpu().numpy()

            for p in preds:
                label = self.id2label[p].lower()
                if "negative" in label:
                    results.append(-1)
                elif "positive" in label:
                    results.append(1)
                else:
                    results.append(0)
        return results


def run_benchmark():
    logger.info("🏁 Starting Benchmark...")

    engine = SentimentEngine()

    # 1. Accuracy Test
    test_cases = [
        ("The company reported a record profit and revenue growth.", 1),
        ("Stock market crashes as fear grips investors.", -1),
        ("The fed kept interest rates unchanged.", 0),
        ("Bankruptcy looms for the struggling retailer.", -1),
        ("Innovator launches generic drug, shares soar.", 1),
    ]

    correct = 0
    for text, expected in test_cases:
        score = engine.predict(text)
        status = "✅" if score == expected else "❌"
        logger.info(
            f"{status} Text: '{text[:40]}...' | Pred: {score} | Exp: {expected}"
        )
        if score == expected:
            correct += 1

    accuracy = (correct / len(test_cases)) * 100
    logger.info(f"🎯 Accuracy on Golden Set: {accuracy:.2f}%")

    if accuracy < 100:
        logger.warning("⚠️ Accuracy check failed perfect score!")

    # 2. Speed Test
    # Generate dummy headlines
    dummy_headlines = [
        "Economy slows down as inflation rises.",
        "Tech giant unveils new AI processor.",
        "Oil prices stabilize after volatile week.",
        "Central bank signals aggressive tightening.",
        "Housing market cools off significantly.",
    ] * 20  # 100 items

    logger.info(f"🚀 Speed Test: Processing {len(dummy_headlines)} headlines...")
    start_time = time.time()
    _ = engine.predict_batch(dummy_headlines, batch_size=32)
    end_time = time.time()

    duration = end_time - start_time
    rate = len(dummy_headlines) / duration

    logger.info(f"⏱️ Processed {len(dummy_headlines)} in {duration:.4f}s")
    logger.info(f"⚡ Throughput: {rate:.2f} headlines/sec")

    if rate > 50:
        logger.info("✅ Speed Requirement MET (>50 Hz)")
    else:
        logger.warning("❌ Speed Requirement MISSED (<50 Hz)")


if __name__ == "__main__":
    run_benchmark()
