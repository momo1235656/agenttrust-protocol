"""SQLAlchemy model for the transactions table."""
from sqlalchemy import Column, String, Integer, Text, DateTime, ForeignKey
from sqlalchemy.sql import func
from server.models.agent import Base


class Transaction(Base):
    """Represents a payment transaction executed by an agent."""

    __tablename__ = "transactions"

    id = Column(String, primary_key=True)
    agent_did = Column(String, ForeignKey("agents.did"), nullable=False)
    amount = Column(Integer, nullable=False)
    currency = Column(String, default='jpy')
    description = Column(Text, nullable=True)
    status = Column(String, nullable=False)
    stripe_payment_intent_id = Column(String, nullable=True)
    idempotency_key = Column(String, unique=True, nullable=True)
    audit_hash = Column(String, nullable=True)
    created_at = Column(DateTime, server_default=func.now())
