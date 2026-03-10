"""AgentWallet - the main class for agent developers."""
import time
import base64
from typing import Any

from sdk.client import AgentTrustClient
from server.crypto.keys import generate_keypair, derive_did
from server.crypto.signing import sign_message_b64


class AgentWallet:
    """Main SDK class for AI agent payment operations.

    Manages DID identity, token caching, and payment execution.
    Can be initialized for a new agent or with an existing DID.
    """

    def __init__(
        self,
        server_url: str = "http://localhost:8000",
        owner: str = None,
        did: str = None,
        private_key: bytes = None,
    ):
        """Initialize the wallet.

        For new agents: AgentWallet(server_url="...")
        For existing DIDs: AgentWallet(did="did:key:...", private_key=b"...")

        Args:
            server_url: URL of the AgentTrust Protocol server
            owner: Optional owner identifier
            did: Existing DID (if resuming)
            private_key: Existing private key bytes (if resuming)
        """
        self.server_url = server_url
        self.owner = owner
        self.did = did
        self._private_key = private_key
        self._access_token: str | None = None
        self._token_expires_at: float = 0
        self._client = AgentTrustClient(server_url=server_url)

    async def create(
        self,
        display_name: str = None,
        max_limit: int = 100000,
        allowed_categories: list[str] = None,
    ) -> "AgentWallet":
        """Create a new DID and register with the server.

        Args:
            display_name: Optional human-readable name
            max_limit: Maximum transaction limit in JPY
            allowed_categories: Allowed payment categories

        Returns:
            self for method chaining
        """
        result = await self._client.create_did(
            display_name=display_name,
            max_limit=max_limit,
            allowed_categories=allowed_categories,
        )
        self.did = result["did"]
        self._private_key = base64.b64decode(result["private_key_base64"])
        return self

    async def get_token(self, scopes: list[str] = None) -> str:
        """Get an access token, using cache if still valid.

        Args:
            scopes: Requested permission scopes

        Returns:
            str: JWT access token
        """
        # Return cached token if still valid (with 60s buffer)
        if self._access_token and time.time() < self._token_expires_at - 60:
            return self._access_token

        if not self.did or not self._private_key:
            raise ValueError("Wallet not initialized. Call create() first or provide did and private_key.")

        # Sign a timestamp message to prove DID ownership
        message = f"auth_request_{int(time.time())}"
        signature = sign_message_b64(self._private_key, message)

        result = await self._client.issue_token(
            did=self.did,
            message=message,
            signature=signature,
            scopes=scopes or ["payment:execute"],
        )

        self._access_token = result["access_token"]
        self._token_expires_at = time.time() + result["expires_in"]
        return self._access_token

    async def pay(
        self,
        amount: int,
        description: str = "",
        idempotency_key: str = None,
        currency: str = "jpy",
    ) -> dict[str, Any]:
        """Execute a payment.

        Args:
            amount: Payment amount in JPY
            description: Payment description
            idempotency_key: Optional unique key for idempotency
            currency: Currency code (default: jpy)

        Returns:
            dict with transaction details
        """
        token = await self.get_token()
        return await self._client.execute_payment(
            token=token,
            amount=amount,
            description=description,
            idempotency_key=idempotency_key,
            currency=currency,
        )

    async def get_audit_chain(self) -> dict[str, Any]:
        """Get the audit chain for this agent.

        Returns:
            dict with chain entries and validation status
        """
        if not self.did:
            raise ValueError("Wallet not initialized.")
        return await self._client.get_audit_chain(self.did)

    async def close(self):
        """Close the HTTP client."""
        await self._client.close()
