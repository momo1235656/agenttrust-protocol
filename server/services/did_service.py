"""DID service for creating, resolving, and verifying DIDs."""
import uuid
import base64
import json
from pathlib import Path
from typing import Any

from server.config import settings
from server.crypto.keys import generate_keypair, derive_did, public_key_to_base64, base64_to_public_key
from server.crypto.signing import verify_signature_b64


class DIDNotFoundError(Exception):
    """Raised when a DID cannot be found."""
    pass


class InvalidSignatureError(Exception):
    """Raised when a signature verification fails."""
    pass


class DIDService:
    """Service for DID operations. Abstracts the DID document store."""

    def __init__(self, store_path: str = None):
        self.store_path = Path(store_path or settings.did_store_path)
        self.store_path.mkdir(parents=True, exist_ok=True)

    def _did_file_path(self, did: str) -> Path:
        """Get the file path for a DID document."""
        # Replace colons and other special chars for filesystem safety
        safe_did = did.replace(":", "_").replace("/", "_")
        return self.store_path / f"{safe_did}.json"

    def _save_document(self, did: str, document: dict[str, Any]) -> None:
        """Save a DID document to the JSON store."""
        file_path = self._did_file_path(did)
        with open(file_path, 'w') as f:
            json.dump(document, f, indent=2)

    def _load_document(self, did: str) -> dict[str, Any] | None:
        """Load a DID document from the JSON store."""
        file_path = self._did_file_path(did)
        if not file_path.exists():
            return None
        with open(file_path, 'r') as f:
            return json.load(f)

    async def create(
        self,
        db_session,
        display_name: str | None = None,
        max_transaction_limit: int = 100000,
        allowed_categories: list[str] = None,
    ) -> dict[str, Any]:
        """Create a new DID with Ed25519 keypair.

        Args:
            db_session: Async SQLAlchemy session
            display_name: Optional human-readable name
            max_transaction_limit: Max transaction amount in JPY
            allowed_categories: List of allowed payment categories

        Returns:
            dict containing did, document, and private_key_base64
        """
        from server.models.agent import Agent
        from sqlalchemy import select

        if allowed_categories is None:
            allowed_categories = []

        private_key, public_key = generate_keypair()
        did = derive_did(public_key)
        public_key_b64 = public_key_to_base64(public_key)
        private_key_b64 = base64.b64encode(private_key).decode()

        document = {
            "@context": ["https://www.w3.org/ns/did/v1"],
            "id": did,
            "authentication": [{
                "type": "Ed25519VerificationKey2020",
                "publicKeyBase64": public_key_b64,
            }],
            "service": [{
                "type": "AgentPayment",
                "maxTransactionLimit": max_transaction_limit,
                "allowedCategories": allowed_categories,
            }],
        }

        # Save DID document to JSON store
        self._save_document(did, document)

        # Save agent to database
        agent_id = str(uuid.uuid4())
        agent = Agent(
            id=agent_id,
            did=did,
            public_key=public_key_b64,
            display_name=display_name,
            max_transaction_limit=max_transaction_limit,
            allowed_categories=json.dumps(allowed_categories),
        )
        db_session.add(agent)
        await db_session.commit()

        return {
            "did": did,
            "document": document,
            "private_key_base64": private_key_b64,
        }

    async def resolve(self, did: str) -> dict[str, Any]:
        """Resolve a DID to its DID Document.

        Args:
            did: DID string to resolve

        Returns:
            dict with did, document, and found status

        Raises:
            DIDNotFoundError: If the DID does not exist
        """
        document = self._load_document(did)
        if document is None:
            raise DIDNotFoundError(f"DID not found: {did}")

        return {
            "did": did,
            "document": document,
            "found": True,
        }

    async def verify(self, did: str, message: str, signature: str) -> bool:
        """Verify an Ed25519 signature against a DID's public key.

        Args:
            did: DID of the signing agent
            message: The message that was signed (base64-encoded)
            signature: Base64-encoded Ed25519 signature

        Returns:
            bool: True if signature is valid

        Raises:
            DIDNotFoundError: If the DID does not exist
            InvalidSignatureError: If the signature is invalid
        """
        document = self._load_document(did)
        if document is None:
            raise DIDNotFoundError(f"DID not found: {did}")

        # Extract public key from DID document
        auth = document.get("authentication", [])
        if not auth:
            raise InvalidSignatureError("No authentication key found in DID document")

        public_key_b64 = auth[0].get("publicKeyBase64", "")
        public_key = base64_to_public_key(public_key_b64)

        # Verify signature
        if not verify_signature_b64(public_key, message, signature):
            raise InvalidSignatureError("Signature verification failed")

        return True

    def get_public_key(self, did: str) -> bytes:
        """Get the public key bytes for a DID.

        Args:
            did: DID string

        Returns:
            bytes: Raw Ed25519 public key bytes

        Raises:
            DIDNotFoundError: If DID not found
        """
        document = self._load_document(did)
        if document is None:
            raise DIDNotFoundError(f"DID not found: {did}")

        auth = document.get("authentication", [])
        if not auth:
            raise DIDNotFoundError(f"No authentication key in DID document for {did}")

        public_key_b64 = auth[0].get("publicKeyBase64", "")
        return base64_to_public_key(public_key_b64)


# Module-level singleton
did_service = DIDService()
