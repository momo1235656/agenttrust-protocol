"""
AutoGen用の決済ツール。
AutoGenのAssistantAgentにツールとして登録可能。

使い方:
    from sdk.autogen_tools import create_payment_tools
    from sdk.wallet import AgentWallet

    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(display_name="autogen-agent")

    tools = create_payment_tools(wallet)

    # AutoGen v0.4+ の場合
    from autogen_agentchat.agents import AssistantAgent
    agent = AssistantAgent(
        name="payment_agent",
        model_client=model_client,
        tools=tools,
    )
"""
from typing import Annotated
from sdk.wallet import AgentWallet


def create_payment_tools(wallet: AgentWallet) -> list:
    """AutoGen用のツール関数リストを返す。

    Args:
        wallet: 初期化済みのAgentWalletインスタンス

    Returns:
        list: AutoGenのツールとして登録可能な非同期関数のリスト
    """

    async def execute_payment(
        amount: Annotated[int, "決済金額（円）"],
        description: Annotated[str, "決済の説明"] = "",
    ) -> str:
        """AIエージェントとして決済を実行します。金額（円）と説明を指定してください。"""
        result = await wallet.pay(amount=amount, description=description)
        return f"決済完了: {result['transaction_id']}, {result['amount']}円, ステータス: {result['status']}"

    async def check_balance(
        dummy: Annotated[str, "任意の文字列（無視されます）"] = "",
    ) -> str:
        """エージェントの取引履歴と監査ログを確認します。"""
        audit = await wallet.get_audit_log()
        return (
            f"総取引数: {audit['total_transactions']}, "
            f"成功率: {audit['success_rate']:.1%}, "
            f"チェーン整合性: {'✅' if audit['chain_valid'] else '❌'}"
        )

    return [execute_payment, check_balance]
