"""Payment API endpoints."""
from fastapi import APIRouter, Depends, HTTPException, Header
from sqlalchemy.ext.asyncio import AsyncSession
from typing import Annotated

from server.dependencies import get_db
from server.schemas.payment import PaymentExecuteRequest, PaymentExecuteResponse, PaymentStatusResponse
from server.services.payment_service import (
    payment_service, PaymentError, ScopeExceededError, DuplicateTransactionError
)
from server.services.auth_service import TokenExpiredError, TokenInvalidError

router = APIRouter(prefix="/payment", tags=["Payment"])


@router.post("/execute", response_model=PaymentExecuteResponse)
async def execute_payment(
    request: PaymentExecuteRequest,
    authorization: Annotated[str | None, Header()] = None,
    db: AsyncSession = Depends(get_db),
):
    """Execute a payment using the agent's scoped JWT."""
    if not authorization:
        raise HTTPException(
            status_code=401,
            detail={"error": {"code": "TOKEN_INVALID", "message": "Missing Authorization header", "details": {}}}
        )

    try:
        result = await payment_service.execute(
            db_session=db,
            authorization=authorization,
            amount=request.amount,
            currency=request.currency,
            description=request.description,
            idempotency_key=request.idempotency_key,
        )
        return result
    except (TokenExpiredError,):
        raise HTTPException(
            status_code=401,
            detail={"error": {"code": "TOKEN_EXPIRED", "message": "Token has expired", "details": {}}}
        )
    except (TokenInvalidError,):
        raise HTTPException(
            status_code=401,
            detail={"error": {"code": "TOKEN_INVALID", "message": "Invalid token", "details": {}}}
        )
    except ScopeExceededError as e:
        raise HTTPException(
            status_code=403,
            detail={"error": {"code": "SCOPE_EXCEEDED", "message": str(e), "details": {}}}
        )
    except DuplicateTransactionError as e:
        # Return existing transaction
        existing_id = str(e)
        existing = await payment_service.get_status(db, existing_id)
        raise HTTPException(
            status_code=409,
            detail={"error": {"code": "DUPLICATE_TRANSACTION", "message": "Transaction with this idempotency key already exists", "details": existing}}
        )
    except PaymentError as e:
        raise HTTPException(
            status_code=502,
            detail={"error": {"code": "PAYMENT_FAILED", "message": str(e), "details": {}}}
        )


@router.get("/{transaction_id}", response_model=PaymentStatusResponse)
async def get_payment_status(
    transaction_id: str,
    db: AsyncSession = Depends(get_db),
):
    """Get the status of a payment transaction."""
    try:
        result = await payment_service.get_status(db, transaction_id)
        return result
    except ValueError:
        raise HTTPException(
            status_code=404,
            detail={"error": {"code": "DID_NOT_FOUND", "message": f"Transaction not found: {transaction_id}", "details": {}}}
        )
