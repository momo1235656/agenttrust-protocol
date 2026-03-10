"""SQLAlchemy model for the audit_logs table."""
from sqlalchemy import Column, String, Integer, Text, ForeignKey
from server.models.agent import Base


class AuditLog(Base):
    """Represents an entry in the immutable hash chain audit log."""

    __tablename__ = "audit_logs"

    id = Column(Integer, primary_key=True, autoincrement=True)
    index_num = Column(Integer, nullable=False)
    agent_did = Column(String, ForeignKey("agents.did"), nullable=False)
    transaction_id = Column(String, ForeignKey("transactions.id"), nullable=False)
    amount = Column(Integer, nullable=False)
    status = Column(String, nullable=False)
    timestamp = Column(String, nullable=False)
    prev_hash = Column(String, nullable=False)
    hash = Column(String, nullable=False)
