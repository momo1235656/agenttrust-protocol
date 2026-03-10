"""LangChain BaseTool integration for AgentTrust payments."""
from __future__ import annotations
from typing import Any, TYPE_CHECKING

from pydantic import BaseModel, Field

try:
    from langchain.tools import BaseTool
    LANGCHAIN_AVAILABLE = True
except ImportError:
    LANGCHAIN_AVAILABLE = False
    # Create a stub for when LangChain is not installed
    class BaseTool:
        pass


class PaymentInput(BaseModel):
    """Input schema for the payment tool."""
    amount: int = Field(description="決済金額（円）")
    description: str = Field(description="決済の説明")


if LANGCHAIN_AVAILABLE:
    class PaymentTool(BaseTool):
        """LangChain tool for executing payments via AgentTrust.

        Add this to your agent's tool list to enable payment capabilities.
        The agent can then make payments using natural language.
        """

        name: str = "agenttrust_payment"
        description: str = "AIエージェントとして決済を実行します。金額（円）と説明を指定してください。"
        args_schema: type = PaymentInput

        wallet: Any = Field(description="AgentWallet instance")

        def _run(self, amount: int, description: str) -> str:
            """Execute payment synchronously.

            Args:
                amount: Payment amount in JPY
                description: Payment description

            Returns:
                str: Result message
            """
            import asyncio
            result = asyncio.run(self.wallet.pay(amount=amount, description=description))
            return f"決済完了: {result['transaction_id']}, {result['amount']}円, ステータス: {result['status']}"

        async def _arun(self, amount: int, description: str) -> str:
            """Execute payment asynchronously.

            Args:
                amount: Payment amount in JPY
                description: Payment description

            Returns:
                str: Result message
            """
            result = await self.wallet.pay(amount=amount, description=description)
            return f"決済完了: {result['transaction_id']}, {result['amount']}円, ステータス: {result['status']}"
else:
    class PaymentTool:
        """Stub PaymentTool for when LangChain is not installed."""

        def __init__(self, wallet):
            self.wallet = wallet
            raise ImportError(
                "LangChain is required to use PaymentTool. "
                "Install it with: pip install langchain"
            )
