"""Audit service for hash chain management."""
from datetime import datetime, timezone
from typing import Any

from server.crypto.hashing import compute_hash, verify_chain as crypto_verify_chain


class AuditService:
    """Service for managing the immutable audit hash chain."""

    async def record(
        self,
        db_session,
        agent_did: str,
        transaction_id: str,
        amount: int,
        status: str,
        timestamp: str,
    ) -> str:
        """Record a transaction in the audit hash chain.

        Args:
            db_session: Async SQLAlchemy session
            agent_did: DID of the agent
            transaction_id: Unique transaction identifier
            amount: Transaction amount
            status: Transaction status
            timestamp: ISO 8601 timestamp

        Returns:
            str: Hash of the new audit log entry
        """
        from server.models.audit_log import AuditLog
        from sqlalchemy import select, func

        # Get the last audit log entry for this agent
        result = await db_session.execute(
            select(AuditLog)
            .where(AuditLog.agent_did == agent_did)
            .order_by(AuditLog.index_num.desc())
            .limit(1)
        )
        last_entry = result.scalar_one_or_none()

        if last_entry is None:
            prev_hash = "0" * 64
            index = 0
        else:
            prev_hash = last_entry.hash
            index = last_entry.index_num + 1

        # Compute hash for new entry
        entry_hash = compute_hash(
            index=index,
            transaction_id=transaction_id,
            amount=amount,
            status=status,
            timestamp=timestamp,
            prev_hash=prev_hash,
        )

        # Save to database
        audit_log = AuditLog(
            index_num=index,
            agent_did=agent_did,
            transaction_id=transaction_id,
            amount=amount,
            status=status,
            timestamp=timestamp,
            prev_hash=prev_hash,
            hash=entry_hash,
        )
        db_session.add(audit_log)

        return entry_hash

    async def get_chain(self, db_session, agent_did: str) -> dict[str, Any]:
        """Get the complete audit chain for an agent.

        Args:
            db_session: Async SQLAlchemy session
            agent_did: DID of the agent

        Returns:
            dict with chain entries and validation status
        """
        from server.models.audit_log import AuditLog
        from sqlalchemy import select

        result = await db_session.execute(
            select(AuditLog)
            .where(AuditLog.agent_did == agent_did)
            .order_by(AuditLog.index_num.asc())
        )
        entries = result.scalars().all()

        chain = []
        for entry in entries:
            chain.append({
                "index": entry.index_num,
                "transaction_id": entry.transaction_id,
                "amount": entry.amount,
                "status": entry.status,
                "timestamp": entry.timestamp,
                "prev_hash": entry.prev_hash,
                "hash": entry.hash,
            })

        chain_valid = crypto_verify_chain(chain) if chain else True

        total = len(chain)
        succeeded = sum(1 for e in chain if e["status"] == "succeeded")
        success_rate = succeeded / total if total > 0 else 1.0

        return {
            "agent_did": agent_did,
            "chain": chain,
            "chain_valid": chain_valid,
            "total_transactions": total,
            "success_rate": success_rate,
        }

    async def verify(self, db_session, agent_did: str) -> dict[str, Any]:
        """Verify the integrity of an agent's audit hash chain.

        Args:
            db_session: Async SQLAlchemy session
            agent_did: DID of the agent

        Returns:
            dict with verification result
        """
        from server.models.audit_log import AuditLog
        from sqlalchemy import select

        result = await db_session.execute(
            select(AuditLog)
            .where(AuditLog.agent_did == agent_did)
            .order_by(AuditLog.index_num.asc())
        )
        entries = result.scalars().all()

        chain = []
        for entry in entries:
            chain.append({
                "index": entry.index_num,
                "transaction_id": entry.transaction_id,
                "amount": entry.amount,
                "status": entry.status,
                "timestamp": entry.timestamp,
                "prev_hash": entry.prev_hash,
                "hash": entry.hash,
            })

        chain_valid = crypto_verify_chain(chain) if chain else True
        verified_at = datetime.now(timezone.utc).isoformat()

        return {
            "agent_did": agent_did,
            "chain_valid": chain_valid,
            "total_entries": len(chain),
            "verified_at": verified_at,
        }


audit_service = AuditService()
