"""Pydantic schemas for payment-related requests and responses."""
from pydantic import BaseModel, Field
from typing import Any
import uuid


class PaymentExecuteRequest(BaseModel):
    """Request schema for executing a payment."""
    amount: int = Field(gt=0, description="Payment amount in JPY")
    currency: str = Field(default="jpy", description="Currency code")
    description: str = Field(default="", description="Payment description")
    idempotency_key: str = Field(
        default_factory=lambda: str(uuid.uuid4()),
        description="Unique key to prevent duplicate payments"
    )


class PaymentExecuteResponse(BaseModel):
    """Response schema for payment execution."""
    transaction_id: str
    status: str
    amount: int
    currency: str
    agent_did: str
    stripe_payment_intent_id: str | None
    audit_hash: str | None
    created_at: str


class PaymentStatusResponse(BaseModel):
    """Response schema for payment status check."""
    transaction_id: str
    status: str
    amount: int
    currency: str
    agent_did: str
    created_at: str
