"""DID API endpoints."""
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession

from server.dependencies import get_db
from server.schemas.did import (
    DIDCreateRequest, DIDCreateResponse,
    DIDResolveResponse, DIDVerifyRequest, DIDVerifyResponse
)
from server.services.did_service import did_service, DIDNotFoundError, InvalidSignatureError

router = APIRouter(prefix="/did", tags=["DID"])


@router.post("/create", response_model=DIDCreateResponse, status_code=201)
async def create_did(
    request: DIDCreateRequest,
    db: AsyncSession = Depends(get_db),
):
    """Create a new DID with Ed25519 keypair and register the agent."""
    result = await did_service.create(
        db_session=db,
        display_name=request.display_name,
        max_transaction_limit=request.max_transaction_limit,
        allowed_categories=request.allowed_categories,
    )
    return result


@router.get("/resolve/{did:path}", response_model=DIDResolveResponse)
async def resolve_did(did: str):
    """Resolve a DID to its DID Document."""
    try:
        result = await did_service.resolve(did)
        return result
    except DIDNotFoundError:
        raise HTTPException(
            status_code=404,
            detail={"error": {"code": "DID_NOT_FOUND", "message": f"DID not found: {did}", "details": {}}}
        )


@router.post("/verify", response_model=DIDVerifyResponse)
async def verify_did(request: DIDVerifyRequest):
    """Verify an Ed25519 signature against a DID's public key."""
    try:
        await did_service.verify(request.did, request.message, request.signature)
        return {"did": request.did, "verified": True}
    except DIDNotFoundError:
        raise HTTPException(
            status_code=404,
            detail={"error": {"code": "DID_NOT_FOUND", "message": f"DID not found: {request.did}", "details": {}}}
        )
    except InvalidSignatureError:
        raise HTTPException(
            status_code=401,
            detail={"error": {"code": "INVALID_SIGNATURE", "message": "The provided signature does not match the DID's public key.", "details": {}}}
        )
