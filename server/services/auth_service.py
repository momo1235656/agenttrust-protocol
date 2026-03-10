"""Authentication service for JWT token issuance and verification."""
import time
from typing import Any
import jwt
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey

from server.config import settings
from server.services.did_service import did_service, DIDNotFoundError, InvalidSignatureError


TOKEN_EXPIRY_SECONDS = 1800  # 30 minutes

VALID_SCOPES = {
    "payment:execute",
    "payment:read",
    "balance:read",
    "audit:read",
}


class TokenError(Exception):
    """Base class for token errors."""
    pass


class TokenExpiredError(TokenError):
    """Raised when a JWT has expired."""
    pass


class TokenInvalidError(TokenError):
    """Raised when a JWT is invalid."""
    pass


class ScopeError(Exception):
    """Raised when requested scopes exceed agent permissions."""
    pass


class AuthService:
    """Service for JWT token operations."""

    def _get_jwt_keys(self) -> tuple[bytes, bytes]:
        """Get server's Ed25519 keypair for JWT signing."""
        return settings.get_server_keypair()

    def _get_jwt_signing_key(self, private_key_bytes: bytes) -> Ed25519PrivateKey:
        """Convert raw Ed25519 bytes to cryptography library format for PyJWT."""
        return Ed25519PrivateKey.from_private_bytes(private_key_bytes)

    def _get_jwt_verify_key(self, public_key_bytes: bytes) -> Ed25519PublicKey:
        """Convert raw Ed25519 bytes to cryptography library format for PyJWT."""
        return Ed25519PublicKey.from_public_bytes(public_key_bytes)

    async def issue_token(
        self,
        db_session,
        did: str,
        message: str,
        signature: str,
        requested_scopes: list[str],
    ) -> dict[str, Any]:
        """Issue a scoped JWT after verifying DID signature.

        Args:
            db_session: Async SQLAlchemy session
            did: Agent DID
            message: Message that was signed
            signature: Base64-encoded Ed25519 signature
            requested_scopes: List of requested permission scopes

        Returns:
            dict with access_token, token_type, expires_in, scopes

        Raises:
            DIDNotFoundError: If DID not found
            InvalidSignatureError: If signature is invalid
            ScopeError: If requested scopes exceed agent permissions
        """
        from server.models.agent import Agent
        from sqlalchemy import select
        import json

        # Verify DID signature
        await did_service.verify(did, message, signature)

        # Get agent from database
        result = await db_session.execute(select(Agent).where(Agent.did == did))
        agent = result.scalar_one_or_none()
        if agent is None:
            raise DIDNotFoundError(f"Agent not found for DID: {did}")

        # Validate requested scopes
        invalid_scopes = set(requested_scopes) - VALID_SCOPES
        if invalid_scopes:
            raise ScopeError(f"Invalid scopes requested: {invalid_scopes}")

        allowed_categories = json.loads(agent.allowed_categories or '[]')

        # Build JWT payload
        now = int(time.time())
        payload = {
            "sub": did,
            "scopes": requested_scopes,
            "max_amount": agent.max_transaction_limit,
            "currency": "jpy",
            "allowed_categories": allowed_categories,
            "iat": now,
            "exp": now + TOKEN_EXPIRY_SECONDS,
        }

        # Sign JWT with server's Ed25519 private key using cryptography library
        private_key_bytes, _ = self._get_jwt_keys()
        private_key = self._get_jwt_signing_key(private_key_bytes)
        token = jwt.encode(payload, private_key, algorithm="EdDSA")

        return {
            "access_token": token,
            "token_type": "Bearer",
            "expires_in": TOKEN_EXPIRY_SECONDS,
            "scopes": requested_scopes,
        }

    def verify_token(self, token: str) -> dict[str, Any]:
        """Verify a JWT and return its payload.

        Args:
            token: JWT string

        Returns:
            dict with valid status and payload

        Raises:
            TokenExpiredError: If token has expired
            TokenInvalidError: If token is invalid
        """
        private_key_bytes, public_key_bytes = self._get_jwt_keys()

        try:
            public_key = self._get_jwt_verify_key(public_key_bytes)
            payload = jwt.decode(
                token,
                public_key,
                algorithms=["EdDSA"],
            )
            return {"valid": True, "payload": payload}
        except jwt.ExpiredSignatureError:
            raise TokenExpiredError("Token has expired")
        except jwt.InvalidTokenError as e:
            raise TokenInvalidError(f"Invalid token: {str(e)}")

    def extract_token_from_header(self, authorization: str) -> str:
        """Extract JWT from Authorization header.

        Args:
            authorization: Authorization header value (e.g., "Bearer eyJ...")

        Returns:
            str: JWT token string

        Raises:
            TokenInvalidError: If header format is invalid
        """
        if not authorization or not authorization.startswith("Bearer "):
            raise TokenInvalidError("Missing or invalid Authorization header")
        return authorization[7:]  # Remove "Bearer " prefix


auth_service = AuthService()
