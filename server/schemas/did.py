"""Pydantic schemas for DID-related requests and responses."""
from pydantic import BaseModel, Field
from typing import Any


class DIDCreateRequest(BaseModel):
    """Request schema for creating a new DID."""
    display_name: str | None = Field(None, description="Optional display name for the agent")
    max_transaction_limit: int = Field(100000, description="Maximum transaction limit in JPY")
    allowed_categories: list[str] = Field(default_factory=list, description="Allowed payment categories")


class DIDDocument(BaseModel):
    """W3C DID Core compliant DID Document."""
    context: list[str] = Field(alias="@context", default=["https://www.w3.org/ns/did/v1"])
    id: str
    authentication: list[dict[str, Any]]
    service: list[dict[str, Any]]

    model_config = {"populate_by_name": True}


class DIDCreateResponse(BaseModel):
    """Response schema for DID creation."""
    did: str
    document: dict[str, Any]
    private_key_base64: str = Field(description="Private key - only returned at creation, store securely")


class DIDResolveResponse(BaseModel):
    """Response schema for DID resolution."""
    did: str
    document: dict[str, Any]
    found: bool


class DIDVerifyRequest(BaseModel):
    """Request schema for DID signature verification."""
    did: str
    message: str = Field(description="Base64-encoded message")
    signature: str = Field(description="Base64-encoded Ed25519 signature")


class DIDVerifyResponse(BaseModel):
    """Response schema for DID verification."""
    did: str
    verified: bool
