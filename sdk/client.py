"""HTTP client for the AgentTrust API."""
import httpx
from typing import Any


class AgentTrustClient:
    """Async HTTP client for the AgentTrust Protocol API."""

    def __init__(self, server_url: str = "http://localhost:8000"):
        self.server_url = server_url.rstrip("/")
        self._client: httpx.AsyncClient | None = None

    async def _get_client(self) -> httpx.AsyncClient:
        if self._client is None or self._client.is_closed:
            self._client = httpx.AsyncClient(base_url=self.server_url, timeout=30.0)
        return self._client

    async def close(self):
        """Close the HTTP client."""
        if self._client and not self._client.is_closed:
            await self._client.aclose()

    async def create_did(self, display_name: str = None, max_limit: int = 100000, allowed_categories: list[str] = None) -> dict[str, Any]:
        """Create a new DID."""
        client = await self._get_client()
        payload = {"max_transaction_limit": max_limit, "allowed_categories": allowed_categories or []}
        if display_name:
            payload["display_name"] = display_name
        response = await client.post("/did/create", json=payload)
        response.raise_for_status()
        return response.json()

    async def resolve_did(self, did: str) -> dict[str, Any]:
        """Resolve a DID."""
        client = await self._get_client()
        response = await client.get(f"/did/resolve/{did}")
        response.raise_for_status()
        return response.json()

    async def issue_token(self, did: str, message: str, signature: str, scopes: list[str] = None) -> dict[str, Any]:
        """Issue an access token."""
        client = await self._get_client()
        payload = {
            "did": did,
            "message": message,
            "signature": signature,
            "requested_scopes": scopes or ["payment:execute"],
        }
        response = await client.post("/auth/token", json=payload)
        response.raise_for_status()
        return response.json()

    async def execute_payment(self, token: str, amount: int, description: str, idempotency_key: str = None, currency: str = "jpy") -> dict[str, Any]:
        """Execute a payment."""
        import uuid
        client = await self._get_client()
        payload = {
            "amount": amount,
            "currency": currency,
            "description": description,
            "idempotency_key": idempotency_key or str(uuid.uuid4()),
        }
        headers = {"Authorization": f"Bearer {token}"}
        response = await client.post("/payment/execute", json=payload, headers=headers)
        response.raise_for_status()
        return response.json()

    async def get_audit_chain(self, agent_did: str) -> dict[str, Any]:
        """Get audit chain for an agent."""
        client = await self._get_client()
        response = await client.get(f"/audit/{agent_did}")
        response.raise_for_status()
        return response.json()
