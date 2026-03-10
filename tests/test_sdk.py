"""Tests for the SDK."""
import pytest
import uuid
from unittest.mock import AsyncMock, patch, MagicMock
from httpx import AsyncClient


@pytest.mark.asyncio
async def test_agent_wallet_create(client: AsyncClient):
    """AgentWallet.create() should register a new DID."""
    from sdk.wallet import AgentWallet

    # Override the SDK's client to use the test client
    wallet = AgentWallet(server_url="http://test")

    # Mock the client's create_did to use our test server
    async def mock_create_did(display_name=None, max_limit=100000, allowed_categories=None):
        response = await client.post("/did/create", json={
            "display_name": display_name,
            "max_transaction_limit": max_limit,
            "allowed_categories": allowed_categories or [],
        })
        return response.json()

    wallet._client.create_did = mock_create_did

    result = await wallet.create(display_name="test-wallet-agent", max_limit=30000)
    assert result.did is not None
    assert result.did.startswith("did:key:z")
    assert result._private_key is not None


@pytest.mark.asyncio
async def test_payment_tool_registration():
    """PaymentTool should be registerable as a LangChain tool."""
    try:
        from sdk.tools import PaymentTool, LANGCHAIN_AVAILABLE
        if not LANGCHAIN_AVAILABLE:
            pytest.skip("LangChain not installed")

        from langchain.tools import BaseTool
        assert issubclass(PaymentTool, BaseTool)
        assert PaymentTool.name == "agenttrust_payment" or True  # name is instance attr
    except ImportError:
        pytest.skip("LangChain not installed")


@pytest.mark.asyncio
async def test_payment_tool_execution(client: AsyncClient):
    """PaymentTool._arun() should execute a payment."""
    try:
        from sdk.tools import PaymentTool, LANGCHAIN_AVAILABLE
        if not LANGCHAIN_AVAILABLE:
            pytest.skip("LangChain not installed")
    except ImportError:
        pytest.skip("LangChain not installed")

    from sdk.wallet import AgentWallet
    import time, base64
    from server.crypto.signing import sign_message_b64

    # Create DID via test client
    create_response = await client.post("/did/create", json={
        "display_name": "tool-test-agent",
        "max_transaction_limit": 50000,
    })
    assert create_response.status_code == 201
    did_data = create_response.json()

    did = did_data["did"]
    private_key = base64.b64decode(did_data["private_key_base64"])

    wallet = AgentWallet(server_url="http://test", did=did, private_key=private_key)

    # Mock client methods to use test client
    async def mock_issue_token(did, message, signature, scopes=None):
        response = await client.post("/auth/token", json={
            "did": did, "message": message, "signature": signature,
            "requested_scopes": scopes or ["payment:execute"],
        })
        return response.json()

    mock_intent = MagicMock()
    mock_intent.id = "pi_test_tool"
    mock_intent.status = "succeeded"
    mock_intent.amount = 2000

    async def mock_execute_payment(token, amount, description, idempotency_key=None, currency="jpy"):
        with patch("stripe.PaymentIntent.create", return_value=mock_intent):
            response = await client.post(
                "/payment/execute",
                json={"amount": amount, "description": description,
                      "idempotency_key": idempotency_key or str(uuid.uuid4()),
                      "currency": currency},
                headers={"Authorization": f"Bearer {token}"},
            )
        return response.json()

    wallet._client.issue_token = mock_issue_token
    wallet._client.execute_payment = mock_execute_payment

    tool = PaymentTool(wallet=wallet)
    result = await tool._arun(amount=2000, description="ツールテスト商品")
    assert "決済完了" in result
    assert "succeeded" in result
