"""Tests for DID endpoints."""
import pytest
import base64
from httpx import AsyncClient

from server.crypto.keys import generate_keypair, derive_did
from server.crypto.signing import sign_message_b64


@pytest.mark.asyncio
async def test_create_did_success(client: AsyncClient):
    """DID creation should succeed and return a valid DID."""
    response = await client.post("/did/create", json={
        "display_name": "test-agent",
        "max_transaction_limit": 50000,
        "allowed_categories": ["electronics"]
    })
    assert response.status_code == 201
    data = response.json()
    assert "did" in data
    assert "document" in data
    assert "private_key_base64" in data


@pytest.mark.asyncio
async def test_created_did_starts_with_prefix(client: AsyncClient):
    """Created DID should start with did:key:z."""
    response = await client.post("/did/create", json={})
    assert response.status_code == 201
    data = response.json()
    assert data["did"].startswith("did:key:z")


@pytest.mark.asyncio
async def test_resolve_existing_did(client: AsyncClient, created_did: dict):
    """Resolving an existing DID should return its document."""
    did = created_did["did"]
    response = await client.get(f"/did/resolve/{did}")
    assert response.status_code == 200
    data = response.json()
    assert data["did"] == did
    assert data["found"] is True


@pytest.mark.asyncio
async def test_resolve_nonexistent_did(client: AsyncClient):
    """Resolving a non-existent DID should return 404."""
    response = await client.get("/did/resolve/did:key:zNONEXISTENT")
    assert response.status_code == 404


@pytest.mark.asyncio
async def test_verify_valid_signature(client: AsyncClient, created_did: dict):
    """Verifying a valid signature should return verified=True."""
    did = created_did["did"]
    private_key = base64.b64decode(created_did["private_key_base64"])
    message = "test_message_to_sign"
    signature = sign_message_b64(private_key, message)

    response = await client.post("/did/verify", json={
        "did": did,
        "message": message,
        "signature": signature,
    })
    assert response.status_code == 200
    data = response.json()
    assert data["verified"] is True


@pytest.mark.asyncio
async def test_verify_invalid_signature(client: AsyncClient, created_did: dict):
    """Verifying an invalid signature should return 401."""
    did = created_did["did"]

    response = await client.post("/did/verify", json={
        "did": did,
        "message": "test_message",
        "signature": base64.b64encode(b"invalid_signature" * 4).decode(),
    })
    assert response.status_code == 401
