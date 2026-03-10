"""Pydantic schemas for authentication-related requests and responses."""
from pydantic import BaseModel, Field
from typing import Any


class TokenRequest(BaseModel):
    """Request schema for JWT token issuance."""
    did: str = Field(description="Agent DID")
    message: str = Field(description="Signed message (typically auth_request_timestamp_...)")
    signature: str = Field(description="Base64-encoded Ed25519 signature of message")
    requested_scopes: list[str] = Field(
        default=["payment:execute"],
        description="Requested permission scopes"
    )


class TokenResponse(BaseModel):
    """Response schema for JWT token issuance."""
    access_token: str
    token_type: str = "Bearer"
    expires_in: int = Field(description="Token validity in seconds")
    scopes: list[str]


class VerifyTokenRequest(BaseModel):
    """Request schema for token verification."""
    token: str


class VerifyTokenResponse(BaseModel):
    """Response schema for token verification."""
    valid: bool
    payload: dict[str, Any] | None = None
