"""Payment service abstracting Stripe and future payment providers."""
import uuid
from datetime import datetime, timezone
from typing import Any

import stripe

from server.config import settings
from server.services.auth_service import auth_service, TokenExpiredError, TokenInvalidError
from server.services.audit_service import audit_service


class PaymentError(Exception):
    """Raised when a payment operation fails."""
    pass


class ScopeExceededError(Exception):
    """Raised when payment exceeds token scope."""
    pass


class DuplicateTransactionError(Exception):
    """Raised when an idempotency key is reused."""
    pass


class PaymentService:
    """Service for executing payments. Abstracts the payment provider."""

    def _get_stripe_client(self):
        """Get configured Stripe client."""
        stripe.api_key = settings.stripe_secret_key
        return stripe

    async def _create_stripe_payment(
        self, amount: int, currency: str, description: str, idempotency_key: str
    ) -> dict[str, Any]:
        """Create a Stripe PaymentIntent (internal method).

        Args:
            amount: Amount in smallest currency unit (e.g., yen for JPY)
            currency: ISO currency code
            description: Payment description
            idempotency_key: Unique key for idempotency

        Returns:
            dict with Stripe PaymentIntent details

        Raises:
            PaymentError: If Stripe returns an error
        """
        client = self._get_stripe_client()
        try:
            payment_intent = client.PaymentIntent.create(
                amount=amount,
                currency=currency,
                description=description,
                payment_method="pm_card_visa",
                confirm=True,
                automatic_payment_methods={"enabled": True, "allow_redirects": "never"},
                idempotency_key=idempotency_key,
            )
            return {
                "id": payment_intent.id,
                "status": payment_intent.status,
                "amount": payment_intent.amount,
            }
        except stripe.error.StripeError as e:
            raise PaymentError(f"Stripe error: {str(e)}")

    async def execute(
        self,
        db_session,
        authorization: str,
        amount: int,
        currency: str,
        description: str,
        idempotency_key: str,
    ) -> dict[str, Any]:
        """Execute a payment after verifying JWT authorization.

        Args:
            db_session: Async SQLAlchemy session
            authorization: Authorization header value
            amount: Payment amount in JPY
            currency: Currency code
            description: Payment description
            idempotency_key: Unique key to prevent duplicate payments

        Returns:
            dict with transaction details

        Raises:
            TokenInvalidError: If JWT is invalid
            ScopeExceededError: If amount exceeds token scope
            DuplicateTransactionError: If idempotency key already used
            PaymentError: If payment execution fails
        """
        from server.models.transaction import Transaction
        from sqlalchemy import select
        import json

        # Extract and verify JWT
        token = auth_service.extract_token_from_header(authorization)
        token_data = auth_service.verify_token(token)
        payload = token_data["payload"]

        agent_did = payload["sub"]

        # Check scope
        if "payment:execute" not in payload.get("scopes", []):
            raise ScopeExceededError("Token does not have payment:execute scope")

        # Check amount limit
        max_amount = payload.get("max_amount", 0)
        if amount > max_amount:
            raise ScopeExceededError(
                f"Amount {amount} exceeds token max_amount {max_amount}"
            )

        # Check for duplicate transaction
        result = await db_session.execute(
            select(Transaction).where(Transaction.idempotency_key == idempotency_key)
        )
        existing = result.scalar_one_or_none()
        if existing is not None:
            # Return existing transaction for idempotent response
            raise DuplicateTransactionError(existing.id)

        # Execute Stripe payment
        transaction_id = "tx_" + str(uuid.uuid4()).replace("-", "")[:20]
        stripe_result = await self._create_stripe_payment(
            amount=amount,
            currency=currency,
            description=description,
            idempotency_key=idempotency_key,
        )

        status = "succeeded" if stripe_result["status"] in ("succeeded", "requires_capture") else "failed"

        created_at = datetime.now(timezone.utc)

        # Record in database
        transaction = Transaction(
            id=transaction_id,
            agent_did=agent_did,
            amount=amount,
            currency=currency,
            description=description,
            status=status,
            stripe_payment_intent_id=stripe_result["id"],
            idempotency_key=idempotency_key,
        )
        db_session.add(transaction)
        await db_session.flush()  # Get the record in DB before audit

        # Record in audit hash chain
        audit_hash = await audit_service.record(
            db_session=db_session,
            agent_did=agent_did,
            transaction_id=transaction_id,
            amount=amount,
            status=status,
            timestamp=created_at.isoformat(),
        )

        # Update transaction with audit hash
        transaction.audit_hash = audit_hash
        await db_session.commit()

        return {
            "transaction_id": transaction_id,
            "status": status,
            "amount": amount,
            "currency": currency,
            "agent_did": agent_did,
            "stripe_payment_intent_id": stripe_result["id"],
            "audit_hash": audit_hash,
            "created_at": created_at.isoformat(),
        }

    async def get_status(self, db_session, transaction_id: str) -> dict[str, Any]:
        """Get the status of a transaction.

        Args:
            db_session: Async SQLAlchemy session
            transaction_id: Transaction ID

        Returns:
            dict with transaction details

        Raises:
            ValueError: If transaction not found
        """
        from server.models.transaction import Transaction
        from sqlalchemy import select

        result = await db_session.execute(
            select(Transaction).where(Transaction.id == transaction_id)
        )
        transaction = result.scalar_one_or_none()
        if transaction is None:
            raise ValueError(f"Transaction not found: {transaction_id}")

        return {
            "transaction_id": transaction.id,
            "status": transaction.status,
            "amount": transaction.amount,
            "currency": transaction.currency,
            "agent_did": transaction.agent_did,
            "created_at": transaction.created_at.isoformat() if transaction.created_at else "",
        }


payment_service = PaymentService()
