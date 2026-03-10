"""Tests for payment endpoints."""
import pytest
import uuid
from httpx import AsyncClient
from unittest.mock import patch, AsyncMock, MagicMock


@pytest.mark.asyncio
async def test_payment_success(client: AsyncClient, auth_token: dict):
    """Payment with valid token should succeed."""
    token = auth_token["token"]
    idempotency_key = f"test-{uuid.uuid4()}"

    mock_intent = MagicMock()
    mock_intent.id = "pi_test_123"
    mock_intent.status = "succeeded"
    mock_intent.amount = 5000

    with patch("stripe.PaymentIntent.create", return_value=mock_intent):
        response = await client.post(
            "/payment/execute",
            json={
                "amount": 5000,
                "currency": "jpy",
                "description": "テスト決済",
                "idempotency_key": idempotency_key,
            },
            headers={"Authorization": f"Bearer {token}"},
        )

    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "succeeded"
    assert data["amount"] == 5000


@pytest.mark.asyncio
async def test_payment_scope_exceeded(client: AsyncClient, auth_token: dict):
    """Payment exceeding token max_amount should be rejected with 403."""
    token = auth_token["token"]
    idempotency_key = f"test-{uuid.uuid4()}"

    response = await client.post(
        "/payment/execute",
        json={
            "amount": 999999,  # Exceeds the 50000 limit
            "currency": "jpy",
            "description": "高額決済",
            "idempotency_key": idempotency_key,
        },
        headers={"Authorization": f"Bearer {token}"},
    )

    assert response.status_code == 403


@pytest.mark.asyncio
async def test_payment_idempotency(client: AsyncClient, auth_token: dict):
    """Second payment with same idempotency key should return 409."""
    token = auth_token["token"]
    idempotency_key = f"test-idempotent-{uuid.uuid4()}"

    mock_intent = MagicMock()
    mock_intent.id = "pi_test_idem"
    mock_intent.status = "succeeded"
    mock_intent.amount = 1000

    with patch("stripe.PaymentIntent.create", return_value=mock_intent):
        response1 = await client.post(
            "/payment/execute",
            json={
                "amount": 1000,
                "currency": "jpy",
                "description": "テスト決済1",
                "idempotency_key": idempotency_key,
            },
            headers={"Authorization": f"Bearer {token}"},
        )
    assert response1.status_code == 200

    response2 = await client.post(
        "/payment/execute",
        json={
            "amount": 1000,
            "currency": "jpy",
            "description": "重複決済",
            "idempotency_key": idempotency_key,
        },
        headers={"Authorization": f"Bearer {token}"},
    )
    assert response2.status_code == 409


@pytest.mark.asyncio
async def test_payment_without_token_returns_401(client: AsyncClient):
    """Payment without Authorization header should return 401."""
    response = await client.post(
        "/payment/execute",
        json={
            "amount": 1000,
            "currency": "jpy",
            "description": "テスト",
            "idempotency_key": str(uuid.uuid4()),
        },
    )
    assert response.status_code == 401


@pytest.mark.asyncio
async def test_payment_with_invalid_token_returns_401(client: AsyncClient):
    """Payment with invalid token should return 401."""
    response = await client.post(
        "/payment/execute",
        json={
            "amount": 1000,
            "currency": "jpy",
            "description": "テスト",
            "idempotency_key": str(uuid.uuid4()),
        },
        headers={"Authorization": "Bearer invalid.token.here"},
    )
    assert response.status_code == 401
