"""
OpenClaw用の決済ツール。
OpenClawのツール定義フォーマットに準拠。

使い方:
    from sdk.openclaw_tools import AgentTrustPaymentAction
    from sdk.wallet import AgentWallet

    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(display_name="openclaw-agent")

    actions = [AgentTrustPaymentAction(wallet=wallet)]
"""
from sdk.wallet import AgentWallet


class AgentTrustPaymentAction:
    """OpenClawのActionインターフェースに準拠した決済アクション。"""

    name: str = "agenttrust_payment"
    description: str = "AIエージェントとして決済を実行します。金額（円）と説明を指定してください。"

    def __init__(self, wallet: AgentWallet):
        """初期化。

        Args:
            wallet: 初期化済みのAgentWalletインスタンス
        """
        self.wallet = wallet

    def get_schema(self) -> dict:
        """OpenClawがツールのパラメータを認識するためのJSON Schemaを返す。

        Returns:
            dict: JSON Schema形式のパラメータ定義
        """
        return {
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "description": "決済金額（円）"
                },
                "description": {
                    "type": "string",
                    "description": "決済の説明"
                }
            },
            "required": ["amount"]
        }

    async def execute(self, params: dict) -> str:
        """OpenClawからの呼び出しを処理して決済を実行する。

        Args:
            params: パラメータdict。amountは必須、descriptionは任意。

        Returns:
            str: 決済結果のメッセージ
        """
        amount = params["amount"]
        description = params.get("description", "")
        result = await self.wallet.pay(amount=amount, description=description)
        return f"決済完了: {result['transaction_id']}, {result['amount']}円, ステータス: {result['status']}"


class AgentTrustAuditAction:
    """OpenClaw用の監査ログ確認アクション。"""

    name: str = "agenttrust_audit"
    description: str = "エージェントの取引履歴と監査ログを確認します。"

    def __init__(self, wallet: AgentWallet):
        """初期化。

        Args:
            wallet: 初期化済みのAgentWalletインスタンス
        """
        self.wallet = wallet

    def get_schema(self) -> dict:
        """パラメータスキーマを返す（パラメータなし）。

        Returns:
            dict: 空のJSON Schema
        """
        return {"type": "object", "properties": {}}

    async def execute(self, params: dict) -> str:
        """監査ログの要約を返す。

        Args:
            params: 使用しない（OpenClawの仕様上必要）

        Returns:
            str: 監査ログの要約メッセージ
        """
        audit = await self.wallet.get_audit_log()
        return (
            f"総取引数: {audit['total_transactions']}, "
            f"成功率: {audit['success_rate']:.1%}, "
            f"チェーン整合性: {'✅' if audit['chain_valid'] else '❌'}"
        )
