import pytest
from service.rule_engine import FraudInput, evaluate


def test_normal_transaction_is_allowed():
    inp = FraudInput(agent_did="did:test", amount=5000, avg_amount_30d=4000)
    result = evaluate(inp)
    assert result.decision == "allow"
    assert result.risk_score < 0.3


def test_10x_amount_is_blocked():
    inp = FraudInput(agent_did="did:test", amount=100_000, avg_amount_30d=5000)
    result = evaluate(inp)
    assert "amount_10x_average" in result.triggered_rules
    assert result.risk_score >= 0.40


def test_night_high_value_triggers():
    inp = FraudInput(agent_did="did:test", amount=60_000, avg_amount_30d=1000, is_night=True)
    result = evaluate(inp)
    assert "night_high_value" in result.triggered_rules


def test_rapid_succession_triggers():
    inp = FraudInput(agent_did="did:test", amount=1000, rapid_count_5min=5)
    result = evaluate(inp)
    assert "rapid_succession" in result.triggered_rules


def test_combined_rules_cap_at_1():
    inp = FraudInput(
        agent_did="did:test",
        amount=200_000,
        avg_amount_30d=1000,
        is_night=True,
        rapid_count_5min=5,
    )
    result = evaluate(inp)
    assert result.risk_score <= 1.0
    assert result.decision == "block"
