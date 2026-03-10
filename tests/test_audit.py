"""Tests for audit log endpoints."""
import pytest
import uuid
from httpx import AsyncClient
from unittest.mock import patch, MagicMock


@pytest.mark.asyncio
async def test_audit_log_recorded_after_payment(client: AsyncClient, auth_token: dict):
    """Audit log should be recorded after a successful payment."""
    token = auth_token["token"]
    did = auth_token["did"]
    idempotency_key = f"test-audit-{uuid.uuid4()}"

    mock_intent = MagicMock()
    mock_intent.id = "pi_test_audit"
    mock_intent.status = "succeeded"
    mock_intent.amount = 2000

    with patch("stripe.PaymentIntent.create", return_value=mock_intent):
        pay_response = await client.post(
            "/payment/execute",
            json={
                "amount": 2000,
                "currency": "jpy",
                "description": "監査ログテスト",
                "idempotency_key": idempotency_key,
            },
            headers={"Authorization": f"Bearer {token}"},
        )
    assert pay_response.status_code == 200

    # Check audit chain
    audit_response = await client.get(f"/audit/{did}")
    assert audit_response.status_code == 200
    data = audit_response.json()
    assert data["total_transactions"] >= 1


@pytest.mark.asyncio
async def test_hash_chain_verification_succeeds(client: AsyncClient, auth_token: dict):
    """Hash chain verification should succeed after payments."""
    token = auth_token["token"]
    did = auth_token["did"]

    mock_intent = MagicMock()
    mock_intent.id = "pi_test_verify"
    mock_intent.status = "succeeded"
    mock_intent.amount = 1500

    with patch("stripe.PaymentIntent.create", return_value=mock_intent):
        await client.post(
            "/payment/execute",
            json={
                "amount": 1500,
                "currency": "jpy",
                "description": "ハッシュチェーンテスト",
                "idempotency_key": f"test-chain-{uuid.uuid4()}",
            },
            headers={"Authorization": f"Bearer {token}"},
        )

    response = await client.post("/audit/verify", json={"agent_did": did})
    assert response.status_code == 200
    data = response.json()
    assert data["chain_valid"] is True


@pytest.mark.asyncio
async def test_hash_chain_prev_hash_linked(client: AsyncClient, auth_token: dict):
    """Each entry's prev_hash should link to the previous entry's hash."""
    token = auth_token["token"]
    did = auth_token["did"]

    mock_intent = MagicMock()
    mock_intent.status = "succeeded"
    mock_intent.amount = 1000

    # Make two payments
    for i in range(2):
        mock_intent.id = f"pi_test_chain_{i}"
        with patch("stripe.PaymentIntent.create", return_value=mock_intent):
            await client.post(
                "/payment/execute",
                json={
                    "amount": 1000,
                    "description": f"チェーンテスト{i}",
                    "idempotency_key": f"test-prev-{did[-8:]}-{i}-{uuid.uuid4()}",
                },
                headers={"Authorization": f"Bearer {token}"},
            )

    audit_response = await client.get(f"/audit/{did}")
    chain = audit_response.json()["chain"]

    if len(chain) >= 2:
        for i in range(1, len(chain)):
            assert chain[i]["prev_hash"] == chain[i-1]["hash"]


@pytest.mark.asyncio
async def test_empty_audit_chain_valid(client: AsyncClient):
    """An empty audit chain should be considered valid."""
    # Create a new agent with no transactions
    create_response = await client.post("/did/create", json={"display_name": "no-tx-agent"})
    assert create_response.status_code == 201
    did = create_response.json()["did"]

    response = await client.post("/audit/verify", json={"agent_did": did})
    assert response.status_code == 200
    data = response.json()
    assert data["chain_valid"] is True
    assert data["total_entries"] == 0
