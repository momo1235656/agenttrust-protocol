"""Tests for authentication endpoints."""
import pytest
import base64
import time
import jwt as pyjwt
from httpx import AsyncClient

from server.crypto.signing import sign_message_b64


@pytest.mark.asyncio
async def test_issue_token_success(client: AsyncClient, created_did: dict):
    """Issuing a token with valid DID signature should succeed."""
    did = created_did["did"]
    private_key = base64.b64decode(created_did["private_key_base64"])
    message = f"auth_request_{int(time.time())}"
    signature = sign_message_b64(private_key, message)

    response = await client.post("/auth/token", json={
        "did": did,
        "message": message,
        "signature": signature,
        "requested_scopes": ["payment:execute"],
    })
    assert response.status_code == 200
    data = response.json()
    assert "access_token" in data
    assert data["token_type"] == "Bearer"
    assert data["expires_in"] == 1800


@pytest.mark.asyncio
async def test_issued_token_is_decodable(client: AsyncClient, created_did: dict):
    """Issued JWT should be decodable with correct structure."""
    did = created_did["did"]
    private_key = base64.b64decode(created_did["private_key_base64"])
    message = f"auth_request_{int(time.time())}"
    signature = sign_message_b64(private_key, message)

    response = await client.post("/auth/token", json={
        "did": did,
        "message": message,
        "signature": signature,
        "requested_scopes": ["payment:execute"],
    })
    assert response.status_code == 200
    token = response.json()["access_token"]

    # Decode without verification to check structure
    payload = pyjwt.decode(token, options={"verify_signature": False})
    assert payload["sub"] == did
    assert "scopes" in payload
    assert "exp" in payload


@pytest.mark.asyncio
async def test_token_payload_contains_scopes(client: AsyncClient, created_did: dict):
    """Issued JWT payload should contain the requested scopes."""
    did = created_did["did"]
    private_key = base64.b64decode(created_did["private_key_base64"])
    message = f"auth_request_{int(time.time())}"
    signature = sign_message_b64(private_key, message)

    response = await client.post("/auth/token", json={
        "did": did,
        "message": message,
        "signature": signature,
        "requested_scopes": ["payment:execute", "balance:read"],
    })
    assert response.status_code == 200
    token = response.json()["access_token"]

    payload = pyjwt.decode(token, options={"verify_signature": False})
    assert "payment:execute" in payload["scopes"]
    assert "balance:read" in payload["scopes"]


@pytest.mark.asyncio
async def test_token_rejected_with_invalid_signature(client: AsyncClient, created_did: dict):
    """Token issuance with invalid signature should be rejected."""
    did = created_did["did"]

    response = await client.post("/auth/token", json={
        "did": did,
        "message": "some_message",
        "signature": base64.b64encode(b"invalid" * 10).decode(),
        "requested_scopes": ["payment:execute"],
    })
    assert response.status_code == 401


@pytest.mark.asyncio
async def test_expired_token_verification_fails(client: AsyncClient):
    """Verifying an expired token should return 401."""
    # Create a token that's already expired
    import time as time_module
    expired_payload = {
        "sub": "did:key:ztest",
        "scopes": ["payment:execute"],
        "iat": int(time_module.time()) - 3600,
        "exp": int(time_module.time()) - 1800,  # Expired 30 min ago
    }

    from server.config import settings
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    import jwt

    private_key_bytes, _ = settings.get_server_keypair()
    private_key = Ed25519PrivateKey.from_private_bytes(private_key_bytes)
    expired_token = jwt.encode(
        expired_payload,
        private_key,
        algorithm="EdDSA"
    )

    response = await client.post("/auth/verify-token", json={"token": expired_token})
    assert response.status_code == 401
