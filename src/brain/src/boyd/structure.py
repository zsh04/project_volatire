from dataclasses import dataclass, field
from typing import Dict, Any, Optional


@dataclass
class TradeDecision:
    """
    Represents the final trading decision output by Boyd.
    """

    action: str  # "LONG", "SHORT", "HOLD"
    confidence: float  # 0.0 to 1.0
    reason: str
    meta: Dict[str, Any] = field(default_factory=dict)
