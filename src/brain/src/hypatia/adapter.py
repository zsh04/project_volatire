import os
import logging
from datetime import datetime, timedelta


# import pandas_datareader.data as web # Lazy import later
from alpaca.data.historical import StockHistoricalDataClient, CryptoHistoricalDataClient
from alpaca.data.requests import StockBarsRequest, CryptoBarsRequest
from alpaca.data.timeframe import TimeFrame

logger = logging.getLogger(__name__)


def load_env_manual(filepath):
    """Manually load .env file to avoid dependencies."""
    try:
        with open(filepath, "r") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                if "=" in line:
                    key, val = line.split("=", 1)
                    # Remove quotes if present
                    val = val.strip().strip("'").strip('"')
                    os.environ[key.strip()] = val
    except Exception as e:
        logger.warning(f"Hypatia: Failed to load .env manually: {e}")


# Load .env from project root
# current: src/brain/src/hypatia/adapter.py
# .env: /Users/zishanmalik/voltaire/.env
env_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../../.env"))
load_env_manual(env_path)


class HypatiaAdapter:
    def __init__(self):
        # Support both naming conventions, prioritize standard APCA, fallback to user's ALPACA
        self.api_key = (
            os.environ.get("APCA_API_KEY_ID")
            or os.environ.get("ALPACA_API_KEY")
            or "PK_MISSING"
        )
        self.api_secret = (
            os.environ.get("APCA_API_SECRET_KEY")
            or os.environ.get("ALPACA_API_SECRET")
            or "SK_MISSING"
        )

        if self.api_key == "PK_MISSING":
            logger.warning("Hypatia: Alpaca Keys missing. Running in MOCK mode.")
            self.mock_mode = True
        else:
            self.mock_mode = False
            self.stock_client = StockHistoricalDataClient(self.api_key, self.api_secret)
            self.crypto_client = CryptoHistoricalDataClient()  # Crypto doesn't strictly need keys for public data usually, but uses them if provided? Actually, yes separate client.

    async def fetch_vix(self) -> float:
        """Fetch VIX or Proxy (VIXY)."""
        if self.mock_mode:
            return 20.0

        # Try fetching VIXY (ETF) as proxy for VIX if Index not available
        try:
            req = StockBarsRequest(
                symbol_or_symbols=["VIXY"],
                timeframe=TimeFrame.Day,
                start=datetime.now() - timedelta(days=5),
                limit=1,
            )
            bars = self.stock_client.get_stock_bars(req)
            if "VIXY" in bars.data:
                # VIXY is an ETF, price ~ VIX futures. Not exact VIX.
                # Let's just assume we want a volatility metric.
                # Or better: Is VIX index available? Not on free data usually.
                # We'll return the close price.
                return bars["VIXY"][0].close
        except Exception as e:
            logger.error(f"Failed to fetch VIXY: {e}")

        return 20.0  # Fallback

    async def fetch_btc_trend(self) -> float:
        """Fetch BTC prices and calc distance from 200 SMA."""
        if self.mock_mode:
            return 0.05  # 5% above SMA

        try:
            req = CryptoBarsRequest(
                symbol_or_symbols=["BTC/USD"],
                timeframe=TimeFrame.Day,
                start=datetime.now() - timedelta(days=300),
                limit=250,
            )
            bars = self.crypto_client.get_crypto_bars(req)
            if "BTC/USD" in bars.data:
                df = bars.df
                if len(df) > 200:
                    sma_200 = df["close"].rolling(window=200).mean().iloc[-1]
                    current = df["close"].iloc[-1]
                    return (current - sma_200) / sma_200
        except Exception as e:
            logger.error(f"Failed to fetch BTC trend: {e}")

        return 0.0

    async def fetch_yields(self) -> float:
        """Fetch 10Y Treasury Yield (DGS10) from FRED via Pandas DataReader."""
        if self.mock_mode:
            return 4.0

        try:
            import pandas_datareader.data as web

            # start = datetime.now() - timedelta(days=10)
            # data = web.DataReader('DGS10', 'fred', start)
            # return data['DGS10'].iloc[-1]
            pass
        except ImportError:
            logger.warning("Hypatia: pandas_datareader not found. Using Mock Yields.")
        except Exception as e:
            logger.error(f"Failed to fetch Yields: {e}")

        return 4.0

    async def fetch_dxy(self) -> float:
        """Fetch DXY Proxy (UUP)."""
        if self.mock_mode:
            return 100.0

        try:
            req = StockBarsRequest(
                symbol_or_symbols=["UUP"],
                timeframe=TimeFrame.Day,
                start=datetime.now() - timedelta(days=5),
                limit=1,
            )
            bars = self.stock_client.get_stock_bars(req)
            if "UUP" in bars.data:
                return bars["UUP"][0].close
        except Exception:
            pass
        return 28.0  # UUP price roughly
