"""SQLAlchemy model for the agents table."""
from sqlalchemy import Column, String, Integer, Text, DateTime
from sqlalchemy.sql import func
from sqlalchemy.orm import DeclarativeBase


class Base(DeclarativeBase):
    pass


class Agent(Base):
    """Represents an AI agent registered in the AgentTrust system."""

    __tablename__ = "agents"

    id = Column(String, primary_key=True)
    did = Column(String, unique=True, nullable=False)
    public_key = Column(Text, nullable=False)
    display_name = Column(String, nullable=True)
    max_transaction_limit = Column(Integer, default=100000)
    allowed_categories = Column(Text, default='[]')
    created_at = Column(DateTime, server_default=func.now())
    updated_at = Column(DateTime, server_default=func.now(), onupdate=func.now())
