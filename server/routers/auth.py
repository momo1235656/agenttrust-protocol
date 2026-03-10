"""Authentication API endpoints."""
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession

from server.dependencies import get_db
from server.schemas.auth import TokenRequest, TokenResponse, VerifyTokenRequest, VerifyTokenResponse
from server.services.auth_service import auth_service, TokenExpiredError, TokenInvalidError
from server.services.did_service import DIDNotFoundError, InvalidSignatureError

router = APIRouter(prefix="/auth", tags=["Auth"])


@router.post("/token", response_model=TokenResponse)
async def issue_token(
    request: TokenRequest,
    db: AsyncSession = Depends(get_db),
):
    """Issue a scoped JWT after verifying DID signature."""
    try:
        result = await auth_service.issue_token(
            db_session=db,
            did=request.did,
            message=request.message,
            signature=request.signature,
            requested_scopes=request.requested_scopes,
        )
        return result
    except DIDNotFoundError:
        raise HTTPException(
            status_code=404,
            detail={"error": {"code": "DID_NOT_FOUND", "message": "DID not found", "details": {}}}
        )
    except InvalidSignatureError:
        raise HTTPException(
            status_code=401,
            detail={"error": {"code": "INVALID_SIGNATURE", "message": "The provided signature does not match the DID's public key.", "details": {}}}
        )
    except Exception as e:
        raise HTTPException(
            status_code=400,
            detail={"error": {"code": "TOKEN_INVALID", "message": str(e), "details": {}}}
        )


@router.post("/verify-token", response_model=VerifyTokenResponse)
async def verify_token(request: VerifyTokenRequest):
    """Verify a JWT and return its payload."""
    try:
        result = auth_service.verify_token(request.token)
        return result
    except TokenExpiredError:
        raise HTTPException(
            status_code=401,
            detail={"error": {"code": "TOKEN_EXPIRED", "message": "The token has expired.", "details": {}}}
        )
    except TokenInvalidError as e:
        raise HTTPException(
            status_code=401,
            detail={"error": {"code": "TOKEN_INVALID", "message": str(e), "details": {}}}
        )
