"""Pydantic schemas for audit log requests and responses."""
from pydantic import BaseModel, Field
from typing import Any


class AuditLogEntry(BaseModel):
    """Represents a single entry in the audit hash chain."""
    index: int
    transaction_id: str
    amount: int
    status: str
    timestamp: str
    prev_hash: str
    hash: str


class AuditChainResponse(BaseModel):
    """Response schema for audit chain retrieval."""
    agent_did: str
    chain: list[AuditLogEntry]
    chain_valid: bool
    total_transactions: int
    success_rate: float


class AuditVerifyRequest(BaseModel):
    """Request schema for hash chain verification."""
    agent_did: str


class AuditVerifyResponse(BaseModel):
    """Response schema for hash chain verification."""
    agent_did: str
    chain_valid: bool
    total_entries: int
    verified_at: str
