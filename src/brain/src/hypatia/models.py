from dataclasses import dataclass
from datetime import datetime


@dataclass
class MarketRegime:
    """
    Represents the current peripheral market state (The Context).

    Attributes:
        score: A scalar from -1.0 (Apocalypse) to +1.0 (Goldilocks).
        vix: The CBOE Volatility Index.
        dxy: The US Dollar Index (or Proxy).
        yield_10y: The 10-Year US Treasury Yield.
        btc_trend_score: Indicator of BTC trend (e.g., dist from 200 SMA).
        is_risk_on: Boolean interpretation of the score.
        timestamp: When this snapshot was taken.
    """

    timestamp: datetime
    score: float
    vix: float = 0.0
    dxy: float = 0.0
    yield_10y: float = 0.0
    btc_trend_score: float = 0.0

    @property
    def is_risk_on(self) -> bool:
        return self.score > 0.0

    @property
    def leverage_scalar(self) -> float:
        """
        Suggested leverage cap based on regime.
        Range: 0.0 (Halt) to 1.0 (Max).
        """
        if self.score < -0.5:
            return 0.0
        elif self.score < 0.0:
            return 0.5
        else:
            return 1.0
