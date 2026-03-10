"""OpenClawでAgentTrustを使うサンプル"""
import asyncio
from sdk.wallet import AgentWallet
from sdk.openclaw_tools import AgentTrustPaymentAction, AgentTrustAuditAction


async def main():
    # 1. ウォレット作成
    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(display_name="openclaw-demo", max_limit=100000)
    print(f"DID: {wallet.did}")

    # 2. アクション作成
    payment_action = AgentTrustPaymentAction(wallet=wallet)
    audit_action = AgentTrustAuditAction(wallet=wallet)

    # 3. スキーマ確認
    print(f"スキーマ: {payment_action.get_schema()}")

    # 4. 決済実行
    result = await payment_action.execute({
        "amount": 4000,
        "description": "OpenClawからのテスト決済"
    })
    print(result)

    # 5. 監査ログ確認
    audit_result = await audit_action.execute({})
    print(audit_result)


if __name__ == "__main__":
    asyncio.run(main())
