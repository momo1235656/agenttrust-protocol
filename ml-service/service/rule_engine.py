"""Rule-based fraud detection engine."""
from dataclasses import dataclass, field
from typing import List


@dataclass
class FraudInput:
    agent_did: str
    amount: int
    avg_amount_30d: int = 1
    hourly_tx_count: int = 0
    normal_hourly_rate: float = 1.0
    is_night: bool = False
    rapid_count_5min: int = 0


@dataclass
class FraudResult:
    risk_score: float
    risk_level: str
    decision: str
    triggered_rules: List[str] = field(default_factory=list)


RULES = [
    ("amount_10x_average", 0.40, lambda i: i.amount / max(i.avg_amount_30d, 1) >= 10),
    ("amount_5x_average",  0.20, lambda i: 5 <= i.amount / max(i.avg_amount_30d, 1) < 10),
    ("frequency_5x_hourly",0.30, lambda i: i.hourly_tx_count >= i.normal_hourly_rate * 5),
    ("night_high_value",   0.25, lambda i: i.is_night and i.amount >= 50_000),
    ("rapid_succession",   0.35, lambda i: i.rapid_count_5min >= 3),
]


def evaluate(inp: FraudInput) -> FraudResult:
    score = 0.0
    triggered = []
    for name, weight, condition in RULES:
        if condition(inp):
            score += weight
            triggered.append(name)
    score = min(score, 1.0)

    risk_level = "low" if score < 0.3 else "medium" if score < 0.7 else "high"
    decision = "allow" if score < 0.3 else "review" if score < 0.7 else "block"

    return FraudResult(
        risk_score=score,
        risk_level=risk_level,
        decision=decision,
        triggered_rules=triggered,
    )
