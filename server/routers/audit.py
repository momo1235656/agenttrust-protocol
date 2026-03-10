"""Audit log API endpoints."""
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession

from server.dependencies import get_db
from server.schemas.audit import AuditChainResponse, AuditVerifyRequest, AuditVerifyResponse
from server.services.audit_service import audit_service

router = APIRouter(prefix="/audit", tags=["Audit"])


@router.get("/{agent_did:path}", response_model=AuditChainResponse)
async def get_audit_chain(
    agent_did: str,
    db: AsyncSession = Depends(get_db),
):
    """Get the complete audit hash chain for an agent."""
    result = await audit_service.get_chain(db, agent_did)
    return result


@router.post("/verify", response_model=AuditVerifyResponse)
async def verify_audit_chain(
    request: AuditVerifyRequest,
    db: AsyncSession = Depends(get_db),
):
    """Verify the integrity of an agent's audit hash chain."""
    result = await audit_service.verify(db, request.agent_did)
    return result
