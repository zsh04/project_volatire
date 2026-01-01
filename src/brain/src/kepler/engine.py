import logging
import os
import time
import pandas as pd
import torch

# Configure Logging
logger = logging.getLogger("KeplerOracle")
logging.basicConfig(level=logging.INFO)


class KeplerOracle:
    """
    The Prophet: Generates probabilistic price forecasts using Chronos-Bolt.
    """

    _instance = None
    _model = None
    _last_prediction_time = 0
    _cached_forecast = None
    _mock_mode = False

    # 1-Minute Heartbeat Cache
    CACHE_TTL = 60.0

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super(KeplerOracle, cls).__new__(cls)
            cls._instance._initialize()
        return cls._instance

    def _initialize(self):
        """
        Loads the Chronos-Bolt model. Falls back to Mock Mode if dependencies fail.
        """
        try:
            # Detect Device (MPS > CUDA > CPU)
            if torch.backends.mps.is_available():
                self.device = "mps"
            elif torch.cuda.is_available():
                self.device = "cuda"
            else:
                self.device = "cpu"

            # Check Environment
            env = os.getenv("VOLTAIRE_ENV", "SIMULATION").upper()
            self._mock_mode = False

            logger.info(
                f"🔮 Kepler Oracle initializing on {self.device.upper()} [ENV={env}]..."
            )

            self._model_path = "amazon/chronos-bolt-small"

            from chronos import ChronosBoltPipeline

            self.pipeline = ChronosBoltPipeline.from_pretrained(
                self._model_path,
                device_map=self.device,
                torch_dtype=torch.float32,
            )
            logger.info(
                f"✅ Chronos-Bolt ({self._model_path}) loaded successfully on {self.device.upper()}."
            )

        except ImportError as e:
            env = os.getenv("VOLTAIRE_ENV", "SIMULATION").upper()
            if env in ["LIVE", "PRODUCTION"]:
                logger.critical(
                    f"❌ CRITICAL: Dependencies missing in {env} mode. PANIC."
                )
                raise e  # Fail fast in Live

            logger.error(
                f"❌ Dependencies missing: {e}. Defaulting to MOCK MODE (Allowed in {env})."
            )
            self._mock_mode = True
        except Exception as load_err:
            env = os.getenv("VOLTAIRE_ENV", "SIMULATION").upper()
            if env in ["LIVE", "PRODUCTION"]:
                logger.critical(
                    f"❌ CRITICAL: Model Load Failed in {env} mode: {load_err}"
                )
                raise load_err

            logger.warning(
                f"⚠️ Failed to load ChronosBoltPipeline: {load_err}. Defaulting to Mock."
            )
            self._mock_mode = True

    def generate_forecast(self, df: pd.DataFrame, horizon: int = 10) -> pd.DataFrame:
        """
        Generates a P10-P90 forecast.
        Input DF columns: [timestamp, price] (or just target series)
        Output DF columns: [timestamp, p10, p20 ... p90]
        """
        current_time = time.time()

        # 1. Check Cache
        if self._cached_forecast is not None:
            if (current_time - self._last_prediction_time) < self.CACHE_TTL:
                logger.debug("✨ Serving Cached Forecast")
                return self._cached_forecast

        # 2. Run Inference
        logger.info(f"🧠 Running Inference (Horizon={horizon}m)...")

        if self._mock_mode:
            forecast = self._mock_inference(df, horizon)
        else:
            forecast = self._real_inference(df, horizon)

        # 3. Update Cache
        self._cached_forecast = forecast
        self._last_prediction_time = current_time

        return forecast

    def _mock_inference(self, df: pd.DataFrame, horizon: int) -> pd.DataFrame:
        """
        Generates a synthetic sine-wave fan chart for testing.
        """
        last_price = (
            df["price"].iloc[-1] if "price" in df.columns else df.iloc[-1].item()
        )
        last_ts = pd.Timestamp.now()

        future_dates = [
            last_ts + pd.Timedelta(minutes=i) for i in range(1, horizon + 1)
        ]

        # Create a "Fan Chart" expanding uncertainty
        results = []
        for i, date in enumerate(future_dates):
            t = i + 1
            volatility = last_price * 0.001 * t  # Vol expand over time

            row = {"timestamp": date}
            # Generate 9 deciles
            for q in range(1, 10):  # 1 to 9
                quantile = q / 10.0
                # Simple drift + noise: Price * (1 + (q-0.5)*vol)
                price_q = last_price + (volatility * (quantile - 0.5) * 2)
                row[f"p{q * 10}"] = price_q

            results.append(row)

        return pd.DataFrame(results)

    def _real_inference(self, df: pd.DataFrame, horizon: int) -> pd.DataFrame:
        """
        Runs real Chronos-Bolt inference.
        """
        # Prepare Context
        price_series = torch.tensor(df["price"].values)

        # Predict
        # Inputs: (batch_size, context_length) or just tensor
        # Returns: (batch_size, num_quantiles, prediction_length)
        # Chronos-Bolt outputs 9 quantiles: 0.1, 0.2, ..., 0.9 directly.
        forecast = self.pipeline.predict(inputs=price_series, prediction_length=horizon)

        # Extract batch 0 -> (9, horizon)
        # Transpose to (horizon, 9)
        quantiles_tensor = forecast[0].numpy().T

        last_ts = pd.Timestamp.now()
        future_dates = [
            last_ts + pd.Timedelta(minutes=i) for i in range(1, horizon + 1)
        ]

        results = []
        # Bolt outputs 0.1 to 0.9 mapping to indices 0 to 8
        qs = [10, 20, 30, 40, 50, 60, 70, 80, 90]

        for i, date in enumerate(future_dates):
            row = {"timestamp": date}
            # quantiles_tensor[i] is array of 9 values (p10..p90)
            step_modals = quantiles_tensor[i]

            for idx, q_label in enumerate(qs):
                row[f"p{q_label}"] = step_modals[idx].item()

            results.append(row)

        return pd.DataFrame(results)
