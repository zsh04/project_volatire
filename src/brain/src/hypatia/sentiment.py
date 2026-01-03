import logging
import torch
from transformers import AutoTokenizer, AutoModelForSequenceClassification

logger = logging.getLogger("HypatiaSentiment")


class SentimentEngine:
    def __init__(
        self,
        model_name="mrm8488/distilroberta-finetuned-financial-news-sentiment-analysis",
    ):
        self.device = "mps" if torch.backends.mps.is_available() else "cpu"
        if torch.cuda.is_available():
            self.device = "cuda"

        self.ready = False
        self.tokenizer = None
        self.model = None
        self.id2label = {}

        try:
            logger.info(f"🤖 Loading Sentiment Model ({self.device})...")
            self.tokenizer = AutoTokenizer.from_pretrained(model_name)
            self.model = AutoModelForSequenceClassification.from_pretrained(
                model_name
            ).to(self.device)
            self.model.eval()
            self.id2label = self.model.config.id2label
            self.ready = True
            logger.info("✅ Sentiment Engine Online.")
        except Exception as e:
            logger.error(f"❌ Sentiment Model Failed: {e}")

    def analyze(self, text: str) -> float:
        """
        Returns score: -1.0 (Negative) to 1.0 (Positive)
        """
        if not self.ready:
            return 0.0

        try:
            inputs = self.tokenizer(
                text, return_tensors="pt", truncation=True, max_length=512
            ).to(self.device)
            with torch.no_grad():
                logits = self.model(**inputs).logits
                predicted_class = torch.argmax(logits, dim=1).item()

            label = self.id2label[predicted_class].lower()

            if "negative" in label:
                return -1.0
            if "positive" in label:
                return 1.0
            return 0.0

        except Exception as e:
            logger.error(f"Inference Error: {e}")
            return 0.0
